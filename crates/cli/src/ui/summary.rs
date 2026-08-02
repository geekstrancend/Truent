//! Summary dashboard for analysis results.

use crate::ui::constants::*;
use crate::ui::utils::{box_line, divider, empty_box_line, severity_bar, term_width};
use serde::Serialize;

/// Represents the analysis summary report.
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisSummary {
    /// The target file or path analyzed
    pub target: String,
    /// The blockchain being analyzed (EVM, Solana, Move)
    pub chain: String,
    /// Total number of checks performed
    pub total_checks: usize,
    /// Number of violations found
    pub violations: usize,
    /// Number of checks that passed
    pub passed: usize,
    /// Number of suppressed violations
    pub suppressed: usize,
    /// Analysis duration in seconds
    pub duration_secs: f64,
    /// Breakdown of violations by severity
    pub severity_breakdown: SeverityBreakdown,
    /// Results demonstrated by execution, with a reproduction.
    pub proven: usize,
    /// Pattern matches that were never executed and remain unconfirmed.
    pub leads: usize,
}

/// Breakdown of violations by severity level.
#[derive(Debug, Clone, Serialize)]
pub struct SeverityBreakdown {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Render the analysis summary dashboard.
///
/// Produces a bordered panel showing:
/// - Target, chain, checks, duration
/// - Severity breakdown with proportional bars
/// - Overall pass/fail status
///
/// # Arguments
/// * `summary` - The analysis summary to display
///
/// # Returns
/// The formatted summary as a string
pub fn render_summary(summary: &AnalysisSummary) -> String {
    let width = term_width();
    let mut output = String::new();

    output.push('\n');

    // Top border
    output.push_str(&format!(
        "{}{}{}\n",
        color_border("╭"),
        divider(width.saturating_sub(2)),
        color_border("╮")
    ));

    // Header line
    let header_line = "─ Analysis Summary ─".to_string();
    output.push_str(&format!("{}\n", box_line(&header_line, width)));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Target line
    let target_line = format!(
        "{}  {}",
        color_label("Target"),
        color_value(&summary.target)
    );
    output.push_str(&format!("{}\n", box_line(&target_line, width)));

    // Chain line
    let chain_line = format!("{}  {}", color_label("Chain"), color_value(&summary.chain));
    output.push_str(&format!("{}\n", box_line(&chain_line, width)));

    // Checks summary line
    let checks_line = format!(
        "{}  {} total  {}  {} proven  {}  {} leads  {}  {} suppressed",
        color_label("Checks"),
        color_value(&summary.total_checks.to_string()),
        color_dim("·"),
        color_value(&summary.proven.to_string()),
        color_dim("·"),
        color_value(&summary.leads.to_string()),
        color_dim("·"),
        color_value(&summary.suppressed.to_string()),
    );
    output.push_str(&format!("{}\n", box_line(&checks_line, width)));

    // Duration line
    let duration_line = format!("{}  {:.2}s", color_label("Duration"), summary.duration_secs);
    output.push_str(&format!("{}\n", box_line(&duration_line, width)));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Middle divider
    output.push_str(&format!(
        "{}{}{}     \n",
        color_border("├"),
        divider(width.saturating_sub(4)),
        color_border("┤")
    ));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Severity breakdown header
    let severity_header = "Severity Breakdown";
    output.push_str(&format!("{}\n", box_line(severity_header, width)));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Top row of severity bars (critical + high)
    let max_severity = summary
        .severity_breakdown
        .critical
        .max(summary.severity_breakdown.high)
        .max(summary.severity_breakdown.medium)
        .max(summary.severity_breakdown.low);

    let critical_bar = severity_bar(summary.severity_breakdown.critical, max_severity, "█", "░");
    let critical_count = summary.severity_breakdown.critical.to_string();
    let critical_line = format!(
        "{}  {}  {}    {}  {}  {}",
        color_critical("CRITICAL"),
        critical_bar,
        color_value(&critical_count),
        color_high("HIGH"),
        severity_bar(summary.severity_breakdown.high, max_severity, "█", "░"),
        color_value(&summary.severity_breakdown.high.to_string()),
    );
    output.push_str(&format!("{}\n", box_line(&critical_line, width)));

    // Bottom row of severity bars (medium + low)
    let medium_bar = severity_bar(summary.severity_breakdown.medium, max_severity, "█", "░");
    let medium_count = summary.severity_breakdown.medium.to_string();
    let low_bar = severity_bar(summary.severity_breakdown.low, max_severity, "█", "░");
    let low_count = summary.severity_breakdown.low.to_string();
    let medium_line = format!(
        "{}  {}  {}    {}  {}  {}",
        color_medium("MEDIUM"),
        medium_bar,
        color_value(&medium_count),
        color_low("LOW"),
        low_bar,
        color_value(&low_count),
    );
    output.push_str(&format!("{}\n", box_line(&medium_line, width)));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Middle divider
    output.push_str(&format!(
        "{}{}{}    \n",
        color_border("├"),
        divider(width.saturating_sub(4)),
        color_border("┤")
    ));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Status line.
    //
    // A lead is a pattern match that was never executed, so it cannot be
    // reported as a failure — saying "FAIL" on unproven results is what makes
    // a tool's output unactionable. Only a result the engine actually
    // reproduced fails the run.
    let (status_icon, status_color_text) = if summary.proven > 0 {
        (
            ICON_CRITICAL,
            color_failure("FAIL — violations reproduced by execution"),
        )
    } else if summary.leads > 0 {
        (
            ICON_ARROW,
            color_medium(&format!(
                "REVIEW — {} unproven lead(s), 0 reproduced",
                summary.leads
            )),
        )
    } else {
        (ICON_PASS, color_success("PASS — nothing found"))
    };

    let status_line = format!(
        "{}  {} {}",
        color_label("Status"),
        status_icon,
        status_color_text
    );
    output.push_str(&format!("{}\n", box_line(&status_line, width)));

    // Empty line
    output.push_str(&format!("{}\n", empty_box_line(width)));

    // Bottom border
    output.push_str(&format!(
        "{}{}{}\n",
        color_border("╰"),
        divider(width.saturating_sub(2)),
        color_border("╯")
    ));

    output
}

/// Render a simple success message when no violations found.
///
/// # Returns
/// A formatted success message
#[allow(dead_code)]
pub fn render_no_violations() -> String {
    let _width = term_width();
    let mut output = String::new();

    output.push('\n');
    output.push_str(&format!(
        "  {}\n",
        color_success(&format!(
            "{} No violations found. All checks passed.",
            ICON_PASS
        ))
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_summary_with_violations() {
        let summary = AnalysisSummary {
            target: "./contracts/Token.sol".to_string(),
            chain: "EVM".to_string(),
            total_checks: 47,
            violations: 3,
            passed: 44,
            proven: 3,
            leads: 0,
            suppressed: 0,
            duration_secs: 1.24,
            severity_breakdown: SeverityBreakdown {
                critical: 1,
                high: 1,
                medium: 1,
                low: 0,
            },
        };

        let rendered = render_summary(&summary);
        assert!(rendered.contains("Analysis Summary"));
        assert!(rendered.contains("./contracts/Token.sol"));
        assert!(rendered.contains("EVM"));
        assert!(rendered.contains("47"));
        // The summary reports evidence, not a "passed" count: every check that
        // did not fire is not a check that passed, and claiming otherwise was
        // the most misleading number in the report.
        assert!(rendered.contains("3"));
        assert!(rendered.contains("proven"));
        assert!(rendered.contains("leads"));
        assert!(rendered.contains("1.24"));
        // Reproduced violations still fail the run.
        assert!(rendered.contains("FAIL"));
    }

    #[test]
    fn test_render_summary_no_violations() {
        let summary = AnalysisSummary {
            target: "./contracts/Safe.sol".to_string(),
            chain: "EVM".to_string(),
            total_checks: 100,
            violations: 0,
            passed: 100,
            proven: 0,
            leads: 0,
            suppressed: 0,
            duration_secs: 2.5,
            severity_breakdown: SeverityBreakdown {
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
            },
        };

        let rendered = render_summary(&summary);
        assert!(rendered.contains("Analysis Summary"));
        assert!(rendered.contains("PASS"));
        assert!(rendered.contains("100"));
    }

    #[test]
    fn test_render_no_violations_message() {
        let msg = render_no_violations();
        assert!(msg.contains("No violations found"));
        assert!(msg.contains("All checks passed"));
    }
}

#[cfg(test)]
mod evidence_status_tests {
    use super::*;

    fn summary(proven: usize, leads: usize) -> AnalysisSummary {
        AnalysisSummary {
            target: "t.sol".to_string(),
            chain: "EVM".to_string(),
            total_checks: 10,
            violations: proven + leads,
            passed: 0,
            proven,
            leads,
            suppressed: 0,
            duration_secs: 0.1,
            severity_breakdown: SeverityBreakdown {
                critical: 0,
                high: 0,
                medium: proven + leads,
                low: 0,
            },
        }
    }

    /// Unproven pattern matches must not be presented as a failure. Telling a
    /// team their build failed on a guess is how a scanner gets switched off.
    #[test]
    fn leads_alone_are_a_review_not_a_failure() {
        let rendered = render_summary(&summary(0, 12));
        assert!(rendered.contains("REVIEW"), "{rendered}");
        assert!(!rendered.contains("FAIL"), "{rendered}");
    }

    #[test]
    fn a_reproduced_violation_fails() {
        let rendered = render_summary(&summary(1, 0));
        assert!(rendered.contains("FAIL"), "{rendered}");
    }

    #[test]
    fn nothing_found_passes() {
        let rendered = render_summary(&summary(0, 0));
        assert!(rendered.contains("PASS"), "{rendered}");
    }
}
