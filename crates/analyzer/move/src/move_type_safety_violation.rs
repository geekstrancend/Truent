/// Move Type Safety Violation Detector
///
/// Detects H52 vulnerability: Type safety violations in Move
///
/// The vulnerability occurs when:
/// 1. Generic types misused with incompatible assets
/// 2. Resource capabilities mixed incorrectly
/// 3. Unsafe casting or type confusion
/// 4. Can lead to fund loss or privilege escalation
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    /// Matches generic type parameters: <T>, <Token>, <T:, <Token:, or generic type keyword
    static ref GENERIC_TYPE: Regex = Regex::new(r"(?i)<T|<Token|generic\s+type").unwrap();
    static ref UNSAFE_AS: Regex = Regex::new(r"(?i)as\s*(&?[A-Z]\w*<|T|Coin)").unwrap();
    static ref RESOURCE_CAST: Regex = Regex::new(r"(?i)(Coin|Asset|Token)<.*>\s*as\s*").unwrap();
    /// Matches explicit type annotations like: let x: Type<T> or let x: &Type<T>
    static ref TYPE_ANNOTATION: Regex = Regex::new(r"(?i)let\s+\w+\s*:\s*&?\w+<").unwrap();
}

pub fn detect_move_type_safety_violation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Check for generic types and unsafe casts across entire source
    let has_generic = GENERIC_TYPE.is_match(source);
    let has_type_annotation = TYPE_ANNOTATION.is_match(source);

    // If generics are used without type annotations and unsafe casts exist
    if has_generic && !has_type_annotation {
        for (line_num, line) in source.lines().enumerate() {
            // Skip comment lines
            if line.trim().starts_with("//") {
                continue;
            }
            // Report on lines with unsafe type casting
            if !UNSAFE_AS.is_match(line) {
                continue;
            }
            findings.push(
                Finding::new(
                    "move_type_safety_violation".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Unsafe type casting in Move. Verify type safety with explicit annotations."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H52".to_string())
                .with_metadata("exploit_name".to_string(), "Move Type Safety".to_string())
                .with_metadata("loss".to_string(), "$2.1M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "type_safety".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Use explicit type annotations and avoid unsafe casts".to_string(),
                ),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsafe_type_cast() {
        let vulnerable = r#"
        fun swap<T>(asset: T) {
            let coin = asset as Coin<USDC>;  // Unsafe cast!
            transfer(coin);
        }
        "#;
        let findings = detect_move_type_safety_violation(vulnerable, "test.move");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_safe_generic_annotation() {
        let safe = r#"
        fun swap<T: Coin>(asset: T) {
            let amount: u64 = get_balance<T>(&asset);
            deposit<T>(asset, amount);
        }
        "#;
        let findings = detect_move_type_safety_violation(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_explicit_type_annotation() {
        let safe = r#"
        fun process<T>(value: T) {
            let x: CoinValue<T> = read_value(&value);
            use_value(&x);
        }
        "#;
        let findings = detect_move_type_safety_violation(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_resource_constraint() {
        let safe = r#"
        fun transfer<T: drop>(coin: T) {
            let value: Coin<T> = coin;
            send_coin(value);
        }
        "#;
        let findings = detect_move_type_safety_violation(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_generic_function_call() {
        let safe = r#"
        fun deposit_token<CoinType>(
            user_account: &signer,
            amount: u64,
        ) {
            let coin = coin::withdraw<CoinType>(user_account, amount);
            register_coin<CoinType>();
        }
        "#;
        let findings = detect_move_type_safety_violation(safe, "test.move");
        assert!(findings.is_empty());
    }
}
