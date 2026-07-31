//! EVM Fee-on-Transfer/Rebasing Token Incompatibility Detector
//!
//! Detects H64 vulnerability: a `transferFrom`/`safeTransferFrom` call
//! whose nominal `amount` parameter is trusted directly for internal
//! accounting, without comparing the recipient's token balance before and
//! after the transfer. Deflationary/fee-on-transfer tokens deduct part of
//! the amount in transit, and rebasing tokens can change balances between
//! the call and the read — either way, the amount that actually arrived
//! can be less than what the contract credited internally, letting a
//! depositor mint more shares/credit than they actually funded.
//!
//! This is one of the most frequently reported findings across public
//! audit contests (code4rena, Sherlock) and has caused real losses,
//! including a documented Balancer-pool exploit against a deflationary
//! token (STA).

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref TRANSFER_FROM_CALL: Regex =
        Regex::new(r"(?i)\.(safeTransferFrom|transferFrom)\s*\(").unwrap();
    static ref BALANCE_DIFF_CHECK: Regex =
        Regex::new(r"(?i)balanceOf\s*\(\s*address\s*\(\s*this\s*\)\s*\)").unwrap();
}

pub fn detect_fee_on_transfer_incompatibility(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("//") || !TRANSFER_FROM_CALL.is_match(line) {
            continue;
        }

        let window_start = i.saturating_sub(10);
        let window_end = lines.len().min(i + 10);
        let window = lines[window_start..window_end].join("\n");
        if BALANCE_DIFF_CHECK.is_match(&window) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_fee_on_transfer_incompatibility".to_string(),
                Severity::Medium,
                file_path.to_string(),
                i + 1,
                0,
                "transferFrom is used without comparing this contract's token balance before \
                 and after the transfer — a fee-on-transfer or rebasing token can deliver less \
                 than the nominal amount, so accounting that trusts the parameter directly can \
                 credit more than was actually received"
                    .to_string(),
                line.trim().to_string(),
            )
            .with_metadata("chain".to_string(), "evm".to_string()),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_transfer_from_without_balance_diff_check() {
        let source = r#"
contract Vault {
    function deposit(uint256 amount) external {
        token.transferFrom(msg.sender, address(this), amount);
        balances[msg.sender] += amount;
    }
}
"#;
        let findings = detect_fee_on_transfer_incompatibility(source, "vault.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_fee_on_transfer_incompatibility"));
    }

    #[test]
    fn does_not_flag_when_balance_diff_checked() {
        let source = r#"
contract Vault {
    function deposit(uint256 amount) external {
        uint256 before = token.balanceOf(address(this));
        token.transferFrom(msg.sender, address(this), amount);
        uint256 received = token.balanceOf(address(this)) - before;
        balances[msg.sender] += received;
    }
}
"#;
        let findings = detect_fee_on_transfer_incompatibility(source, "safe_vault.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_fee_on_transfer_incompatibility"));
    }
}
