//! EVM ERC-4337 Validation Side-Effects Detector
//!
//! Detects H67 vulnerability: an ERC-4337 `validateUserOp` (or
//! `validatePaymasterUserOp`) implementation that performs a state-mutating
//! token call (`approve`/`transfer`/`transferFrom`/`mint`/`burn`) instead of
//! staying side-effect-free.
//!
//! ERC-4337 validation must be side-effect-free by design — a bundler may
//! simulate a UserOperation's validation multiple times, and downstream
//! code assumes validation cannot itself change state. This is the exact
//! root cause of the Lumi Finance hack ($270K, Jul 2026): its Sodium smart
//! accounts performed token `approve` calls as an unintended side effect
//! during `validateUserOp`, letting an attacker-controlled paymaster obtain
//! allowances from many accounts and drain them.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref VALIDATION_FN: Regex =
        Regex::new(r"(?i)function\s+(validateUserOp|validatePaymasterUserOp)\s*\(").unwrap();
    static ref STATE_MUTATING_TOKEN_CALL: Regex =
        Regex::new(r"(?i)\.(approve|transfer|transferFrom|mint|burn)\s*\(").unwrap();
}

pub fn detect_erc4337_validation_side_effects(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let Some(caps) = VALIDATION_FN.captures(line) else {
            continue;
        };
        let fn_name = &caps[1];

        // Crude function-body bound: scan until the next top-level `function`
        // declaration, matching this codebase's established window-based
        // heuristic style rather than a full brace-matching parser.
        let mut end = lines.len();
        for (j, l) in lines.iter().enumerate().skip(i + 1) {
            if l.trim_start().starts_with("function ") {
                end = j;
                break;
            }
        }
        let body = lines[i..end].join("\n");

        if STATE_MUTATING_TOKEN_CALL.is_match(&body) {
            findings.push(
                Finding::new(
                    "evm_erc4337_validation_side_effects".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    i + 1,
                    0,
                    format!(
                        "{fn_name} performs a state-mutating token call (approve/transfer/mint/\
                         burn) — ERC-4337 validation must be side-effect-free; an attacker-\
                         controlled paymaster or UserOperation can trigger unintended \
                         approvals during validation, the exact pattern behind the Lumi \
                         Finance hack ($270K, Jul 2026)"
                    ),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "evm".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_approve_call_inside_validate_user_op() {
        let source = r#"
contract SodiumAccount {
    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
        external returns (uint256 validationData)
    {
        IERC20(token).approve(paymaster, type(uint256).max);
        return 0;
    }
}
"#;
        let findings = detect_erc4337_validation_side_effects(source, "sodium.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_erc4337_validation_side_effects"));
    }

    #[test]
    fn does_not_flag_side_effect_free_validation() {
        let source = r#"
contract SafeAccount {
    function validateUserOp(UserOperation calldata userOp, bytes32 userOpHash, uint256 missingAccountFunds)
        external returns (uint256 validationData)
    {
        address recovered = ECDSA.recover(userOpHash, userOp.signature);
        return recovered == owner ? 0 : 1;
    }
}
"#;
        let findings = detect_erc4337_validation_side_effects(source, "safe_account.sol");
        assert!(findings.is_empty());
    }
}
