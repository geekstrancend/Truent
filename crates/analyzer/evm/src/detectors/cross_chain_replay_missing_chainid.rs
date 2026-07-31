//! EVM Cross-Chain Signature Replay Detector
//!
//! Detects H65 vulnerability: a signature-verification path (`ecrecover`)
//! with no `block.chainid` bound into the signed payload anywhere in the
//! file. A signature valid on one EVM chain can then be replayed verbatim
//! on any other chain the same contract is deployed to.
//!
//! This caused a $20M Wintermute-targeted exploit on Optimism from a
//! missing chain-ID check in a cross-chain-deployed contract, and a
//! Multichain contract vulnerability from hardcoding the wrong chain ID.
//! Contracts inheriting OpenZeppelin's `EIP712` base handle this
//! internally via `_domainSeparatorV4()`, so this detector suppresses
//! itself when it sees that pattern rather than flag a false positive.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref ECRECOVER_CALL: Regex = Regex::new(r"(?i)ecrecover\s*\(").unwrap();
    static ref CHAIN_ID_USAGE: Regex = Regex::new(r"(?i)block\.chainid|chainid").unwrap();
    static ref EIP712_BASE: Regex = Regex::new(r"(?i)\bEIP712\b").unwrap();
}

pub fn detect_cross_chain_replay_missing_chainid(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    if EIP712_BASE.is_match(source) || CHAIN_ID_USAGE.is_match(source) {
        return findings;
    }

    for (line_num, line) in source.lines().enumerate() {
        if line.trim_start().starts_with("//") || !ECRECOVER_CALL.is_match(line) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_cross_chain_replay_missing_chainid".to_string(),
                Severity::High,
                file_path.to_string(),
                line_num + 1,
                0,
                "Signature verification via ecrecover found, but block.chainid never appears \
                 anywhere in this file — a signature valid on one chain can be replayed \
                 verbatim on any other chain this contract is deployed to. See the $20M \
                 Wintermute-targeted Optimism exploit and Multichain's hardcoded-chainId bug"
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
    fn flags_ecrecover_with_no_chainid_anywhere() {
        let source = r#"
contract Forwarder {
    function execute(bytes memory data, uint8 v, bytes32 r, bytes32 s) external {
        bytes32 hash = keccak256(abi.encodePacked(data));
        address signer = ecrecover(hash, v, r, s);
        require(signer == owner);
    }
}
"#;
        let findings = detect_cross_chain_replay_missing_chainid(source, "forwarder.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_cross_chain_replay_missing_chainid"));
    }

    #[test]
    fn does_not_flag_when_chainid_bound_into_hash() {
        let source = r#"
contract Forwarder {
    function execute(bytes memory data, uint8 v, bytes32 r, bytes32 s) external {
        bytes32 hash = keccak256(abi.encodePacked(data, block.chainid));
        address signer = ecrecover(hash, v, r, s);
        require(signer == owner);
    }
}
"#;
        let findings = detect_cross_chain_replay_missing_chainid(source, "safe_forwarder.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn does_not_flag_eip712_base_contracts() {
        let source = r#"
import "@openzeppelin/contracts/utils/cryptography/EIP712.sol";

contract Forwarder is EIP712 {
    function execute(bytes memory data, uint8 v, bytes32 r, bytes32 s) external {
        bytes32 hash = _hashTypedDataV4(keccak256(abi.encode(data)));
        address signer = ecrecover(hash, v, r, s);
        require(signer == owner);
    }
}
"#;
        let findings = detect_cross_chain_replay_missing_chainid(source, "eip712.sol");
        assert!(findings.is_empty());
    }
}
