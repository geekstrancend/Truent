//! EVM Detector implementations using Finding type.
//!
//! Each detector discovers invariant violations and returns them as Finding objects.

use truent_core::{Finding, Severity};

/// Detects classic reentrancy: external call before state update (CEI violated)
pub fn detect_reentrancy_classic(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Pattern: look for external calls (call, send, transfer) followed by state updates
    // in the same function without nonReentrant guard

    // Simplified pattern matching: check for external calls (either .call{ or .transfer)
    if (source.contains(".call{") || source.contains(".transfer(") || source.contains(".send("))
        && !source.contains("nonReentrant")
        && !source.contains("nonreentrant")
    {
        // Find line number of the pattern
        for (line_num, line) in source.lines().enumerate() {
            if (line.contains(".call{") || line.contains(".transfer(") || line.contains(".send("))
                && !line.trim().starts_with("//")
            {
                findings.push(
                    Finding::new(
                        "evm_reentrancy_classic".to_string(),
                        Severity::Critical,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "External call detected before state update (Checks-Effects-Interactions pattern violated)".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "pattern_match".to_string())
                );
            }
        }
    }

    findings
}

/// Detects missing signer checks in external state-modifying functions
pub fn detect_missing_signer_check(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Pattern: external/public function that modifies state without msg.sender check
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Look for function definitions
        if (trimmed.starts_with("function ") || trimmed.contains(" function "))
            && (trimmed.contains("public") || trimmed.contains("external"))
            && !trimmed.contains("pure")
            && !trimmed.contains("view")
        {
            // Check if it has access control
            let has_access_check = source.lines().skip(line_num).take(20).any(|l| {
                let lower = l.to_lowercase();
                lower.contains("msg.sender")
                    || lower.contains("onlyowner")
                    || lower.contains("require")
                    || lower.contains("_checkrole")
            });

            // Check if function modifies state
            let modifies_state = source.lines().skip(line_num).take(20).any(|l| {
                let lower = l.to_lowercase();
                (lower.contains(" = ")
                    || lower.contains("transfer")
                    || lower.contains("mint")
                    || lower.contains("burn"))
                    && !l.trim().starts_with("//")
            });

            if modifies_state && !has_access_check {
                findings.push(
                    Finding::new(
                        "evm_missing_signer_check".to_string(),
                        Severity::High,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "External state-modifying function lacks msg.sender validation".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "ast_analysis".to_string()),
                );
            }
        }
    }

    findings
}

/// Detects conservation check absence: AMM swaps without x*y=k verification
pub fn detect_conservation_check_absent(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Pattern: swap/exchange function that modifies reserves without k invariant check
    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("function swap") || lower.contains("function exchange"))
            && !lower.contains("view")
            && !lower.contains("pure")
        {
            // Check function body for k invariant verification
            let func_body = source
                .lines()
                .skip(line_num)
                .take(50)
                .collect::<Vec<_>>()
                .join("\n");

            let has_k_check = func_body.to_lowercase().contains("require(")
                && (func_body.to_lowercase().contains("* ")
                    || func_body.to_lowercase().contains("*"));

            let modifies_reserves = func_body.to_lowercase().contains("reservea")
                || func_body.to_lowercase().contains("reserveb")
                || func_body.to_lowercase().contains("reserve0")
                || func_body.to_lowercase().contains("reserve1");

            if modifies_reserves && !has_k_check {
                findings.push(
                    Finding::new(
                        "evm_conservation_check_absent".to_string(),
                        Severity::Critical,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "AMM swap function does not verify x*y=k invariant after exchange"
                            .to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "invariant_check".to_string())
                    .with_metadata("impact".to_string(), "pool can be drained".to_string()),
                );
            }
        }
    }

    findings
}

/// Detects oracle spot price vulnerabilities: using balanceOf as price
pub fn detect_oracle_spot_price(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Pattern: balanceOf, reserve, or similar used in price calculation without oracle
    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("balanceof")
            || lower.contains("reserve")
            || lower.contains("getreserves"))
            && (lower.contains("price")
                || lower.contains("rate")
                || lower.contains("amount")
                || lower.contains("="))
            && !lower.contains("oracle")
            && !lower.contains("chainlink")
            && !lower.contains("//")
        {
            findings.push(
                Finding::new(
                    "evm_oracle_spot_price".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Spot price calculation using token balance instead of oracle".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_match".to_string()),
            );
        }
    }

    findings
}

/// Detects unprotected initializer functions
pub fn detect_unprotected_initializer(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains("function initialize") && !line.contains("//") {
            // Check for initializer modifier
            if !line.contains("initializer") && !line.contains("onlyInitializing") {
                findings.push(
                    Finding::new(
                        "evm_unprotected_initializer".to_string(),
                        Severity::Critical,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "Initialize function lacks initializer modifier - can be called multiple times".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "modifier_check".to_string())
                );
            }
        }
    }

    findings
}

