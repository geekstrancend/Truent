/// EVM Upgrade Path Verification Detector
///
/// Detects H47 vulnerability: Unvalidated upgrade paths in proxy contracts
///
/// The vulnerability occurs when:
/// 1. Implementation upgrade doesn't verify new code
/// 2. No storage layout validation after upgrade
/// 3. Malicious implementation can steal funds
/// 4. No time locks or governance checks on upgrades
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref UPGRADE_FUNCTION: Regex =
        Regex::new(r"(?i)(upgradeTo|setImplementation|updateImplementation|_setImplementation)")
            .unwrap();
    static ref NEW_IMPL_PARAM: Regex =
        Regex::new(r"(?i)newImplementation|newImpl|impl|implementation\s*:").unwrap();
    static ref IMPLEMENTATION_CHECK: Regex =
        Regex::new(r"(?i)(code\.(size|length)|ERC1967)").unwrap();
    static ref TIMELOCK_CHECK: Regex =
        Regex::new(r"(?i)timelock|delay|pendingImplementation|schedule|execute").unwrap();
    static ref INTERFACE_CHECK: Regex =
        Regex::new(r"(?i)supportsInterface|implementsInterface|INTERFACE_ID").unwrap();
}

pub fn detect_upgrade_path_verification(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !UPGRADE_FUNCTION.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 150, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        let has_impl_check = IMPLEMENTATION_CHECK.is_match(&function_body);
        let has_timelock = TIMELOCK_CHECK.is_match(&function_body);
        let _has_interface = INTERFACE_CHECK.is_match(&function_body);

        if !has_impl_check && !has_timelock {
            findings.push(
                Finding::new(
                    "evm_upgrade_path_verification".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Upgrade function lacks implementation validation or timelock. Add code size check and delay mechanism.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H47".to_string())
                .with_metadata("exploit_name".to_string(), "Unvalidated Upgrade".to_string())
                .with_metadata("loss".to_string(), "$1.2M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "unsafe_upgrade".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add implementation validation and timelock delay".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_upgrade_validation() {
        let vulnerable = r#"
        function upgradeTo(address newImplementation) external onlyAdmin {
            _implementation = newImplementation;
        }
        "#;
        let findings = detect_upgrade_path_verification(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_with_implementation_check() {
        let safe = r#"
        function upgradeTo(address newImplementation) external onlyAdmin {
            require(newImplementation.code.length > 0, "No code");
            _implementation = newImplementation;
        }
        "#;
        let findings = detect_upgrade_path_verification(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_with_timelock() {
        let safe = r#"
        function upgradeTo(address newImplementation) external onlyAdmin {
            require(block.timestamp >= pendingImplementationTime + UPGRADE_DELAY, "Too soon");
            _implementation = newImplementation;
            pendingImplementationTime = 0;
        }
        "#;
        let findings = detect_upgrade_path_verification(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_erc1967_upgrade() {
        let safe = r#"
        function upgradeTo(address newImplementation) external {
            require(ERC1967Utils.getImplementation() != address(0), "No current impl");
            ERC1967Utils.upgradeToAndCall(newImplementation, "");
        }
        "#;
        let findings = detect_upgrade_path_verification(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_pending_implementation() {
        let safe = r#"
        function scheduleUpgrade(address newImplementation) external onlyAdmin {
            pendingImplementation = newImplementation;
            pendingImplementationTime = block.timestamp + TIMELOCK;
        }
        
        function confirmUpgrade() external onlyAdmin {
            require(block.timestamp >= pendingImplementationTime, "Too early");
            _implementation = pendingImplementation;
        }
        "#;
        let findings = detect_upgrade_path_verification(safe, "test.sol");
        assert!(findings.is_empty());
    }
}
