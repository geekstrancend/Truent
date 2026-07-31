/// Comprehensive Security Report Generator
///
/// Generates multi-format security analysis reports with severity aggregation,
/// remediation guidance, and industry-standard formatting.
use truent_core::{Finding, Severity};
use std::collections::HashMap;

/// Report format enumeration
#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// CSV format
    Csv,
}

/// Severity statistics
#[derive(Debug, Clone)]
pub struct SeverityStats {
    /// Critical severity count
    pub critical: usize,
    /// High severity count
    pub high: usize,
    /// Medium severity count
    pub medium: usize,
    /// Low severity count
    pub low: usize,
    /// Info severity count
    pub info: usize,
}

impl SeverityStats {
    /// Create from findings
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut stats = Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
        };

        for finding in findings {
            match finding.severity {
                Severity::Critical => stats.critical += 1,
                Severity::High => stats.high += 1,
                Severity::Medium => stats.medium += 1,
                Severity::Low => stats.low += 1,
                Severity::Info => stats.info += 1,
            }
        }

        stats
    }

    /// Total findings count
    pub fn total(&self) -> usize {
        self.critical + self.high + self.medium + self.low + self.info
    }

    /// Risk score (0.0-100.0)
    pub fn risk_score(&self) -> f64 {
        let total = self.total() as f64;
        if total == 0.0 {
            return 0.0;
        }

        let weighted = (self.critical as f64 * 100.0)
            + (self.high as f64 * 75.0)
            + (self.medium as f64 * 50.0)
            + (self.low as f64 * 25.0)
            + (self.info as f64 * 10.0);

        (weighted / (total * 100.0)).min(100.0)
    }
}

/// Security analysis report
pub struct SecurityReport {
    /// Report title
    pub title: String,
    /// Analysis timestamp
    pub timestamp: String,
    /// Analyzed files/contracts
    pub analyzed_targets: Vec<String>,
    /// All findings
    pub findings: Vec<Finding>,
    /// Severity statistics
    pub severity_stats: SeverityStats,
    /// Detector chain breakdown
    pub chain_breakdown: HashMap<String, usize>,
    /// Executive summary
    pub executive_summary: String,
}

impl SecurityReport {
    /// Create new security report
    pub fn new(
        title: String,
        analyzed_targets: Vec<String>,
        findings: Vec<Finding>,
        executive_summary: String,
    ) -> Self {
        let severity_stats = SeverityStats::from_findings(&findings);

        let mut chain_breakdown = HashMap::new();
        for finding in &findings {
            let count = chain_breakdown
                .entry(finding.invariant_id.clone())
                .or_insert(0);
            *count += 1;
        }

        Self {
            title,
            timestamp: chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            analyzed_targets,
            findings,
            severity_stats,
            chain_breakdown,
            executive_summary,
        }
    }

    /// Generate report in specified format
    pub fn generate(&self, format: ReportFormat) -> String {
        match format {
            ReportFormat::Markdown => self.generate_markdown(),
            ReportFormat::Json => self.generate_json(),
            ReportFormat::Html => self.generate_html(),
            ReportFormat::Csv => self.generate_csv(),
        }
    }

    /// Generate Markdown report
    fn generate_markdown(&self) -> String {
        let mut report = format!("# {}\n\n", self.title);
        report.push_str("**Generated:** ");
        report.push_str(&self.timestamp);
        report.push_str("\n\n");

        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&self.executive_summary);
        report.push_str("\n\n");

        // Statistics
        report.push_str("## Security Statistics\n\n");
        report.push_str(&format!(
            "- **Total Findings:** {}\n",
            self.severity_stats.total()
        ));
        report.push_str(&format!(
            "- **Critical:** {}\n",
            self.severity_stats.critical
        ));
        report.push_str(&format!("- **High:** {}\n", self.severity_stats.high));
        report.push_str(&format!("- **Medium:** {}\n", self.severity_stats.medium));
        report.push_str(&format!("- **Low:** {}\n", self.severity_stats.low));
        report.push_str(&format!("- **Info:** {}\n", self.severity_stats.info));
        report.push_str(&format!(
            "- **Risk Score:** {:.1}/100.0\n\n",
            self.severity_stats.risk_score()
        ));

        // Analyzed Targets
        report.push_str("## Analyzed Targets\n\n");
        for target in &self.analyzed_targets {
            report.push_str(&format!("- {}\n", target));
        }
        report.push('\n');

        // Findings by Severity
        report.push_str("## Detailed Findings\n\n");

        for severity_level in &[
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            let severity_findings: Vec<_> = self
                .findings
                .iter()
                .filter(|f| f.severity == *severity_level)
                .collect();

            if !severity_findings.is_empty() {
                report.push_str(&format!("### {:?} Severity\n\n", severity_level));

                for finding in severity_findings {
                    report.push_str(&format!("#### {}\n", finding.message));
                    report.push_str(&format!("- **File:** {}\n", finding.file));
                    report.push_str(&format!(
                        "- **Location:** Line {}, Column {}\n",
                        finding.line, finding.col
                    ));
                    report.push_str(&format!("- **Code:** {}\n", finding.snippet));
                    report.push('\n');
                }
            }
        }