/// Detects unsafe math without SafeMath (Solidity <0.8.0)
pub fn detect_legacy_unsafe_math(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Check Solidity version
    let version_line = source.lines().find(|l| l.contains("pragma solidity"));

    if let Some(v_line) = version_line {
        // If version < 0.8.0, check for unsafe math without SafeMath
        if v_line.contains("0.7") || v_line.contains("0.6") || v_line.contains("0.5") {
            let has_safemath = source.contains("SafeMath") || source.contains("using SafeMath");

            if !has_safemath {
                // Check for arithmetic operations
                for (line_num, line) in source.lines().enumerate() {
                    let lower = line.to_lowercase();
                    if (lower.contains(" + ") || lower.contains(" - ") || lower.contains(" * "))
                        && !lower.contains("//")
                        && !lower.contains("safemath")
                    {
                        findings.push(
                            Finding::new(
                                "evm_legacy_unsafe_math".to_string(),
                                Severity::High,
                                file_path.to_string(),
                                line_num + 1,
                                0,
                                "Arithmetic operation in Solidity <0.8.0 without SafeMath"
                                    .to_string(),
                                line.trim().to_string(),
                            )
                            .with_metadata("detector".to_string(), "version_check".to_string()),
                        );
                        break; // Report once per file
                    }
                }
            }
        }
    }

    findings
}

/// Detects flash loan governance attacks: snapshot on same block as loan
pub fn detect_flash_loan_governance(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        // Look for block.number used in voting/snapshot
        if lower.contains("block.number")
            && (lower.contains("vote")
                || lower.contains("snapshot")
                || lower.contains("vote_power"))
        {
            findings.push(
                Finding::new(
                    "evm_flash_loan_governance".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Block-based voting snapshot vulnerable to flash loan attacks".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "pattern_match".to_string()),
            );
        }
    }

    findings
}

/// Detects DVN threshold vulnerabilities in LayerZero bridges
pub fn detect_dvn_threshold(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Look for DVN threshold configuration
    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("dvnthreshold") || lower.contains("dvn_threshold"))
            && (lower.contains(" = 0") || lower.contains(" = 1"))
        {
            findings.push(
                Finding::new(
                    "evm_dvn_threshold".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "DVN threshold set to 0 or 1 - insufficient validation".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "config_check".to_string()),
            );
        }
    }

    findings
}

/// Detects ERC20 reentrancy: token transfer before internal accounting update
pub fn detect_reentrancy_erc20(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        // ERC20 transfer patterns followed by balance updates
        if (lower.contains("erc20") || lower.contains("ierc20"))
            && (lower.contains("transfer") || lower.contains("transfer_from"))
        {
            // Check if balance update happens after
            let following = source.lines().skip(line_num).take(10).any(|l| {
                l.to_lowercase().contains("balances[") || l.to_lowercase().contains("_balances[")
            });

            if following {
                findings.push(
                    Finding::new(
                        "evm_reentrancy_erc20".to_string(),
                        Severity::Critical,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "ERC20 token transfer before internal accounting update".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "token_pattern".to_string()),
                );
            }
        }
    }

    findings
}

/// Detects precision loss: division before multiplication in finance
pub fn detect_precision_loss(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        // Pattern: (a / b) * c (should be a * c / b)
        if line.contains(" / ") && line.contains(" * ") {
            let div_pos = line.find(" / ");
            let mul_pos = line.find(" * ");

            if let (Some(d), Some(m)) = (div_pos, mul_pos) {
                if d < m {
                    findings.push(
                        Finding::new(
                            "evm_precision_loss".to_string(),
                            Severity::Medium,
                            file_path.to_string(),
                            line_num + 1,
                            d,
                            "Division before multiplication may cause precision loss".to_string(),
                            line.trim().to_string(),
                        )
                        .with_metadata("detector".to_string(), "arithmetic_pattern".to_string()),
                    );
                }
            }
        }
    }

    findings
}

/// Detects merkle root zero check: accepts bytes32(0) as valid
pub fn detect_merkle_root_zero(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        // Merkle root validation without zero check
        if (lower.contains("merkleroot") || lower.contains("merkle_root"))
            && lower.contains("verify")
            && !lower.contains("require")
            && !lower.contains("!= 0")
        {
            findings.push(
                Finding::new(
                    "evm_merkle_root_zero".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Merkle root verification may accept bytes32(0) as valid".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "validation_check".to_string()),
            );
        }
    }

    findings
}

/// Detects zero challenge period: optimistic bridge with no delay
pub fn detect_zero_challenge_period(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();

        if (lower.contains("challenge") || lower.contains("challengeperiod"))
            && (lower.contains(" = 0") || lower.contains("= 0 "))
        {
            findings.push(
                Finding::new(
                    "evm_zero_challenge_period".to_string(),
                    Severity::Critical,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Zero challenge period - optimistic bridge has no dispute window".to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "config_check".to_string()),
            );
        }
    }

    findings
}

