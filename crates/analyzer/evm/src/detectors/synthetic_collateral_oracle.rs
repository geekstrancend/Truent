/// EVM Synthetic Collateral Oracle Dependency Detector
///
/// Detects H45 (Rhea Finance $7.6M) and H40 (Makina $1.2M) vulnerabilities:
/// Synthetic collateral valuation relying on external oracle without fallback protection
///
/// The vulnerability occurs when:
/// 1. Synthetic assets use external oracle price for collateral valuation
/// 2. No fallback oracle or safety checks are present
/// 3. Oracle failure can cause sudden collateral devaluation
/// 4. Users can be liquidated due to oracle manipulation/failure
///
/// Example Vulnerable Pattern:
/// ```solidity
/// function getCollateralValue(address synthetic) external view returns (uint) {
///     return synthBalance[synthetic] * oracle.getPrice(synthetic);  // No fallback!
/// }
/// ```
///
/// Safe Pattern:
/// ```solidity
/// function getCollateralValue(address synthetic) external view returns (uint) {
///     uint price = oracle.getPrice(synthetic);
///     require(price > 0 && lastUpdate[synthetic] > block.timestamp - 1 hour, "Stale");
///     uint fallbackPrice = fallbackOracle.getPrice(synthetic);
///     require(abs(price - fallbackPrice) / fallbackPrice < 0.05, "Price divergence");
///     return synthBalance[synthetic] * price;
/// }
/// ```
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref SYNTHETIC_COLLATERAL_PATTERN: Regex =
        Regex::new(r"(?i)(synthetic|synth|_synth|derivatives)").unwrap();
    static ref ORACLE_CALL_PATTERN: Regex =
        Regex::new(r"(?i)(oracle|pricer|price_feed)\s*\.\s*(getPrice|getPriceOf|getLatestPrice|fetchPrice|price)\s*\(")
            .unwrap();
    static ref FALLBACK_PATTERN: Regex =
        Regex::new(r"(?i)(fallback|backup|secondary|alternative)\s*(oracle|price|feed|source)|try\s*\{.*?\}\s*catch")
            .unwrap();
    static ref SAFETY_CHECK_PATTERN: Regex =
        Regex::new(r"(?i)require\s*\(.*?(stale|timeout|lastUpdate|updatedAt|freshness|deviation|divergence|abs).*?\)").unwrap();
}

pub fn detect_synthetic_collateral_oracle(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim().starts_with("//") {
            continue;
        }

        // Look for function definitions that might use synthetic collateral
        if !SYNTHETIC_COLLATERAL_PATTERN.is_match(line) && !ORACLE_CALL_PATTERN.is_match(line) {
            continue;
        }

        // Extract function context (100 lines following)
        let context_end = std::cmp::min(line_num + 100, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        // Check for oracle calls in synthetic context
        let has_synthetic_oracle = SYNTHETIC_COLLATERAL_PATTERN.is_match(&function_body)
            && ORACLE_CALL_PATTERN.is_match(&function_body);

        if !has_synthetic_oracle {
            continue;
        }

        // Check for safety mechanisms
        let has_fallback = FALLBACK_PATTERN.is_match(&function_body);
        let has_safety_checks = SAFETY_CHECK_PATTERN.is_match(&function_body);

        if !has_fallback && !has_safety_checks {
            findings.push(
                Finding::new(
                    "evm_synthetic_collateral_oracle".to_string(),
                    truent_core::Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Synthetic collateral uses external oracle without fallback or safety checks. \
                     Vulnerable to oracle manipulation, failure, or price manipulation attacks."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H45".to_string())
                .with_metadata(
                    "exploit_name".to_string(),
                    "Rhea Finance Oracle Failure".to_string(),
                )
                .with_metadata("loss".to_string(), "$7.6M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "oracle_dependency".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Add fallback oracle, price sanity checks, staleness validation".to_string(),
                ),
            );
        } else if has_fallback && !has_safety_checks {
            // Fallback exists but no validation
            findings.push(
                Finding::new(
                    "evm_synthetic_collateral_oracle".to_string(),
                    truent_core::Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Synthetic collateral has fallback oracle but lacks safety validation. \
                     Consider adding price divergence checks and staleness validation."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H45".to_string())
                .with_metadata(
                    "exploit_name".to_string(),
                    "Rhea Finance - Weak Safety Checks".to_string(),
                )
                .with_metadata("loss".to_string(), "$7.6M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "oracle_dependency".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Add price divergence validation between oracles".to_string(),
                ),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_synthetic_oracle_no_fallback() {
        let vulnerable = r#"
        function getCollateralValue(address synthetic) external view returns (uint) {
            uint balance = synthBalance[synthetic];
            uint price = oracle.getPrice(synthetic);  // No fallback or validation
            return balance * price;
        }
        "#;

        let findings = detect_synthetic_collateral_oracle(vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect unprotected oracle call"
        );
        assert_eq!(
            findings[0].metadata.get("exploit_id"),
            Some(&"H45".to_string())
        );
    }

    #[test]
    fn test_safe_synthetic_with_fallback_and_checks() {
        let safe = r#"
        function getCollateralValue(address synthetic) external view returns (uint) {
            uint price = oracle.getPrice(synthetic);
            uint fallbackPrice = fallbackOracle.getPrice(synthetic);
            
            require(price > 0, "Invalid price");
            require(lastUpdate[synthetic] > block.timestamp - 1 hours, "Stale price");
            require(abs(price - fallbackPrice) / fallbackPrice < 50, "Price divergence");
            
            return synthBalance[synthetic] * price;
        }
        "#;

        let findings = detect_synthetic_collateral_oracle(safe, "test.sol");
        // Safe pattern with proper checks should not trigger
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should not flag properly protected oracle"
        );
    }

    #[test]
    fn test_synthetic_with_fallback_but_weak_validation() {
        let weak = r#"
        function getCollateralValue(address synthetic) external view returns (uint) {
            uint price = oracle.getPrice(synthetic);
            uint fallbackPrice = fallbackOracle.getPrice(synthetic);  // Has fallback
            // Missing divergence check
            return synthBalance[synthetic] * price;
        }
        "#;

        let findings = detect_synthetic_collateral_oracle(weak, "test.sol");
        let high_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::High)
            .collect();
        assert!(
            !high_findings.is_empty(),
            "Should flag fallback without proper validation"
        );
    }

    #[test]
    fn test_synthetic_oracle_real_exploit_rhea() {
        let rhea_pattern = r#"
        // Rhea Finance pattern: uses synthetic collateral with unprotected oracle
        mapping(address => uint) public synthBalance;
        
        function redeemSynthetic(address synth, uint amount) external {
            uint price = priceOracle.getPriceOf(synth);  // Single oracle
            require(amount <= synthBalance[msg.sender], "Insufficient balance");
            
            uint value = amount * price / 1e18;
            synth.transfer(msg.sender, amount);
            synthBalance[msg.sender] -= amount;
        }
        "#;

        let findings = detect_synthetic_collateral_oracle(rhea_pattern, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect Rhea-style synthetic oracle vulnerability"
        );
    }

    #[test]
    fn test_non_synthetic_oracle_not_flagged() {
        let normal = r#"
        function getPrice(address token) external view returns (uint) {
            return oracle.getPrice(token);
        }
        "#;

        let findings = detect_synthetic_collateral_oracle(normal, "test.sol");
        assert!(
            findings.is_empty(),
            "Should not flag regular non-synthetic oracle usage"
        );
    }
}
