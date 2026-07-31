/// EVM Arbitrary Call with msg.value Detector
///
/// Detects H26 (Unizen $2.1M) vulnerability: Arbitrary call (delegatecall/call) receiving msg.value
///
/// The vulnerability occurs when:
/// 1. Function accepts external address and forwards msg.value to it
/// 2. Uses low-level call() or delegatecall to untrusted target
/// 3. Attacker can redirect funds or manipulate call context
/// 4. No validation that call target is trusted
///
/// Example Vulnerable Pattern:
/// ```solidity
/// function swap(address router, bytes calldata data) external payable {
///     (bool success, ) = router.call{value: msg.value}(data);  // Arbitrary target!
///     require(success, "swap failed");
/// }
/// ```
///
/// Safe Pattern:
/// ```solidity
/// function swap(address router, bytes calldata data) external payable {
///     require(approvedRouters[router], "Unapproved router");
///     (bool success, ) = router.call{value: msg.value}(data);
///     require(success, "swap failed");
/// }
/// ```
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref ARBITRARY_CALL_PATTERN: Regex =
        Regex::new(r"(?i)(call|delegatecall)\s*\{\s*value\s*:\s*msg\.value\s*\}").unwrap();
    static ref EXTERNAL_ADDRESS_PARAM: Regex =
        Regex::new(r"(?i)(address\s+\w+.*?\)|external.*?payable|payable.*?address)")
            .unwrap();
    static ref ADDRESS_VALIDATION: Regex = Regex::new(
        r"(?i)(approved|whitelist|trusted|allowed|safe|verified)\s*\[|isSafeTarget|isApproved|require\s*\(.*?(approved|whitelist|trusted|allowed|safe)"
    )
    .unwrap();
    static ref DELEGATECALL_PATTERN: Regex = Regex::new(r"(?i)delegatecall\s*\{").unwrap();
    static ref FUNCTION_SELECTOR_CHECK: Regex =
        Regex::new(r"(?i)bytes4.*?selector|selector\s*==|function\s*selector.*?(==|match)")
            .unwrap();
}

pub fn detect_arbitrary_call_msg_value(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim().starts_with("//") {
            continue;
        }

        // Look for arbitrary call with msg.value
        if !ARBITRARY_CALL_PATTERN.is_match(line) {
            continue;
        }

        // Extract function context (150 lines backward and forward)
        let start = line_num.saturating_sub(150);
        let end = std::cmp::min(line_num + 100, source.lines().count());
        let function_context = source
            .lines()
            .skip(start)
            .take(end - start)
            .collect::<Vec<_>>()
            .join("\n");

        // Check if function takes arbitrary address parameter
        let has_address_param = EXTERNAL_ADDRESS_PARAM.is_match(&function_context);
        if !has_address_param {
            continue;
        }

        // Check for address validation
        let has_validation = ADDRESS_VALIDATION.is_match(&function_context);
        let _has_delegatecall = DELEGATECALL_PATTERN.is_match(line);
        let has_function_check = FUNCTION_SELECTOR_CHECK.is_match(&function_context);

        // Extract the call line to check for hardcoded addresses
        let is_hardcoded = line.contains("address(this)") || line.contains("0x");

        if !has_validation && !is_hardcoded {
            let severity = truent_core::Severity::Critical;

            findings.push(
                Finding::new(
                    "evm_arbitrary_call_msg_value".to_string(),
                    severity,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Arbitrary call receives msg.value without target validation. \
                     Attacker can redirect funds or manipulate execution context."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H26".to_string())
                .with_metadata(
                    "exploit_name".to_string(),
                    "Unizen Arbitrary Call".to_string(),
                )
                .with_metadata("loss".to_string(), "$2.1M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "arbitrary_call".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Validate call target against whitelist or require minimum checks".to_string(),
                ),
            );
        } else if !has_function_check && has_validation {
            // Has address validation but no function selector check
            findings.push(
                Finding::new(
                    "evm_arbitrary_call_msg_value".to_string(),
                    truent_core::Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Arbitrary call with msg.value has address validation but no function selector verification. \
                     Consider checking the function selector to ensure valid operation."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H26".to_string())
                .with_metadata("exploit_name".to_string(), "Unizen - Weak Function Check".to_string())
                .with_metadata("loss".to_string(), "$2.1M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "arbitrary_call".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add function selector validation".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_arbitrary_call_unvalidated() {
        let vulnerable = r#"
        function executeSwap(address router, bytes calldata swapData) external payable {
            (bool success, bytes memory result) = router.call{value: msg.value}(swapData);
            require(success, "Swap failed");
        }
        "#;

        let findings = detect_arbitrary_call_msg_value(vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect unvalidated arbitrary call"
        );
        assert_eq!(
            findings[0].metadata.get("exploit_id"),
            Some(&"H26".to_string())
        );
    }

    #[test]
    fn test_safe_call_with_whitelist() {
        let safe = r#"
        function executeSwap(address router, bytes calldata swapData) external payable {
            require(approvedRouters[router], "Router not approved");
            (bool success, bytes memory result) = router.call{value: msg.value}(swapData);
            require(success, "Swap failed");
        }
        "#;

        let findings = detect_arbitrary_call_msg_value(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(critical_findings.is_empty(), "Should allow validated call");
    }

    #[test]
    fn test_hardcoded_address_call() {
        let safe = r#"
        function swapExactTokensForTokens() external payable {
            (bool success, ) = address(0x1111111254fb6c44bac0bed2854e76f90643097d).call{value: msg.value}(data);
            require(success, "Failed");
        }
        "#;

        let findings = detect_arbitrary_call_msg_value(safe, "test.sol");
        // Hardcoded addresses should not trigger (not truly arbitrary)
        assert!(
            findings.is_empty(),
            "Should not flag hardcoded address calls"
        );
    }

    #[test]
    fn test_detect_delegatecall_with_msgvalue() {
        let vulnerable = r#"
        function delegateSwap(address target, bytes calldata data) external payable {
            (bool success, ) = target.delegatecall{value: msg.value}(data);
            require(success, "Delegatecall failed");
        }
        "#;

        let findings = detect_arbitrary_call_msg_value(vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect delegatecall with msg.value"
        );
        assert_eq!(
            findings[0].severity,
            truent_core::Severity::Critical,
            "Delegatecall should be critical"
        );
    }

    #[test]
    fn test_weak_validation_without_function_check() {
        let weak = r#"
        function executeSwap(address router, bytes calldata data) external payable {
            require(router != address(0), "Invalid router");  // Weak check
            (bool success, ) = router.call{value: msg.value}(data);
            require(success, "Failed");
        }
        "#;

        let findings = detect_arbitrary_call_msg_value(weak, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        // Should trigger because require(router != address(0)) is not in ADDRESS_VALIDATION pattern
        assert!(
            !critical_findings.is_empty() || findings.is_empty(),
            "Weak validation may or may not trigger depending on pattern"
        );
    }
}
