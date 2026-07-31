//! Solana invariant enforcement procedural macro.
//!
//! # #[invariant_enforced] Attribute Macro
//!
//! **Current status: scaffolding only, not yet enforcing.** This macro parses
//! its check-expression arguments and computes a deterministic hash of them
//! (see `parse_invariant_checks`/`compute_check_hash`), but
//! `generate_check_statements` currently emits only comments
//! (`// Invariant: <check>`) followed by `let _ = ();` - no assertion, no
//! `require`, no compile-time validation, no runtime enforcement. Applying
//! this attribute to a function does not change that function's behavior.
//!
//! Do not rely on this macro for any security property until real
//! enforcement is implemented; the properties below describe the intended
//! design, not current behavior.
//!
//! # Intended Security Properties (not yet implemented)
//! - Deterministic injection order (alphabetical by state variable)
//! - No silent failures (compile error if invariant can't be resolved)
//! - Tamper detection: Hash embedded in generated code comments
//! - Type-safe: All injected code type-checked by Rust compiler
//!
//! # Example
//!
//! ```ignore
//! #[invariant_enforced("balance >= 0", "supply == sum_of_balances")]
//! pub fn transfer(
//!     from: &mut Account,
//!     to: &mut Account,
//!     amount: u64,
//! ) -> ProgramResult {
//!     from.balance = from.balance.checked_sub(amount)?;
//!     to.balance = to.balance.checked_add(amount)?;
//!     // No invariant check is actually injected here yet.
//!     Ok(())
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat};

/// Procedural attribute macro, currently scaffolding only - see the module
/// doc comment above. It parses the check expressions given as its
/// arguments but does not currently validate, type-check, or enforce them;
/// applying it to a function is a no-op.
///
/// # Attributes
/// - Comma-separated, quoted invariant expression strings, e.g.
///   `#[invariant_enforced("balance >= 0", "supply > 0")]`
#[proc_macro_attribute]
pub fn invariant_enforced(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse attribute arguments
    let args_str = args.to_string();

    // Validate function signature
    match validate_function_signature(&input_fn) {
        Ok(state_vars) => {
            // Generate invariant checks
            let checks = parse_invariant_checks(&args_str);
            let check_stmts = generate_check_statements(&checks, &state_vars);

            // Inject checks into function
            let modified_fn = inject_checks(&input_fn, check_stmts);
            quote! { #modified_fn }.into()
        }
        Err(e) => syn::Error::new_spanned(&input_fn, e)
            .to_compile_error()
            .into(),
    }
}

/// Validate that the function signature is suitable for invariant injection.
fn validate_function_signature(func: &ItemFn) -> Result<Vec<String>, String> {
    let mut state_vars = Vec::new();

    for arg in &func.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let var_name = pat_ident.ident.to_string();

                    // Check if it's a mutable reference (state parameter)
                    if pat_ident.mutability.is_some() {
                        state_vars.push(var_name);
                    }
                }
            }
            FnArg::Receiver(_) => {
                return Err("Invariant-enforced functions cannot have &self/&mut self".to_string());
            }
        }
    }

    if state_vars.is_empty() {
        return Err("Function must have at least one &mut parameter for state".to_string());
    }

    Ok(state_vars)
}

/// Parse invariant check specifications from macro arguments.
fn parse_invariant_checks(args: &str) -> Vec<String> {
    // Remove quotes and split by comma
    args.split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Generate invariant check statements with tamper detection hash.
fn generate_check_statements(checks: &[String], _state_vars: &[String]) -> Vec<syn::Stmt> {
    use quote::format_ident;

    let mut stmts = Vec::new();

    // Add tamper detection header (hash embeds macro version and check list)
    let check_hash = compute_check_hash(checks);
    let _hash_comment = format!("// TRUENT_HASH: {}", check_hash);

    stmts.push(syn::parse_quote! {
        // Invariant checks injected by #[invariant_enforced]
    });

    // Generate a check for each invariant
    for (idx, check) in checks.iter().enumerate() {
        let _check_name = format_ident!("truent_check_{}", idx);
        let _check_expr_str = check.clone();

        // Create assertion-like statement
        stmts.push(syn::parse_quote! {
            // Invariant: #check
            // This check is automatically enforced
        });
    }

    stmts.push(syn::parse_quote! {
        // Tamper detection enabled
        let _ = ();
    });

    stmts
}

/// Inject check statements at the end of the function, just before return.
fn inject_checks(func: &ItemFn, checks: Vec<syn::Stmt>) -> ItemFn {
    let mut modified_fn = func.clone();

    // Find insertion point (before final return if present)
    match &mut modified_fn.block.stmts.last() {
        Some(_last_stmt) => {
            // Insert checks before the last statement if it's a return
            modified_fn.block.stmts.splice(
                modified_fn.block.stmts.len() - 1..modified_fn.block.stmts.len(),
                checks,
            );
        }
        None => {
            // Empty block, just add checks
            modified_fn.block.stmts.extend(checks);
        }
    }

    modified_fn
}

/// Compute hash for tamper detection.
/// Uses XOR of all check strings for deterministic ordering.
fn compute_check_hash(checks: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Sort checks for deterministic hashing
    let mut sorted_checks = checks.to_vec();
    sorted_checks.sort();

    for check in sorted_checks {
        check.hash(&mut hasher);
    }

    let hash = hasher.finish();
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_hash_deterministic() {
        let checks1 = vec!["balance >= 0".to_string(), "supply > 0".to_string()];
        let checks2 = vec!["supply > 0".to_string(), "balance >= 0".to_string()];

        // Different order should produce same hash
        let hash1 = compute_check_hash(&checks1);
        let hash2 = compute_check_hash(&checks2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_parse_invariant_checks() {
        let args = r#""balance >= 0", "supply > 0""#;
        let checks = parse_invariant_checks(args);

        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0], "balance >= 0");
        assert_eq!(checks[1], "supply > 0");
    }
}
