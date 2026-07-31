/// Comprehensive Test Infrastructure for Truent Detectors
///
/// Provides unified testing framework for validating detector implementations
/// across EVM, Solana, and Move chains with consistent test patterns.
use std::collections::HashMap;

/// Test result for a detector
#[derive(Debug, Clone)]
pub struct DetectorTestResult {
    /// Detector name
    pub detector_name: String,
    /// Chain (evm, solana, move)
    pub chain: String,
    /// Number of vulnerable patterns tested
    pub vulnerable_count: usize,
    /// Number of safe patterns tested
    pub safe_count: usize,
    /// Number of true positives (correctly detected vulnerabilities)
    pub true_positives: usize,
    /// Number of false positives (safe code detected as vulnerable)
    pub false_positives: usize,
    /// Number of false negatives (vulnerable code not detected)
    pub false_negatives: usize,
    /// Test execution time in ms
    pub execution_time_ms: u64,
}

impl DetectorTestResult {
    /// Calculate precision (TP / (TP + FP))
    pub fn precision(&self) -> f64 {
        let total = self.true_positives + self.false_positives;
        if total == 0 {
            1.0
        } else {
            self.true_positives as f64 / total as f64
        }
    }

    /// Calculate recall (TP / (TP + FN))
    pub fn recall(&self) -> f64 {
        let total = self.true_positives + self.false_negatives;
        if total == 0 {
            1.0
        } else {
            self.true_positives as f64 / total as f64
        }
    }

    /// Calculate F1 score
    pub fn f1_score(&self) -> f64 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }

    /// Check if test passes quality threshold
    pub fn passes_quality_threshold(&self) -> bool {
        self.precision() >= 0.90 && self.recall() >= 0.90 && self.f1_score() >= 0.90
    }
}

/// Test case for detector validation
#[derive(Debug, Clone)]
pub struct DetectorTestCase {
    /// Test name
    pub name: String,
    /// Test code pattern
    pub code: String,
    /// Whether this code is vulnerable
    pub is_vulnerable: bool,
    /// Expected findings count
    pub expected_findings: usize,
    /// Category (vulnerable, safe, edge_case, real_exploit)
    pub category: String,
}

/// Comprehensive test suite for all detectors
pub struct DetectorTestSuite {
    test_cases: HashMap<String, Vec<DetectorTestCase>>,
    results: HashMap<String, DetectorTestResult>,
}

impl Default for DetectorTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectorTestSuite {
    /// Create new test suite
    pub fn new() -> Self {
        Self {
            test_cases: HashMap::new(),
            results: HashMap::new(),
        }
    }

    /// Add test case to suite
    pub fn add_test_case(&mut self, detector_name: String, test_case: DetectorTestCase) {
        self.test_cases
            .entry(detector_name)
            .or_default()
            .push(test_case);
    }

    /// Get test cases for detector
    pub fn get_test_cases(&self, detector_name: &str) -> Option<&[DetectorTestCase]> {
        self.test_cases.get(detector_name).map(|v| v.as_slice())
    }

    /// Store test result
    pub fn add_result(&mut self, detector_name: String, result: DetectorTestResult) {
        self.results.insert(detector_name, result);
    }

    /// Get test result for detector
    pub fn get_result(&self, detector_name: &str) -> Option<&DetectorTestResult> {
        self.results.get(detector_name)
    }

