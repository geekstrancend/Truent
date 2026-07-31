//! Report formatter with ANSI colors, JSON, and SARIF output.
//!
//! Formats findings for terminal (with colors), JSON (NDJSON), and SARIF 2.1.0.

use truent_core::{Finding, Severity};
use serde_json::{json, Value};
use std::fmt::Write;

/// ANSI color codes for terminal output
const ANSI_RESET: &str = "\x1b[0m";
const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_BOLD_RED: &str = "\x1b[1;31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_BOLD_YELLOW: &str = "\x1b[1;33m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_GREEN: &str = "\x1b[32m";

/// Format findings for terminal output with ANSI colors
pub fn format_terminal(findings: &[Finding], use_color: bool) -> String {
    let mut output = String::new();

    if findings.is_empty() {
        writeln!(
            &mut output,
            "{}✓ No findings - all checks passed{}",
            if use_color { ANSI_GREEN } else { "" },
            if use_color { ANSI_RESET } else { "" }
        )
        .unwrap();
        return output;
    }

    // Header
    writeln!(
        &mut output,
        "\n{}╔════════════════════════════════════════════════════════════════╗{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();
    writeln!(
        &mut output,
        "{}║{}  Truent Security Analysis Results{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" },
        if use_color { ANSI_BOLD } else { "" }
    )
    .unwrap();
    writeln!(
        &mut output,
        "{}║{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();
    writeln!(
        &mut output,
        "{}╚════════════════════════════════════════════════════════════════╝{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    // Findings grouped by severity
    let critical: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .collect();
    let high: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .collect();
    let medium: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .collect();
    let low: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .collect();
    let info: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .collect();

    // Print findings by severity
    if !critical.is_empty() {
        writeln!(
            &mut output,
            "\n{}",
            format_severity_section("CRITICAL", &critical, use_color)
        )
        .unwrap();
    }
    if !high.is_empty() {
        writeln!(
            &mut output,
            "\n{}",
            format_severity_section("HIGH", &high, use_color)
        )
        .unwrap();
    }
    if !medium.is_empty() {
        writeln!(
            &mut output,
            "\n{}",
            format_severity_section("MEDIUM", &medium, use_color)
        )
        .unwrap();
    }
    if !low.is_empty() {
        writeln!(
            &mut output,
            "\n{}",
            format_severity_section("LOW", &low, use_color)
        )
        .unwrap();
    }
    if !info.is_empty() {
        writeln!(
            &mut output,
            "\n{}",
            format_severity_section("INFO", &info, use_color)
        )
        .unwrap();
    }

    // Summary table
    writeln!(
        &mut output,
        "\n{}",
        format_summary_table(findings, use_color)
    )
    .unwrap();

    output
}

fn format_severity_section(severity: &str, findings: &[&Finding], use_color: bool) -> String {
    let mut output = String::new();

    let color = if use_color {
        match severity {
            "CRITICAL" => ANSI_BOLD_RED,
            "HIGH" => ANSI_BOLD_YELLOW,
            "MEDIUM" => ANSI_YELLOW,
            "LOW" => ANSI_BLUE,
            "INFO" => ANSI_CYAN,
            _ => "",
        }
    } else {
        ""
    };

    writeln!(
        &mut output,
        "{}[{}]{} ({})",
        if use_color { ANSI_BOLD } else { "" },
        severity,
        if use_color { ANSI_RESET } else { "" },
        findings.len()
    )
    .unwrap();

    for (idx, finding) in findings.iter().enumerate() {
        writeln!(
            &mut output,
            "\n  {}{}. {}{}{}:{}",
            if use_color { ANSI_BOLD } else { "" },
            idx + 1,
            if use_color { color } else { "" },
            finding.invariant_id,
            if use_color { ANSI_RESET } else { "" },
            if use_color { ANSI_RESET } else { "" }
        )
        .unwrap();
        writeln!(
            &mut output,
            "     Location: {}:{}:{}",
            finding.file, finding.line, finding.col
        )
        .unwrap();
        writeln!(&mut output, "     Message:  {}", finding.message).unwrap();

        if !finding.snippet.is_empty() {
            writeln!(&mut output, "     Code:     {}", finding.snippet).unwrap();
        }
    }

    output
}

fn format_summary_table(findings: &[Finding], use_color: bool) -> String {
    let critical = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let high = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    let medium = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .count();
    let low = findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .count();
    let info = findings
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .count();

    let mut output = String::new();
    writeln!(
        &mut output,
        "{}┌─────────────┬────────┐{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();
    writeln!(
        &mut output,
        "{}│ Severity    │ Count  │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();
    writeln!(
        &mut output,
        "{}├─────────────┼────────┤{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}│ {}CRITICAL{} │   {}   │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_BOLD_RED } else { "" },
        if use_color {
            format!("{}{}", ANSI_RESET, ANSI_BOLD)
        } else {
            String::new()
        },
        critical,
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}│ {}HIGH{}     │   {}   │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_BOLD_YELLOW } else { "" },
        if use_color {
            format!("{}{}", ANSI_RESET, ANSI_BOLD)
        } else {
            String::new()
        },
        high,
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}│ {}MEDIUM{}   │   {}   │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_YELLOW } else { "" },
        if use_color {
            format!("{}{}", ANSI_RESET, ANSI_BOLD)
        } else {
            String::new()
        },
        medium,
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}│ {}LOW{}      │   {}   │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_BLUE } else { "" },
        if use_color {
            format!("{}{}", ANSI_RESET, ANSI_BOLD)
        } else {
            String::new()
        },
        low,
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}│ {}INFO{}     │   {}   │{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_CYAN } else { "" },
        if use_color {
            format!("{}{}", ANSI_RESET, ANSI_BOLD)
        } else {
            String::new()
        },
        info,
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    writeln!(
        &mut output,
        "{}└─────────────┴────────┘{}",
        if use_color { ANSI_BOLD } else { "" },
        if use_color { ANSI_RESET } else { "" }
    )
    .unwrap();

    output
}