        report
    }

    /// Generate JSON report.
    ///
    /// Built via `serde_json` rather than hand-formatted strings so that any
    /// attacker-influenced content (a crafted contract/file name ending up in
    /// `title`, or a finding `message`/`file` containing quotes or control
    /// characters) is always escaped correctly instead of corrupting the JSON
    /// structure.
    fn generate_json(&self) -> String {
        let report = serde_json::json!({
            "title": self.title,
            "timestamp": self.timestamp,
            "statistics": {
                "total_findings": self.severity_stats.total(),
                "critical": self.severity_stats.critical,
                "high": self.severity_stats.high,
                "medium": self.severity_stats.medium,
                "low": self.severity_stats.low,
                "info": self.severity_stats.info,
                "risk_score": self.severity_stats.risk_score(),
            },
            "target_count": self.analyzed_targets.len(),
            "targets": self.analyzed_targets,
            "summary": self.executive_summary,
            "findings": self.findings,
        });

        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate HTML report, escaping any content that isn't a fixed literal
    /// (title comes from the scan target and can contain attacker-chosen text).
    fn generate_html(&self) -> String {
        let title = html_escape(&self.title);
        let timestamp = html_escape(&self.timestamp);

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .critical {{ color: red; font-weight: bold; }}
        .high {{ color: orange; font-weight: bold; }}
        .medium {{ color: #FFD700; }}
        .low {{ color: blue; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid black; padding: 8px; text-align: left; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p><strong>Generated:</strong> {}</p>
    <h2>Summary</h2>
    <p>Total Findings: {}</p>
    <p>Risk Score: {:.1}/100.0</p>
</body>
</html>"#,
            title,
            title,
            timestamp,
            self.severity_stats.total(),
            self.severity_stats.risk_score()
        )
    }

    /// Generate CSV report using RFC 4180 field quoting (every field wrapped
    /// in double quotes, internal quotes doubled) rather than naive comma
    /// stripping, so quotes/commas/newlines in a finding can't corrupt columns.
    fn generate_csv(&self) -> String {
        let mut csv = "Severity,Vulnerability_ID,File,Line,Message\n".to_string();

        for finding in &self.findings {
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                csv_escape(&format!("{:?}", finding.severity)),
                csv_escape(&finding.invariant_id),
                csv_escape(&finding.file),
                finding.line,
                csv_escape(&finding.message)
            ));
        }

        csv
    }
}

/// Escape a single CSV field per RFC 4180: always quote, and double any
/// embedded quote characters. Safe regardless of commas/quotes/newlines.
fn csv_escape(field: &str) -> String {
    format!("\"{}\"", field.replace('"', "\"\""))
}

/// Escape a string for safe interpolation into HTML text content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_stats_from_findings() {
        let findings = vec![
            Finding::new(
                "test".to_string(),
                Severity::Critical,
                "file.sol".to_string(),
                1,
                0,
                "Test".to_string(),
                "code".to_string(),
            ),
            Finding::new(
                "test".to_string(),
                Severity::High,
                "file.sol".to_string(),
                2,
                0,
                "Test".to_string(),
                "code".to_string(),
            ),
        ];

        let stats = SeverityStats::from_findings(&findings);
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.total(), 2);
    }

    #[test]
    fn risk_score_calculation() {
        let stats = SeverityStats {
            critical: 1,
            high: 2,
            medium: 3,
            low: 4,
            info: 0,
        };

        let score = stats.risk_score();
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn report_generation() {
        let findings = vec![];
        let report = SecurityReport::new(
            "Test Report".to_string(),
            vec!["test.sol".to_string()],
            findings,
            "No issues found".to_string(),
        );

        let md = report.generate(ReportFormat::Markdown);
        assert!(md.contains("Test Report"));
    }

    /// A malicious title (e.g. derived from an attacker-chosen file/contract
    /// name) must never corrupt the JSON structure or let content escape its
    /// string field.
    #[test]
    fn json_report_escapes_untrusted_title() {
        let report = SecurityReport::new(
            r#"Evil" , "injected": true, "x": ""#.to_string(),
            vec!["test.sol".to_string()],
            vec![],
            "summary".to_string(),
        );

        let json = report.generate(ReportFormat::Json);
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("generated JSON must always parse");
        assert!(parsed.get("injected").is_none(), "must not inject new keys");
    }

    /// A finding message containing HTML/script content must be escaped, not
    /// interpolated raw, when embedded in an HTML report.
    #[test]
    fn html_report_escapes_script_content() {
        let report = SecurityReport::new(
            "<script>alert(1)</script>".to_string(),
            vec!["test.sol".to_string()],
            vec![],
            "summary".to_string(),
        );

        let html = report.generate(ReportFormat::Html);
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    /// A finding message containing a comma and a quote must not break CSV
    /// column alignment - every field is quoted and internal quotes doubled.
    #[test]
    fn csv_report_escapes_commas_and_quotes() {
        let findings = vec![Finding::new(
            "test_id".to_string(),
            Severity::High,
            "file.sol".to_string(),
            1,
            0,
            r#"message, with "quotes" and, commas"#.to_string(),
            "code".to_string(),
        )];
        let report = SecurityReport::new(
            "Test".to_string(),
            vec!["test.sol".to_string()],
            findings,
            "summary".to_string(),
        );

        let csv = report.generate(ReportFormat::Csv);
        let data_line = csv.lines().nth(1).expect("must have a data row");
        // Exactly 5 quoted fields, not split apart by the embedded commas.
        assert_eq!(data_line.matches('"').count() % 2, 0, "quotes must balance");
        assert!(data_line.contains(r#""message, with ""quotes"" and, commas""#));
    }
}
