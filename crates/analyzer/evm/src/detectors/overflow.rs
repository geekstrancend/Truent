//! Integer overflow/underflow detector for EVM smart contracts.
//!
//! Detects arithmetic operations on integer types without SafeMath or Solidity 0.8+
//! checked arithmetic. Solidity 0.8+ has built-in overflow checks, so this only
//! flags unsafe patterns in <0.8.0 code.

use crate::ast_types::*;
use crate::ast_walker::AstVisitor;
use crate::bytecode::Severity;
/// Placeholder for violation type until proper integration with truent_core
#[derive(Debug, Clone)]
struct Violation;
/// Detects integer overflow vulnerabilities
pub struct OverflowDetector<'a> {
    source: &'a str,
    file_name: &'a str,
    /// Accumulated violations
    pub violations: Vec<Violation>,
    solidity_version: Option<String>,
}

impl<'a> OverflowDetector<'a> {
    /// Create a new overflow detector
    pub fn new(source: &'a str, file_name: &'a str) -> Self {
        Self {
            source,
            file_name,
            violations: Vec::new(),
            solidity_version: None,
        }
    }

    /// Set the Solidity version (to skip checks if 0.8+)
    pub fn with_solidity_version(mut self, version: String) -> Self {
        self.solidity_version = Some(version);
        self
    }

    fn is_solidity_0_8_or_higher(&self) -> bool {
        self.solidity_version.as_ref().map_or(false, |v| {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() >= 2 {
                if let Ok(major) = parts[0].parse::<u32>() {
                    if major > 0 {
                        return true;
                    }
                    if major == 0 {
                        if let Ok(minor) = parts[1].parse::<u32>() {
                            return minor >= 8;
                        }
                    }
                }
            }
            false
        })
    }
}

impl<'a> AstVisitor for OverflowDetector<'a> {
    fn visit_binary_op(&mut self, op: &BinaryOperation, func: &FunctionDefinition, contract: &str) {
        // Only check arithmetic operators
        if !matches!(op.operator.as_str(), "+" | "-" | "*" | "**") {
            return;
        }

        // Skip if Solidity 0.8+ (has built-in overflow checks)
        if self.is_solidity_0_8_or_higher() {
            return;
        }

        // Check if operands are integer types
        let left_type = op.left_expression.get_type_string().unwrap_or_default();
        let right_type = op.right_expression.get_type_string().unwrap_or_default();

        let is_integer_op = (left_type.contains("uint") || left_type.contains("int"))
            && (right_type.contains("uint") || right_type.contains("int"));

        if !is_integer_op {
            return;
        }

        let (offset, _, _) = parse_src(&op.src);
        let line = offset_to_line(self.source, offset);
        let lines: Vec<&str> = self.source.lines().collect();

        let left_type_str = if !left_type.is_empty() {
            left_type.clone()
        } else {
            "integer".to_string()
        };

        self.violations.push(Violation {
            id: "evm_integer_overflow".to_string(),
            severity: Severity::High,
            title: "Integer Overflow/Underflow".to_string(),
            message: format!(
                "Arithmetic operation '{}' in '{}::{}' at line {} \
                 on type '{}' without overflow protection. \
                 Pre-0.8 Solidity does not check for overflow by default.",
                op.operator, contract, func.name, line, left_type_str,
            ),
            recommendation:
                "Use Solidity ^0.8.0 (has built-in overflow checks) or \
                 OpenZeppelin SafeMath for older versions. \
                 Example: amount.add(fee) instead of amount + fee."
                    .to_string(),
            location: Some(format!("{}:{}", self.file_name, line)),
            context: Some(extract_context(&lines, line)),
            cwe: Some("CWE-190".to_string()),
            references: vec![
                "https://docs.soliditylang.org/en/v0.8.0/080-breaking-changes.html#silent-changes-of-the-semantics"
                    .to_string(),
            ],
        });
    }
}

fn extract_context(lines: &[&str], line_num: usize) -> String {
    let line_idx = line_num.saturating_sub(1);
    let start = line_idx.saturating_sub(1);
    let end = std::cmp::min(line_idx + 2, lines.len());

    let mut context = String::new();
    for i in start..end {
        if i < lines.len() {
            let marker = if i == line_idx { ">>> " } else { "    " };
            context.push_str(&format!("{}{:4} {}\n", marker, i + 1, lines[i]));
        }
    }
    context
}