/// Format findings as NDJSON (one Finding per line)
pub fn format_ndjson(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| serde_json::to_string(f).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format findings as SARIF 2.1.0 (GitHub Code Scanning compatible)
pub fn format_sarif(findings: &[Finding], tool_version: &str) -> Value {
    let rules: Vec<Value> = findings
        .iter()
        .map(|f| {
            json!({
                "id": f.invariant_id,
                "shortDescription": {
                    "text": f.message.lines().next().unwrap_or(&f.message)
                },
                "fullDescription": {
                    "text": f.message
                },
                "help": {
                    "text": format!("Invariant: {}", f.invariant_id)
                },
                "defaultConfiguration": {
                    "level": severity_to_sarif_level(f.severity)
                },
                "name": f.invariant_id
            })
        })
        .collect();

    let results: Vec<Value> = findings
        .iter()
        .map(|f| {
            json!({
                "ruleId": f.invariant_id,
                "ruleIndex": 0,
                "level": severity_to_sarif_level(f.severity),
                "message": {
                    "text": f.message
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": f.file
                            },
                            "region": {
                                "startLine": f.line,
                                "startColumn": f.col + 1
                            }
                        }
                    }
                ]
            })
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "Truent",
                        "version": tool_version,
                        "informationUri": "https://github.com/geekstrancend/Truent",
                        "rules": rules
                    }
                },
                "results": results
            }
        ]
    })
}

fn severity_to_sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "error",
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "note",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_terminal() {
        let findings = vec![Finding::new(
            "test_invariant".to_string(),
            Severity::Critical,
            "contract.sol".to_string(),
            42,
            10,
            "Test vulnerability".to_string(),
            "code line".to_string(),
        )];

        let output = format_terminal(&findings, true);
        assert!(output.contains("CRITICAL"));
        assert!(output.contains("test_invariant"));
    }

    #[test]
    fn test_format_sarif() {
        let findings = vec![Finding::new(
            "test".to_string(),
            Severity::High,
            "file.sol".to_string(),
            1,
            0,
            "msg".to_string(),
            "code".to_string(),
        )];

        let sarif = format_sarif(&findings, "0.3.0");
        assert_eq!(sarif["version"], "2.1.0");
        assert!(sarif["runs"][0]["results"].is_array());
    }
}
