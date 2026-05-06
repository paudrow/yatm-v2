use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use anyhow::{Context, Result};
use askama::Template;
use octocrab::models::issues::Issue;
use octocrab::models::IssueState;

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

pub fn generate_report(
    issues: &[&Issue],
    open_issues: &[&Issue],
    closed_completed: &[&Issue],
    closed_wont_fix: &[&Issue],
    closed_duplicate: &[&Issue],
    permutation_keys_values: &BTreeMap<String, BTreeSet<String>>,
    report_path: &PathBuf,
) -> Result<()> {
    let total_issues = issues.len();
    let open_count = open_issues.len();
    let open_pct = format!("{:.2}", (open_issues.len() as f64 / issues.len() as f64) * 100.0);
    let completed_count = closed_completed.len();
    let completed_pct = format!("{:.2}", (closed_completed.len() as f64 / issues.len() as f64) * 100.0);
    let wont_fix_count = closed_wont_fix.len();
    let wont_fix_pct = format!("{:.2}", (closed_wont_fix.len() as f64 / issues.len() as f64) * 100.0);
    let duplicate_count = closed_duplicate.len();
    let duplicate_pct = format!("{:.2}", (closed_duplicate.len() as f64 / issues.len() as f64) * 100.0);

    let mut max_term_issues = 0;
    if !permutation_keys_values.is_empty() {
        for (key, values) in permutation_keys_values {
            for value in values {
                let label_str = crate::helpers::sanitize_label(format!(
                    "{}: {}",
                    key, value
                ));
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
                let label_str = crate::helpers::sanitize_label(format!(
                    "{}: {}",
                    key, value
                ));

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
                        let is_wont_fix = i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned);
                        let is_duplicate = i.labels.iter().any(|l| l.name.to_lowercase() == "duplicate");
                        !is_wont_fix && !is_duplicate
                    })
                    .collect::<Vec<_>>();

                let term_wont_fix = term_closed
                    .iter()
                    .filter(|i| i.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned))
                    .collect::<Vec<_>>();

                let term_duplicate = term_closed
                    .iter()
                    .filter(|i| {
                        i.labels
                            .iter()
                            .any(|l| l.name.to_lowercase() == "duplicate")
                    })
                    .collect::<Vec<_>>();

                let closed_pct = (term_closed.len() as f64
                    / term_issues.len() as f64)
                    * 100.0;
                let open_pct =
                    (term_open.len() as f64 / term_issues.len() as f64) * 100.0;
                let completed_pct = (term_completed.len() as f64
                    / term_issues.len() as f64)
                    * 100.0;
                let wont_fix_pct = (term_wont_fix.len() as f64
                    / term_issues.len() as f64)
                    * 100.0;
                let duplicate_pct = (term_duplicate.len() as f64
                    / term_issues.len() as f64)
                    * 100.0;

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

    let mut pairwise_matrices = vec![];
    if !permutation_keys_values.is_empty() {
        let keys: Vec<String> =
            permutation_keys_values.keys().cloned().collect();
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
                        let label_a = crate::helpers::sanitize_label(format!(
                            "{}: {}",
                            key_a, val_a
                        ));
                        issues
                            .iter()
                            .filter(|i| {
                                i.labels.iter().any(|l| l.name == label_a)
                            })
                            .count()
                    })
                    .sum();

                // Calculate column total issues sum and individual column widths
                let sum_col_issues: usize = values_b
                    .iter()
                    .map(|val_b| {
                        let label_b = crate::helpers::sanitize_label(format!(
                            "{}: {}",
                            key_b, val_b
                        ));
                        issues
                            .iter()
                            .filter(|i| {
                                i.labels.iter().any(|l| l.name == label_b)
                            })
                            .count()
                    })
                    .sum();

                let col_widths: Vec<f64> = values_b
                    .iter()
                    .map(|val_b| {
                        let label_b = crate::helpers::sanitize_label(format!(
                            "{}: {}",
                            key_b, val_b
                        ));
                        let col_issues_cnt = issues
                            .iter()
                            .filter(|i| {
                                i.labels.iter().any(|l| l.name == label_b)
                            })
                            .count();
                        if sum_col_issues > 0 {
                            (col_issues_cnt as f64 / sum_col_issues as f64)
                                * 100.0
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
                    let label_a = crate::helpers::sanitize_label(format!(
                        "{}: {}",
                        key_a, val_a
                    ));
                    let row_issues_cnt = issues
                        .iter()
                        .filter(|i| i.labels.iter().any(|l| l.name == label_a))
                        .count();
                    let row_height = if sum_row_issues > 0 {
                        50.0 + ((row_issues_cnt as f64 / sum_row_issues as f64)
                            * 150.0)
                    } else {
                        50.0
                    };

                    let mut cells = vec![];
                    for val_b in &values_b {
                        let label_b = crate::helpers::sanitize_label(format!(
                            "{}: {}",
                            key_b, val_b
                        ));

                        let cell_issues = issues
                            .iter()
                            .filter(|issue| {
                                issue.labels.iter().any(|l| l.name == label_a)
                                    && issue
                                        .labels
                                        .iter()
                                        .any(|l| l.name == label_b)
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

                        let cell_completed = cell_closed.iter().filter(|issue| {
                            issue.state_reason != Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                            && !issue.labels.iter().any(|l| l.name.to_lowercase() == "duplicate")
                        }).count();

                        let cell_wont_fix = cell_closed.iter().filter(|issue| {
                            issue.state_reason == Some(octocrab::models::issues::IssueStateReason::NotPlanned)
                        }).count();

                        let cell_duplicate = cell_closed
                            .iter()
                            .filter(|issue| {
                                issue.labels.iter().any(|l| {
                                    l.name.to_lowercase() == "duplicate"
                                })
                            })
                            .count();

                        let cell_total = cell_completed
                            + cell_open.len()
                            + cell_wont_fix
                            + cell_duplicate;

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
                        let cell_lightness =
                            100.0 - (cell_completed_ratio * 12.0);

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
        total_issues,
        open_count,
        open_pct,
        completed_count,
        completed_pct,
        wont_fix_count,
        wont_fix_pct,
        duplicate_count,
        duplicate_pct,
        permutations,
        pairwise_matrices,
    };

    let report_str = template.render().context("Failed to render the metrics report template")?;
    std::fs::write(report_path, report_str)?;
    Ok(())
}
