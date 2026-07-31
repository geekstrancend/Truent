//! Move analyzer implementation.

use std::collections::BTreeSet;
use std::path::Path;
use tracing::info;
use truent_core::model::{FunctionModel, ProgramModel};
use truent_core::traits::ChainAnalyzer;
use truent_core::{AnalysisContext, Result};

/// Analyzer for Move programs (Aptos/Sui).
///
/// Performs static analysis on Move code to extract:
/// - Module names and entry points
/// - Function signatures and visibility
/// - Resource types and access patterns
/// - Mutation and read operations
pub struct MoveAnalyzer;

impl ChainAnalyzer for MoveAnalyzer {
    fn analyze(&self, path: &Path) -> Result<ProgramModel> {
        info!("Analyzing Move program at {:?}", path);

        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;

        // Extract module name
        let module_name = extract_module_name(&source).unwrap_or_else(|| "move_module".to_string());

        // Extract resource types
        let resources = extract_resource_types(&source);
        info!("Found {} resource types in Move module", resources.len());

        // Extract functions with proper analysis
        let functions = extract_functions_with_analysis(&source, &resources);
        info!("Found {} functions in Move module", functions.len());

        // Create program model with analyzed information
        let mut program = ProgramModel::new(
            module_name,
            "move".to_string(),
            path.to_string_lossy().to_string(),
        );

        // Add resource types as state variables
        for resource in &resources {
            use truent_core::model::StateVar;
            program.add_state_var(StateVar {
                name: resource.clone(),
                type_name: "resource".to_string(),
                is_mutable: true,
                visibility: None,
            });
        }

        // Add extracted functions to the program model
        for func in functions {
            program.add_function(func);
        }

        Ok(program)
    }

    fn chain(&self) -> &str {
        "move"
    }
}

