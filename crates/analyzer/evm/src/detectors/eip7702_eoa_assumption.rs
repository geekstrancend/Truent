//! EVM EIP-7702 EOA Assumption Detector (proactive)
//!
//! Detects H68 vulnerability: `tx.origin` used anywhere in the contract,
//! historically an anti-pattern for access control because it can be
//! phished through an intermediate contract, but now doubly unsafe since
//! EIP-7702 (live on Ethereum mainnet as part of the Pectra upgrade) lets
//! an EOA delegate to arbitrary contract code. Any logic assuming
//! `tx.origin`/`msg.sender` identifies a "plain" externally-owned account
//! with no code can be defeated once that address has been delegated.
//!
//! This is a **proactive** check: unlike this file's siblings, it is not
//! citing a confirmed public exploit of this exact composition. Security
//! researchers have flagged the EIP-7702 + ERC-4337 interaction as "an
//! attack surface that isn't visible from either spec alone" — this
//! detector exists to catch the underlying antipattern before it is
//! exploited at scale, not after.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref TX_ORIGIN_USAGE: Regex = Regex::new(r"(?i)tx\.origin").unwrap();
}

pub fn detect_eip7702_eoa_assumption(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("//") || !TX_ORIGIN_USAGE.is_match(line) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_eip7702_eoa_assumption".to_string(),
                Severity::Medium,
                file_path.to_string(),
                line_num + 1,
                0,
                "tx.origin is used here, historically to assume the caller is a plain \
                 externally-owned account with no code. EIP-7702 (live on Ethereum mainnet \
                 since the Pectra upgrade) lets an EOA delegate to arbitrary contract code, so \
                 this assumption no longer holds — any call routed through a 7702-delegated \
                 EOA now executes that delegated code. Proactive check: no public exploit of \
                 this exact composition has been reported yet, but it has been flagged by \
                 security researchers as an emerging attack surface"
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
    fn flags_tx_origin_usage() {
        let source = r#"
contract Vault {
    function withdraw() external {
        require(tx.origin == owner, "not owner");
    }
}
"#;
        let findings = detect_eip7702_eoa_assumption(source, "vault.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_eip7702_eoa_assumption"));
    }

    #[test]
    fn does_not_flag_msg_sender_usage() {
        let source = r#"
contract Vault {
    function withdraw() external {
        require(msg.sender == owner, "not owner");
    }
}
"#;
        let findings = detect_eip7702_eoa_assumption(source, "safe_vault.sol");
        assert!(findings.is_empty());
    }
}
