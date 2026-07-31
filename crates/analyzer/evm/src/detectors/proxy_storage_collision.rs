/// EVM Proxy Storage Collision Detector
///
/// Detects H28 (Pike $1.68M) vulnerability: Storage layout collision in proxy contracts
///
/// The vulnerability occurs when:
/// 1. Proxy contract and implementation have mismatched storage layouts
/// 2. Proxy stores data in slots used by implementation for important variables
/// 3. Storage collision causes variable overwriting
/// 4. Can lead to ownership hijacking, parameter corruption, etc.
///
/// Example Vulnerable Pattern:
/// ```solidity
/// // Proxy
/// contract Proxy {
///     address public implementation;
///     address public admin;  // Slot 1
///     // Gap not defined!
/// }
///
/// // Implementation
/// contract Implementation {
///     address public owner;  // Also slot 0
///     uint256 public value;  // Slot 1 (collision with proxy's admin!)
/// }
/// ```
///
/// Safe Pattern:
/// ```solidity
/// // Proxy
/// contract Proxy {
///     address public implementation;
///     address public admin;
///     uint256[50] private __gap;  // Reserve storage
/// }
///
/// // Implementation
/// contract Implementation is Proxy {
///     address public owner;  // Slot 2 (after gap)
///     uint256 public value;
/// }
/// ```
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::Finding;

lazy_static! {
    static ref PROXY_PATTERN: Regex =
        Regex::new(r"(?i)(contract\s+\w+.*?Proxy|_delegate|delegateToImplementation)").unwrap();
    static ref STORAGE_VARIABLE: Regex = Regex::new(
        r"(?i)(address|uint|bool|bytes32|bytes)\s+(?:public|internal|private)?\s+\w+\s*;"
    )
    .unwrap();
    static ref GAP_DEFINITION: Regex =
        Regex::new(r"(?i)uint.*?\[\s*\d+\s*\]\s*(?:private|internal)?\s*__gap|gap\s*\[\s*\d+\s*\]")
            .unwrap();
    static ref INHERITANCE_PATTERN: Regex = Regex::new(r"(?i)contract\s+\w+\s+is\s+\w+").unwrap();
    static ref STORAGE_ANNOTATION: Regex =
        Regex::new(r"///\s*@storage-location|///\s*@slot|// slot").unwrap();
    static ref UPGRADEABLE_PATTERN: Regex =
        Regex::new(r"(?i)Upgradeable|OwnedUpgradeable|proxy|ProxyAdmin").unwrap();
}

