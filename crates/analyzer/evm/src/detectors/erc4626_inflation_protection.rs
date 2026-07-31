/// EVM ERC4626 Vault Inflation Attack Detector
///
/// Detects inflation attacks on ERC4626 vault implementations:
/// Attacker can force early shareholders to lose shares through share inflation via large deposit + transfer pattern
///
/// The vulnerability occurs when:
/// 1. Vault uses totalAssets() for share calculation without rounding protections
/// 2. First depositor can donate assets to inflate vault balance
/// 3. Later depositors get rounded down to 0 shares
/// 4. Early depositor captures value from later depositors
///
/// Example Vulnerable Pattern:
/// ```solidity
/// function deposit(uint assets, address receiver) external returns (uint shares) {
///     shares = (assets * totalSupply()) / totalAssets();  // No minimum or rounding protection
///     require(shares > 0, "zero shares");
///     vault.transferFrom(msg.sender, address(this), assets);
///     _mint(receiver, shares);
/// }
/// ```
///
/// Safe Pattern:
/// ```solidity
/// function deposit(uint assets, address receiver) external returns (uint shares) {
///     require(assets > 0, "zero assets");
///     uint _totalAssets = totalAssets();
///     uint _supply = totalSupply();
///     
///     if (_supply == 0) {
///         shares = assets;
///     } else {
///         shares = (assets * _supply) / _totalAssets;
///     }
///     
///     require(shares >= MIN_SHARES, "insufficient shares");
///     vault.transferFrom(msg.sender, address(this), assets);
///     _mint(receiver, shares);
/// }
/// ```
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref DEPOSIT_PATTERN: Regex = Regex::new(r"(?i)function\s+deposit\s*\(").unwrap();
    static ref SHARE_CALC_PATTERN: Regex =
        Regex::new(r"(?i)(shares\s*=|return).*?\(.*?assets.*?\).*?/").unwrap();
    static ref MINIMUM_SHARES_CHECK: Regex =
        Regex::new(r"(?i)require\s*\(.*?(shares\s*>=|shares\s*>|MIN_SHARES)\s*.*?\)").unwrap();
    static ref ZERO_SUPPLY_PROTECTION: Regex = Regex::new(
        r"(?i)(if|require).*?totalSupply.*?(==\s*0|isZero)|(if|require).*?_supply.*?(==\s*0)"
    )
    .unwrap();
    static ref ROUNDING_PROTECTION: Regex =
        Regex::new(r"(?i)(roundUp|roundDown|ceil|floor|mulDiv|FixedPointMathLib)").unwrap();
}

pub fn detect_erc4626_inflation_protection(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim().starts_with("//") {
            continue;
        }

        // Look for deposit functions
        if !DEPOSIT_PATTERN.is_match(line) {
            continue;
        }

        // Extract function body (150 lines)
        let context_end = std::cmp::min(line_num + 150, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        // Check for share calculation
        let has_share_calc = SHARE_CALC_PATTERN.is_match(&function_body);
        if !has_share_calc {
            continue;
        }

        // Check for protections
        let has_min_check = MINIMUM_SHARES_CHECK.is_match(&function_body);
        let has_zero_supply_protection = ZERO_SUPPLY_PROTECTION.is_match(&function_body);
        let has_rounding = ROUNDING_PROTECTION.is_match(&function_body);

        // Critical if no min check and no rounding and no zero supply protection
        if !has_min_check && !has_rounding && !has_zero_supply_protection {
            findings.push(
                Finding::new(
                    "evm_erc4626_inflation_protection".to_string(),
                    truent_core::Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "ERC4626 deposit uses vulnerable share calculation without minimum shares protection. \
                     Vulnerable to inflation attacks where early depositor can force subsequent depositors to receive 0 shares."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H52".to_string())
                .with_metadata("exploit_name".to_string(), "ERC4626 Inflation Attack".to_string())
                .with_metadata("loss".to_string(), "Varies".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "share_inflation".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Require MIN_SHARES per deposit or use rounding protections".to_string()
                ),
            );
        } else if !has_zero_supply_protection && has_min_check && !has_rounding {
            // Has min check but might miss zero supply case
            findings.push(
                Finding::new(
                    "evm_erc4626_inflation_protection".to_string(),
                    truent_core::Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "ERC4626 deposit has minimum shares check but lacks zero supply protection. \
                     Consider explicit zero supply handling to prevent first-depositor inflation attacks."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H52".to_string())
                .with_metadata("exploit_name".to_string(), "ERC4626 - Weak Zero Supply Handling".to_string())
                .with_metadata("loss".to_string(), "Varies".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "share_inflation".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add explicit if (totalSupply == 0) check".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_unprotected_share_calculation() {
        let vulnerable = r#"
        function deposit(uint assets, address receiver) external returns (uint shares) {
            uint _totalAssets = totalAssets();
            uint _supply = totalSupply();
            
            shares = (assets * _supply) / _totalAssets;  // No minimum check!
            require(shares > 0, "zero shares");
            
            asset.transferFrom(msg.sender, address(this), assets);
            _mint(receiver, shares);
        }
        "#;

        let findings = detect_erc4626_inflation_protection(vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect unprotected share calculation"
        );
        assert_eq!(
            findings[0].metadata.get("exploit_id"),
            Some(&"H52".to_string())
        );
    }

    #[test]
    fn test_safe_deposit_with_min_shares() {
        let safe = r#"
        function deposit(uint assets, address receiver) external returns (uint shares) {
            require(assets > 0, "zero assets");
            
            uint _totalAssets = totalAssets();
            uint _supply = totalSupply();
            
            shares = (assets * _supply) / _totalAssets;
            require(shares >= MIN_SHARES, "insufficient shares");  // Protected!
            
            asset.transferFrom(msg.sender, address(this), assets);
            _mint(receiver, shares);
        }
        "#;

        let findings = detect_erc4626_inflation_protection(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should allow properly protected deposits"
        );
    }

    #[test]
    fn test_safe_deposit_with_zero_supply_check() {
        let safe = r#"
        function deposit(uint assets, address receiver) external returns (uint shares) {
            if (totalSupply() == 0) {
                shares = assets;
            } else {
                shares = (assets * totalSupply()) / totalAssets();
            }
            
            asset.transferFrom(msg.sender, address(this), assets);
            _mint(receiver, shares);
        }
        "#;

        let findings = detect_erc4626_inflation_protection(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should allow deposits with zero supply protection"
        );
    }

    #[test]
    fn test_deposit_with_fixed_point_math() {
        let safe = r#"
        function deposit(uint assets, address receiver) external returns (uint shares) {
            shares = FixedPointMathLib.mulDiv(assets, totalSupply(), totalAssets());
            require(shares > 0, "zero shares");
            
            asset.transferFrom(msg.sender, address(this), assets);
            _mint(receiver, shares);
        }
        "#;

        let findings = detect_erc4626_inflation_protection(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should allow deposits with rounding protection"
        );
    }

    #[test]
    fn test_weak_zero_supply_handling() {
        let weak = r#"
        function deposit(uint assets, address receiver) external returns (uint shares) {
            shares = (assets * totalSupply()) / totalAssets();
            require(shares >= 1000, "min shares");  // Has min check
            
            asset.transferFrom(msg.sender, address(this), assets);
            _mint(receiver, shares);
        }
        "#;

        let findings = detect_erc4626_inflation_protection(weak, "test.sol");
        let high_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::High)
            .collect();
        assert!(
            !high_findings.is_empty(),
            "Should flag missing explicit zero supply check"
        );
    }
}
