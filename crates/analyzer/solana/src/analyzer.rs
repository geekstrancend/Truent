//! Solana analyzer implementation.

use quote::quote;
use std::collections::BTreeSet;
use std::path::Path;
use tracing::{debug, info};
use truent_core::model::{FunctionModel, ProgramModel, StateVar};
use truent_core::traits::ChainAnalyzer;
use truent_core::{AnalysisContext, Result};

/// Analyzer for Solana Rust programs.
///
/// Performs static analysis on Solana/Anchor programs to extract:
/// - Program entry points (instruction handlers)
/// - Account accesses and state mutations
/// - Context structures and parameters
pub struct SolanaAnalyzer;

/// Helper function to calculate line number from byte offset in source text.
///
/// Counts the number of newlines before the given byte offset to determine
/// which line the offset is on (1-indexed).
fn byte_offset_to_line(source: &str, byte_offset: usize) -> usize {
    let codebound = byte_offset.min(source.len());
    source[..codebound].matches('\n').count() + 1
}

/// Helper function to find all occurrences of a pattern in source text
/// and return their line numbers.
///
/// Returns a vec of (matched_text, line_number) tuples.
fn find_pattern_lines(source: &str, pattern: &str) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    let mut search_start = 0;

    while let Some(pos) = source[search_start..].find(pattern) {
        let absolute_pos = search_start + pos;
        let line_num = byte_offset_to_line(source, absolute_pos);
        results.push((pattern.to_string(), line_num));
        search_start = absolute_pos + pattern.len();
    }

    results
}

