/// EVM Reentrancy via Whitelisted Contract Detector
///
/// Detects H29 (Penpie $27M) vulnerability: Reentrancy through whitelisted token/contract calls
///
/// The vulnerability occurs when:
/// 1. Function transfers whitelisted tokens without checking for reentrancy
/// 2. Whitelisted contract is assumed safe but contains malicious fallback/hooks
/// 3. Reentrancy guards only check for external callers, not whitelisted contracts
/// 4. Attacker can create malicious token that reenters via callback
///
/// Example Vulnerable Pattern:
/// ```solidity
/// function withdraw(uint amount) external nonReentrant {
///     require(whitelistedTokens[token], "Token not whitelisted");
///     token.transfer(msg.sender, amount);  // Assumes whitelisted = safe
///     balances[msg.sender] -= amount;
/// }
/// ```
///
/// Safe Pattern:
/// ```solidity
/// function withdraw(uint amount) external nonReentrant {
///     require(whitelistedTokens[token], "Token not whitelisted");
///     balances[msg.sender] -= amount;  // Update state FIRST
///     token.transfer(msg.sender, amount);  // Then transfer (CEI pattern)
/// }
/// ```
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref TRANSFER_PATTERN: Regex =
        Regex::new(r"(?i)(transfer|transferFrom|safeTransfer|safeTransferFrom|send|call)\s*\(")
            .unwrap();
    static ref WHITELIST_CHECK: Regex =
        Regex::new(r"(?i)(?:whitelisted|approved|verified)\s*[\[\(]").unwrap();
    static ref STATE_UPDATE_PATTERN: Regex = Regex::new(
        r"(?i)(balances|amount|supply|shares|value)\s*\[\s*\w+\s*\]\s*(-=|\+=|=\s*0|=\s*\w+\s*-)"
    )
    .unwrap();
    static ref TRANSFER_AFTER_BEFORE: Regex =
        Regex::new(r"transfer.*?balances|transfer.*?amount.*?-=").unwrap();
    static ref STATE_BEFORE_TRANSFER: Regex =
        Regex::new(r"balances.*?-=.*?transfer|amount.*?-=.*?transfer").unwrap();
    static ref REENTRANCY_GUARD: Regex =
        Regex::new(r"(?i)nonReentrant|noReentrant|ReentrancyGuard").unwrap();
}

