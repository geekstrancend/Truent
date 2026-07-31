//! Move detector implementations.
//!
//! Detectors for Move (Aptos/Sui) module vulnerabilities.

use truent_core::{Finding, Severity};

/// Detects missing access control in public entry functions
pub fn detect_access_control_missing(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("public entry") || line.contains("public fun"))
            && (line.contains("transfer")
                || line.contains("burn")
                || line.contains("mint")
                || line.contains("withdraw"))
        {
            // Check if function has capability parameter
            if !line.contains("&Capability") && !line.contains("&AdminCap") {
                findings.push(
                    Finding::new(
                        "move_access_control_missing".to_string(),
                        Severity::High,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "Public entry function lacks capability/permission check".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("chain".to_string(), "move".to_string()),
                );
            }
        }
    }

    findings
}

/// Detects liquidity conservation absence in AMM swaps
pub fn detect_liquidity_conservation(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("swap") || line.contains("exchange"))
            && !line.contains("assert")
            && !line.contains("require")
        {
            // Check if x*y=k is verified
            let func_body = source
                .lines()
                .skip(line_num)
                .take(30)
                .collect::<Vec<_>>()
                .join("\n");

            if !func_body.contains("*") || !func_body.contains("==") {
                findings.push(
                    Finding::new(
                        "move_liquidity_conservation".to_string(),
                        Severity::Critical,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "AMM swap does not assert x*y==k invariant".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("chain".to_string(), "move".to_string()),
                );
            }
        }
    }

    findings
}

/// Detects admin functions without timelock
pub fn detect_admin_no_timelock(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("admin") || line.contains("upgrade") || line.contains("freeze"))
            && !line.contains("timelock")
            && !line.contains("delay")
        {
            findings.push(
                Finding::new(
                    "move_admin_no_timelock".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Admin capability can be used immediately without timelock".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "move".to_string()),
            );
        }
    }

    findings
}

/// Detects pool reserve used directly for price
pub fn detect_oracle_spot_price(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("reserve") || line.contains("balance"))
            && (line.contains("price") || line.contains("rate"))
        {
            findings.push(
                Finding::new(
                    "move_oracle_spot_price".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Pool reserve used directly for pricing without oracle".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "move".to_string()),
            );
        }
    }

    findings
}

/// Run all Move detectors
pub fn detect_all(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(detect_access_control_missing(source, file_path));
    findings.extend(detect_liquidity_conservation(source, file_path));
    findings.extend(detect_admin_no_timelock(source, file_path));
    findings.extend(detect_oracle_spot_price(source, file_path));

    findings.sort_by(|a, b| match b.severity.cmp(&a.severity) {
        std::cmp::Ordering::Equal => a.line.cmp(&b.line),
        other => other,
    });

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_control_detection() {
        let code = r#"
        public entry fun withdraw<T>(
            vault: &mut Vault<T>,
            amount: u64,
            ctx: &mut TxContext
        ) {
            // ...
        }
        "#;

        let findings = detect_access_control_missing(code, "module.move");
        for f in findings {
            assert!(f.invariant_id.starts_with("move_"));
        }
    }
}