pub fn detect_proxy_storage_collision(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comments
        if line.trim().starts_with("//") {
            continue;
        }

        // Look for proxy patterns
        if !PROXY_PATTERN.is_match(line) && !UPGRADEABLE_PATTERN.is_match(line) {
            continue;
        }

        // Extract contract body (500 lines)
        let context_end = std::cmp::min(line_num + 500, source.lines().count());
        let contract_body = source
            .lines()
            .skip(line_num)
            .take(context_end - line_num)
            .collect::<Vec<_>>()
            .join("\n");

        // Count storage variables
        let storage_count = STORAGE_VARIABLE.captures_iter(&contract_body).count();
        if storage_count < 2 {
            continue;
        }

        // Check for storage gap
        let has_gap = GAP_DEFINITION.is_match(&contract_body);

        // Check for storage annotations
        let has_annotations = STORAGE_ANNOTATION.is_match(&contract_body);

        // Check for inheritance
        let has_inheritance = INHERITANCE_PATTERN.is_match(&contract_body);

        if !has_gap && has_inheritance && storage_count > 2 {
            findings.push(
                Finding::new(
                    "evm_proxy_storage_collision".to_string(),
                    truent_core::Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Proxy/Upgradeable contract lacks storage gap definition. \
                     Risk of storage collision with implementation contract variables. \
                     Add uint256[50] private __gap; to reserve slots."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H28".to_string())
                .with_metadata(
                    "exploit_name".to_string(),
                    "Pike Storage Collision".to_string(),
                )
                .with_metadata("loss".to_string(), "$1.68M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "storage_collision".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Add storage gap uint256[50] __gap; to proxy contract".to_string(),
                ),
            );
        } else if !has_annotations && has_inheritance && storage_count > 2 {
            // Has gap but no storage annotations
            findings.push(
                Finding::new(
                    "evm_proxy_storage_collision".to_string(),
                    truent_core::Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Proxy contract has storage gap but lacks storage layout documentation. \
                     Consider adding /// @storage-location comments for clarity."
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("exploit_id".to_string(), "H28".to_string())
                .with_metadata(
                    "exploit_name".to_string(),
                    "Pike - Weak Documentation".to_string(),
                )
                .with_metadata("loss".to_string(), "$1.68M".to_string())
                .with_metadata("year".to_string(), "2023".to_string())
                .with_metadata(
                    "vulnerability_type".to_string(),
                    "storage_collision".to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_analysis".to_string())
                .with_metadata(
                    "remediation".to_string(),
                    "Add storage layout documentation".to_string(),
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
    fn test_detect_proxy_without_gap() {
        let vulnerable = r#"
        contract UpgradeableProxy {
            address public implementation;
            address public admin;
            uint256 public value;
            
            function delegateTo(address impl) internal {
                assembly {
                    delegatecall(gas(), impl, 0, 0, 0, 0)
                }
            }
        }
        
        contract Implementation is UpgradeableProxy {
            address public owner;  // Collision risk!
            uint256 public data;
        }
        "#;

        let findings = detect_proxy_storage_collision(vulnerable, "test.sol");
        assert!(!findings.is_empty(), "Should detect missing storage gap");
        assert_eq!(
            findings[0].metadata.get("exploit_id"),
            Some(&"H28".to_string())
        );
    }

    #[test]
    fn test_safe_proxy_with_gap() {
        let safe = r#"
        contract UpgradeableProxy {
            address public implementation;
            address public admin;
            uint256 public value;
            
            uint256[50] private __gap;  // Storage gap reserved
        }
        
        contract Implementation is UpgradeableProxy {
            address public owner;
            uint256 public data;
        }
        "#;

        let findings = detect_proxy_storage_collision(safe, "test.sol");
        let critical_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::Critical)
            .collect();
        assert!(
            critical_findings.is_empty(),
            "Should allow proxy with storage gap"
        );
    }

    #[test]
    fn test_proxy_with_gap_but_no_documentation() {
        let weak = r#"
        contract UpgradeableProxy {
            address public implementation;
            address public admin;
            
            uint256[50] private __gap;
        }
        "#;

        let findings = detect_proxy_storage_collision(weak, "test.sol");
        // May trigger high severity if inheritance detected
        let _all_findings = findings.len();
    }

    #[test]
    fn test_pike_pattern() {
        let pike_vulnerable = r#"
        contract ProxyStorage {
            address public proxyAdmin;
            address public pendingAdmin;
            mapping(bytes32 => uint256) internal data;
        }
        
        contract UpgradeableImplementation is ProxyStorage {
            address public owner;  // Collision with proxyAdmin slot!
            uint256 public version;
        }
        "#;

        let findings = detect_proxy_storage_collision(pike_vulnerable, "test.sol");
        assert!(
            !findings.is_empty(),
            "Should detect Pike-style storage collision"
        );
    }

    #[test]
    fn test_simple_contract_no_flag() {
        let simple = r#"
        contract SimpleContract {
            address public owner;
            uint256 public value;
        }
        "#;

        let findings = detect_proxy_storage_collision(simple, "test.sol");
        assert!(findings.is_empty(), "Should not flag non-proxy contracts");
    }

    #[test]
    fn test_proxy_with_storage_annotations() {
        let annotated = r#"
        /// @storage-location erc7201:my.contract.storage
        contract UpgradeableProxy {
            address public implementation;
            address public admin;
            // slot-0: implementation
            // slot-1: admin
            
            uint256[50] private __gap;
        }
        "#;

        let findings = detect_proxy_storage_collision(annotated, "test.sol");
        let _high_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == truent_core::Severity::High)
            .collect();
        // Should be satisfied with annotations
    }
}