pub fn detect_reentrancy_via_whitelisted(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let source_lower = source.to_lowercase();

    // Quick check: must have both whitelist and transfer patterns
    if !source_lower.contains("transfer") || !source_lower.contains("whitelisted") {
        return findings;
    }

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim().starts_with("//") {
            continue;
        }

        // Look for transfer operations
        let line_lower = line.to_lowercase();
        if !line_lower.contains("transfer")
            && !line_lower.contains("send")
            && !line_lower.contains("call")
        {
            continue;
        }

        // Extract function context (200 lines backward)
        let start = line_num.saturating_sub(200);
        let end = std::cmp::min(line_num + 50, source.lines().count());
        let function_context = source
            .lines()
            .skip(start)
            .take(end - start)
            .collect::<Vec<_>>()
            .join("\n");

        let function_context_lower = function_context.to_lowercase();

        // Check for whitelist check
        let has_whitelist = function_context_lower.contains("whitelisted")
            || function_context_lower.contains("approved")
            || function_context_lower.contains("verified");
        if !has_whitelist {
            continue;
        }

        // Check CEI pattern: state update BEFORE transfer
        let has_proper_order = (function_context_lower.contains("balances")
            && function_context_lower.contains("-=")
            && function_context_lower.find("balances").unwrap_or(0)
                < function_context_lower
                    .find("transfer")
                    .unwrap_or(usize::MAX))
            || (function_context_lower.contains("amount")
                && function_context_lower.contains("-=")
                && function_context_lower.find("amount").unwrap_or(0)
                    < function_context_lower
                        .find("transfer")
                        .unwrap_or(usize::MAX));

        // Check for reentrancy guard
        let has_guard = function_context_lower.contains("nonreentrant")
            || function_context_lower.contains("noreentrant")
            || function_context_lower.contains("reentrancyguard");

        if !has_proper_order {
            findings.push(
                Finding::new(
                    "evm_reentrancy_via_whitelisted".to_string(),
                    truent_core::Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Transfer of whitelisted token happens before state update. \
                     Vulnerable to reentrancy even though token is whitelisted (whitelist doesn't guarantee safety)."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H29".to_string())
                .with_metadata("exploit_name".to_string(), "Penpie Whitelisted Reentrancy".to_string())
                .with_metadata("loss".to_string(), "$27M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "reentrancy".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Update state before transfer (CEI pattern)".to_string()),
            );
        } else if !has_guard {
            // Has proper order but no reentrancy guard
            findings.push(
                Finding::new(
                    "evm_reentrancy_via_whitelisted".to_string(),
                    truent_core::Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Transfer of whitelisted token lacks reentrancy guard. \
                     While CEI pattern is used, consider adding nonReentrant modifier for defense in depth."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H29".to_string())
                .with_metadata("exploit_name".to_string(), "Penpie - Weak Guard".to_string())
                .with_metadata("loss".to_string(), "$27M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "reentrancy".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add nonReentrant modifier".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_transfer_before_state_update() {
        let vulnerable = r#"
        function withdraw(uint amount) external {
            require(whitelistedTokens[tokenAddress], "Token not whitelisted");
            tokenAddress.transfer(msg.sender, amount);  // Transfer BEFORE state update!
            balances[msg.sender] -= amount;
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect transfer before state update"
        );
        assert_eq!(
            findings[0].metadata.get("exploit_id"),
            Some(&"H29".to_string())
        );
    }

    #[test]
    fn test_safe_cei_pattern() {
        let safe = r#"
        function withdraw(uint amount) external nonReentrant {
            require(whitelistedTokens[tokenAddress], "Token not whitelisted");
            balances[msg.sender] -= amount;  // State update FIRST
            tokenAddress.transfer(msg.sender, amount);  // Then transfer
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should allow proper CEI pattern with guard"
        );
    }

    #[test]
    fn test_proper_order_without_guard() {
        let weak = r#"
        function withdraw(uint amount) external {
            require(whitelistedTokens[tokenAddress], "Token not whitelisted");
            balances[msg.sender] -= amount;  // State update first
            tokenAddress.transfer(msg.sender, amount);  // Then transfer (but no guard)
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(weak, "test.sol");
        let high_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::High)
            .collect();
        assert!(
            !high_findings.is_empty(),
            "Should flag missing reentrancy guard despite proper order"
        );
    }

    #[test]
    fn test_no_whitelist_check_ignored() {
        let ignored = r#"
        function transfer(address to, uint amount) external {
            balances[msg.sender] -= amount;
            tokenAddress.transfer(to, amount);
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(ignored, "test.sol");
        assert!(
            findings.is_empty(),
            "Should ignore transfers without whitelist check"
        );
    }

    #[test]
    fn test_penpie_pattern() {
        let penpie_vulnerable = r#"
        function withdraw(uint amount) external {
            require(whitelistedTokens[msg.sender], "Not whitelisted");
            _vault.transfer(msg.sender, amount);
            userDeposits[msg.sender] -= amount;
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(penpie_vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect Penpie-style reentrancy vulnerability"
        );
    }

    #[test]
    fn test_safe_transfer_from() {
        let safe = r#"
        function withdraw(uint amount) external nonReentrant {
            require(whitelistedTokens[token], "Not whitelisted");
            balances[msg.sender] -= amount;
            token.safeTransfer(msg.sender, amount);
        }
        "#;

        let findings = detect_reentrancy_via_whitelisted(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Safe pattern should not trigger"
        );
    }
}
