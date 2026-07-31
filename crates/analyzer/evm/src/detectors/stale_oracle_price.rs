//! EVM Stale Oracle Price Detector
//!
//! Detects H63 vulnerability: a Chainlink-style `latestRoundData()` call
//! whose `updatedAt` return value is never checked against a staleness
//! threshold. If the underlying feed stalls (keeper outage, chain
//! congestion, an oracle-network incident), the contract keeps trusting an
//! arbitrarily old price with no on-chain signal that anything is wrong.
//!
//! This is a widely documented, frequently-audited pattern — Venus
//! Protocol was exploited on BSC during the LUNA crash via a related
//! oracle-answer validation gap, and "some protocols check the updatedAt
//! timestamp, most don't" per security research covering dozens of
//! audited codebases.

use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref LATEST_ROUND_DATA: Regex = Regex::new(r"(?i)\.latestRoundData\s*\(\s*\)").unwrap();
    static ref STALENESS_CHECK: Regex = Regex::new(
        r"(?i)(block\.timestamp|block\.number)\s*-\s*\w*updatedat|updatedat\s*[<>]|require\([^)]*updatedat"
    )
    .unwrap();
}

pub fn detect_stale_oracle_price(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("//") || !LATEST_ROUND_DATA.is_match(line) {
            continue;
        }

        let window_end = lines.len().min(i + 15);
        let window = lines[i..window_end].join("\n").to_lowercase();
        if STALENESS_CHECK.is_match(&window) {
            continue;
        }

        findings.push(
            Finding::new(
                "evm_stale_oracle_price".to_string(),
                Severity::High,
                file_path.to_string(),
                i + 1,
                0,
                "latestRoundData() is called but its updatedAt timestamp is never checked \
                 against a staleness threshold — if the feed stalls, this contract keeps \
                 trusting an arbitrarily old price. Require \
                 (block.timestamp - updatedAt) <= maxAge before using the returned price"
                    .to_string(),
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
    fn flags_latest_round_data_without_staleness_check() {
        let source = r#"
contract Lending {
    function getPrice() public view returns (int256) {
        (, int256 price, , , ) = priceFeed.latestRoundData();
        return price;
    }
}
"#;
        let findings = detect_stale_oracle_price(source, "lending.sol");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "evm_stale_oracle_price"));
    }

    #[test]
    fn does_not_flag_when_staleness_checked() {
        let source = r#"
contract Lending {
    function getPrice() public view returns (int256) {
        (, int256 price, , uint256 updatedAt, ) = priceFeed.latestRoundData();
        require(block.timestamp - updatedAt <= 3600, "stale price");
        return price;
    }
}
"#;
        let findings = detect_stale_oracle_price(source, "safe_lending.sol");
        assert!(!findings
            .iter()
            .any(|f| f.invariant_id == "evm_stale_oracle_price"));
    }
}
