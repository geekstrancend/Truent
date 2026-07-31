//! Invariant library management.

use truent_core::model::{Expression, Invariant};
use truent_dsl_parser::parse_invariant;
use std::collections::BTreeMap;
use tracing::warn;

/// Compile a constraint string (e.g. "call_order_respected AND no_recursive_calls")
/// into a real `Expression` tree via the DSL parser, the same way
/// `LibraryLoader` compiles TOML-defined invariants. Falls back to a single
/// `Expression::Var` (the constraint text as one opaque variable) if parsing
/// ever fails, so a malformed built-in constraint degrades instead of panicking.
fn compile_constraint(id: &str, constraint: &str) -> Expression {
    // The built-in constraint text uses readable "AND"/"OR" keywords, but the
    // DSL grammar's logical operators are "&&"/"||" (see grammar.rs) - normalize
    // before compiling. Safe because every identifier here is lower_snake_case,
    // so the uppercase tokens can only ever be these keywords.
    let normalized = constraint
        .split_whitespace()
        .map(|tok| match tok {
            "AND" => "&&",
            "OR" => "||",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ");

    let full = format!("invariant {} {{ {} }}", id, normalized);
    match parse_invariant(&full) {
        Ok(inv) => inv.expression,
        Err(e) => {
            warn!(
                "Failed to compile built-in constraint for '{}' ({}): {}. Falling back to opaque variable.",
                id, constraint, e
            );
            Expression::Var(id.to_string())
        }
    }
}

/// A collection of invariants organized by category.
pub struct InvariantLibrary {
    /// Invariants by category.
    pub categories: BTreeMap<String, Vec<Invariant>>,
}

impl InvariantLibrary {
    /// Create a new empty library.
    pub fn new() -> Self {
        Self {
            categories: BTreeMap::new(),
        }
    }

    /// Create a library with default built-in invariants for a chain.
    pub fn with_defaults(chain: &str) -> Self {
        let mut lib = Self::new();
        lib.add_defaults(chain);
        lib
    }

    /// Add default invariants for a specific blockchain.
    fn add_defaults(&mut self, chain: &str) {
        match chain.to_lowercase().as_str() {
            "evm" => self.add_evm_defaults(),
            "solana" => self.add_solana_defaults(),
            "move" => self.add_move_defaults(),
            "soroban" => self.add_soroban_defaults(),
            _ => {}
        }
    }

    /// Add EVM-specific invariants.
    fn add_evm_defaults(&mut self) {
        let evm_invariants = vec![
            (
                "evm_reentrancy_protection",
                "Reentrancy Protection",
                "call_order_respected AND no_recursive_calls",
            ),
            (
                "evm_integer_overflow",
                "Integer Overflow",
                "arithmetic_values_bounded AND checked_arithmetic",
            ),
            (
                "evm_integer_underflow",
                "Integer Underflow",
                "subtraction_checked AND minimum_value_respected",
            ),
            (
                "evm_unchecked_returns",
                "Unchecked Return Values",
                "all_external_calls_checked AND return_validation",
            ),
            (
                "evm_delegatecall_injection",
                "Delegatecall Injection",
                "delegatecall_target_whitelist AND no_user_delegatecall",
            ),
            (
                "evm_access_control",
                "Access Control",
                "caller_authentication AND permission_respected",
            ),
            (
                "evm_timestamp_dependence",
                "Timestamp Dependence",
                "no_timestamp_for_security AND block_properties_consistent",
            ),
            (
                "evm_frontrunning",
                "Front-running",
                "state_order_independent AND tx_ordering_irrelevant",
            ),
            (
                "evm_uninitialized_pointers",
                "Uninitialized Pointers",
                "memory_initialized_before_use AND storage_initialized",
            ),
            (
                "evm_division_by_zero",
                "Division by Zero",
                "divisor_nonzero AND modulo_nonzero",
            ),
        ];

        for (id, name, constraint) in evm_invariants {
            self.add(
                "EVM".to_string(),
                Invariant {
                    name: id.to_string(),
                    description: Some(format!("EVM invariant: {}", name)),
                    expression: compile_constraint(id, constraint),
                    severity: "high".to_string(),
                    category: "evm".to_string(),
                    is_always_true: true,
                    layers: vec!["control_flow".to_string(), "data_flow".to_string()],
                    phases: vec!["execution".to_string(), "finalization".to_string()],
                },
            );
        }
    }

    /// Add Solana-specific invariants.
    fn add_solana_defaults(&mut self) {
        let solana_invariants = vec![
            (
                "sol_signer_checks",
                "Signer Checks",
                "all_required_signers_present AND signatures_valid",
            ),
            (
                "sol_account_validation",
                "Account Validation",
                "expected_accounts_provided AND account_state_valid",
            ),
            (
                "sol_integer_overflow",
                "Integer Overflow",
                "arithmetic_checked AND values_within_bounds",
            ),
            (
                "sol_rent_exemption",
                "Rent Exemption",
                "rent_paid_or_exempt AND account_cleanup_proper",
            ),
            (
                "sol_pda_derivation",
                "PDA Derivation",
                "pda_seeds_deterministic AND derivation_consistent",
            ),
            (
                "sol_lamport_balance",
                "Lamport Balance",
                "lamports_conserved AND no_lamport_leaks",
            ),
            (
                "sol_instruction_parsing",
                "Instruction Parsing",
                "instruction_data_valid AND account_order_correct",
            ),
        ];

        for (id, name, constraint) in solana_invariants {
            self.add(
                "Solana".to_string(),
                Invariant {
                    name: id.to_string(),
                    description: Some(format!("Solana invariant: {}", name)),
                    expression: compile_constraint(id, constraint),
                    severity: "high".to_string(),
                    category: "solana".to_string(),
                    is_always_true: true,
                    layers: vec!["account_layer".to_string(), "instruction_layer".to_string()],
                    phases: vec!["parsing".to_string(), "execution".to_string()],
                },
            );
        }
    }

    /// Add Move-specific invariants.
    fn add_move_defaults(&mut self) {
        let move_invariants = vec![
            (
                "move_access_control",
                "Access Control",
                "caller_has_required_capability AND resource_protected",
            ),
            (
                "move_integer_overflow",
                "Integer Overflow",
                "addition_checked AND values_bounded",
            ),
            (
                "move_resource_leaks",
                "Resource Leaks",
                "all_resources_returned AND no_abort_without_cleanup",
            ),
            (
                "move_type_safety",
                "Type Safety",
                "types_match_at_boundaries AND resources_typed",
            ),
            (
                "move_signer_requirement",
                "Signer Requirement",
                "signer_passed_and_verified AND signer_not_optional",
            ),
        ];

        for (id, name, constraint) in move_invariants {
            self.add(
                "Move".to_string(),
                Invariant {
                    name: id.to_string(),
                    description: Some(format!("Move invariant: {}", name)),
                    expression: compile_constraint(id, constraint),
                    severity: "high".to_string(),
                    category: "move".to_string(),
                    is_always_true: true,
                    layers: vec!["module_layer".to_string(), "transaction_layer".to_string()],
                    phases: vec!["execution".to_string()],
                },
            );
        }
    }

    /// Add Soroban-specific invariants.
    fn add_soroban_defaults(&mut self) {
        let soroban_invariants = vec![
            (
                "sor_require_auth_checks",
                "Require-Auth Checks",
                "privileged_functions_require_auth AND auth_address_matches_actor",
            ),
            (
                "sor_no_unprotected_upgrade",
                "Protected Upgrade",
                "upgrade_requires_auth AND upgrade_target_verified",
            ),
            (
                "sor_init_guard",
                "Initializer Guard",
                "initialize_checks_not_already_set AND admin_set_exactly_once",
            ),
            (
                "sor_checked_arithmetic",
                "Checked Arithmetic",
                "arithmetic_uses_checked_ops AND values_within_bounds",
            ),
            (
                "sor_storage_ttl_extended",
                "Storage TTL Extended",
                "persistent_entries_ttl_extended AND no_unexpected_archival",
            ),
            (
                "sor_no_reentrancy",
                "No Reentrancy",
                "state_written_before_external_call AND checks_effects_interactions_respected",
            ),
        ];

        for (id, name, constraint) in soroban_invariants {
            self.add(
                "Soroban".to_string(),
                Invariant {
                    name: id.to_string(),
                    description: Some(format!("Soroban invariant: {}", name)),
                    expression: compile_constraint(id, constraint),
                    severity: "high".to_string(),
                    category: "soroban".to_string(),
                    is_always_true: true,
                    layers: vec!["contract_layer".to_string(), "storage_layer".to_string()],
                    phases: vec!["parsing".to_string(), "execution".to_string()],
                },
            );
        }
    }

    /// Add an invariant to the library.
    pub fn add(&mut self, category: String, invariant: Invariant) {
        self.categories.entry(category).or_default().push(invariant);
    }

    /// Get all invariants in a category.
    pub fn get_category(&self, category: &str) -> Option<&[Invariant]> {
        self.categories.get(category).map(|v| v.as_slice())
    }

    /// Get all invariants.
    pub fn all(&self) -> Vec<&Invariant> {
        self.categories.values().flat_map(|v| v.iter()).collect()
    }

    /// Count total invariants.
    pub fn count(&self) -> usize {
        self.categories.values().map(|v| v.len()).sum()
    }
}

impl Default for InvariantLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Built-in invariants must carry a real compiled expression tree, not the
    /// old `Expression::Var(id)` placeholder that discarded the constraint text.
    #[test]
    fn evm_defaults_compile_to_real_expressions() {
        let lib = InvariantLibrary::with_defaults("evm");
        let invariants = lib.all();
        assert_eq!(invariants.len(), 10);

        for inv in &invariants {
            match &inv.expression {
                Expression::Logical { .. } => {}
                other => panic!(
                    "expected invariant '{}' to compile to Expression::Logical, got {:?}",
                    inv.name, other
                ),
            }
        }
    }

    #[test]
    fn solana_defaults_compile_to_real_expressions() {
        let lib = InvariantLibrary::with_defaults("solana");
        let invariants = lib.all();
        assert_eq!(invariants.len(), 7);
        for inv in &invariants {
            assert!(matches!(inv.expression, Expression::Logical { .. }));
        }
    }

    #[test]
    fn move_defaults_compile_to_real_expressions() {
        let lib = InvariantLibrary::with_defaults("move");
        let invariants = lib.all();
        assert_eq!(invariants.len(), 5);
        for inv in &invariants {
            assert!(matches!(inv.expression, Expression::Logical { .. }));
        }
    }

    #[test]
    fn soroban_defaults_compile_to_real_expressions() {
        let lib = InvariantLibrary::with_defaults("soroban");
        let invariants = lib.all();
        assert_eq!(invariants.len(), 6);
        for inv in &invariants {
            assert!(matches!(inv.expression, Expression::Logical { .. }));
        }
    }

    #[test]
    fn unknown_chain_yields_empty_library() {
        let lib = InvariantLibrary::with_defaults("cosmwasm");
        assert_eq!(lib.count(), 0);
    }
}
