/// EVM Token Balance Manipulation Detector
///
/// Detects H50 vulnerability: Unsafe external call for balance checks
///
/// The vulnerability occurs when:
/// 1. Code assumes balanceOf call is always safe
/// 2. Malicious token contract can reenter
/// 3. No guards against balance manipulation via callbacks
/// 4. Can drain protocols via re-entrancy loops
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref BALANCE_OF_CALL: Regex =
        Regex::new(r"(?i)(token\.balanceOf|ERC20\(.*?\)\.balanceOf|IERC20\(.*?\)\.balanceOf)")
            .unwrap();
    static ref TRANSFER_AFTER_CHECK: Regex =
        Regex::new(r"(?i)(balance|amount)\s*=.*?balanceOf|balanceOf.*?(transfer|transferFrom)")
            .unwrap();
    static ref REENTRANCY_GUARD: Regex = Regex::new(r"(?i)nonReentrant|ReentrancyGuard").unwrap();
}

pub fn detect_token_balance_manipulation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") || !BALANCE_OF_CALL.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 100, lines.len());
        let function_body = lines[line_num..context_end].join("\n");

        // Check if function has a guard in its signature (look backwards for 'function' keyword)
        let function_sig_start = lines[..=line_num]
            .iter()
            .rposition(|l| l.contains("function"))
            .unwrap_or(0);

        let function_sig_end = lines[line_num..]
            .iter()
            .position(|l| l.contains("{"))
            .map(|p| std::cmp::min(p + line_num, lines.len() - 1))
            .unwrap_or(std::cmp::min(line_num + 10, lines.len() - 1));

        let sig_end_safe = std::cmp::min(function_sig_end, lines.len() - 1);
        let function_signature = if function_sig_start <= sig_end_safe {
            lines[function_sig_start..=sig_end_safe].join(" ")
        } else {
            String::new()
        };

        let has_transfer_after = TRANSFER_AFTER_CHECK.is_match(&function_body);
        let has_guard = REENTRANCY_GUARD.is_match(&function_signature);

        if has_transfer_after && !has_guard {
            findings.push(
                Finding::new(
                    "evm_token_balance_manipulation".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Token balance check followed by transfer without reentrancy guard. Vulnerable to reentrant calls via token callback.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H50".to_string())
                .with_metadata("exploit_name".to_string(), "Token Balance Manipulation".to_string())
                .with_metadata("loss".to_string(), "$3.8M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "reentrancy".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add nonReentrant modifier to function".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_check_without_guard() {
        let vulnerable = r#"
        function deposit(address token) external {
            uint balance = IERC20(token).balanceOf(msg.sender);
            require(balance > 0, "no balance");
            IERC20(token).transferFrom(msg.sender, address(this), balance);
        }
        "#;
        let findings = detect_token_balance_manipulation(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_with_reentrancy_guard() {
        let safe = r#"
        function deposit(address token) external nonReentrant {
            uint balance = IERC20(token).balanceOf(msg.sender);
            require(balance > 0, "no balance");
            IERC20(token).transferFrom(msg.sender, address(this), balance);
        }
        "#;
        let findings = detect_token_balance_manipulation(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_with_reentrant_guard() {
        let safe = r#"
        function withdraw(address token) external ReentrancyGuard {
            uint balance = token.balanceOf(address(this));
            token.transfer(msg.sender, balance);
        }
        "#;
        let findings = detect_token_balance_manipulation(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_simple_balance_query() {
        let ignored = r#"
        function checkBalance(address token, address user) external view returns (uint) {
            return IERC20(token).balanceOf(user);
        }
        "#;
        let findings = detect_token_balance_manipulation(ignored, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_multiple_balance_operations() {
        let vulnerable = r#"
        function swapAndTransfer(address token) external {
            uint balance = ERC20(token).balanceOf(address(this));
            uint amountOut = router.swap(balance);
            token.transfer(msg.sender, amountOut);
        }
        "#;
        let findings = detect_token_balance_manipulation(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }
}