    /// Generate test report
    pub fn generate_report(&self) -> String {
        let mut report = "# Detector Test Report\n\n".to_string();
        report.push_str("## Summary\n\n");

        let mut total_tests = 0;
        let mut passed_tests = 0;
        let mut avg_precision = 0.0;
        let mut avg_recall = 0.0;
        let mut avg_f1 = 0.0;

        for (detector, result) in &self.results {
            report.push_str(&format!("### {} ({})\n", detector, result.chain));

            report.push_str(&format!(
                "- **Vulnerable Patterns Tested:** {}\n",
                result.vulnerable_count
            ));
            report.push_str(&format!(
                "- **Safe Patterns Tested:** {}\n",
                result.safe_count
            ));
            report.push_str(&format!(
                "- **True Positives:** {}\n",
                result.true_positives
            ));
            report.push_str(&format!(
                "- **False Positives:** {}\n",
                result.false_positives
            ));
            report.push_str(&format!(
                "- **False Negatives:** {}\n",
                result.false_negatives
            ));
            report.push_str(&format!(
                "- **Precision:** {:.2}%\n",
                result.precision() * 100.0
            ));
            report.push_str(&format!("- **Recall:** {:.2}%\n", result.recall() * 100.0));
            report.push_str(&format!("- **F1 Score:** {:.4}\n", result.f1_score()));
            report.push_str(&format!(
                "- **Execution Time:** {} ms\n",
                result.execution_time_ms
            ));

            let status = if result.passes_quality_threshold() {
                "✓ PASS"
            } else {
                "✗ FAIL"
            };
            report.push_str(&format!("- **Status:** {}\n\n", status));

            total_tests += 1;
            if result.passes_quality_threshold() {
                passed_tests += 1;
            }

            avg_precision += result.precision();
            avg_recall += result.recall();
            avg_f1 += result.f1_score();
        }

        if !self.results.is_empty() {
            let count = self.results.len() as f64;
            report.push_str("\n## Aggregate Metrics\n\n");
            report.push_str(&format!(
                "- **Total Detectors Tested:** {}\n",
                self.results.len()
            ));
            report.push_str(&format!(
                "- **Passed Quality Threshold:** {}/{}\n",
                passed_tests, total_tests
            ));
            report.push_str(&format!(
                "- **Average Precision:** {:.2}%\n",
                (avg_precision / count) * 100.0
            ));
            report.push_str(&format!(
                "- **Average Recall:** {:.2}%\n",
                (avg_recall / count) * 100.0
            ));
            report.push_str(&format!("- **Average F1 Score:** {:.4}\n", avg_f1 / count));
        }

        report
    }
}

/// Test infrastructure utilities
pub mod utils {
    /// Extract code snippet from finding
    pub fn extract_code_context(code: &str, line: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let start = line.saturating_sub(context_lines);
        let end = std::cmp::min(line + context_lines, lines.len());

        lines[start..end].join("\n")
    }

    /// Compare two findings for equality
    pub fn findings_match(f1: &crate::Finding, f2: &crate::Finding) -> bool {
        f1.invariant_id == f2.invariant_id && f1.file == f2.file && f1.line == f2.line
    }

    /// Generate test code variant
    pub fn generate_variant(base_code: &str, mutation: &str) -> String {
        base_code.replace("// MUTATION_POINT", mutation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_metrics() {
        let result = DetectorTestResult {
            detector_name: "test".to_string(),
            chain: "evm".to_string(),
            vulnerable_count: 50,
            safe_count: 50,
            true_positives: 48,
            false_positives: 1,
            false_negatives: 2,
            execution_time_ms: 150,
        };

        assert!(result.precision() > 0.95);
        assert!(result.recall() > 0.95);
        assert!(result.f1_score() > 0.95);
    }

    #[test]
    fn test_suite_stores_cases() {
        let mut suite = DetectorTestSuite::new();
        let test_case = DetectorTestCase {
            name: "vulnerable_case".to_string(),
            code: "malicious code".to_string(),
            is_vulnerable: true,
            expected_findings: 1,
            category: "vulnerable".to_string(),
        };

        suite.add_test_case("detector1".to_string(), test_case);
        assert!(suite.get_test_cases("detector1").is_some());
    }

    #[test]
    fn test_suite_generates_report() {
        let mut suite = DetectorTestSuite::new();
        let result = DetectorTestResult {
            detector_name: "test_detector".to_string(),
            chain: "evm".to_string(),
            vulnerable_count: 50,
            safe_count: 50,
            true_positives: 45,
            false_positives: 2,
            false_negatives: 5,
            execution_time_ms: 200,
        };

        suite.add_result("test_detector".to_string(), result);
        let report = suite.generate_report();

        assert!(report.contains("test_detector"));
        assert!(report.contains("Precision"));
        assert!(report.contains("Recall"));
    }
}
