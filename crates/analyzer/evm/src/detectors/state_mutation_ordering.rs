/// EVM State Mutation Ordering Detector
///
/// Detects H45 vulnerability: Unsafe state mutation order leading to exploits
///
/// The vulnerability occurs when:
/// 1. State is updated after external calls
/// 2. Reentrancy possible before state is finalized
/// 3. Incorrect order exposes intermediate states
/// 4. Can lead to double-spending or fund loss
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref EXTERNAL_CALL: Regex = Regex::new(
        r"(?i)\.call\s*[\(\{]|\.delegatecall|\.transfer\(|\.send\(|safeTransfer|safeTransferFrom"
    )
    .unwrap();
    static ref STATE_UPDATE: Regex =
        Regex::new(r"(?i)(balances|amounts|supply|shares|reserves)\s*\[\s*\w+\s*\]\s*(-=|\+=|=)")
            .unwrap();
    static ref EXTERNAL_CALL_BEFORE_STATE: Regex =
        Regex::new(r"(?is)\.call[\s\S]*?balances|(?is)transfer[\s\S]*?balances\[[\s\S]*?\]\s*-=")
            .unwrap();
}

pub fn detect_state_mutation_ordering(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !EXTERNAL_CALL.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 100, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        if EXTERNAL_CALL_BEFORE_STATE.is_match(&function_body) {
            findings.push(
                Finding::new(
                    "evm_state_mutation_ordering".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "External call precedes state update. Reorder to update state before calling external functions (CEI pattern).".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H45".to_string())
                .with_metadata("exploit_name".to_string(), "State Mutation Order".to_string())
                .with_metadata("loss".to_string(), "$2.6M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "ordering_vulnerability".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Update state before external calls (CEI)".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_before_state_update() {
        let vulnerable = r#"
        function withdraw(uint amount) external {
            (bool success, ) = msg.sender.call{value: amount}("");
            require(success, "Transfer failed");
            balances[msg.sender] -= amount;  // State update AFTER call!
        }
        "#;
        let findings = detect_state_mutation_ordering(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_correct_ordering() {
        let safe = r#"
        function withdraw(uint amount) external {
            balances[msg.sender] -= amount;  // Update state FIRST
            (bool success, ) = msg.sender.call{value: amount}("");
            require(success, "Transfer failed");
        }
        "#;
        let findings = detect_state_mutation_ordering(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_transfer_before_balance_update() {
        let vulnerable = r#"
        function swap(address token, uint amount) external {
            token.transfer(msg.sender, amount);
            balances[msg.sender] -= amount;
        }
        "#;
        let findings = detect_state_mutation_ordering(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_safe_transfer_pattern() {
        let safe = r#"
        function swap(address token, uint amount) external {
            balances[msg.sender] -= amount;
            token.transfer(msg.sender, amount);
        }
        "#;
        let findings = detect_state_mutation_ordering(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_safe_transfer_from() {
        let safe = r#"
        function deposit(address token, uint amount) external {
            balances[msg.sender] += amount;
            token.transferFrom(msg.sender, address(this), amount);
        }
        "#;
        let findings = detect_state_mutation_ordering(safe, "test.sol");
        assert!(findings.is_empty());
    }
}
