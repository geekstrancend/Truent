/// EVM Account Abstraction Entropy Detector
///
/// Detects H54 vulnerability: Insufficient entropy in account abstraction implementations
///
/// The vulnerability occurs when:
/// 1. Account abstraction uses weak randomness for nonce or salt generation
/// 2. Predictable patterns in account creation
/// 3. Attacker can predict and precompute account addresses
/// 4. Can be combined with other attacks for account takeover
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref AA_FACTORY: Regex =
        Regex::new(r"(?i)(createAccount|create2Account|deployAccount|AccountFactory)").unwrap();
    static ref NONCE_PATTERN: Regex = Regex::new(
        r"(?i)nonce\s*\+\+|nonce\s*=\s*block\.(timestamp|number)|nonce\s*=\s*msg\.sender"
    )
    .unwrap();
    static ref RANDOMNESS_GOOD: Regex = Regex::new(
        r"(?i)(blockhash.*?prevrandao|keccak256.*?abi\.encode.*?block)|random\s*=\s*oracle"
    )
    .unwrap();
}

pub fn detect_aa_entropy_weakness(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !AA_FACTORY.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 150, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        let has_weak_nonce = NONCE_PATTERN.is_match(&function_body);
        let has_good_randomness = RANDOMNESS_GOOD.is_match(&function_body);

        if has_weak_nonce && !has_good_randomness {
            findings.push(
                Finding::new(
                    "evm_aa_entropy_weakness".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Account abstraction uses weak entropy for nonce/salt generation. Predictable nonces enable account prediction attacks.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H54".to_string())
                .with_metadata("exploit_name".to_string(), "AA Entropy Weakness".to_string())
                .with_metadata("loss".to_string(), "$5.2M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "weak_randomness".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Use prevrandao or oracle randomness for salt generation".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_nonce_entropy() {
        let vulnerable = r#"
        function createAccount(address owner) external returns (address) {
            uint256 nonce = msg.sender;  // Weak entropy!
            bytes32 salt = keccak256(abi.encode(owner, nonce));
            return create2Account(salt, owner);
        }
        "#;
        let findings = detect_aa_entropy_weakness(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_block_timestamp_entropy() {
        let vulnerable = r#"
        function createAccount(address owner) external returns (address) {
            uint256 nonce = block.timestamp;  // Predictable!
            bytes32 salt = keccak256(abi.encode(owner, nonce));
            return create2Account(salt, owner);
        }
        "#;
        let findings = detect_aa_entropy_weakness(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_good_randomness_prevrandao() {
        let safe = r#"
        function createAccount(address owner) external returns (address) {
            uint256 entropy = uint256(keccak256(abi.encode(blockhash(block.number - 1), block.prevrandao)));
            bytes32 salt = keccak256(abi.encode(owner, entropy));
            return create2Account(salt, owner);
        }
        "#;
        let findings = detect_aa_entropy_weakness(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_oracle_randomness() {
        let safe = r#"
        function createAccount(address owner) external returns (address) {
            uint256 random = randomOracle.getRandomness();
            bytes32 salt = keccak256(abi.encode(owner, random));
            return deployAccount(salt, owner);
        }
        "#;
        let findings = detect_aa_entropy_weakness(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_block_number_entropy() {
        let weak = r#"
        function AccountFactory(address owner) external {
            uint256 nonce = block.number;
            salt = keccak256(abi.encode(owner, nonce));
        }
        "#;
        let findings = detect_aa_entropy_weakness(weak, "test.sol");
        assert!(!findings.is_empty());
    }
}
