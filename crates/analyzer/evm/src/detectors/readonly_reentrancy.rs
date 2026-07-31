//! EVM Read-Only Reentrancy Detector
//!
//! Detects H57 vulnerability: a public/external `view`/`pure` getter with no
//! reentrancy guard, in a contract that also updates balance/supply/reserve
//! state *after* an external call elsewhere (the classic checks-effects-
//! interactions violation `evm_reentrancy_classic`/`evm_state_mutation_ordering`
//! already flag) — but the risk here isn't reentering a state-changing
//! function. It's that another protocol reads the *view* function mid-
//! transaction and gets a stale/manipulated value, because that view
//! function has no guard of its own.
//!
//! This pattern was exploited in H57 dForce ($3.7M, Feb 2023) via Curve's
//! `get_virtual_price()` — LP tokens were burned before the underlying
//! assets were transferred out, so a reentrant read during that window saw
//! an inflated virtual price — and drove roughly $70M in further losses
//! across other Curve-pool integrators in August 2023.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref EXTERNAL_CALL: Regex =
        Regex::new(r"(?i)\.call\s*[\(\{]|\.transfer\(|\.send\(").unwrap();
    static ref STATE_WRITE: Regex = Regex::new(
        r"(?i)(balances|totalsupply|total_supply|reserves|shares)\s*(\[[^\]]*\])?\s*(-=|\+=|=[^=])"
    )
    .unwrap();
    static ref VIEW_FN: Regex = Regex::new(
        r"(?i)function\s+(\w*(?:price|rate|virtual|exchange|share|value|balance|supply)\w*)\s*\([^)]*\)\s*(?:public|external)\s+(?:view|pure)"
    )
    .unwrap();
}

/// Whether this file updates balance/supply/reserve-shaped state *after* an
/// external call somewhere — the window a read-only reentrancy attack reads
/// through.
fn has_external_call_before_state_write(source: &str) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("//") || !EXTERNAL_CALL.is_match(line) {
            continue;
        }
        let window_end = lines.len().min(i + 30);
        let window = lines[i..window_end].join("\n");
        if STATE_WRITE.is_match(&window) {
            return true;
        }
    }
    false
}

pub fn detect_readonly_reentrancy(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !has_external_call_before_state_write(source) {
        return findings;
    }

    for caps in VIEW_FN.captures_iter(source) {
        let full_match = caps.get(0).unwrap();
        let fn_name = &caps[1];
        let line_num = source[..full_match.start()].matches('\n').count() + 1;
        let decl_line = source.lines().nth(line_num - 1).unwrap_or("");
        if decl_line.to_lowercase().contains("nonreentrant") {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_readonly_reentrancy".to_string(),
                Severity::High,
                file_path.to_string(),
                line_num,
                0,
                format!(
                    "View function '{fn_name}' has no reentrancy guard, and this contract \
                     updates balance/supply/reserve state after an external call elsewhere \
                     in the file; a protocol reading '{fn_name}' mid-transaction can observe \
                     a stale or manipulated value (read-only reentrancy) — see H57 dForce \
                     ($3.7M, Feb 2023) via Curve's get_virtual_price()"
                ),
                decl_line.trim().to_string(),
            )
            .with_metadata("chain".to_string(), "evm".to_string()),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    const VULNERABLE: &str = r#"
contract VulnerableVault {
    mapping(address => uint256) balances;
    uint256 totalSupply;

    function removeLiquidity(uint256 amount) external {
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok);
        balances[msg.sender] -= amount;
        totalSupply -= amount;
    }

    function getVirtualPrice() public view returns (uint256) {
        return address(this).balance * 1e18 / totalSupply;
    }
}
"#;

    #[test]
    fn flags_unguarded_view_function_when_state_written_after_external_call() {
        let findings = detect_readonly_reentrancy(VULNERABLE, "vault.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_readonly_reentrancy"));
    }

    #[test]
    fn does_not_flag_guarded_view_function() {
        let guarded = VULNERABLE.replace(
            "function getVirtualPrice() public view returns (uint256) {",
            "function getVirtualPrice() public view nonReentrant returns (uint256) {",
        );
        let findings = detect_readonly_reentrancy(&guarded, "vault.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_readonly_reentrancy"));
    }

    #[test]
    fn does_not_flag_when_no_external_call_before_state_write() {
        let safe = r#"
contract SafeVault {
    mapping(address => uint256) balances;
    uint256 totalSupply;

    function removeLiquidity(uint256 amount) external {
        balances[msg.sender] -= amount;
        totalSupply -= amount;
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok);
    }

    function getVirtualPrice() public view returns (uint256) {
        return address(this).balance * 1e18 / totalSupply;
    }
}
"#;
        let findings = detect_readonly_reentrancy(safe, "vault.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_readonly_reentrancy"));
    }
}