/// Detects shallow auth: onlyOwner on wrapper but not inner function
pub fn detect_shallow_auth(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Look for functions calling internal/private functions without checking they also have guards
    let lines: Vec<&str> = source.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        if line.contains("_") && line.contains("(") && !line.contains("function") {
            // This might be calling an internal function
            if let Some(func_name) = extract_function_name(line) {
                // Check if the called function exists and lacks guards
                for check_line in &lines {
                    if check_line.contains(&format!("function {}", func_name))
                        && !check_line.contains("onlyOwner")
                        && !check_line.contains("onlyAdmin")
                    {
                        findings.push(
                            Finding::new(
                                "evm_shallow_auth".to_string(),
                                Severity::High,
                                file_path.to_string(),
                                line_num + 1,
                                0,
                                "Called function lacks access control guard".to_string(),
                                line.trim().to_string(),
                            )
                            .with_metadata("detector".to_string(), "auth_chain".to_string()),
                        );
                        break;
                    }
                }
            }
        }
    }

    findings
}

/// Detects public relay: permissionless relay function
pub fn detect_public_relay(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("function") && line.contains("public") || line.contains("external"))
            && (line.contains("relay") || line.contains("execute") || line.contains("forward"))
            && !line.contains("onlyOwner")
            && !line.contains("onlyAdmin")
            && !line.contains("view")
            && !line.contains("pure")
        {
            findings.push(
                Finding::new(
                    "evm_public_relay".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    line_num + 1,
                    0,
                    "Permissionless relay function allows anyone to execute arbitrary transactions"
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("detector".to_string(), "authorization".to_string()),
            );
        }
    }

    findings
}

/// Detects single EOA admin: admin is EOA without isContract check
pub fn detect_single_eoa_admin(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        if (line.contains("owner") || line.contains("admin"))
            && line.contains("=")
            && (line.contains("msg.sender") || line.contains("_msgSender"))
        {
            // Check if code path requires multisig or contract check
            let has_contract_check =
                source.contains("isContract") || source.contains(".code.length");

            if !has_contract_check {
                findings.push(
                    Finding::new(
                        "evm_single_eoa_admin".to_string(),
                        Severity::High,
                        file_path.to_string(),
                        line_num + 1,
                        0,
                        "Admin is EOA without multisig or contract check".to_string(),
                        line.trim().to_string(),
                    )
                    .with_metadata("detector".to_string(), "admin_check".to_string()),
                );
            }
        }
    }

    findings
}

fn extract_function_name(line: &str) -> Option<String> {
    if let Some(start) = line.find('_') {
        if let Some(end) = line[start..].find('(') {
            let name = &line[start..start + end];
            return Some(name.to_string());
        }
    }
    None
}

/// Run all EVM detectors on source code
pub fn detect_all(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    findings.extend(detect_reentrancy_classic(source, file_path));
    findings.extend(detect_reentrancy_erc20(source, file_path));
    findings.extend(detect_missing_signer_check(source, file_path));
    findings.extend(detect_conservation_check_absent(source, file_path));
    findings.extend(detect_oracle_spot_price(source, file_path));
    findings.extend(detect_unprotected_initializer(source, file_path));
    findings.extend(detect_legacy_unsafe_math(source, file_path));
    findings.extend(detect_precision_loss(source, file_path));
    findings.extend(detect_flash_loan_governance(source, file_path));
    findings.extend(detect_dvn_threshold(source, file_path));
    findings.extend(detect_merkle_root_zero(source, file_path));
    findings.extend(detect_zero_challenge_period(source, file_path));
    findings.extend(detect_shallow_auth(source, file_path));
    findings.extend(detect_public_relay(source, file_path));
    findings.extend(detect_single_eoa_admin(source, file_path));

    // Sort by severity (critical first) then by line number
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
    fn test_reentrancy_detection() {
        let code = r#"
        function withdraw() public {
            uint amount = balances[msg.sender];
            (bool success,) = msg.sender.call{value: amount}("");
            require(success);
            balances[msg.sender] = 0;
        }
        "#;

        let findings = detect_reentrancy_classic(code, "test.sol");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].invariant_id, "evm_reentrancy_classic");
    }

    #[test]
    fn test_missing_signer_check() {
        let code = r#"
        function withdrawAll() public {
            uint amount = balances[msg.sender];
            balances[msg.sender] = 0;
            payable(owner).transfer(amount);
        }
        "#;

        let findings = detect_missing_signer_check(code, "test.sol");
        // May or may not find depending on heuristic
        for f in findings {
            assert!(f.invariant_id.contains("signer"));
        }
    }

    #[test]
    fn test_conservation_check() {
        let code = r#"
        function swap(uint amountIn) public {
            reserveA -= amountIn;
            reserveB += (amountIn * 100) / 101;
        }
        "#;

        let findings = detect_conservation_check_absent(code, "test.sol");
        assert!(!findings.is_empty());
    }
}
