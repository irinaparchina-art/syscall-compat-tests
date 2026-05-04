use std::collections::HashMap;
use std::io::Write;

use chrono::Local;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::runner::{TestResult, TestStatus};

/// Summary statistics for a test run.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub unimplemented: usize,
    pub errors: usize,
    pub pass_rate: f64,
    pub timestamp: String,
    pub target_os: String,
}

impl RunSummary {
    pub fn from_results(results: &[TestResult], target_os: &str) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let failed = results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Fail { .. }))
            .count();
        let unimplemented = results
            .iter()
            .filter(|r| r.status == TestStatus::Unimplemented)
            .count();
        let errors = results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Error(_)))
            .count();

        Self {
            total,
            passed,
            failed,
            unimplemented,
            errors,
            pass_rate: if total > 0 {
                passed as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            target_os: target_os.to_string(),
        }
    }
}

/// Generates a human-readable report to stdout.
pub fn print_console_report(results: &[TestResult], target_os: &str) {
    let summary = RunSummary::from_results(results, target_os);

    println!("\n{}", "═".repeat(60));
    println!(
        "  {} — {}",
        "syscall-compat-tests".bold().cyan(),
        target_os.bold()
    );
    println!("  {}", summary.timestamp.dimmed());
    println!("{}", "═".repeat(60));

    // Results by category
    let mut by_category: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for r in results {
        by_category
            .entry(r.category.to_string())
            .or_default()
            .push(r);
    }

    let mut categories: Vec<_> = by_category.keys().collect();
    categories.sort();

    for cat in categories {
        let cat_results = &by_category[cat];
        println!("\n  {}", format!("[{}]", cat).bold().yellow());
        for r in cat_results {
            let status_str = match &r.status {
                TestStatus::Pass => "PASS".green().to_string(),
                TestStatus::Fail { actual_ret, expected_ret, .. } => {
                    format!("FAIL (ret: expected={}, got={})", expected_ret, actual_ret).red().to_string()
                }
                TestStatus::Unimplemented => "ENOSYS".yellow().to_string(),
                TestStatus::Error(e) => format!("ERROR: {}", e).red().to_string(),
            };
            println!(
                "    {:40} {} {:>6}μs",
                r.name.dimmed(),
                status_str,
                r.duration_us
            );
        }
    }

    println!("\n{}", "─".repeat(60));
    println!(
        "  Total: {}  Pass: {}  Fail: {}  Unimplemented: {}  Error: {}",
        summary.total.to_string().bold(),
        summary.passed.to_string().green(),
        summary.failed.to_string().red(),
        summary.unimplemented.to_string().yellow(),
        summary.errors.to_string().red(),
    );
    println!(
        "  Compatibility: {:.1}%",
        summary.pass_rate.to_string().bold().cyan()
    );
    println!("{}\n", "═".repeat(60));
}

/// Generates a JSON report file.
pub fn write_json_report(
    results: &[TestResult],
    target_os: &str,
    path: &str,
) -> anyhow::Result<()> {
    #[derive(Serialize)]
    struct JsonReport<'a> {
        summary: RunSummary,
        results: &'a [TestResult],
    }

    let report = JsonReport {
        summary: RunSummary::from_results(results, target_os),
        results,
    };

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(path, json)?;
    println!("JSON report written to: {}", path);
    Ok(())
}

/// Generates a Markdown compatibility matrix report.
pub fn write_markdown_report(
    results: &[TestResult],
    target_os: &str,
    path: &str,
) -> anyhow::Result<()> {
    let summary = RunSummary::from_results(results, target_os);
    let mut buf = Vec::new();

    writeln!(buf, "# syscall-compat-tests Report")?;
    writeln!(buf, "")?;
    writeln!(buf, "**Target OS:** {}  ", target_os)?;
    writeln!(buf, "**Date:** {}  ", summary.timestamp)?;
    writeln!(
        buf,
        "**Compatibility:** {:.1}% ({}/{} passed)  ",
        summary.pass_rate, summary.passed, summary.total
    )?;
    writeln!(buf, "")?;
    writeln!(buf, "## Compatibility Matrix")?;
    writeln!(buf, "")?;
    writeln!(
        buf,
        "| Test | Syscall | Category | {} | Notes |",
        target_os
    )?;
    writeln!(buf, "|------|---------|----------|:---:|-------|")?;

    for r in results {
        let status_icon = match &r.status {
            TestStatus::Pass => "✅",
            TestStatus::Fail { .. } => "❌",
            TestStatus::Unimplemented => "⚠️",
            TestStatus::Error(_) => "💥",
        };
        let notes = match &r.status {
            TestStatus::Fail {
                expected_ret,
                actual_ret,
                expected_errno,
                actual_errno,
            } => format!(
                "ret exp={} got={}, errno exp={:?} got={:?}",
                expected_ret, actual_ret, expected_errno, actual_errno
            ),
            TestStatus::Error(e) => e.clone(),
            _ => String::new(),
        };
        writeln!(
            buf,
            "| {} | `{}` | {} | {} | {} |",
            r.name, r.syscall, r.category, status_icon, notes
        )?;
    }

    writeln!(buf, "")?;
    writeln!(buf, "## Legend")?;
    writeln!(buf, "")?;
    writeln!(buf, "| Icon | Meaning |")?;
    writeln!(buf, "|------|---------|")?;
    writeln!(buf, "| ✅ | Fully compatible |")?;
    writeln!(buf, "| ❌ | Incompatible (wrong return value or errno) |")?;
    writeln!(buf, "| ⚠️ | Not implemented (ENOSYS) |")?;
    writeln!(buf, "| 💥 | Test error |")?;

    std::fs::write(path, String::from_utf8(buf)?)?;
    println!("Markdown report written to: {}", path);
    Ok(())
}
