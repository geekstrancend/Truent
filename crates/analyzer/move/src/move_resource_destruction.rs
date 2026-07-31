/// Move Resource Destruction Detector
///
/// Detects H51 vulnerability: Improper resource destruction/cleanup
///
/// The vulnerability occurs when:
/// 1. Resources not properly dropped/destroyed
/// 2. Undroppable resources cause compilation bypass
/// 3. Resource leaks or infinite loops
/// 4. Can freeze protocol or cause state corruption
///
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref RESOURCE_STRUCT: Regex =
        Regex::new(r"(?i)struct.*has\s+key|struct.*has\s+store|#\[resource\]").unwrap();
    /// Matches move_from function calls that extract resources
    static ref MOVE_FROM: Regex = Regex::new(r"(?i)move_from\s*<").unwrap();
    /// Matches drop/destroy functions - use word boundaries to avoid matching "destroyed"
    static ref DROP_CALL: Regex = Regex::new(r"(?i)drop\s*\(|drop\s*<|destroy\s*\(").unwrap();
    /// Matches move_to transfers (moving resource to new location)
    static ref MOVE_TO: Regex = Regex::new(r"(?i)move_to\s*<").unwrap();
    /// Matches unpacking/destructuring patterns like: let Token { ... } or let Coin { x: _ }
    static ref UNPACK_PATTERN: Regex = Regex::new(r"(?i)let\s+\w+\s*\{").unwrap();
    /// Matches explicit discard: _ = variable or let _
    static ref DISCARD_PATTERN: Regex = Regex::new(r"_\s*=\s*\w+|let\s+_").unwrap();
}

pub fn detect_move_resource_destruction(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comment lines
        if line.trim().starts_with("//") {
            continue;
        }
        // Skip lines that don't have resource definitions
        if !RESOURCE_STRUCT.is_match(line) {
            continue;
        }

        let context_end = std::cmp::min(line_num + 300, source.lines().count());
        let function_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        let has_move_from = MOVE_FROM.is_match(&function_body);
        let has_drop = DROP_CALL.is_match(&function_body);
        let has_move_to = MOVE_TO.is_match(&function_body);
        let has_unpack = UNPACK_PATTERN.is_match(&function_body);
        let has_discard = DISCARD_PATTERN.is_match(&function_body);

        // Check if resource is extracted but not properly handled
        // It must be either: destroyed (drop), transferred (move_to), unpacked, or discarded
        if has_move_from && !has_drop && !has_move_to && !has_unpack && !has_discard {
            findings.push(
                Finding::new(
                    "move_resource_destruction".to_string(),
                    truent_core::Severity::Medium,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Resource moved without proper destruction. Ensure resources are destroyed or discarded with _.".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H51".to_string())
                .with_metadata("exploit_name".to_string(), "Move Resource Destruction".to_string())
                .with_metadata("loss".to_string(), "$1.8M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata("vulnerability_type".to_string(), "resource_leak".to_string())
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata("remediation".to_string(), "Destroy resources or use _ = resource to discard".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_not_destroyed() {
        let vulnerable = r#"
        struct Coin has key {
            value: u64,
        }
        
        fun extract_coin(account: &signer) {
            let coin = move_from<Coin>(account);
            // Coin not destroyed!
        }
        "#;
        let findings = detect_move_resource_destruction(vulnerable, "test.move");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_resource_destroyed() {
        let safe = r#"
        struct Coin has key, drop {
            value: u64,
        }
        
        fun extract_coin(account: &signer) {
            let coin = move_from<Coin>(account);
            drop(coin);  // Properly destroyed
        }
        "#;
        let findings = detect_move_resource_destruction(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_resource_discarded() {
        let safe = r#"
        struct Token has key {
            amount: u64,
        }
        
        fun cleanup(signer_ref: &signer) {
            let token: Token = move_from<Token>(signer_ref);
            let Token { amount: _ } = token;  // Unpacked and discarded
        }
        "#;
        let findings = detect_move_resource_destruction(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_resource_with_underscore() {
        let safe = r#"
        struct Data has key {
            value: u64,
        }
        
        fun remove_data(account: &signer) {
            let data = move_from<Data>(account);
            _ = data;  // Explicitly discarded
        }
        "#;
        let findings = detect_move_resource_destruction(safe, "test.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_resource_transferred() {
        let safe = r#"
        struct Asset has key, store {
            id: u64,
        }
        
        fun transfer_asset(from: &signer, to: address) {
            let asset = move_from<Asset>(from);
            move_to<Asset>(to, asset);  // Moved to new location
        }
        "#;
        let findings = detect_move_resource_destruction(safe, "test.move");
        assert!(findings.is_empty());
    }
}
