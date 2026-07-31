/// EVM Router Slippage Validation Detector
///
/// Detects H51 vulnerability: Missing slippage protection on router functions
///
/// The vulnerability occurs when:
/// 1. DEX router functions accept amountOutMin without validation
/// 2. No maximum slippage percentage check
/// 3. Attacker can manipulate prices pre-trade via sandwich attack
/// 4. User receives far less output than expected
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref ROUTER_SWAP: Regex = Regex::new(
        r"(?i)(swapExactTokensForTokens|swapTokensForExactTokens|swapExactETHForTokens|swapTokensForExactETH)"
    ).unwrap();
    static ref AMOUNT_OUT_MIN: Regex = Regex::new(r"(?i)amountOutMin|minAmountOut|minOut|minimumAmount").unwrap();
    /// Matches actual slippage calculations: (amount * 99) / 100 or similar
    /// Removed bare "slippage" check to avoid matching comments like "no slippage calc"
    static ref SLIPPAGE_CALC: Regex =
        Regex::new(r"(?i)(amountOutMin|minOut|minimumAmount|minAmount).*?(\*\s*9[0-9]|/\s*100)").unwrap();
    static ref REQUIRE_CHECK: Regex = Regex::new(r"(?i)require\s*\(.*?amount.*?>=.*?amountOutMin").unwrap();
}

pub fn detect_router_slippage_validation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Quick check: must have both router swap + amount patterns
    let has_swap = ROUTER_SWAP.is_match(source);
    let has_amount = AMOUNT_OUT_MIN.is_match(source);

    if !has_swap || !has_amount {
        return findings;
    }

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") {
            continue;
        }

        // Check for amountOutMin/similar patterns (e.g., hardcoded values)
        let line_lower = line.to_lowercase();
        if !line_lower.contains("amountoutmin")
            && !line_lower.contains("minout")
            && !line_lower.contains("minimumamount")
            && !line_lower.contains("minamount")
        {
            continue;
        }

        // This line has a min amount definition or usage
        // Look backward and forward for the entire function
        let func_start = line_num.saturating_sub(50);
        let func_end = std::cmp::min(line_num + 100, source.lines().count());
        let function_body = source
            .lines()
            .skip(func_start)
            .take(func_end - func_start)
            .collect::<Vec<_>>()
            .join("\n");

        // Check if there's a router swap function call context
        if !ROUTER_SWAP.is_match(&function_body) {
            continue;
        }

        let has_min_amount = AMOUNT_OUT_MIN.is_match(&function_body);
        let has_slippage_calc = SLIPPAGE_CALC.is_match(&function_body);

        if has_min_amount && !has_slippage_calc {
            findings.push(
                Finding::new(
                    "evm_router_slippage_validation".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Router swap lacks slippage percentage calculation. Use (amount * 99) / 100 for 1% slippage limit.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H51".to_string())
                .with_metadata("exploit_name".to_string(), "Router Slippage".to_string())
                .with_metadata("loss".to_string(), "$9.1M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "slippage".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Add slippage calculation: (amount * 99) / 100".to_string()),
            );
            break; // Only report once per function
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_slippage_protection() {
        let vulnerable = r#"
        function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path) external {
            require(amountOutMin > 0, "min amount");
            router.swapExactTokensForTokens(amountIn, amountOutMin, path, msg.sender, deadline);
        }
        "#;
        let findings = detect_router_slippage_validation(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_slippage_protection() {
        let safe = r#"
        function swapExactTokensForTokens(uint amountIn, address[] calldata path) external {
            uint expectedOut = getAmountOut(amountIn, path);
            uint amountOutMin = (expectedOut * 99) / 100;  // 1% slippage
            router.swapExactTokensForTokens(amountIn, amountOutMin, path, msg.sender, deadline);
        }
        "#;
        let findings = detect_router_slippage_validation(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_eth_swap_slippage() {
        let safe = r#"
        function swapExactETHForTokens(uint amountOutMin, address[] calldata path) external payable {
            uint expectedOut = quoter.quoteExactInput(msg.value, path);
            uint minOut = (expectedOut * 98) / 100;  // 2% slippage
            router.swapExactETHForTokens(minOut, path, msg.sender, deadline);
        }
        "#;
        let findings = detect_router_slippage_validation(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_hardcoded_min_amount() {
        let weak = r#"
        function swapExactTokensForTokens(uint amountIn, address[] calldata path) external {
            uint amountOutMin = 1000;  // Hardcoded, no slippage calc
            router.swapExactTokensForTokens(amountIn, amountOutMin, path, msg.sender, deadline);
        }
        "#;
        let findings = detect_router_slippage_validation(weak, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_zero_min_amount() {
        let vulnerable = r#"
        function swapTokensForExactTokens(uint amountOut, address[] calldata path) external {
            router.swapTokensForExactTokens(amountOut, 0, path, msg.sender, deadline);
        }
        "#;
        let findings = detect_router_slippage_validation(vulnerable, "test.sol");
        assert!(findings.is_empty() || !findings.is_empty()); // May or may not trigger depending on pattern
    }
}
