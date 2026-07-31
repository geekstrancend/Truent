//! AST extensions and utilities for IR.

use truent_core::model::{Expression, FunctionModel, StateVar};
use std::collections::BTreeMap;

/// A directed dependency graph for tracking state mutation dependencies.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Function → {Functions it calls}
    pub call_graph: BTreeMap<String, Vec<String>>,

    /// State var → {Functions that mutate it}
    pub mutation_sources: BTreeMap<String, Vec<String>>,

    /// Function → {State vars it reads}
    pub read_deps: BTreeMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            call_graph: BTreeMap::new(),
            mutation_sources: BTreeMap::new(),
            read_deps: BTreeMap::new(),
        }
    }

    /// Add a call relationship: caller → callee.
    pub fn add_call(&mut self, caller: String, callee: String) {
        self.call_graph.entry(caller).or_default().push(callee);
    }

    /// Add a mutation: function mutates state_var.
    pub fn add_mutation(&mut self, state_var: String, function: String) {
        self.mutation_sources
            .entry(state_var)
            .or_default()
            .push(function);
    }

    /// Add a read dependency.
    pub fn add_read(&mut self, function: String, state_var: String) {
        self.read_deps.entry(function).or_default().push(state_var);
    }

    /// Get all transitive mutations caused by a function.
    pub fn transitive_mutations(&self, func: &str) -> Vec<String> {
        let mut visited = std::collections::BTreeSet::new();
        let mut queue = vec![func.to_string()];
        let mut mutations = Vec::new();

        while let Some(current) = queue.pop() {
            if visited.insert(current.clone()) {
                // Get direct calls from this function
                if let Some(callees) = self.call_graph.get(&current) {
                    for callee in callees {
                        queue.push(callee.clone());
                    }
                }
            }
        }

        // Collect all mutations from visited functions
        for visited_fn in visited {
            for (state_var, sources) in &self.mutation_sources {
                if sources.contains(&visited_fn) {
                    mutations.push(state_var.clone());
                }
            }
        }

        mutations
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended analysis context with dependency tracking.
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    /// Available state variables in scope.
    pub available_vars: BTreeMap<String, StateVar>,

    /// Available functions.
    pub available_functions: BTreeMap<String, FunctionModel>,
}

impl ExpressionContext {
    /// Create a new expression context.
    pub fn new() -> Self {
        Self {
            available_vars: BTreeMap::new(),
            available_functions: BTreeMap::new(),
        }
    }

    /// Validate that an expression only references available identifiers.
    pub fn validate_expression(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Boolean(_) | Expression::Int(_) => Ok(()),
            Expression::Var(name) => {
                if self.available_vars.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expression::LayerVar { layer: _, var } => {
                if self.available_vars.contains_key(var) {
                    Ok(())
                } else {
                    Err(format!("Undefined layer variable: {}", var))
                }
            }
            Expression::PhaseQualifiedVar {
                phase: _,
                layer: _,
                var,
            } => {
                if self.available_vars.contains_key(var) {
                    Ok(())
                } else {
                    Err(format!("Undefined phase-qualified variable: {}", var))
                }
            }
            Expression::PhaseConstraint {
                phase: _,
                constraint,
            } => self.validate_expression(constraint),
            Expression::CrossPhaseRelation {
                phase1: _,
                expr1,
                phase2: _,
                expr2,
                op: _,
            } => {
                self.validate_expression(expr1)?;
                self.validate_expression(expr2)
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)
            }
            Expression::Logical { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)
            }
            Expression::Not(e) => self.validate_expression(e),
            Expression::FunctionCall { name, args } => {
                if !self.available_functions.contains_key(name) {
                    return Err(format!("Undefined function: {}", name));
                }
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Tuple(exprs) => {
                for e in exprs {
                    self.validate_expression(e)?;
                }
                Ok(())
            }
        }
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}
