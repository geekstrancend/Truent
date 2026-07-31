//! EVM Insufficient Multisig/Validator Threshold Detector
//!
//! Detects H58 vulnerability: a multisig/validator-set contract whose
//! signature threshold is a bare majority or less relative to its total
//! signer count. A low threshold means compromising only a minority of
//! keys is enough to authorize arbitrary actions — exactly the pattern
//! behind the Ronin Bridge ($625M, Mar 2022, 5-of-9 validators) and Harmony
//! Horizon Bridge ($100M, Jun 2022, 2-of-5 multisig) key-compromise thefts.
//! Both were ultimately private-key thefts, not smart-contract bugs, but
//! the on-chain threshold configuration is exactly what determined how few
//! keys the attacker needed.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref THRESHOLD_DECL: Regex = Regex::new(
        r"(?i)(?:uint\d*|int\d*)\s+(?:public\s+|private\s+|internal\s+|constant\s+)*(required|threshold|quorum|minsignatures|min_signatures|signaturesrequired)\b\s*=\s*(\d+)",
    )
    .unwrap();
    static ref TOTAL_DECL: Regex = Regex::new(
        r"(?i)(?:uint\d*|int\d*)\s+(?:public\s+|private\s+|internal\s+|constant\s+)*(total_?validators|total_?signers|total_?owners|num_?validators|num_?signers|owner_?count|validator_?count)\b\s*=\s*(\d+)",
    )
    .unwrap();
    static ref OWNERS_ARRAY_LITERAL: Regex =
        Regex::new(r"(?i)(?:owners|validators|signers)\s*=\s*\[([^\]]*)\]").unwrap();
}

pub fn detect_insufficient_multisig_threshold(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let Some(threshold_caps) = THRESHOLD_DECL.captures(source) else {
        return findings;
    };
    let Ok(threshold) = threshold_caps[2].parse::<u32>() else {
        return findings;
    };
    if threshold == 0 {
        return findings;
    }

    let total = TOTAL_DECL
        .captures(source)
        .and_then(|c| c[2].parse::<u32>().ok())
        .or_else(|| {
            OWNERS_ARRAY_LITERAL
                .captures(source)
                .map(|c| c[1].split(',').filter(|s| !s.trim().is_empty()).count() as u32)
        });

    let Some(total) = total else {
        return findings;
    };
    if total == 0 || threshold > total {
        return findings;
    }

    let ratio = f64::from(threshold) / f64::from(total);
    if ratio > 0.6 {
        return findings;
    }

    let full_match = threshold_caps.get(0).unwrap();
    let line_num = source[..full_match.start()].matches('\n').count() + 1;

    findings.push(
        Finding::new(
            "evm_insufficient_multisig_threshold".to_string(),
            Severity::High,
            file_path.to_string(),
            line_num,
            0,
            format!(
                "Multisig/validator threshold requires only {threshold} of {total} signatures \
                 ({:.0}%) — a minority of compromised keys suffices to authorize actions, the \
                 exact pattern behind the Ronin Bridge ($625M, 2022, 5-of-9) and Harmony Horizon \
                 Bridge ($100M, 2022, 2-of-5) key-compromise thefts",
                ratio * 100.0
            ),
            full_match.as_str().trim().to_string(),
        )
        .with_metadata("chain".to_string(), "evm".to_string()),
    );

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_low_threshold_relative_to_owner_count() {
        let source = r#"
contract Bridge {
    uint256 public required = 2;
    address[] public owners;

    constructor() {
        owners = [addr1, addr2, addr3, addr4, addr5];
    }
}
"#;
        let findings = detect_insufficient_multisig_threshold(source, "bridge.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_insufficient_multisig_threshold"));
    }

    #[test]
    fn flags_low_threshold_with_explicit_total_constant() {
        let source = r#"
contract Validators {
    uint8 constant threshold = 5;
    uint8 constant total_validators = 9;
}
"#;
        let findings = detect_insufficient_multisig_threshold(source, "validators.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_insufficient_multisig_threshold"));
    }

    #[test]
    fn does_not_flag_supermajority_threshold() {
        let source = r#"
contract SafeMultisig {
    uint256 public required = 4;
    address[] public owners;

    constructor() {
        owners = [addr1, addr2, addr3, addr4, addr5];
    }
}
"#;
        let findings = detect_insufficient_multisig_threshold(source, "safe.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_insufficient_multisig_threshold"));
    }

    #[test]
    fn does_not_flag_when_no_threshold_declared() {
        let source = r#"
contract PlainToken {
    mapping(address => uint256) balances;
}
"#;
        let findings = detect_insufficient_multisig_threshold(source, "token.sol");
        assert!(findings.is_empty());
    }
}
