//! EVM Unbounded Pricing Input Detector
//!
//! Detects H66 vulnerability: a `buy`/`mint`/`purchase`-style function whose
//! parameter is used directly in pricing arithmetic with no upper-bound
//! `require` anywhere in the function.
//!
//! This is the shape behind the Truebit hack ($26.2M, Jan 2026): an attacker
//! passed an enormous input into the token-purchase pricing function,
//! wrapping the computed mint price to near-zero, then called `buyTRU()` to
//! mint a large amount of tokens for almost nothing. Capping the input is a
//! defense-in-depth measure independent of whether the arithmetic itself is
//! overflow-checked — an unbounded input can still distort a pricing
//! formula (e.g. an exponential bonding curve) even without wrapping.

use regex::Regex;
use truent_core::{Finding, Severity};

fn pricing_function_regex() -> Regex {
    Regex::new(r"(?i)function\s+(buy\w*|mint\w*|purchase\w*)\s*\(\s*(?:uint\d*\s+)?(\w+)").unwrap()
}

pub fn detect_unbounded_pricing_input(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let pricing_fn = pricing_function_regex();

    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(caps) = pricing_fn.captures(line) else {
            continue;
        };
        let fn_name = caps[1].to_string();
        let param = caps[2].to_string();

        let window_end = lines.len().min(i + 30);
        let window = lines[i..window_end].join("\n");

        let escaped = regex::escape(&param);
        let arithmetic_pattern =
            Regex::new(&format!(r"(?i){escaped}\s*\*|\*\s*{escaped}")).unwrap();
        if !arithmetic_pattern.is_match(&window) {
            continue;
        }

        let bound_pattern = Regex::new(&format!(r"(?i)require\s*\(\s*{escaped}\s*<=?")).unwrap();
        if bound_pattern.is_match(&window) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_unbounded_pricing_input".to_string(),
                Severity::Medium,
                file_path.to_string(),
                i + 1,
                0,
                format!(
                    "Function '{fn_name}' uses parameter '{param}' directly in pricing \
                     arithmetic with no upper-bound require() check — an extreme input can \
                     wrap or distort the computed price, the pattern behind the Truebit hack \
                     ($26.2M, Jan 2026), where an oversized input wrapped the computed mint \
                     price to near-zero"
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
    fn flags_mint_function_with_no_upper_bound() {
        let source = r#"
contract TruToken {
    function buyTRU(uint256 amount) external payable {
        uint256 price = basePrice * amount;
        require(msg.value >= price);
        _mint(msg.sender, amount);
    }
}
"#;
        let findings = detect_unbounded_pricing_input(source, "tru.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_unbounded_pricing_input"));
    }

    #[test]
    fn does_not_flag_when_upper_bound_checked() {
        let source = r#"
contract TruToken {
    function buyTRU(uint256 amount) external payable {
        require(amount <= MAX_PURCHASE);
        uint256 price = basePrice * amount;
        require(msg.value >= price);
        _mint(msg.sender, amount);
    }
}
"#;
        let findings = detect_unbounded_pricing_input(source, "safe_tru.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_unbounded_pricing_input"));
    }

    #[test]
    fn does_not_flag_when_param_not_used_in_arithmetic() {
        let source = r#"
contract TruToken {
    function mintTo(address recipient) external {
        _mint(recipient, 1);
    }
}
"#;
        let findings = detect_unbounded_pricing_input(source, "no_arith.sol");
        assert!(findings.is_empty());
    }
}
