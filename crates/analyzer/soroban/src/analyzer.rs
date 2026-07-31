//! Soroban analyzer implementation.

use crate::soroban_parser::parse_contract_functions;
use truent_core::model::{FunctionModel, ProgramModel, StateVar};
use truent_core::traits::ChainAnalyzer;
use truent_core::Result;
use std::collections::BTreeSet;
use std::path::Path;
use tracing::info;

/// Analyzer for Soroban (Stellar) Rust contracts.
///
/// Performs static analysis on `#[contract]`/`#[contractimpl]` source to
/// extract entry points and the facts the Soroban detectors need,
/// following the same `syn`-based approach `truent-analyzer-solana` uses.
pub struct SorobanAnalyzer;

impl ChainAnalyzer for SorobanAnalyzer {
    fn analyze(&self, path: &Path) -> Result<ProgramModel> {
        info!("Analyzing Soroban contract at {:?}", path);

        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;

        let file = syn::parse_file(&source).map_err(|e| {
            truent_core::InvarError::AnalysisFailed(format!("Failed to parse Rust: {}", e))
        })?;

        let mut program = ProgramModel::new(
            "soroban_contract".to_string(),
            "soroban".to_string(),
            path.to_string_lossy().to_string(),
        );

        for item in &file.items {
            if let syn::Item::Struct(item_struct) = item {
                let is_contract_struct = item_struct
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("contract"));
                if is_contract_struct {
                    program.add_state_var(StateVar {
                        name: item_struct.ident.to_string(),
                        type_name: "contract".to_string(),
                        is_mutable: false,
                        visibility: Some("public".to_string()),
                    });
                }
            }
        }

        let functions = parse_contract_functions(&source).map_err(|e| {
            truent_core::InvarError::AnalysisFailed(format!(
                "Failed to parse Soroban contract functions: {}",
                e
            ))
        })?;

        for f in &functions {
            let mut mutates = BTreeSet::new();
            if !f.has_require_auth {
                mutates.insert(format!("SOROBAN_MISSING_REQUIRE_AUTH:{}", f.line));
            }
            if f.upgrades_wasm && !f.has_require_auth {
                mutates.insert(format!("SOROBAN_UNPROTECTED_UPGRADE:{}", f.line));
            }
            if f.is_initialize && !f.has_init_guard {
                mutates.insert(format!("SOROBAN_REINITIALIZATION:{}", f.line));
            }
            if f.uses_unchecked_arithmetic {
                mutates.insert(format!("SOROBAN_UNCHECKED_ARITHMETIC:{}", f.line));
            }
            if f.writes_persistent_storage && !f.extends_ttl {
                mutates.insert(format!("SOROBAN_STORAGE_TTL_NOT_EXTENDED:{}", f.line));
            }
            if f.writes_temporary_storage_of_sensitive_state {
                mutates.insert(format!("SOROBAN_TEMP_STORAGE_CRITICAL:{}", f.line));
            }
            if f.external_call_before_storage_write {
                mutates.insert(format!("SOROBAN_REENTRANCY:{}", f.line));
            }
            if f.unwrap_count > 0 {
                mutates.insert(format!("SOROBAN_UNHANDLED_PANIC:{}", f.line));
            }

            program.add_function(FunctionModel {
                name: f.name.clone(),
                parameters: vec![],
                return_type: None,
                mutates,
                reads: BTreeSet::new(),
                is_entry_point: true,
                is_pure: false,
            });
        }

        info!(
            "Extracted {} state vars and {} entry points",
            program.state_vars.len(),
            program.functions.len()
        );
        Ok(program)
    }

    fn chain(&self) -> &str {
        "soroban"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const CONTRACT: &str = r#"
#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn withdraw(env: Env, to: Address, amount: i128) {
        env.storage().persistent().set(&DataKey::Balance(to), &amount);
    }
}
"#;

    #[test]
    fn test_analyze_soroban_contract() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        fs::write(&path, CONTRACT).unwrap();

        let analyzer = SorobanAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.chain, "soroban");
        assert_eq!(result.functions.len(), 2);
        assert!(result.state_vars.contains_key("TokenContract"));
    }

    #[test]
    fn test_analyze_empty_rust_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.rs");
        fs::write(&path, "").unwrap();

        let analyzer = SorobanAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.functions.len(), 0);
    }

    #[test]
    fn test_analyze_nonexistent_path() {
        let analyzer = SorobanAnalyzer;
        let result = analyzer.analyze(Path::new("/nonexistent/path/contract.rs"));
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_identifier() {
        let analyzer = SorobanAnalyzer;
        assert_eq!(analyzer.chain(), "soroban");
    }

    #[test]
    fn test_analyze_malformed_rust() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.rs");
        fs::write(&path, r#"fn broken_function( { invalid rust code }"#).unwrap();

        let analyzer = SorobanAnalyzer;
        let result = analyzer.analyze(&path);
        assert!(result.is_err());
    }
}
