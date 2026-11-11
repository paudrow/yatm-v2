//! Explain why an existing GitHub issue is considered changed relative to the
//! locally generated issue, using the `similar` crate for unified diffs.

use similar::TextDiff;

/// Render a unified diff (`---`/`+++`/`@@` hunks) between `old` and `new` with
/// `context` lines of surrounding context. Returns an empty string if the
/// inputs are identical.
pub fn unified_diff(
    old: &str,
    new: &str,
    old_label: &str,
    new_label: &str,
    context: usize,
) -> String {
    if old == new {
        return String::new();
    }
    TextDiff::from_lines(old, new)
        .unified_diff()
        .context_radius(context)
        .header(old_label, new_label)
        .to_string()
}

/// Explain the differences between an existing GitHub issue and the locally
/// generated issue (title and body). Returns an empty string if identical.
pub fn describe_issue_diff(
    github_title: &str,
    github_body: &str,
    local_title: &str,
    local_body: &str,
) -> String {
    let mut out = String::new();
    if github_title != local_title {
        out.push_str(&format!("Title:\n- {}\n+ {}\n", github_title, local_title));
    }
    if github_body != local_body {
        // GitHub may return bodies with CRLF line endings; diff with normalized
        // line endings so visually identical lines aren't all reported as changed.
        let github_norm = github_body.replace("\r\n", "\n");
        let local_norm = local_body.replace("\r\n", "\n");
        let crlf_differs = github_body.contains('\r') != local_body.contains('\r');

        if github_norm.trim_end_matches('\n') == local_norm.trim_end_matches('\n') {
            out.push_str(if crlf_differs {
                "Body: only line ending differences (CRLF vs LF)\n"
            } else {
                "Body: only trailing newline differences\n"
            });
        } else {
            out.push_str("Body:\n");
            out.push_str(&unified_diff(
                &github_norm,
                &local_norm,
                "github",
                "local",
                3,
            ));
            if crlf_differs {
                out.push_str("(Line endings also differ: CRLF vs LF)\n");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_inputs_produce_no_diff() {
        assert_eq!(unified_diff("a\nb\n", "a\nb\n", "old", "new", 3), "");
        assert_eq!(describe_issue_diff("t", "a\nb", "t", "a\nb"), "");
    }

    #[test]
    fn single_line_change_with_context() {
        let old = "1\n2\n3\n4\n5\n6\n7\n8\n";
        let new = "1\n2\n3\n4\nFIVE\n6\n7\n8\n";
        let diff = unified_diff(old, new, "github", "local", 2);
        assert_eq!(
            diff,
            "--- github\n+++ local\n@@ -3,5 +3,5 @@\n 3\n 4\n-5\n+FIVE\n 6\n 7\n"
        );
    }

    #[test]
    fn insertions_and_separate_hunks() {
        let old = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n";
        let new = "a\nB\nc\nd\ne\nf\ng\nh\ni\nj\nk\n";
        let diff = unified_diff(old, new, "o", "n", 1);
        assert_eq!(
            diff,
            "--- o\n+++ n\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n@@ -10 +10,2 @@\n j\n+k\n"
        );
    }

    #[test]
    fn describes_title_and_line_ending_only_changes() {
        let d = describe_issue_diff("Old title", "a\r\nb", "New title", "a\nb");
        assert!(d.contains("Title:\n- Old title\n+ New title\n"));
        assert!(d.contains("Body: only line ending differences (CRLF vs LF)"));

        let d2 = describe_issue_diff("t", "a\nb\n", "t", "a\nb");
        assert_eq!(d2, "Body: only trailing newline differences\n");
    }

    #[test]
    fn crlf_body_with_real_change_diffs_normalized_lines() {
        let d = describe_issue_diff("t", "a\r\nb\r\nc\r\n", "t", "a\nB\nc\n");
        assert!(d.contains("-b\n+B\n"), "{}", d);
        assert!(
            !d.contains("-a"),
            "unchanged lines must not show as changed: {}",
            d
        );
        assert!(d.contains("(Line endings also differ: CRLF vs LF)"));
    }
}
