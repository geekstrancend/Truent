//! Detector for LST (Liquid Staking Token) depeg collateral risk vulnerabilities.
//!
//! This detector identifies when protocols accept Liquid Staking Tokens (stETH, rsETH, etc.)
//! as collateral without depeg protection. This pattern was exploited in H47 KelpDAO/rsETH
//! ($292M, 2026) when rsETH depegged from ETH.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    /// List of known liquid staking tokens
    static ref KNOWN_LSTS: Vec<&'static str> = vec![
        "stETH", "rETH", "eETH", "rsETH", "mETH", "osETH",
        "sfrxETH", "swETH", "wstETH", "ankrETH", "cbETH", "frxETH",
        "lsETH", "SETH", "aETH",
    ];

    /// Regex to match borrow/deposit functions
    static ref BORROW_DEPOSIT_REGEX: Regex =
        Regex::new(r"(?i)function\s+(borrow|deposit|supply|collateralize)\s*[\(\{]").unwrap();

    /// Regex to match oracle price functions
    static ref PRICE_REGEX: Regex =
        Regex::new(r"(?i)(price|rate|exchange|value)").unwrap();
}

/// Detects LST depeg collateral risk in lending protocols.
pub fn detect_lst_depeg_collateral_risk(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let source_lower = source.to_lowercase();

    // Quick check: must have both lending keyword + LST keywords
    if !source_lower.contains("collateral") && !source_lower.contains("deposit") {
        return findings;
    }

    // Check for LST tokens in the code
    let mut lst_found = "";
    for lst in KNOWN_LSTS.iter() {
        if source_lower.contains(&lst.to_lowercase()) {
            lst_found = lst;
            break;
        }
    }

    if lst_found.is_empty() {
        return findings;
    }

    // Check if code has depeg protection - if it does, no findings
    let has_protection = has_depeg_protection(&source_lower);
    if has_protection {
        return findings;
    }

    // Found LST usage without protection - flag it
    for (line_num, line) in source.lines().enumerate() {
        let line_lower = line.to_lowercase();
        // Report first line that mentions deposit, borrow, transfer, collateral, or the LST token
        if line_lower.contains("deposit")
            || line_lower.contains("borrow")
            || line_lower.contains("transfer")
            || line_lower.contains("collateral")
            || line_lower.contains(&lst_found.to_lowercase())
        {
            findings.push(
                Finding::new(
                    "evm_lst_depeg".to_string(),
                    truent_core::Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    format!(
                        "Code accepts {} as collateral without depeg protection. \
                         LSTs can depeg from their underlying asset. \
                         H47 KelpDAO ($292M) was exploited when rsETH depegged, \
                         causing cascade liquidations. Add price band validation.",
                        lst_found
                    ),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H47".to_string())
                .with_metadata("exploit_name".to_string(), "KelpDAO LST Depeg".to_string())
                .with_metadata("loss".to_string(), "$292M".to_string())
                .with_metadata("year".to_string(), "2024".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "collateral_depeg".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Add depeg protection: price bands, fallback oracle, reduce LTV".to_string(),
                ),
            );
            break;
        }
    }

    findings
}

/// Extract function name from function declaration
#[allow(dead_code)]
fn extract_function_name(line: &str) -> String {
    if let Some(start) = line.find("function ") {
        let after_function = &line[start + 9..];
        if let Some(end) = after_function.find('(') {
            return after_function[..end].trim().to_string();
        }
    }
    "collateralize".to_string()
}

/// Check if function has depeg protection
fn has_depeg_protection(source_lower: &str) -> bool {
    // Patterns indicating depeg checks
    // NOTE: Removed bare "depeg" check because it matches comments like "No depeg check!"
    // Rely on specific protection patterns instead
    source_lower.contains("peg_band")
        || source_lower.contains("max_deviation")
        || source_lower.contains("deviation_tolerance")
        || source_lower.contains("peg_threshold")
        || source_lower.contains("price_band")
        || source_lower.contains("ratio_band")
        || source_lower.contains("95)")
        || source_lower.contains("90)")
        || source_lower.contains("* 95")
        || source_lower.contains("* 90")
        || source_lower.contains("require(") && source_lower.contains("price")
        || source_lower.contains("max_collateral_ratio")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerable_no_depeg_protection_kelp_dao() {
        let code = r#"
            contract KelpDAO {
                function deposit(uint256 amount, address token) external {
                    require(supportedTokens[token], "Unsupported token");
                    // Accepts rsETH without any depeg check!
                    if (token == rsETH) {
                        userCollateral[msg.sender][token] += amount;
                    }
                }
                
                function borrow(uint amount) external {
                    // Can borrow against rsETH that later depegs
                    require(getHealthFactor(msg.sender) >= 1.5);
                    _borrow(amount);
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "kelpDAO.sol");
        assert!(!findings.is_empty(), "Should detect LST depeg risk");
        assert!(findings[0].invariant_id.contains("lst_depeg"));
    }

    #[test]
    fn test_vulnerable_steth_no_band() {
        let code = r#"
            contract Lender {
                function deposit(uint amount, address token) external {
                    if (token == stETH) {
                        collateral[msg.sender] += amount;  // No depeg check!
                    }
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "lender.sol");
        assert!(
            !findings.is_empty(),
            "Should detect stETH without depeg protection"
        );
    }

    #[test]
    fn test_safe_with_depeg_band() {
        let code = r#"
            contract SafeLender {
                function deposit(uint amount, address token) external {
                    if (token == rsETH) {
                        uint ethPrice = getPrice(ETH);
                        uint lstPrice = getPrice(rsETH);
                        
                        // Check depeg band - allow max 5% deviation
                        require(lstPrice >= (ethPrice * 95) / 100, "LST depegged");
                        
                        collateral[msg.sender][token] += amount;
                    }
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "safe.sol");
        assert!(findings.is_empty(), "Should not flag with depeg band");
    }

    #[test]
    fn test_safe_with_peg_threshold() {
        let code = r#"
            contract SafeProtocol {
                uint constant PEG_BAND = 0.9;  // Allow 10% deviation
                
                function deposit(uint amount, address token) external {
                    require(supportedLSTs[token], "Not LST");
                    
                    uint deviation = calculateDeviation(token);
                    require(deviation < (1 - PEG_BAND), "Too much depeg");
                    
                    userCollateral[msg.sender][token] += amount;
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "safe.sol");
        assert!(findings.is_empty(), "Should detect peg_band check");
    }

    #[test]
    fn test_vulnerable_multiple_lsts() {
        let code = r#"
            contract MultiLST {
                function deposit(address[] calldata tokens, uint[] calldata amounts) external {
                    for (uint i = 0; i < tokens.length; i++) {
                        // Accept stETH, rETH, rsETH without any depeg checks
                        collateral[msg.sender] += amounts[i];
                    }
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "multilst.sol");
        assert!(
            !findings.is_empty(),
            "Should detect LST depeg risk for multiple tokens"
        );
    }

    #[test]
    fn test_non_lst_token() {
        let code = r#"
            contract TokenProtocol {
                function deposit(uint amount, address token) external {
                    if (token == USDC) {
                        collateral[msg.sender] += amount;  // USDC doesn't depeg like LSTs
                    }
                }
            }
        "#;

        let findings = detect_lst_depeg_collateral_risk(code, "token.sol");
        assert!(findings.is_empty(), "Should not flag non-LST tokens");
    }
}
