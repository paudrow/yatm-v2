use anyhow::{bail, Result};

pub struct TocOptions {
    pub toc_title: Option<String>,
    pub toc_title_level: Option<usize>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub spaces_per_indent: Option<usize>,
}

impl Default for TocOptions {
    fn default() -> Self {
        TocOptions {
            toc_title: None,
            toc_title_level: None,
            min_depth: None,
            max_depth: None,
            spaces_per_indent: Some(2),
        }
    }
}

pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
}

pub fn prepend_markdown_table_of_contents(
    content: &String,
    options: Option<&TocOptions>,
) -> String {
    let toc = make_markdown_table_of_contents(content, options).unwrap_or_default();
    if toc.is_empty() {
        return content.clone();
    }
    format!("{}\n\n{}", toc, content)
}

pub fn make_markdown_table_of_contents(
    content: &String,
    options: Option<&TocOptions>,
) -> Result<String> {
    let default_options = TocOptions::default();
    let options = options.unwrap_or(&default_options);

    let mut toc = String::new();

    let mut is_in_code_block = false;
    for line in content.lines() {
        if line.starts_with("```") {
            is_in_code_block = !is_in_code_block;
        }
        if is_in_code_block {
            continue;
        }
        if line.starts_with("#") {
            let level = line.chars().take_while(|&c| c == '#').count();
            let title = line[level..].trim();
            let slug = slugify(title);

            if options.min_depth.is_some() && level < options.min_depth.unwrap() {
                continue;
            }
            if options.max_depth.is_some() && level > options.max_depth.unwrap() {
                continue;
            }
            let indent = " ".repeat(
                (level - options.min_depth.unwrap_or(1)) * options.spaces_per_indent.unwrap_or(2),
            );
            toc.push_str(&format!("{}- [{}](#{})\n", indent, title, slug));
        }
    }
    if toc.is_empty() {
        bail!("No headings found in the document");
    }

    if let Some(toc_title) = &options.toc_title {
        let toc_title = match options.toc_title_level {
            Some(toc_title_level) => format!("{} {}\n", "#".repeat(toc_title_level), toc_title),
            None => format!("{}\n", toc_title),
        };
        toc = format!("{}\n{}", toc_title, toc);
    }
    Ok(toc)
}

#[cfg(test)]
mod test_make_markdown_table_of_contents {
    use super::*;

    #[test]
    fn test_make_markdown_table_of_contents() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(&content, None).unwrap();

        assert_eq!(
            result,
            r#"- [Title](#title)
  - [Subtitle](#subtitle)
    - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_special_characters() {
        let content = r#"
# Title with special characters: `&`
        "#
        .to_string();
        let result = make_markdown_table_of_contents(&content, None).unwrap();

        assert_eq!(
            result,
            "- [Title with special characters: `&`](#title-with-special-characters-)\n".to_string()
        );
    }

    #[test]
    fn test_handles_min_depth() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: Some(2),
                max_depth: None,
                toc_title: None,
                toc_title_level: None,
                spaces_per_indent: Some(2),
            }),
        )
        .unwrap();

        assert_eq!(
            result,
            r#"- [Subtitle](#subtitle)
  - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_max_depth() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: None,
                max_depth: Some(1),
                toc_title: None,
                toc_title_level: None,
                spaces_per_indent: Some(2),
            }),
        )
        .unwrap();

        assert_eq!(
            result,
            r#"- [Title](#title)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_min_and_max_depth() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: Some(2),
                max_depth: Some(2),
                toc_title: None,
                toc_title_level: None,
                spaces_per_indent: Some(2),
            }),
        )
        .unwrap();

        assert_eq!(
            result,
            r#"- [Subtitle](#subtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_toc_title_with_level() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: None,
                max_depth: None,
                toc_title: Some("Table of Contents".to_string()),
                toc_title_level: None,
                spaces_per_indent: Some(2),
            }),
        )
        .unwrap();

        assert_eq!(
            result,
            r#"Table of Contents

- [Title](#title)
  - [Subtitle](#subtitle)
    - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_toc_title_without_level() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: None,
                max_depth: None,
                toc_title: Some("Table of Contents".to_string()),
                toc_title_level: None,
                spaces_per_indent: Some(2),
            }),
        )
        .unwrap();

        assert_eq!(
            result,
            r#"Table of Contents

- [Title](#title)
  - [Subtitle](#subtitle)
    - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_toc_title_level() {
        let content = r#"
# Title
## Subtitle
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(
            &content,
            Some(&TocOptions {
                min_depth: None,
                max_depth: None,
                toc_title: None,
                toc_title_level: None,
                spaces_per_indent: Some(4),
            }),
        )
        .unwrap();
        assert_eq!(
            result,
            r#"- [Title](#title)
    - [Subtitle](#subtitle)
        - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }

    #[test]
    fn test_handles_code_blocks() {
        let content = r#"
# Title
## Subtitle
```bash
# a comment
```
### Subsubtitle
        "#
        .to_string();
        let result = make_markdown_table_of_contents(&content, None).unwrap();
        assert_eq!(
            result,
            r#"- [Title](#title)
  - [Subtitle](#subtitle)
    - [Subsubtitle](#subsubtitle)
"#
            .to_string()
        );
    }
}