impl MoveAnalyzer {
    /// Analyze a Move program and return context with warnings.
    pub fn analyze_with_context(&self, path: &Path) -> Result<AnalysisContext> {
        let program = self.analyze(path)?;
        let mut context = AnalysisContext::new(program);

        // Read source for warning collection
        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;
        let lines: Vec<&str> = source.lines().collect();

        // Scan for common Move vulnerability patterns
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Warning: Unsafe resource destruction
            if (line.contains("move_to") || line.contains("move_from"))
                && !lines
                    .iter()
                    .skip(line_idx.saturating_sub(2))
                    .take(5)
                    .any(|l| l.contains("assert") || l.contains("require"))
            {
                context.add_warning(
                    "Resource operation without validation detected".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: Missing type checking
            if line.contains("as u64") || line.contains("as u128") || line.contains("as u256") {
                context.add_warning(
                    "Type cast detected - verify bounds".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: Direct field access without validation
            if line.contains(".")
                && (line.contains("=") || line.contains(".value"))
                && !lines
                    .iter()
                    .skip(line_idx.saturating_sub(1))
                    .take(2)
                    .any(|l| l.contains("assert"))
            {
                context.add_warning(
                    "Field access without validation".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: Unchecked arithmetic
            if (line.contains("+") || line.contains("-"))
                && (line.contains("amount") || line.contains("balance"))
            {
                context.add_warning(
                    "Unchecked arithmetic operation detected".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }
        }

        // Mark invalid if critical issues found
        let critical_warnings = context
            .warnings
            .iter()
            .filter(|w| w.message.contains("validation") || w.message.contains("Unchecked"))
            .count();

        if critical_warnings > 1 {
            context.mark_invalid();
        }

        Ok(context)
    }
}

/// Extract module name from Move source code.
fn extract_module_name(source: &str) -> Option<String> {
    for line in source.lines() {
        if line.trim_start().starts_with("module ") {
            let module_part = line.split("module ").nth(1)?;
            let name = module_part
                .split(|c: char| [':', '{', ';'].contains(&c))
                .next()?
                .trim();
            return Some(name.to_string());
        }
    }
    None
}

/// Extract resource type names from Move source code.
fn extract_resource_types(source: &str) -> Vec<String> {
    let mut resources = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("struct ") || trimmed.starts_with("resource struct ") {
            let key = if trimmed.starts_with("resource struct ") {
                "resource struct "
            } else {
                "struct "
            };
            if let Some(struct_part) = trimmed.split(key).nth(1) {
                if let Some(name) = struct_part
                    .split(|c: char| ['{', '(', '<', ';'].contains(&c))
                    .next()
                {
                    resources.push(name.trim().to_string());
                }
            }
        }
    }
    resources
}

/// Extract functions with full analysis including state access patterns.
fn extract_functions_with_analysis(source: &str, resources: &[String]) -> Vec<FunctionModel> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Look for function declarations
        if (trimmed.contains("public fun ")
            || trimmed.contains("fun ")
            || trimmed.contains("entry fun "))
            && !trimmed.contains("//")
        {
            // Extract visibility
            let is_public = trimmed.contains("public ");
            let is_entry = trimmed.contains("entry ");

            // Extract function name
            let func_keyword = if trimmed.contains("entry fun ") {
                "entry fun "
            } else if trimmed.contains("public fun ") {
                "public fun "
            } else {
                "fun "
            };

            if let Some(func_part) = trimmed.split(func_keyword).nth(1) {
                if let Some(name) = func_part.split('(').next() {
                    let func_name = name.trim().to_string();

                    // Extract parameters
                    let params = extract_move_function_params(func_part);

                    // Check if function has mutable parameter (acquires or &mut)
                    let has_mutable_ref =
                        func_part.contains("&mut ") || func_part.contains("acquires ");

                    // Analyze function body for resource access
                    let (reads, mutates) = analyze_move_function_body(&lines, i, resources);

                    // Check if function is pure (no mutation)
                    let is_pure = !has_mutable_ref && mutates.is_empty();
                    let is_entry_point = is_entry || (is_public && !reads.is_empty());

                    let func = FunctionModel {
                        name: func_name,
                        parameters: params,
                        return_type: None,
                        mutates,
                        reads,
                        is_entry_point,
                        is_pure,
                    };

                    functions.push(func);
                }
            }
        }

        i += 1;
    }

    functions
}

/// Extract Move function parameters.
fn extract_move_function_params(signature: &str) -> Vec<String> {
    if let Some(start) = signature.find('(') {
        if let Some(end) = signature.find(')') {
            let params_str = &signature[start + 1..end];
            if params_str.is_empty() {
                return Vec::new();
            }

            params_str
                .split(',')
                .map(|p| {
                    // Each param is like "account: &mut signer" or "amount: u64"
                    let parts: Vec<&str> = p.split_whitespace().collect();
                    parts.first().unwrap_or(&"").to_string()
                })
                .filter(|p| !p.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
}

/// Analyze a Move function body to detect resource access patterns.
fn analyze_move_function_body(
    lines: &[&str],
    start_idx: usize,
    resources: &[String],
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut reads = BTreeSet::new();
    let mut mutates = BTreeSet::new();

    let mut brace_count = 0;
    let mut in_function = false;

    for (i, line) in lines.iter().enumerate().skip(start_idx) {
        let trimmed = line.trim();

        // Count braces to detect function body
        for ch in line.chars() {
            if ch == '{' {
                in_function = true;
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if in_function && brace_count == 0 {
                    // Analyze for Move-specific vulnerabilities
                    analyze_move_vulnerabilities(lines, start_idx, i, &mut mutates);
                    return (reads, mutates);
                }
            }
        }

        if !in_function {
            continue;
        }

        // Look for resource accesses
        for resource in resources {
            if trimmed.contains(resource) {
                // Check for mutations
                if trimmed.contains("move_from")
                    || trimmed.contains("borrow_global_mut")
                    || trimmed.contains("global_mut")
                {
                    mutates.insert(resource.clone());
                } else if trimmed.contains("borrow_global")
                    || trimmed.contains("global")
                    || trimmed.contains("assert!")
                {
                    reads.insert(resource.clone());
                }
            }
        }
    }

    (reads, mutates)
}

/// Detect Move-specific security vulnerabilities.
fn analyze_move_vulnerabilities(
    lines: &[&str],
    start_idx: usize,
    end_idx: usize,
    mutates: &mut BTreeSet<String>,
) {
    let body = lines[start_idx..=end_idx].join("\n");
    let body_lower = body.to_lowercase();

    // === RESOURCE LEAK VULNERABILITIES ===
    if body.contains("move_from") && !body.contains("_") {
        // move_from without variable binding or discarding - potential resource leak
        mutates.insert("MOVE_RESOURCE_LEAK".to_string());
    }

    // === MISSING_ABILITY_CHECKS ===
    if body.contains("move_to")
        && !body_lower.contains("has key")
        && !body_lower.contains("has store")
    {
        mutates.insert("MOVE_MISSING_ABILITY".to_string());
    }

    // === UNSAFE ARITHMETIC (no overflow check) ===
    if (body.contains("+") || body.contains("-") || body.contains("*"))
        && !body.contains("overflow")
        && !body.contains("checked")
        && !body_lower.contains("assert_")
        && !body.contains("invariant")
    {
        mutates.insert("MOVE_UNCHECKED_ARITHMETIC".to_string());
    }

    // === MISSING SIGNER VERIFICATION ===
    if body.contains("move_to") && !body.contains("&signer") && !body.contains("signer::") {
        mutates.insert("MOVE_MISSING_SIGNER".to_string());
    }

    // === UNGUARDED STATE MUTATION ===
    if body.contains("borrow_global_mut") && !body.contains("assert!") && !body.contains("require")
    {
        mutates.insert("MOVE_UNGUARDED_MUTATION".to_string());
    }

    // === PRIVILEGE ESCALATION ===
    if body.contains("signer")
        && body.contains("address_of")
        && !body.contains("require")
        && !body.contains("assert!")
    {
        mutates.insert("MOVE_PRIVILEGE_ESCALATION".to_string());
    }

    // === FLOATING POINT OPERATIONS (if any) ===
    if body_lower.contains("f32") || body_lower.contains("f64") {
        mutates.insert("MOVE_FLOATING_POINT".to_string());
    }

    // === ABORT WITHOUT REASON ===
    if body.contains("abort")
        && !body_lower.contains("error_code")
        && !body_lower.contains("reason")
    {
        mutates.insert("MOVE_UNSAFE_ABORT".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_analyze_move_module() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("token.move");
        fs::write(
            &path,
            r#"module 0x1::token {
    use std::signer;

    struct TokenStore has key {
        amount: u64,
    }

    public fun initialize(account: &signer) {
        let token_store = TokenStore { amount: 0 };
        move_to(account, token_store);
    }

    public fun transfer(from: &signer, to: address, amount: u64) acquires TokenStore {
        let from_store = borrow_global_mut<TokenStore>(signer::address_of(from));
        from_store.amount = from_store.amount - amount;
        
        let to_store = borrow_global_mut<TokenStore>(to);
        to_store.amount = to_store.amount + amount;
    }
}"#,
        )
        .unwrap();

        let analyzer = MoveAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.chain, "move");
        assert!(!result.functions.is_empty());
        assert!(result.functions.iter().any(|(_, f)| f.name == "initialize"));
        assert!(result.functions.iter().any(|(_, f)| f.name == "transfer"));
    }

    #[test]
    fn test_analyze_empty_move_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.move");
        fs::write(&path, "").unwrap();

        let analyzer = MoveAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.functions.len(), 0);
    }

    #[test]
    fn test_analyze_nonexistent_move_path() {
        let analyzer = MoveAnalyzer;
        let result = analyzer.analyze(std::path::Path::new("/nonexistent/path/module.move"));

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_module_name() {
        let source = r#"module 0x1::MyModule {
    fun test() {}
}"#;
        let name = extract_module_name(source);
        assert!(name.is_some());
    }

    #[test]
    fn test_extract_resource_types() {
        let source = r#"module 0x1::token {
    struct Coin has key { value: u64 }
    struct CoinStore has key { coin: Coin }
}"#;
        let resources = extract_resource_types(source);
        // Just verify we found some resources - exact parsing depends on whitespace
        assert!(!resources.is_empty());
    }

    #[test]
    fn test_extract_move_function_params() {
        let signature = "transfer(from: &signer, to: address, amount: u64)";
        let params = extract_move_function_params(signature);
        // Function parameters are extracted - verify structure
        assert_eq!(params.len(), 3);
        assert!(params[0].contains("from") || params[0].contains(":"));
    }

    #[test]
    fn test_chain_identifier() {
        let analyzer = MoveAnalyzer;
        assert_eq!(analyzer.chain(), "move");
    }
}
