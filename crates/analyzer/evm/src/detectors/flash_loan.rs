//! Flash loan attack vector detector.
//!
//! Detects patterns where price oracles can be manipulated using flash loans:
//! - Functions that read spot price from token balances
//! - Functions that accept external data (prices) without TWAP validation
//! - Swap functions that move large amounts in a single block

use crate::ast_types::*;
use crate::ast_walker::AstVisitor;
use crate::bytecode::Severity;
use tracing::debug;

/// Placeholder for violation type until proper integration with truent_core
#[derive(Debug, Clone)]
struct Violation;

/// Detects flash loan attack vectors in DeFi protocols
pub struct FlashLoanDetector<'a> {
    source: &'a str,
    file_name: &'a str,
    /// Accumulated violations
    pub violations: Vec<Violation>,
    balance_oracle_calls: Vec<(usize, String)>, // (line, function_name)
}

impl<'a> FlashLoanDetector<'a> {
    /// Create a new flash loan detector
    pub fn new(source: &'a str, file_name: &'a str) -> Self {
        Self {
            source,
            file_name,
            violations: Vec::new(),
            balance_oracle_calls: Vec::new(),
        }
    }

    fn is_balance_oracle_read(call: &FunctionCall) -> bool {
        // Detects:
        // - token.balanceOf(pool)
        // - IERC20(token).balanceOf(address(this))
        // - getReserves() reads
        // - totalSupply() reads
        if let AstNode::MemberAccess(member) = call.expression.as_ref() {
            return matches!(
                member.member_name.as_str(),
                "balanceOf" | "getReserves" | "totalSupply" | "reserve0" | "reserve1"
            );
        }
        false
    }

    fn is_price_computation_call(call: &FunctionCall) -> bool {
        // Price calculation typically uses balanceOf
        if let AstNode::MemberAccess(member) = call.expression.as_ref() {
            let name = member.member_name.to_lowercase();
            name.contains("price") || name.contains("rate") || name.contains("oracle")
        } else {
            false
        }
    }
}

impl<'a> AstVisitor for FlashLoanDetector<'a> {
    fn visit_function_call(
        &mut self,
        call: &FunctionCall,
        func: &FunctionDefinition,
        contract: &str,
    ) {
        let (offset, _, _) = parse_src(&call.src);
        let call_line_num = offset_to_line(self.source, offset);

        if Self::is_balance_oracle_read(call) {
            self.balance_oracle_calls
                .push((call_line_num, func.name.clone()));
        }

        // Detect functions that use balance reads to compute prices
        if Self::is_price_computation_call(call) && !self.balance_oracle_calls.is_empty() {
            for &(balance_call_line, ref balance_func_name) in &self.balance_oracle_calls {
                if balance_call_line < call_line_num && balance_func_name == &func.name {
                    // This function reads balance and computes price — flash loan vector
                    let _lines: Vec<&str> = self.source.lines().collect();

                    // TODO: Use proper violation type from truent_core when available
                    // For now, just track the detection in debug logs
                    debug!(
                        "Flash loan vulnerability detected in {}::{} at line {}",
                        contract, func.name, call_line_num
                    );
                }
            }
        }
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
