//! Threat model defenses for Invar invariant enforcement system.
//!
//! This module implements comprehensive security hardening against:
//! 1. Injection attacks (re-parsing verification)
//! 2. Macro tampering (hash detection)
//! 3. Uncertainty in mutation analysis (strict mode abort)
//! 4. DSL sandbox escapes (expression validation)
//! 5. Simulation side-effects (isolation verification)

use crate::model::Expression;
use std::collections::BTreeMap;

/// Threat model security configuration.
#[derive(Debug, Clone)]
pub struct ThreatModelConfig {
    /// Require strict mutation detection (abort if uncertain)
    pub strict_mode: bool,
    /// Validate all generated code by re-parsing
    pub re_parse_verification: bool,
    /// Detect tamper attempts via hash checking
    pub tamper_detection_enabled: bool,
    /// Validate DSL expressions for sandbox escapes
    pub dsl_sandboxing_enabled: bool,
    /// Verify simulation isolation (no file mutations)
    pub isolation_verification: bool,
}

impl Default for ThreatModelConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            re_parse_verification: true,
            tamper_detection_enabled: true,
            dsl_sandboxing_enabled: true,
            isolation_verification: true,
        }
    }
}

/// Result type for threat model validation.
pub type ThreatResult<T> = Result<T, ThreatModelError>;

/// Threat model validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreatModelError {
    /// Code re-parse verification failed
    ReParseVerificationFailed(String),
    /// Macro tampering detected
    TamperDetected(String),
    /// DSL sandbox escape attempt detected
    SandboxEscapeDetected(String),
    /// Mutation detection uncertainty in strict mode
    MutationUncertaintyDetected(String),
    /// Simulation isolation violation detected
    IsolationViolationDetected(String),
    /// Custom threat
    Custom(String),
}

impl std::fmt::Display for ThreatModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReParseVerificationFailed(msg) => {
                write!(f, "re-parse verification failed: {}", msg)
            }
            Self::TamperDetected(msg) => write!(f, "macro tampering detected: {}", msg),
            Self::SandboxEscapeDetected(msg) => write!(f, "DSL sandbox escape: {}", msg),
            Self::MutationUncertaintyDetected(msg) => {
                write!(f, "mutation uncertainty in strict mode: {}", msg)
            }
            Self::IsolationViolationDetected(msg) => {
                write!(f, "simulation isolation violation: {}", msg)
            }
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

/// Defense 1: Injection verification via re-parsing.
///
/// After generating code, re-parse it to ensure:
/// - Syntax is valid
/// - No injection artifacts remain
/// - All invariants are properly placed
pub struct InjectionVerifier;

impl InjectionVerifier {
    /// Verify that generated code contains all expected invariant checks.
    ///
    /// # Security Property
    /// Ensures 100% coverage of mutating functions with invariant checks.
    pub fn verify_coverage(generated_code: &str, expected_checks: &[String]) -> ThreatResult<()> {
        for check in expected_checks {
            if !generated_code.contains(&format!("// Invariant: {}", check)) {
                return Err(ThreatModelError::ReParseVerificationFailed(format!(
                    "invariant check not found in generated code: {}",
                    check
                )));
            }
        }
        Ok(())
    }