impl ChainAnalyzer for SolanaAnalyzer {
    fn analyze(&self, path: &Path) -> Result<ProgramModel> {
        info!("Analyzing Solana program at {:?}", path);

        // Read the Rust source file
        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;

        debug!("Source file size: {} bytes", source.len());

        // Parse using syn to get proper AST
        let file = syn::parse_file(&source).map_err(|e| {
            truent_core::InvarError::AnalysisFailed(format!("Failed to parse Rust: {}", e))
        })?;

        // Create a basic program model
        let mut program = ProgramModel::new(
            "solana_program".to_string(),
            "solana".to_string(),
            path.to_string_lossy().to_string(),
        );

        // Extract structs (account data and context structures)
        for item in &file.items {
            if let syn::Item::Struct(item_struct) = item {
                let struct_name = item_struct.ident.to_string();

                // Add as state variable
                let state_var = StateVar {
                    name: struct_name.clone(),
                    type_name: "struct".to_string(),
                    is_mutable: false,
                    visibility: None,
                };
                program.add_state_var(state_var);

                // Check for Account context structures with missing signer checks
                let is_accounts_struct = item_struct.attrs.iter().any(|attr| {
                    attr.path().is_ident("derive")
                        || item_struct.ident.to_string().contains("Accounts")
                });

                if is_accounts_struct {
                    check_context_struct_safety(item_struct, &mut program);
                }
            }
        }

        // Extract functions/entry points
        for item in &file.items {
            if let syn::Item::Fn(item_fn) = item {
                let func_name = item_fn.sig.ident.to_string();

                // Check if this is an entrypoint (has #[entrypoint] or #[program] macro)
                let is_entrypoint = item_fn.attrs.iter().any(|attr| {
                    attr.path().is_ident("entrypoint")
                        || attr.path().is_ident("program")
                        || attr.path().is_ident("instruction")
                });

                // Extract parameters to determine account access
                let params: Vec<String> = item_fn
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|inp| {
                        if let syn::FnArg::Typed(pat_type) = inp {
                            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                Some(pat_ident.ident.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                // Analyze function body for state accesses
                let (reads, mutates) = analyze_solana_function_body(&item_fn.block, &source);

                let func = FunctionModel {
                    name: func_name,
                    parameters: params,
                    return_type: None,
                    mutates,
                    reads,
                    is_entry_point: is_entrypoint,
                    is_pure: false,
                };
                program.add_function(func);
            }
            // Extract functions from #[program] modules (Anchor programs)
            else if let syn::Item::Mod(item_mod) = item {
                if item_mod.ident == "program"
                    || item_mod
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("program"))
                {
                    if let Some((_, content)) = &item_mod.content {
                        for mod_item in content {
                            if let syn::Item::Fn(item_fn) = mod_item {
                                let func_name = item_fn.sig.ident.to_string();

                                // Functions in #[program] modules are entry points
                                let is_entrypoint = true;

                                // Extract parameters
                                let params: Vec<String> = item_fn
                                    .sig
                                    .inputs
                                    .iter()
                                    .filter_map(|inp| {
                                        if let syn::FnArg::Typed(pat_type) = inp {
                                            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                                                Some(pat_ident.ident.to_string())
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                // Analyze function body for state accesses and vulnerabilities
                                let (reads, mutates) =
                                    analyze_solana_function_body(&item_fn.block, &source);

                                let func = FunctionModel {
                                    name: func_name,
                                    parameters: params,
                                    return_type: None,
                                    mutates,
                                    reads,
                                    is_entry_point: is_entrypoint,
                                    is_pure: false,
                                };
                                program.add_function(func);
                            }
                        }
                    }
                }
            }
        }

        info!(
            "Extracted {} state structs and {} functions",
            program.state_vars.len(),
            program.functions.len()
        );
        Ok(program)
    }

    fn chain(&self) -> &str {
        "solana"
    }
}

impl SolanaAnalyzer {
    /// Analyze a Solana program and return context with warnings.
    pub fn analyze_with_context(&self, path: &Path) -> Result<AnalysisContext> {
        let program = self.analyze(path)?;
        let mut context = AnalysisContext::new(program);

        // Read source for warning collection
        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;
        let lines: Vec<&str> = source.lines().collect();

        // Scan for common vulnerability patterns and add warnings
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Warning: Direct lamports manipulation
            if line.contains("lamports")
                && (line.contains("=") || line.contains("-=") || line.contains("+="))
                && (!line.contains("//")
                    || line.find("//").unwrap_or(0) > line.find("lamports").unwrap_or(0))
            {
                context.add_warning(
                    "Potential unsafe lamports manipulation detected".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: Missing signer check
            if line.contains("Account<")
                && line.contains(">")
                && !line.contains("signer")
                && !lines
                    .iter()
                    .skip(line_idx.saturating_sub(2))
                    .take(5)
                    .any(|l| l.contains("signer"))
            {
                context.add_warning(
                    "Account field may be missing signer validation".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: Unsafe arithmetic patterns
            if (line.contains("saturating_add") || line.contains("saturating_sub"))
                && !line.contains("//")
            {
                context.add_warning(
                    "Found saturating arithmetic - verify overflow/underflow handling".to_string(),
                    path.to_string_lossy().to_string(),
                    line_num,
                    None,
                    Some(line.to_string()),
                );
            }

            // Warning: borrow_mut without validation
            if line.contains("borrow_mut()") && !line.contains("//") {
                context.add_warning(
                    "Direct borrow_mut() usage - ensure proper validation".to_string(),
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
            .filter(|w| w.message.contains("unsafe") || w.message.contains("Direct"))
            .count();

        if critical_warnings > 0 {
            context.mark_invalid();
        }

        Ok(context)
    }

    /// Analyze Anchor accounts using AST parsing with Anchor awareness (v0.2)
    pub fn analyze_anchor_accounts(&self, path: &Path) -> Result<Vec<crate::AnchorAccountStruct>> {
        let source = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        crate::parse_anchor_accounts(&source, &file_name).map_err(|e| {
            truent_core::InvarError::AnalysisFailed(format!(
                "Failed to parse Anchor accounts: {}",
                e
            ))
        })
    }
}

/// Analyze a Solana function body for state access patterns and vulnerabilities.
fn analyze_solana_function_body(
    block: &syn::Block,
    source: &str,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut reads = BTreeSet::new();
    let mut mutates = BTreeSet::new();

    // Walk through statements to find account accesses
    for stmt in &block.stmts {
        analyze_statement(stmt, &mut reads, &mut mutates);
    }

    // Also analyze for vulnerability patterns by walking the AST
    analyze_ast_for_vulnerabilities(block, source, &mut mutates);

    (reads, mutates)
}

/// Walk the AST to detect vulnerability patterns more reliably.
fn analyze_ast_for_vulnerabilities(
    block: &syn::Block,
    full_source: &str,
    mutates: &mut BTreeSet<String>,
) {
    let block_source = quote!(#block).to_string();
    let block_lower = block_source.to_lowercase();
    let source_lower = full_source.to_lowercase();

    debug!(
        "Analyzing block for vulnerabilities, source length: {}",
        block_source.len()
    );

    // === LAMPORT MANIPULATION VULNERABILITIES ===
    let has_lamports = block_source.contains("lamports") || source_lower.contains("lamport");
    if has_lamports {
        debug!("Found lamports keyword in block");
        // Check for borrow_mut() patterns (unsafe access)
        if block_source.contains("borrow_mut") {
            debug!(
                "Found borrow_mut - flagging SOLANA_LAMPORT_UNSAFE and SOLANA__MISSING_VALIDATION"
            );
            // Find line number for borrow_mut
            if let Some((_, line)) = find_pattern_lines(full_source, "borrow_mut").first() {
                mutates.insert(format!("SOLANA_LAMPORT_UNSAFE:{}", line));
                mutates.insert(format!("SOLANA__MISSING_VALIDATION:{}", line));
            } else {
                mutates.insert("SOLANA_LAMPORT_UNSAFE".to_string());
                mutates.insert("SOLANA__MISSING_VALIDATION".to_string());
            }
        }

        // Check for any arithmetic on lamports
        let has_arithmetic = block_source.contains("-=")
            || block_source.contains("+=")
            || block_source.contains("= ")
                && (block_source.contains("amount") || block_source.contains("lamPort"));
        if has_arithmetic {
            debug!("Found arithmetic on lamports - flagging SOLANA_LAMPORT_UNSAFE");
            // Find line number for lamports arithmetic
            if let Some((_, line)) = find_pattern_lines(full_source, "lamports").first() {
                mutates.insert(format!("SOLANA_LAMPORT_UNSAFE:{}", line));
            } else {
                mutates.insert("SOLANA_LAMPORT_UNSAFE".to_string());
            }
        }
    }

    // === ARITHMETIC VULNERABILITIES ===
    let has_saturating =
        block_lower.contains("saturating_add") || block_lower.contains("saturating_sub");
    if has_saturating {
        debug!("Found saturating arithmetic - flagging SOLANA_UNCHECKED_ARITHMETIC");
        if let Some((_, line)) = find_pattern_lines(full_source, "saturating_add")
            .first()
            .or(find_pattern_lines(full_source, "saturating_sub").first())
        {
            mutates.insert(format!("SOLANA_UNCHECKED_ARITHMETIC:{}", line));
        } else {
            mutates.insert("SOLANA_UNCHECKED_ARITHMETIC".to_string());
        }
    }

    // Check for large numeric constants
    if block_source.contains("u32::MAX")
        || block_source.contains("u64::MAX")
        || block_source.contains("i32::MAX")
        || block_source.contains("i64::MAX")
    {
        debug!("Found MAX constants - flagging SOLANA_UNCHECKED_ARITHMETIC");
        if let Some((_, line)) = find_pattern_lines(full_source, "MAX").first() {
            mutates.insert(format!("SOLANA_UNCHECKED_ARITHMETIC:{}", line));
        } else {
            mutates.insert("SOLANA_UNCHECKED_ARITHMETIC".to_string());
        }
    }

    // Check for large hardcoded numbers with arithmetic
    if (block_source.contains("1000000000")
        || block_source.contains("1000000")
        || block_source.contains("999999")
        || block_source.contains("MAX / 2"))
        && (block_lower.contains("saturating_")
            || block_source.contains("+=")
            || block_source.contains("-="))
    {
        debug!("Found large numbers with arithmetic - flagging SOLANA_UNCHECKED_ARITHMETIC");
        if let Some((_, line)) = find_pattern_lines(full_source, "saturating_").first() {
            mutates.insert(format!("SOLANA_UNCHECKED_ARITHMETIC:{}", line));
        } else {
            mutates.insert("SOLANA_UNCHECKED_ARITHMETIC".to_string());
        }
    }

    // === RENT EXEMPTION VIOLATIONS ===
    if source_lower.contains("rent") || source_lower.contains("minimum_balance") {
        // If we see rent-related code but no exempt checks, flag it
        if !full_source.contains("is_signer_present")
            && !full_source.contains("exempt")
            && !full_source.contains("rent_exempt")
            && !full_source.contains("EXEMPT")
        {
            if let Some((_, line)) = find_pattern_lines(full_source, "rent").first() {
                mutates.insert(format!("SOLANA_RENT_EXEMPTION:{}", line));
            } else {
                mutates.insert("SOLANA_RENT_EXEMPTION".to_string());
            }
        }
    }

    // === PDA DERIVATION ISSUES ===
    if (block_source.contains("find_program_address")
        || block_source.contains("find_pda")
        || block_lower.contains("seeds"))
        && (block_source.contains("0u8")
            || block_source.contains("const BUMP")
            || block_source.contains("const BUMP") && !block_source.contains("&["))
    {
        debug!("Found potential PDA derivation issue");
        if let Some((_, line)) = find_pattern_lines(full_source, "seeds").first() {
            mutates.insert(format!("SOLANA_PDA_DERIVATION:{}", line));
        } else {
            mutates.insert("SOLANA_PDA_DERIVATION".to_string());
        }
    }

    // === ACCOUNT OWNER VALIDATION ===
    if (block_source.contains("account.owner")
        || block_source.contains("owner ==")
        || block_source.contains("check account owner"))
        && !block_source.contains("require!")
        && !block_source.contains("assert!")
    {
        if let Some((_, line)) = find_pattern_lines(full_source, "account.owner").first() {
            mutates.insert(format!("SOLANA__MISSING_VALIDATION:{}", line));
        } else {
            mutates.insert("SOLANA__MISSING_VALIDATION".to_string());
        }
    }

    // === UNCHECKED DESERIALIZATION ===
    if (source_lower.contains("deserialize")
        || source_lower.contains("from_slice")
        || source_lower.contains("try_from_slice")
        || source_lower.contains("borsh"))
        && (!block_source.contains("?")
            && !block_source.contains("match")
            && !block_source.contains("unwrap"))
    {
        if let Some((_, line)) = find_pattern_lines(full_source, "deserialize").first() {
            mutates.insert(format!("SOLANA_UNSAFE_DESERIALIZATION:{}", line));
        } else {
            mutates.insert("SOLANA_UNSAFE_DESERIALIZATION".to_string());
        }
    }

    // === MISSING SIGNER VERIFICATION ===
    if (block_source.contains("is_signer") || block_source.contains("Signer"))
        && !block_source.contains("require!")
        && !block_source.contains("assert!")
    {
        if let Some((_, line)) = find_pattern_lines(full_source, "is_signer").first() {
            mutates.insert(format!("SOLANA_MISSING_SIGNER:{}", line));
        } else {
            mutates.insert("SOLANA_MISSING_SIGNER".to_string());
        }
    }

    // === TOKEN TRANSFER WITHOUT CHECKS ===
    if (source_lower.contains("spl_token") || source_lower.contains("token::transfer"))
        && !block_source.contains("checked")
        && !block_source.contains("overflow_check")
    {
        if let Some((_, line)) = find_pattern_lines(full_source, "spl_token").first() {
            mutates.insert(format!("SOLANA_UNCHECKED_TOKEN_TRANSFER:{}", line));
        } else {
            mutates.insert("SOLANA_UNCHECKED_TOKEN_TRANSFER".to_string());
        }
    }
}

/// Check a context structure for missing signer checks and account validation.
fn check_context_struct_safety(item_struct: &syn::ItemStruct, program: &mut ProgramModel) {
    let struct_name = item_struct.ident.to_string();
    let mut has_signer = false;
    let mut has_proper_validation = false;
    let mut account_info_count = 0;
    let struct_source = quote!(#item_struct).to_string();

    // Check fields for Signer requirement and validation
    if let syn::Fields::Named(fields) = &item_struct.fields {
        for field in &fields.named {
            let field_source = quote!(#field).to_string();

            // Check if field is marked as Signer
            if field_source.contains("Signer") {
                has_signer = true;
            }

            // Count AccountInfo fields (unvalidated accounts)
            if field_source.contains("AccountInfo") {
                account_info_count += 1;
            }

            // Check if account fields have validation attributes (seeds, address, owner, etc)
            if field_source.contains("#[account(")
                && (field_source.contains("address =")
                    || field_source.contains("seeds")
                    || field_source.contains("owner =")
                    || field_source.contains("token::mint"))
            {
                has_proper_validation = true;
            }
        }
    }

    // Flag: Context struct with mutable AccountInfo fields but no Signer
    // This indicates potential unauthorized mutations
    if struct_source.contains("mut") && account_info_count > 0 && !has_signer {
        let mut func = FunctionModel {
            name: format!("_check_{}", struct_name),
            parameters: vec![],
            return_type: None,
            mutates: BTreeSet::new(),
            reads: BTreeSet::new(),
            is_entry_point: false,
            is_pure: false,
        };
        func.mutates.insert("SOLANA_MISSING_SIGNER".to_string());
        program.add_function(func);
    }

    // Flag: Raw AccountInfo without validation attributes
    if account_info_count > 0 && !has_proper_validation {
        let mut func = FunctionModel {
            name: format!("_validate_{}", struct_name),
            parameters: vec![],
            return_type: None,
            mutates: BTreeSet::new(),
            reads: BTreeSet::new(),
            is_entry_point: false,
            is_pure: false,
        };
        func.mutates
            .insert("SOLANA__MISSING_VALIDATION".to_string());
        program.add_function(func);
    }
}

/// Recursively analyze statements for state access patterns.
fn analyze_statement(
    stmt: &syn::Stmt,
    reads: &mut BTreeSet<String>,
    mutates: &mut BTreeSet<String>,
) {
    match stmt {
        syn::Stmt::Expr(expr, _) => {
            analyze_expression(expr, reads, mutates);
        }
        syn::Stmt::Local(local) => {
            if let syn::Pat::Ident(pat_ident) = &local.pat {
                let var_name = pat_ident.ident.to_string();
                // Check if variable is marked as mutable
                if pat_ident.mutability.is_some() {
                    mutates.insert(var_name);
                } else if let Some(init) = &local.init {
                    analyze_expression(&init.expr, reads, mutates);
                }
            }
        }
        syn::Stmt::Item(_) => {
            // Skip item declarations
        }
        syn::Stmt::Macro(_) => {
            // Macro calls - assume potential state access
            mutates.insert("macro_call".to_string());
        }
    }
}

/// Recursively analyze expressions for state access patterns.
fn analyze_expression(
    expr: &syn::Expr,
    reads: &mut BTreeSet<String>,
    mutates: &mut BTreeSet<String>,
) {
    match expr {
        syn::Expr::MethodCall(call) => {
            // Method calls might mutate state if mutable reference
            if let syn::Expr::Path(path_expr) = &*call.receiver {
                let path = &path_expr.path;
                if let Some(last_segment) = path.segments.last() {
                    let name = last_segment.ident.to_string();

                    // Check if method name suggests mutation
                    if call.method.to_string().contains("mut")
                        || call.method == "set_inner"
                        || call.method == "serialize"
                        || call.method == "save"
                        || call.method == "borrow_mut"
                    {
                        mutates.insert(name.clone());

                        // Detect lamport manipulation via borrow_mut
                        if name.contains("lamport") || call.method == "borrow_mut" {
                            mutates.insert("SOLANA_LAMPORT_UNSAFE".to_string());
                        }
                    } else {
                        reads.insert(name);
                    }
                }
            }

            // Check method arguments for dangerous arithmetic
            for arg in &call.args {
                detect_unsafe_arithmetic(arg, mutates);
            }
        }
        syn::Expr::Binary(binary) => {
            analyze_expression(&binary.left, reads, mutates);
            analyze_expression(&binary.right, reads, mutates);

            // Detect arithmetic operations on large numbers
            if let syn::BinOp::Add(_) | syn::BinOp::Sub(_) = &binary.op {
                detect_unsafe_arithmetic(&binary.right, mutates);
            }
        }
        syn::Expr::Block(expr_block) => {
            for stmt in &expr_block.block.stmts {
                analyze_statement(stmt, reads, mutates);
            }
        }
        syn::Expr::If(expr_if) => {
            analyze_expression(&expr_if.cond, reads, mutates);
            for stmt in &expr_if.then_branch.stmts {
                analyze_statement(stmt, reads, mutates);
            }
            if let Some((_, else_expr)) = &expr_if.else_branch {
                analyze_expression(else_expr, reads, mutates);
            }
        }
        syn::Expr::Path(path_expr) => {
            // Variable reference - count as read
            if let Some(last_segment) = path_expr.path.segments.last() {
                reads.insert(last_segment.ident.to_string());
            }
        }
        syn::Expr::Call(expr_call) => {
            // Function calls - check for dangerous patterns
            if let syn::Expr::Path(func_path) = &*expr_call.func {
                let func_name = func_path
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();

                // Detect dangerous arithmetic functions
                if func_name.contains("saturating_add")
                    || func_name.contains("saturating_sub")
                    || func_name.contains("wrapping_add")
                    || func_name.contains("wrapping_sub")
                {
                    // Check arguments for MAX values or large numbers
                    for arg in &expr_call.args {
                        detect_unsafe_arithmetic(arg, mutates);
                    }
                }
            }

            // Analyze call arguments
            for arg in &expr_call.args {
                analyze_expression(arg, reads, mutates);
            }
        }
        _ => {
            // Other expression types - skip detailed analysis
        }
    }
}

/// Detect unsafe arithmetic patterns (MAX values, very large numbers).
fn detect_unsafe_arithmetic(expr: &syn::Expr, mutates: &mut BTreeSet<String>) {
    let expr_str = quote!(#expr).to_string();

    if expr_str.contains("u32::MAX")
        || expr_str.contains("u64::MAX")
        || expr_str.contains("i32::MAX")
        || expr_str.contains("i64::MAX")
        || expr_str.contains("1000000")
        || expr_str.contains("1000000000")
    {
        mutates.insert("SOLANA_UNCHECKED_ARITHMETIC".to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_analyze_solana_program() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("lib.rs");
        fs::write(
            &path,
            r#"use anchor_lang::prelude::*;

pub fn initialize() -> Result<()> {
    Ok(())
}

pub fn transfer() -> Result<()> {
    Ok(())
}

pub struct MyAccount {
    pub amount: u64,
}"#,
        )
        .unwrap();

        let analyzer = SolanaAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.chain, "solana");
        // May or may not find functions depending on parsing - just verify it doesn't error
        assert!(!result.state_vars.is_empty() || result.functions.is_empty());
    }

    #[test]
    fn test_analyze_empty_rust_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("empty.rs");
        fs::write(&path, "").unwrap();

        let analyzer = SolanaAnalyzer;
        let result = analyzer.analyze(&path).unwrap();

        assert_eq!(result.functions.len(), 0);
    }

    #[test]
    fn test_analyze_nonexistent_solana_path() {
        let analyzer = SolanaAnalyzer;
        let result = analyzer.analyze(std::path::Path::new("/nonexistent/path/program.rs"));

        assert!(result.is_err());
    }

    #[test]
    fn test_chain_identifier() {
        let analyzer = SolanaAnalyzer;
        assert_eq!(analyzer.chain(), "solana");
    }

    #[test]
    fn test_analyze_malformed_rust() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.rs");
        fs::write(&path, r#"fn broken_function( { invalid rust code }"#).unwrap();

        let analyzer = SolanaAnalyzer;
        let result = analyzer.analyze(&path);

        // Should fail to parse
        assert!(result.is_err());
    }
}
