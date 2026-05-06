use anyhow::{Context, Result};
use askama::Template;
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "metrics_report.md", escape = "none")]
struct MetricsReportTemplate {
    total_issues: usize,
    open_count: usize,
    open_pct: String,
    completed_count: usize,
    completed_pct: String,
    wont_fix_count: usize,
    wont_fix_pct: String,
    duplicate_count: usize,
    duplicate_pct: String,
    permutations: Vec<PermutationKeyBreakdown>,
    pairwise_matrices: Vec<PairwiseMatrix>,
}

#[derive(Template)]
#[template(path = "console_metrics.txt", escape = "none")]
struct ConsoleMetricsTemplate<'a> {
    metrics: &'a OverallMetrics,
    permutations: &'a [PermutationKeyBreakdown],
}

struct PermutationKeyBreakdown {
    key: String,
    values: Vec<PermutationValueBreakdown>,
}

struct PermutationValueBreakdown {
    value: String,
    closed_count: usize,
    total_count: usize,
    closed_pct: String,
    open_count: usize,
    open_pct: String,
    completed_count: usize,
    completed_pct: String,
    wont_fix_count: usize,
    wont_fix_pct: String,
    duplicate_count: usize,
    duplicate_pct: String,
    bar_width_pct: String,
    has_completed: bool,
    has_open: bool,
    has_wont_fix: bool,
    has_duplicate: bool,
}

struct PairwiseMatrix {
    key_a: String,
    key_b: String,
    headers: Vec<MatrixHeader>,
    rows: Vec<MatrixRow>,
}

struct MatrixHeader {
    value: String,
    width_pct: String,
}

struct MatrixRow {
    val_a: String,
    height: String,
    cells: Vec<MatrixCell>,
}

struct MatrixCell {
    total_cases: usize,
    completed: usize,
    total_valid: usize,
    completed_pct: String,
    open: usize,
    open_pct: String,
    wont_fix: usize,
    wont_fix_pct: String,
    duplicate: usize,
    duplicate_pct: String,
    has_mini_bar: bool,
    has_completed: bool,
    has_open: bool,
    has_wont_fix: bool,
    has_duplicate: bool,
    lightness: String,
}

pub struct OverallMetrics {
    pub total: usize,
    pub open: usize,
    pub open_pct: f64,
    pub completed: usize,
    pub completed_pct: f64,
    pub wont_fix: usize,
    pub wont_fix_pct: f64,
    pub duplicate: usize,
    pub duplicate_pct: f64,
}

impl OverallMetrics {
    pub fn open_pct_str(&self) -> String {
        format!("{:.2}", self.open_pct)
    }
    pub fn completed_pct_str(&self) -> String {
        format!("{:.2}", self.completed_pct)
    }
    pub fn wont_fix_pct_str(&self) -> String {
        format!("{:.2}", self.wont_fix_pct)
    }
    pub fn duplicate_pct_str(&self) -> String {
        format!("{:.2}", self.duplicate_pct)
    }
}

