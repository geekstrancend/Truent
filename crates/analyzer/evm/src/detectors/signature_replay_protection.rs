/// EVM Signature Replay Protection Detector
///
/// Detects H48 vulnerability: Missing replay protection in signatures
///
/// The vulnerability occurs when:
/// 1. Signature validation doesn't include chain ID
/// 2. No nonce or unique identifier in signed message
/// 3. Same signature works across different chains or multiple times
/// 4. Attacker can replay transactions
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref ECDSA_RECOVER: Regex = Regex::new(r"(?i)(ECDSA\.recover|ecrecover)\s*\(").unwrap();
    static ref CHAIN_ID_CHECK: Regex =
        Regex::new(r"(?i)chainid|chain\.id|block\.chainid|CHAIN_ID").unwrap();
    static ref NONCE_CHECK: Regex =
        Regex::new(r"(?i)nonce\[.*?\]\+\+|require\s*\(.*?nonce.*?\)").unwrap();
    static ref MESSAGE_HASH: Regex = Regex::new(r"(?i)keccak256\s*\(\s*abi\.encode").unwrap();
}

pub fn detect_signature_replay_protection(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let source_lower = source.to_lowercase();

    // Check if code uses EIP712 safe patterns
    let has_eip712_safe = source_lower.contains("_hashtypeddatav4")
        || source_lower.contains("_domaintypehash")
        || source_lower.contains("eip712domain")
        || source_lower.contains("_buildseparator");

    if has_eip712_safe {
        return findings; // EIP712 is safe
    }

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !ECDSA_RECOVER.is_match(line) {
            continue;
        }

        // Look backward and forward for function context
        let func_start = line_num.saturating_sub(100);
        let context_end = std::cmp::min(line_num + 100, source.lines().count());
        let function_body = source
            .lines()
            .skip(func_start)
            .take(context_end - func_start)
            .collect::<Vec<_>>()
            .join("\n");

        let has_chain_id = CHAIN_ID_CHECK.is_match(&function_body);
        let has_nonce = NONCE_CHECK.is_match(&function_body);
        let has_message_hash = MESSAGE_HASH.is_match(&function_body);

        if has_message_hash && !has_chain_id && !has_nonce {
            findings.push(
                Finding::new(
                    "evm_signature_replay_protection".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Signature validation missing replay protection (chain ID and/or nonce). Add chainid and nonce to signed message digest.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H48".to_string())
                .with_metadata("exploit_name".to_string(), "Signature Replay".to_string())
                .with_metadata("loss".to_string(), "$2.4M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "replay_attack".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Include chainid and nonce in message digest".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_replay_protection() {
        let vulnerable = r#"
        function executeWithSignature(address recipient, uint amount, bytes calldata sig) external {
            bytes32 digest = keccak256(abi.encode(recipient, amount));
            address signer = ECDSA.recover(digest, sig);
            require(signer == authorizedSigner, "Invalid sig");
        }
        "#;
        let findings = detect_signature_replay_protection(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_with_chain_id() {
        let safe = r#"
        function executeWithSignature(address recipient, uint amount, bytes calldata sig) external {
            bytes32 digest = keccak256(abi.encode(recipient, amount, block.chainid));
            address signer = ECDSA.recover(digest, sig);
            require(signer == authorizedSigner, "Invalid sig");
        }
        "#;
        let findings = detect_signature_replay_protection(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_with_nonce() {
        let safe = r#"
        function executeWithSignature(address recipient, uint amount, bytes calldata sig) external {
            bytes32 digest = keccak256(abi.encode(recipient, amount, nonce[msg.sender]));
            address signer = ECDSA.recover(digest, sig);
            require(signer == authorizedSigner, "Invalid sig");
            nonce[msg.sender]++;
        }
        "#;
        let findings = detect_signature_replay_protection(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_with_both_chain_and_nonce() {
        let safe = r#"
        function executeWithSignature(address recipient, uint amount, bytes calldata sig) external {
            bytes32 digest = keccak256(abi.encode(recipient, amount, block.chainid, nonce[msg.sender]));
            address signer = ecrecover(digest, v, r, s);
            require(signer == trusted, "Bad sig");
        }
        "#;
        let findings = detect_signature_replay_protection(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_eip712_domain_separator() {
        let safe = r#"
        function executeWithSignature(address recipient, uint amount, bytes calldata sig) external {
            bytes32 digest = _hashTypedDataV4(keccak256(abi.encode(recipient, amount)));
            address signer = ECDSA.recover(digest, sig);
            require(signer == authorizedSigner, "Invalid");
        }
        "#;
        let findings = detect_signature_replay_protection(safe, "test.sol");
        assert!(findings.is_empty());
    }
}
