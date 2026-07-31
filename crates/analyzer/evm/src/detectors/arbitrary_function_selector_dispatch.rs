//! EVM Arbitrary Function Selector Dispatch Detector
//!
//! Detects H62 vulnerability: a low-level `.call(...)`/`.delegatecall(...)`
//! whose calldata argument is a plain variable (attacker/relayer-supplied
//! data), with no allowlist restricting which function selector that data
//! is allowed to invoke.
//!
//! This is the shape behind the Poly Network hack ($611M, Aug 2021, still
//! one of the largest DeFi exploits ever): its `EthCrossChainManager`
//! contract generically dispatched relayer-supplied calldata to
//! `EthCrossChainData` with no restriction on which function could be
//! called, so an attacker crafted calldata that invoked the
//! keeper-rotation function directly and replaced the keeper set with
//! their own key — after which they could authorize withdrawing anything.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref RAW_CALL_WITH_VARIABLE_DATA: Regex =
        Regex::new(r"(?i)\.(call|delegatecall)\s*\(\s*(\w+)\s*\)").unwrap();
    static ref SELECTOR_ALLOWLIST_EVIDENCE: Regex = Regex::new(
        r"(?i)allowedselector|selectorallowlist|selectorwhitelist|require\([^)]*selector"
    )
    .unwrap();
}

pub fn detect_arbitrary_function_selector_dispatch(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let Some(caps) = RAW_CALL_WITH_VARIABLE_DATA.captures(line) else {
            continue;
        };
        // A literal string argument like `.call("")` would also match `\w+`
        // if unquoted, but our pattern requires a bare identifier already
        // excluding quotes; still skip common non-identifier false matches.
        let arg = &caps[2];
        if arg.eq_ignore_ascii_case("true") || arg.eq_ignore_ascii_case("false") {
            continue;
        }

        let window_start = i.saturating_sub(20);
        let window = lines[window_start..=i].join("\n");
        if SELECTOR_ALLOWLIST_EVIDENCE.is_match(&window) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_arbitrary_function_selector_dispatch".to_string(),
                Severity::Critical,
                file_path.to_string(),
                i + 1,
                0,
                format!(
                    "Low-level call forwards '{arg}' as calldata with no allowlist \
                     restricting which function selector it may invoke — if '{arg}' is \
                     externally/relayer-supplied, an attacker can target any function \
                     including privileged ones, the exact pattern behind the Poly Network \
                     hack ($611M, Aug 2021), where relayer-supplied calldata reached a \
                     keeper-rotation function with no restriction"
                ),
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
    fn flags_generic_dispatch_with_no_allowlist() {
        let source = r#"
contract CrossChainManager {
    function executeCrossChainTx(address target, bytes memory data) external {
        (bool ok, ) = target.call(data);
        require(ok);
    }
}
"#;
        let findings = detect_arbitrary_function_selector_dispatch(source, "manager.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_arbitrary_function_selector_dispatch"));
    }

    #[test]
    fn does_not_flag_when_selector_allowlist_present() {
        let source = r#"
contract CrossChainManager {
    mapping(bytes4 => bool) public allowedSelector;

    function executeCrossChainTx(address target, bytes memory data) external {
        bytes4 selector = bytes4(data);
        require(allowedSelector[selector], "not allowed");
        (bool ok, ) = target.call(data);
        require(ok);
    }
}
"#;
        let findings = detect_arbitrary_function_selector_dispatch(source, "safe_manager.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_arbitrary_function_selector_dispatch"));
    }

    #[test]
    fn does_not_flag_hardcoded_calldata() {
        let source = r#"
contract Vault {
    function withdraw(address to, uint256 amount) external {
        (bool ok, ) = to.call{value: amount}("");
        require(ok);
    }
}
"#;
        let findings = detect_arbitrary_function_selector_dispatch(source, "vault.sol");
        assert!(findings.is_empty());
    }
}
