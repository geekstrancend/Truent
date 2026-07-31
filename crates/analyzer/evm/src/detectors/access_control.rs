//! Access control vulnerability detector.
//!
//! Detects missing or inadequate access control on functions that should be restricted.

use crate::ast_types::*;
use crate::ast_walker::AstVisitor;
use crate::bytecode::Severity;

/// Placeholder for violation type until proper integration with truent_core
#[derive(Debug, Clone)]
struct Violation;

/// Detects missing or inadequate access control
pub struct AccessControlDetector<'a> {
    source: &'a str,
    file_name: &'a str,
    /// Accumulated violations
    pub violations: Vec<Violation>,
}

impl<'a> AccessControlDetector<'a> {
    /// Create a new access control detector
    pub fn new(source: &'a str, file_name: &'a str) -> Self {
        Self {
            source,
            file_name,
            violations: Vec::new(),
        }
    }
}

impl<'a> AstVisitor for AccessControlDetector<'a> {
    fn visit_function(&mut self, func: &FunctionDefinition, contract: &str) {
        // Skip constructors and internal functions
        if func.is_constructor || func.visibility == "internal" || func.visibility == "private" {
            return;
        }

        // Functions that modify state and are public/external
        if func.state_mutability == "nonpayable" || func.state_mutability == "payable" {
            // Check if function has modifiers (access control guards)
            if func.modifiers.is_empty() {
                // Check for sensitive names
                let name_lower = func.name.to_lowercase();
                if self.is_sensitive_function(&name_lower) {
                    let (offset, _, _) = parse_src(&func.src);
                    let line = offset_to_line(self.source, offset);
                    let lines: Vec<&str> = self.source.lines().collect();

                    self.violations.push(Violation {
                        id: "evm_missing_access_control".to_string(),
                        severity: Severity::High,
                        title: "Missing Access Control".to_string(),
                        message: format!(
                            "Function '{}::{}' at line {} appears to modify state \
                             but has no access control modifiers. \
                             Any account can call this function.",
                            contract, func.name, line
                        ),
                        recommendation:
                            "Add an access control modifier like onlyOwner, onlyAdmin, \
                             or use role-based access control (RBAC). \
                             OpenZeppelin provides Ownable, AccessControl, and other \
                             ready-to-use implementations."
                                .to_string(),
                        location: Some(format!("{}:{}", self.file_name, line)),
                        context: Some(extract_context(&lines, line)),
                        cwe: Some("CWE-276".to_string()),
                        references: vec![
                            "https://docs.openzeppelin.com/contracts/4.x/access-control"
                                .to_string(),
                        ],
                    });
                }
            }
        }
    }
}

impl<'a> AccessControlDetector<'a> {
    fn is_sensitive_function(&self, name: &str) -> bool {
        // Functions that typically need access control
        matches!(
            name,
            "withdraw"
                | "transfer"
                | "burn"
                | "mint"
                | "pause"
                | "unpause"
                | "setrole"
                | "addadmin"
                | "removeadmin"
                | "settle"
                | "execute"
                | "drain"
                | "upgrade"
        )
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