pub fn calculate_overall_metrics(
    issues: &[&Issue],
    open_issues: &[&Issue],
    closed_completed: &[&Issue],
    closed_wont_fix: &[&Issue],
    closed_duplicate: &[&Issue],
) -> OverallMetrics {
    let total = issues.len();
    let open = open_issues.len();
    let open_pct = if total > 0 {
        (open as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let completed = closed_completed.len();
    let completed_pct = if total > 0 {
        (completed as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let wont_fix = closed_wont_fix.len();
    let wont_fix_pct = if total > 0 {
        (wont_fix as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    let duplicate = closed_duplicate.len();
    let duplicate_pct = if total > 0 {
        (duplicate as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    OverallMetrics {
        total,
        open,
        open_pct,
        completed,
        completed_pct,
        wont_fix,
        wont_fix_pct,
        duplicate,
        duplicate_pct,
    }
}

fn calculate_permutations_breakdown(
    issues: &[&Issue],
    permutation_keys_values: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<PermutationKeyBreakdown> {
    let mut max_term_issues = 0;
    if !permutation_keys_values.is_empty() {
        for (key, values) in permutation_keys_values {
            for value in values {
                let label_str = crate::helpers::sanitize_label(format!("{}: {}", key, value));
                let cnt = issues
                    .iter()
                    .filter(|i| i.labels.iter().any(|l| l.name == label_str))
                    .count();
                if cnt > max_term_issues {
                    max_term_issues = cnt;
                }
            }
        }
    }

    let mut permutations = vec![];
    if !permutation_keys_values.is_empty() {
        for (key, values) in permutation_keys_values {
            let mut value_breakdowns = vec![];
            for value in values {
                let label_str = crate::helpers::sanitize_label(format!("{}: {}", key, value));

                let term_issues = issues
                    .iter()
                    .filter(|i| i.labels.iter().any(|l| l.name == label_str))
                    .collect::<Vec<_>>();

                if term_issues.is_empty() {
                    continue;
                }

                let term_closed = term_issues
                    .iter()
                    .filter(|i| i.state == IssueState::Closed)
                    .collect::<Vec<_>>();

                let term_open = term_issues
                    .iter()
                    .filter(|i| i.state == IssueState::Open)
                    .collect::<Vec<_>>();

                let term_completed = term_closed
                    .iter()
                    .filter(|i| {
                        let is_wont_fix = i.state_reason
                            == Some(octocrab::models::issues::IssueStateReason::NotPlanned);
                        let is_duplicate = i
                            .labels
                            .iter()
                            .any(|l| l.name.to_lowercase() == "duplicate");
                        !is_wont_fix && !is_duplicate
                    })
                    .collect::<Vec<_>>();

                let term_wont_fix = term_closed
                    .iter()
                    .filter(|i| {
                        i.state_reason
                            == Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                    })
                    .collect::<Vec<_>>();

                let term_duplicate = term_closed
                    .iter()
                    .filter(|i| {
                        i.labels
                            .iter()
                            .any(|l| l.name.to_lowercase() == "duplicate")
                    })
                    .collect::<Vec<_>>();

                let closed_pct = (term_closed.len() as f64 / term_issues.len() as f64) * 100.0;
                let open_pct = (term_open.len() as f64 / term_issues.len() as f64) * 100.0;
                let completed_pct =
                    (term_completed.len() as f64 / term_issues.len() as f64) * 100.0;
                let wont_fix_pct = (term_wont_fix.len() as f64 / term_issues.len() as f64) * 100.0;
                let duplicate_pct =
                    (term_duplicate.len() as f64 / term_issues.len() as f64) * 100.0;

                let bar_width_pct = if max_term_issues > 0 {
                    (term_issues.len() as f64 / max_term_issues as f64) * 100.0
                } else {
                    0.0
                };

                value_breakdowns.push(PermutationValueBreakdown {
                    value: value.clone(),
                    closed_count: term_closed.len(),
                    total_count: term_issues.len(),
                    closed_pct: format!("{:.2}", closed_pct),
                    open_count: term_open.len(),
                    open_pct: format!("{:.2}", open_pct),
                    completed_count: term_completed.len(),
                    completed_pct: format!("{:.2}", completed_pct),
                    wont_fix_count: term_wont_fix.len(),
                    wont_fix_pct: format!("{:.2}", wont_fix_pct),
                    duplicate_count: term_duplicate.len(),
                    duplicate_pct: format!("{:.2}", duplicate_pct),
                    bar_width_pct: format!("{:.2}", bar_width_pct),
                    has_completed: completed_pct > 0.0,
                    has_open: open_pct > 0.0,
                    has_wont_fix: wont_fix_pct > 0.0,
                    has_duplicate: duplicate_pct > 0.0,
                });
            }
            if !value_breakdowns.is_empty() {
                permutations.push(PermutationKeyBreakdown {
                    key: key.clone(),
                    values: value_breakdowns,
                });
            }
        }
    }
    permutations
}

pub fn print_overall_metrics(
    metrics: &OverallMetrics,
    issues: &[&Issue],
    permutation_keys_values: &BTreeMap<String, BTreeSet<String>>,
) {
    let permutations = calculate_permutations_breakdown(issues, permutation_keys_values);
    let template = ConsoleMetricsTemplate {
        metrics,
        permutations: &permutations,
    };
    match template.render() {
        Ok(rendered) => print!("{}", rendered),
        Err(e) => eprintln!("Failed to render console metrics template: {:?}", e),
    }
}

pub fn generate_report(
    issues: &[&Issue],
    metrics: &OverallMetrics,
    permutation_keys_values: &BTreeMap<String, BTreeSet<String>>,
    report_path: &PathBuf,
) -> Result<()> {
    let permutations = calculate_permutations_breakdown(issues, permutation_keys_values);

    let mut pairwise_matrices = vec![];
    if !permutation_keys_values.is_empty() {
        let keys: Vec<String> = permutation_keys_values.keys().cloned().collect();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                let key_a = &keys[i];
                let key_b = &keys[j];

                let values_a: Vec<String> = permutation_keys_values
                    .get(key_a)
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect();
                let values_b: Vec<String> = permutation_keys_values
                    .get(key_b)
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect();

                // Calculate row total issues sum
                let sum_row_issues: usize = values_a
                    .iter()
                    .map(|val_a| {
                        let label_a =
                            crate::helpers::sanitize_label(format!("{}: {}", key_a, val_a));
                        issues
                            .iter()
                            .filter(|i| i.labels.iter().any(|l| l.name == label_a))
                            .count()
                    })
                    .sum();

                // Calculate column total issues sum and individual column widths
                let sum_col_issues: usize = values_b
                    .iter()
                    .map(|val_b| {
                        let label_b =
                            crate::helpers::sanitize_label(format!("{}: {}", key_b, val_b));
                        issues
                            .iter()
                            .filter(|i| i.labels.iter().any(|l| l.name == label_b))
                            .count()
                    })
                    .sum();

                let col_widths: Vec<f64> = values_b
                    .iter()
                    .map(|val_b| {
                        let label_b =
                            crate::helpers::sanitize_label(format!("{}: {}", key_b, val_b));
                        let col_issues_cnt = issues
                            .iter()
                            .filter(|i| i.labels.iter().any(|l| l.name == label_b))
                            .count();
                        if sum_col_issues > 0 {
                            (col_issues_cnt as f64 / sum_col_issues as f64) * 100.0
                        } else {
                            100.0 / (values_b.len() as f64)
                        }
                    })
                    .collect();

                let headers = values_b
                    .iter()
                    .zip(&col_widths)
                    .map(|(val_b, col_w)| MatrixHeader {
                        value: val_b.clone(),
                        width_pct: format!("{:.2}", col_w),
                    })
                    .collect();

                let mut rows = vec![];
                for val_a in &values_a {
                    let label_a = crate::helpers::sanitize_label(format!("{}: {}", key_a, val_a));
                    let row_issues_cnt = issues
                        .iter()
                        .filter(|i| i.labels.iter().any(|l| l.name == label_a))
                        .count();
                    let row_height = if sum_row_issues > 0 {
                        50.0 + ((row_issues_cnt as f64 / sum_row_issues as f64) * 150.0)
                    } else {
                        50.0
                    };

                    let mut cells = vec![];
                    for val_b in &values_b {
                        let label_b =
                            crate::helpers::sanitize_label(format!("{}: {}", key_b, val_b));

                        let cell_issues = issues
                            .iter()
                            .filter(|issue| {
                                issue.labels.iter().any(|l| l.name == label_a)
                                    && issue.labels.iter().any(|l| l.name == label_b)
                            })
                            .collect::<Vec<_>>();

                        let cell_closed = cell_issues
                            .iter()
                            .filter(|issue| issue.state == IssueState::Closed)
                            .collect::<Vec<_>>();
                        let cell_open = cell_issues
                            .iter()
                            .filter(|issue| issue.state == IssueState::Open)
                            .collect::<Vec<_>>();

                        let cell_completed = cell_closed
                            .iter()
                            .filter(|issue| {
                                issue.state_reason
                                    != Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                                    && !issue
                                        .labels
                                        .iter()
                                        .any(|l| l.name.to_lowercase() == "duplicate")
                            })
                            .count();

                        let cell_wont_fix = cell_closed
                            .iter()
                            .filter(|issue| {
                                issue.state_reason
                                    == Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                            })
                            .count();

                        let cell_duplicate = cell_closed
                            .iter()
                            .filter(|issue| {
                                issue
                                    .labels
                                    .iter()
                                    .any(|l| l.name.to_lowercase() == "duplicate")
                            })
                            .count();

                        let cell_total =
                            cell_completed + cell_open.len() + cell_wont_fix + cell_duplicate;

                        let cell_completed_pct = if cell_total > 0 {
                            (cell_completed as f64 / cell_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        let cell_open_pct = if cell_total > 0 {
                            (cell_open.len() as f64 / cell_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        let cell_wont_fix_pct = if cell_total > 0 {
                            (cell_wont_fix as f64 / cell_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        let cell_duplicate_pct = if cell_total > 0 {
                            (cell_duplicate as f64 / cell_total as f64) * 100.0
                        } else {
                            0.0
                        };

                        let cell_completed_ratio = if cell_total > 0 {
                            cell_completed as f64 / cell_total as f64
                        } else {
                            0.0
                        };
                        let cell_lightness = 100.0 - (cell_completed_ratio * 12.0);

                        cells.push(MatrixCell {
                            total_cases: cell_issues.len(),
                            completed: cell_completed,
                            total_valid: cell_total,
                            completed_pct: format!("{:.1}", cell_completed_pct),
                            open: cell_open.len(),
                            open_pct: format!("{:.1}", cell_open_pct),
                            wont_fix: cell_wont_fix,
                            wont_fix_pct: format!("{:.1}", cell_wont_fix_pct),
                            duplicate: cell_duplicate,
                            duplicate_pct: format!("{:.1}", cell_duplicate_pct),
                            has_mini_bar: cell_total > 0,
                            has_completed: cell_completed_pct > 0.0,
                            has_open: cell_open_pct > 0.0,
                            has_wont_fix: cell_wont_fix_pct > 0.0,
                            has_duplicate: cell_duplicate_pct > 0.0,
                            lightness: format!("{:.1}", cell_lightness),
                        });
                    }
                    rows.push(MatrixRow {
                        val_a: val_a.clone(),
                        height: format!("{:.0}", row_height),
                        cells,
                    });
                }

                pairwise_matrices.push(PairwiseMatrix {
                    key_a: key_a.clone(),
                    key_b: key_b.clone(),
                    headers,
                    rows,
                });
            }
        }
    }

    let template = MetricsReportTemplate {
        total_issues: metrics.total,
        open_count: metrics.open,
        open_pct: format!("{:.2}", metrics.open_pct),
        completed_count: metrics.completed,
        completed_pct: format!("{:.2}", metrics.completed_pct),
        wont_fix_count: metrics.wont_fix,
        wont_fix_pct: format!("{:.2}", metrics.wont_fix_pct),
        duplicate_count: metrics.duplicate,
        duplicate_pct: format!("{:.2}", metrics.duplicate_pct),
        permutations,
        pairwise_matrices,
    };

    let report_str = template
        .render()
        .context("Failed to render the metrics report template")?;
    std::fs::write(report_path, report_str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::models::issues::Issue;

    #[test]
    fn test_generate_report_tables_no_cellular_newlines() {
        let issues: Vec<Issue> = vec![];
        let mut permutation_keys_values = BTreeMap::new();
        let mut values = BTreeSet::new();
        values.insert("amd64".to_string());
        permutation_keys_values.insert("chip".to_string(), values);

        let mut os_values = BTreeSet::new();
        os_values.insert("ubuntu".to_string());
        permutation_keys_values.insert("os".to_string(), os_values);

        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let open_issues: Vec<&Issue> = vec![];
        let closed_completed: Vec<&Issue> = vec![];
        let closed_wont_fix: Vec<&Issue> = vec![];
        let closed_duplicate: Vec<&Issue> = vec![];

        let metrics = calculate_overall_metrics(
            &issue_refs,
            &open_issues,
            &closed_completed,
            &closed_wont_fix,
            &closed_duplicate,
        );

        let permutations = vec![];
        let mut pairwise_matrices = vec![];

        let headers = vec![MatrixHeader {
            value: "ubuntu".to_string(),
            width_pct: "100".to_string(),
        }];
        let cells = vec![MatrixCell {
            total_cases: 0,
            completed: 0,
            total_valid: 0,
            completed_pct: "0.0".to_string(),
            open: 0,
            open_pct: "0.0".to_string(),
            wont_fix: 0,
            wont_fix_pct: "0.0".to_string(),
            duplicate: 0,
            duplicate_pct: "0.0".to_string(),
            has_mini_bar: false,
            has_completed: false,
            has_open: false,
            has_wont_fix: false,
            has_duplicate: false,
            lightness: "100.0".to_string(),
        }];
        let rows = vec![MatrixRow {
            val_a: "amd64".to_string(),
            height: "76".to_string(),
            cells,
        }];
        pairwise_matrices.push(PairwiseMatrix {
            key_a: "chip".to_string(),
            key_b: "os".to_string(),
            headers,
            rows,
        });

        let template = MetricsReportTemplate {
            total_issues: metrics.total,
            open_count: metrics.open,
            open_pct: format!("{:.2}", metrics.open_pct),
            completed_count: metrics.completed,
            completed_pct: format!("{:.2}", metrics.completed_pct),
            wont_fix_count: metrics.wont_fix,
            wont_fix_pct: format!("{:.2}", metrics.wont_fix_pct),
            duplicate_count: metrics.duplicate,
            duplicate_pct: format!("{:.2}", metrics.duplicate_pct),
            permutations,
            pairwise_matrices,
        };

        let rendered = template
            .render()
            .expect("Failed to render template in test");

        assert!(rendered.contains("<table"));
        assert!(rendered.contains("</table>"));

        let mut rest = rendered.as_str();
        while let Some(start_idx) = rest.find("<td") {
            let end_idx = rest[start_idx..]
                .find("</td>")
                .expect("Malformed <td> in output");
            let td_content = &rest[start_idx..start_idx + end_idx];
            assert!(
                !td_content.contains('\n'),
                "Found newline inside cell content: {:?}",
                td_content
            );
            rest = &rest[start_idx + end_idx + 5..];
        }
    }

    #[test]
    fn test_calculate_overall_metrics() {
        let issues: Vec<Issue> = vec![];
        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let open_issues: Vec<&Issue> = vec![];
        let closed_completed: Vec<&Issue> = vec![];
        let closed_wont_fix: Vec<&Issue> = vec![];
        let closed_duplicate: Vec<&Issue> = vec![];

        let metrics = calculate_overall_metrics(
            &issue_refs,
            &open_issues,
            &closed_completed,
            &closed_wont_fix,
            &closed_duplicate,
        );

        assert_eq!(metrics.total, 0);
        assert_eq!(metrics.open, 0);
        assert_eq!(metrics.open_pct, 0.0);
        assert_eq!(metrics.completed, 0);
        assert_eq!(metrics.completed_pct, 0.0);
        assert_eq!(metrics.wont_fix, 0);
        assert_eq!(metrics.wont_fix_pct, 0.0);
        assert_eq!(metrics.duplicate, 0);
        assert_eq!(metrics.duplicate_pct, 0.0);
    }

    #[test]
    fn test_overall_metrics_getters() {
        let metrics = OverallMetrics {
            total: 10,
            open: 4,
            open_pct: 40.0,
            completed: 3,
            completed_pct: 30.0,
            wont_fix: 2,
            wont_fix_pct: 20.0,
            duplicate: 1,
            duplicate_pct: 10.0,
        };

        assert_eq!(metrics.open_pct_str(), "40.00");
        assert_eq!(metrics.completed_pct_str(), "30.00");
        assert_eq!(metrics.wont_fix_pct_str(), "20.00");
        assert_eq!(metrics.duplicate_pct_str(), "10.00");
    }

    #[test]
    fn test_calculate_permutations_breakdown() {
        let issues: Vec<Issue> = vec![];
        let mut permutation_keys_values = BTreeMap::new();
        let mut values = BTreeSet::new();
        values.insert("amd64".to_string());
        permutation_keys_values.insert("chip".to_string(), values);

        let breakdowns = calculate_permutations_breakdown(
            &issues.iter().collect::<Vec<_>>(),
            &permutation_keys_values,
        );
        assert_eq!(breakdowns.len(), 0);
    }

    #[test]
    fn test_print_overall_metrics_rendering() {
        let metrics = OverallMetrics {
            total: 0,
            open: 0,
            open_pct: 0.0,
            completed: 0,
            completed_pct: 0.0,
            wont_fix: 0,
            wont_fix_pct: 0.0,
            duplicate: 0,
            duplicate_pct: 0.0,
        };
        let issues: Vec<Issue> = vec![];
        let permutation_keys_values = BTreeMap::new();

        print_overall_metrics(
            &metrics,
            &issues.iter().collect::<Vec<_>>(),
            &permutation_keys_values,
        );
    }

    #[test]
    fn test_generate_report_file_writing() {
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let report_path = tmp_dir.path().join("metrics_report.md");

        let issues: Vec<Issue> = vec![];
        let issue_refs: Vec<&Issue> = issues.iter().collect();
        let open_issues: Vec<&Issue> = vec![];
        let closed_completed: Vec<&Issue> = vec![];
        let closed_wont_fix: Vec<&Issue> = vec![];
        let closed_duplicate: Vec<&Issue> = vec![];

        let metrics = calculate_overall_metrics(
            &issue_refs,
            &open_issues,
            &closed_completed,
            &closed_wont_fix,
            &closed_duplicate,
        );

        let permutation_keys_values = BTreeMap::new();

        let result = generate_report(
            &issue_refs,
            &metrics,
            &permutation_keys_values,
            &report_path,
        );
        assert!(result.is_ok());
        assert!(report_path.exists());

        let report_content =
            std::fs::read_to_string(report_path).expect("Failed to read written report in test");
        assert!(report_content.contains("# GitHub Test Case Metrics Report"));
    }
}
