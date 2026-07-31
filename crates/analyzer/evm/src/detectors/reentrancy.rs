//! Reentrancy vulnerability detector using AST analysis.
//!
//! Detects the classic reentrancy pattern: external calls before state updates.
//! This is only possible to detect correctly with AST analysis that provides
//! statement ordering within functions.

use crate::ast_types::*;
use crate::ast_walker::AstVisitor;
use crate::bytecode::Severity;

/// Placeholder for violation type until proper integration with truent_core
#[derive(Debug, Clone)]
struct Violation;

/// Detects reentrancy vulnerabilities via AST statement analysis
pub struct ReentrancyDetector<'a> {
    source: &'a str,
    file_name: &'a str,
    /// Accumulated violations
    pub violations: Vec<Violation>,

    // Per-function state tracking
    current_func_name: String,
    external_call_lines: Vec<usize>,
    state_update_lines: Vec<usize>,
    has_reentrancy_guard: bool,
    state_vars: Vec<String>,
}

impl<'a> ReentrancyDetector<'a> {
    /// Create a new reentrancy detector
    pub fn new(source: &'a str, file_name: &'a str) -> Self {
        Self {
            source,
            file_name,
            violations: Vec::new(),
            current_func_name: String::new(),
            external_call_lines: Vec::new(),
            state_update_lines: Vec::new(),
            has_reentrancy_guard: false,
            state_vars: Vec::new(),
        }
    }

    fn is_external_call(call: &FunctionCall) -> bool {
        // Detect patterns like:
        // - addr.call{value: x}("")
        // - addr.send(x)
        // - addr.transfer(x)
        // - IERC20(token).transfer(to, amount)

        match call.expression.as_ref() {
            AstNode::MemberAccess(member) => {
                matches!(
                    member.member_name.as_str(),
                    "call" | "send" | "transfer" | "delegatecall" | "staticcall"
                )
            }
            _ => false,
        }
    }

    fn is_state_update(assign: &Assignment, state_vars: &[String]) -> bool {
        // Check if LHS is a state variable or indexed state variable
        match assign.left_hand_side.as_ref() {
            AstNode::Identifier(id) => state_vars.contains(&id.name),
            AstNode::IndexAccess(idx) => {
                if let AstNode::Identifier(base) = idx.base_expression.as_ref() {
                    state_vars.contains(&base.name)
                } else {
                    false
                }
            }
            AstNode::MemberAccess(member) => {
                if let AstNode::Identifier(base) = member.expression.as_ref() {
                    state_vars.contains(&base.name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn has_nonreentrant_modifier(func: &FunctionDefinition) -> bool {
        func.modifiers.iter().any(|m| {
            let name = m.modifier_name.name.to_lowercase();
            name.contains("reentrant") || name.contains("lock") || name == "guard"
        })
    }
}

impl<'a> AstVisitor for ReentrancyDetector<'a> {
    fn visit_function(&mut self, func: &FunctionDefinition, _contract: &str) {
        self.current_func_name = func.name.clone();
        self.external_call_lines.clear();
        self.state_update_lines.clear();
        self.has_reentrancy_guard = Self::has_nonreentrant_modifier(func);
    }

    fn visit_function_call(
        &mut self,
        call: &FunctionCall,
        _func: &FunctionDefinition,
        _contract: &str,
    ) {
        if self.has_reentrancy_guard {
            return;
        }

        if Self::is_external_call(call) {
            let (offset, _, _) = parse_src(&call.src);
            let line = offset_to_line(self.source, offset);
            self.external_call_lines.push(line);
        }
    }

    fn visit_assignment(
        &mut self,
        assign: &Assignment,
        _func: &FunctionDefinition,
        _contract: &str,
    ) {
        if self.has_reentrancy_guard {
            return;
        }

        if Self::is_state_update(assign, &self.state_vars) {
            let (offset, _, _) = parse_src(&assign.src);
            let line = offset_to_line(self.source, offset);
            self.state_update_lines.push(line);
        }
    }

    fn visit_state_variable(&mut self, var: &VariableDeclaration, _contract: &str) {
        if var.state_variable {
            self.state_vars.push(var.name.clone());
        }
    }

    fn leave_function(
        &mut self,
        func: &FunctionDefinition,
        _statements: &[AstNode],
        contract: &str,
    ) {
        if self.has_reentrancy_guard || self.external_call_lines.is_empty() {
            return;
        }

        // Check if any external call happens without prior state update
        for &call_line in &self.external_call_lines {
            let has_prior_state_update = self
                .state_update_lines
                .iter()
                .any(|&update_line| update_line < call_line);

            if !has_prior_state_update {
                let lines: Vec<&str> = self.source.lines().collect();
                let context = extract_code_context(&lines, call_line);

                self.violations.push(Violation {
                    id: "evm_reentrancy_protection".to_string(),
                    severity: Severity::Critical,
                    title: "Reentrancy Vulnerability".to_string(),
                    message: format!(
                        "Function '{}::{}' makes an external call at line {} \
                         before updating state. An attacker can re-enter the \
                         function before state is updated, potentially \
                         draining funds.",
                        contract, func.name, call_line
                    ),
                    recommendation:
                        "Apply the checks-effects-interactions pattern: \
                         (1) Check conditions first, \
                         (2) Update all state variables, \
                         (3) Make external calls last. \
                         Or add the nonReentrant modifier from OpenZeppelin."
                            .to_string(),
                    location: Some(format!("{}:{}", self.file_name, call_line)),
                    context: Some(context),
                    cwe: Some("CWE-841".to_string()),
                    references: vec![
                        "https://docs.openzeppelin.com/contracts/4.x/api/security#ReentrancyGuard"
                            .to_string(),
                        "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/"
                            .to_string(),
                    ],
                });
            }
        }
    }
}

/// Extract code context around a line for violation reporting
fn extract_code_context(lines: &[&str], line_num: usize) -> String {
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