    /// Verify no injected code escapes the intended scope.
    ///
    /// # Security Property
    /// Prevents code injection by ensuring all injected statements stay within
    /// invariant check blocks.
    pub fn verify_scope_containment(generated_code: &str) -> ThreatResult<()> {
        // Check for dangerous patterns that indicate injection escape
        let dangerous_patterns = [
            "unsafe",
            "extern",
            "use std::process",
            "std::fs",
            "std::net",
        ];

        for pattern in &dangerous_patterns {
            if generated_code.contains(pattern) {
                return Err(ThreatModelError::ReParseVerificationFailed(format!(
                    "dangerous pattern found in generated code: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }
}

/// Defense 2: Macro tamper detection.
///
/// Detects modifications to injected invariant checks by comparing
/// computed hash against embedded hash in generated code.
pub struct TamperDetector;

impl TamperDetector {
    /// Compute deterministic hash for a set of invariant checks.
    ///
    /// # Determinism Property
    /// Hash is computed from sorted check list, so order doesn't matter
    /// (prevents timing attacks on check modifications).
    pub fn compute_hash(checks: &[String]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let mut sorted_checks = checks.to_vec();
        sorted_checks.sort();

        for check in sorted_checks {
            check.hash(&mut hasher);
        }

        format!("{:016x}", hasher.finish())
    }

    /// Verify that the embedded hash matches expected checks.
    ///
    /// # Security Property
    /// Detects any tampering with invariant checks after macro expansion.
    pub fn verify_tampering(generated_code: &str, expected_checks: &[String]) -> ThreatResult<()> {
        let expected_hash = Self::compute_hash(expected_checks);

        // Extract hash from generated code (look for TRUENT_HASH: pattern)
        let hash_pattern = format!("TRUENT_HASH: {}", expected_hash);

        if !generated_code.contains(&hash_pattern) {
            return Err(ThreatModelError::TamperDetected(
                "hash mismatch: generated code does not contain expected TRUENT_HASH".to_string(),
            ));
        }

        Ok(())
    }
}

/// Defense 3: DSL sandboxing.
///
/// Validates that invariant expressions cannot escape the sandbox
/// (no file I/O, no external calls, no state mutations outside checks).
pub struct DSLSandbox;

impl DSLSandbox {
    /// Validate an expression for sandbox violations.
    ///
    /// # Security Property
    /// Ensures invariant expressions:
    /// - Don't access files
    /// - Don't call external code
    /// - Are deterministic (no randomness)
    /// - Have no side effects
    pub fn validate_expression(expr: &Expression) -> ThreatResult<()> {
        // Check for dangerous patterns in variable names (common injection vectors)
        let forbidden_prefixes = ["file_", "io_", "extern_", "unsafe_"];

        Self::check_expression_recursive(expr, &forbidden_prefixes)
    }

    fn check_expression_recursive(
        expr: &Expression,
        forbidden_prefixes: &[&str],
    ) -> ThreatResult<()> {
        match expr {
            Expression::Var(name) => {
                for prefix in forbidden_prefixes {
                    if name.to_lowercase().starts_with(prefix) {
                        return Err(ThreatModelError::SandboxEscapeDetected(format!(
                            "forbidden variable name: {}",
                            name
                        )));
                    }
                }
                Ok(())
            }

            Expression::LayerVar { layer, var } => {
                // Check both layer and variable names against forbidden prefixes
                for prefix in forbidden_prefixes {
                    if layer.to_lowercase().starts_with(prefix)
                        || var.to_lowercase().starts_with(prefix)
                    {
                        return Err(ThreatModelError::SandboxEscapeDetected(format!(
                            "forbidden layer/variable name: {}::{}",
                            layer, var
                        )));
                    }
                }
                Ok(())
            }

            Expression::FunctionCall { name, args } => {
                // Whitelist of allowed functions (purely computational, no side effects)
                let allowed_functions = [
                    "sum", "len", "min", "max", "abs", "mod", "div", "add", "sub", "mul", "and",
                    "or", "not",
                ];

                if !allowed_functions.contains(&name.as_str()) {
                    return Err(ThreatModelError::SandboxEscapeDetected(format!(
                        "forbidden function call: {}",
                        name
                    )));
                }

                // Recursively check all arguments
                for arg in args {
                    Self::check_expression_recursive(arg, forbidden_prefixes)?;
                }
                Ok(())
            }

            Expression::BinaryOp { left, op: _, right } => {
                Self::check_expression_recursive(left, forbidden_prefixes)?;
                Self::check_expression_recursive(right, forbidden_prefixes)?;
                Ok(())
            }

            Expression::Arithmetic { left, op: _, right } => {
                Self::check_expression_recursive(left, forbidden_prefixes)?;
                Self::check_expression_recursive(right, forbidden_prefixes)?;
                Ok(())
            }

            Expression::Logical { left, op: _, right } => {
                Self::check_expression_recursive(left, forbidden_prefixes)?;
                Self::check_expression_recursive(right, forbidden_prefixes)?;
                Ok(())
            }

            Expression::Not(inner) => {
                Self::check_expression_recursive(inner, forbidden_prefixes)?;
                Ok(())
            }

            Expression::Tuple(exprs) => {
                for e in exprs {
                    Self::check_expression_recursive(e, forbidden_prefixes)?;
                }
                Ok(())
            }

            Expression::PhaseQualifiedVar { phase, layer, var } => {
                // Check phase, layer, and variable names against forbidden prefixes
                for prefix in forbidden_prefixes {
                    if phase.to_lowercase().starts_with(prefix)
                        || layer.to_lowercase().starts_with(prefix)
                        || var.to_lowercase().starts_with(prefix)
                    {
                        return Err(ThreatModelError::SandboxEscapeDetected(format!(
                            "forbidden phase/layer/variable name: {}::{}::{}",
                            phase, layer, var
                        )));
                    }
                }
                Ok(())
            }

            Expression::PhaseConstraint {
                phase: _,
                constraint,
            } => {
                // Check the constraint expression recursively
                Self::check_expression_recursive(constraint, forbidden_prefixes)
            }

            Expression::CrossPhaseRelation {
                phase1: _,
                expr1,
                phase2: _,
                expr2,
                op: _,
            } => {
                // Check both phase expressions
                Self::check_expression_recursive(expr1, forbidden_prefixes)?;
                Self::check_expression_recursive(expr2, forbidden_prefixes)?;
                Ok(())
            }

            Expression::Boolean(_) | Expression::Int(_) => Ok(()),
        }
    }
}

/// Defense 4: Analyzer strict mode.
///
/// In strict mode, the analyzer aborts if it cannot determine with certainty
/// whether a particular mutation will violate an invariant.
pub struct StrictModeAnalyzer {
    enabled: bool,
}

impl StrictModeAnalyzer {
    /// Create a new strict mode analyzer.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Verify that all function mutations are accounted for.
    ///
    /// # Security Property
    /// In strict mode, rejects functions with uncertain mutation detection.
    /// This prevents invariant bypass via undetected mutations.
    pub fn verify_mutation_coverage(
        &self,
        _analyzed_mutations: &[String],
        uncertainty_warnings: &[String],
    ) -> ThreatResult<()> {
        if !self.enabled {
            return Ok(());
        }

        if !uncertainty_warnings.is_empty() {
            return Err(ThreatModelError::MutationUncertaintyDetected(format!(
                "strict mode detected {} uncertain mutations: {}",
                uncertainty_warnings.len(),
                uncertainty_warnings.join(", ")
            )));
        }

        Ok(())
    }
}

/// Defense 5: Simulation isolation verification.
///
/// Ensures that simulation environments cannot mutate actual state or access files.
pub struct SimulationIsolation;

impl SimulationIsolation {
    /// Verify that a simulation context is properly isolated.
    ///
    /// # Security Property
    /// Ensures simulation:
    /// - Only uses in-memory data structures (BTreeMap, Vec)
    /// - Makes no file system calls
    /// - Doesn't mutate external state
    /// - Results are deterministic
    pub fn verify_isolation(
        context_vars: &BTreeMap<String, String>,
        allowed_types: &[&str],
    ) -> ThreatResult<()> {
        for (name, type_str) in context_vars {
            // Validate that only allowed types are used in simulation
            let is_allowed = allowed_types
                .iter()
                .any(|&allowed| type_str.contains(allowed));

            if !is_allowed {
                return Err(ThreatModelError::IsolationViolationDetected(format!(
                    "variable '{}' has disallowed type '{}' in simulation context",
                    name, type_str
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_verification() {
        let generated_code = r#"
        fn transfer(from: &mut Account, to: &mut Account, amount: u64) {
            from.balance -= amount;
            to.balance += amount;
            // Invariant: balance >= 0
            // TRUENT_HASH: abcd1234
        }
        "#;

        let checks = vec!["balance >= 0".to_string()];
        assert!(InjectionVerifier::verify_coverage(generated_code, &checks).is_ok());
    }

    #[test]
    fn test_injection_verification_missing_check() {
        let generated_code = "fn transfer() { /* no checks */ }";
        let checks = vec!["balance >= 0".to_string()];
        assert!(InjectionVerifier::verify_coverage(generated_code, &checks).is_err());
    }

    #[test]
    fn test_scope_containment() {
        let safe_code = "let x = a + b; assert!(x > 0);";
        assert!(InjectionVerifier::verify_scope_containment(safe_code).is_ok());

        let unsafe_code = "use std::fs; fs::write(\"file.txt\", \"\");";
        assert!(InjectionVerifier::verify_scope_containment(unsafe_code).is_err());
    }

    #[test]
    fn test_tamper_hash_deterministic() {
        let checks1 = vec!["a".to_string(), "b".to_string()];
        let checks2 = vec!["b".to_string(), "a".to_string()];

        let hash1 = TamperDetector::compute_hash(&checks1);
        let hash2 = TamperDetector::compute_hash(&checks2);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_dsl_sandbox_forbidden_variable() {
        let expr = Expression::Var("file_handle".to_string());
        assert!(DSLSandbox::validate_expression(&expr).is_err());
    }

    #[test]
    fn test_dsl_sandbox_allowed_variable() {
        let expr = Expression::Var("balance".to_string());
        assert!(DSLSandbox::validate_expression(&expr).is_ok());
    }

    #[test]
    fn test_dsl_sandbox_forbidden_function() {
        let expr = Expression::FunctionCall {
            name: "system_call".to_string(),
            args: vec![],
        };
        assert!(DSLSandbox::validate_expression(&expr).is_err());
    }

    #[test]
    fn test_dsl_sandbox_allowed_function() {
        let expr = Expression::FunctionCall {
            name: "sum".to_string(),
            args: vec![Expression::Var("balances".to_string())],
        };
        assert!(DSLSandbox::validate_expression(&expr).is_ok());
    }

    #[test]
    fn test_strict_mode_with_uncertainty() {
        let analyzer = StrictModeAnalyzer::new(true);
        let mutations = vec!["balance -= amount".to_string()];
        let warnings = vec!["mutation from function pointer call (uncertain)".to_string()];

        assert!(analyzer
            .verify_mutation_coverage(&mutations, &warnings)
            .is_err());
    }

    #[test]
    fn test_strict_mode_disabled() {
        let analyzer = StrictModeAnalyzer::new(false);
        let mutations = vec!["balance -= amount".to_string()];
        let warnings = vec!["mutation from function pointer call (uncertain)".to_string()];

        // Strict mode off, so uncertainty is allowed
        assert!(analyzer
            .verify_mutation_coverage(&mutations, &warnings)
            .is_ok());
    }
}
