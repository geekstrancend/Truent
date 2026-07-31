/// EVM Arithmetic Rounding Detector
///
/// Detects H44 vulnerability: Incorrect rounding in division operations
///
/// The vulnerability occurs when:
/// 1. Division rounds down causing precision loss
/// 2. Accumulates over time to significant loss
/// 3. No rounding protection using mulDiv or similar
/// 4. Can be weaponized in share calculations
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref DIVISION_OP: Regex =
        Regex::new(r"(?i)(\w+)\s*=\s*\(.*?\)\s*/\s*(\w+|[0-9]+)").unwrap();
    static ref MULDIV_SAFE: Regex =
        Regex::new(r"(?i)(mulDiv|FixedPointMathLib|PRBMath|Math\.(mul|div))").unwrap();
    static ref ROUNDING_UP: Regex =
        Regex::new(r"(?i)(\+\s*\w+\s*-\s*1\)|ceil|roundUp|-\s*1\)\s*/)").unwrap();
    static ref SHARES_CALCULATION: Regex =
        Regex::new(r"(?i)(shares|amount)\s*=.*?(\*|/|totalSupply|totalAssets)").unwrap();
}

pub fn detect_arithmetic_rounding(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim().starts_with("//") || !DIVISION_OP.is_match(line) {
            continue;
        }

        // Look specifically for vulnerable patterns
        if !SHARES_CALCULATION.is_match(line) {
            continue;
        }

        let has_safe_math = MULDIV_SAFE.is_match(line);
        let has_rounding_up = ROUNDING_UP.is_match(line);

        if !has_safe_math && !has_rounding_up {
            findings.push(
                Finding::new(
                    "evm_arithmetic_rounding".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Share/amount calculation uses simple division without rounding protection. Use mulDiv or add ceil rounding.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H44".to_string())
                .with_metadata("exploit_name".to_string(), "Arithmetic Rounding".to_string())
                .with_metadata("loss".to_string(), "$4.3M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "rounding_vulnerability".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Use mulDiv or add rounding: (amount + divisor - 1) / divisor".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_division_rounding() {
        let vulnerable = r#"
        shares = (amount * totalSupply()) / totalAssets();
        "#;
        let findings = detect_arithmetic_rounding(vulnerable, "test.sol");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_muldiv_safe() {
        let safe = r#"
        shares = mulDiv(amount, totalSupply(), totalAssets());
        "#;
        let findings = detect_arithmetic_rounding(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_fixed_point_math() {
        let safe = r#"
        shares = FixedPointMathLib.mulDiv(amount, totalSupply(), totalAssets());
        "#;
        let findings = detect_arithmetic_rounding(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_ceil_rounding() {
        let safe = r#"
        shares = (amount * totalSupply() + totalAssets() - 1) / totalAssets();
        "#;
        let findings = detect_arithmetic_rounding(safe, "test.sol");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_prb_math() {
        let safe = r#"
        shares = PRBMath.mulDiv(amount, totalSupply(), totalAssets());
        "#;
        let findings = detect_arithmetic_rounding(safe, "test.sol");
        assert!(findings.is_empty());
    }
}
