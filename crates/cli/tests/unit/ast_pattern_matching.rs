//! Comprehensive pattern matching tests for Expression AST nodes.
//!
//! These tests verify that all Expression enum variants can be properly
//! matched and their field values extracted correctly. Coverage includes:
//! - All 12 Expression variants
//! - Both Boolean and Int literals
//! - Nested expressions
//! - Field extraction via pattern matching

mod expression_pattern_tests {
    use truent_core::model::{BinaryOp, Expression, LogicalOp};

    // ============================================================================
    // 1. LITERAL TESTS - Boolean and Int
    // ============================================================================

    /// Test Boolean literal pattern matching
    #[test]
    fn test_pattern_match_boolean_literal() {
        let expr = Expression::Boolean(true);

        // Test match with value extraction
        match expr {
            Expression::Boolean(b) => {
                assert!(b, "Boolean value should be extracted correctly");
            }
            _ => panic!("Expected Boolean variant"),
        }

        let expr_false = Expression::Boolean(false);
        match expr_false {
            Expression::Boolean(b) => {
                assert!(!b, "Boolean false value should be extracted correctly");
            }
            _ => panic!("Expected Boolean variant"),
        }
    }

    /// Test Int literal pattern matching (i128)
    #[test]
    fn test_pattern_match_int_literal() {
        let test_values = vec![0i128, 1, 42, 1000, -100, i128::MAX, i128::MIN];

        for value in test_values {
            let expr = Expression::Int(value);

            match expr {
                Expression::Int(i) => {
                    assert_eq!(i, value, "Int value should be extracted correctly");
                }
                _ => panic!("Expected Int variant for value {}", value),
            }
        }
    }

    // ============================================================================
    // 2. VARIABLE REFERENCE TESTS
    // ============================================================================

    /// Test simple Var pattern matching
    #[test]
    fn test_pattern_match_var() {
        let var_names = vec!["balance", "total", "state_value", "x", "my_variable"];

        for name in var_names {
            let expr = Expression::Var(name.to_string());

            match expr {
                Expression::Var(v) => {
                    assert_eq!(v, name, "Variable name should be extracted correctly");
                }
                _ => panic!("Expected Var variant for name {}", name),
            }
        }
    }

    /// Test LayerVar pattern matching with multiple layer types
    #[test]
    fn test_pattern_match_layer_var() {
        let test_cases = vec![
            ("bundler", "nonce"),
            ("account", "balance"),
            ("paymaster", "deposit"),
            ("protocol", "version"),
            ("entrypoint", "safe_mode"),
        ];

        for (layer, var) in test_cases {
            let expr = Expression::LayerVar {
                layer: layer.to_string(),
                var: var.to_string(),
            };

            match expr {
                Expression::LayerVar { layer: l, var: v } => {
                    assert_eq!(l, layer, "Layer should be extracted correctly");
                    assert_eq!(v, var, "Variable should be extracted correctly");
                }
                _ => panic!("Expected LayerVar variant for {}::{}", layer, var),
            }
        }
    }

    /// Test PhaseQualifiedVar pattern matching
    #[test]
    fn test_pattern_match_phase_qualified_var() {
        let test_cases = vec![
            ("validation", "bundler", "nonce"),
            ("execution", "account", "balance"),
            ("settlement", "paymaster", "deposit"),
        ];

        for (phase, layer, var) in test_cases {
            let expr = Expression::PhaseQualifiedVar {
                phase: phase.to_string(),
                layer: layer.to_string(),
                var: var.to_string(),
            };

            match expr {
                Expression::PhaseQualifiedVar {
                    phase: p,
                    layer: l,
                    var: v,
                } => {
                    assert_eq!(p, phase, "Phase should be extracted correctly");
                    assert_eq!(l, layer, "Layer should be extracted correctly");
                    assert_eq!(v, var, "Variable should be extracted correctly");
                }
                _ => panic!(
                    "Expected PhaseQualifiedVar variant for {}::{}::{}",
                    phase, layer, var
                ),
            }
        }
    }

    // ============================================================================
    // 3. CONSTRAINT AND RELATION TESTS
    // ============================================================================

    /// Test PhaseConstraint pattern matching
    #[test]
    fn test_pattern_match_phase_constraint() {
        let inner_expr = Expression::Int(100);
        let expr = Expression::PhaseConstraint {
            phase: "validation".to_string(),
            constraint: Box::new(inner_expr),
        };

        match expr {
            Expression::PhaseConstraint {
                phase: p,
                constraint: c,
            } => {
                assert_eq!(p, "validation", "Phase should be extracted correctly");

                // Test nested pattern matching on constraint
                match *c {
                    Expression::Int(i) => {
                        assert_eq!(i, 100, "Nested Int should be extracted correctly");
                    }
                    _ => panic!("Expected Int in constraint"),
                }
            }
            _ => panic!("Expected PhaseConstraint variant"),
        }
    }

    /// Test CrossPhaseRelation pattern matching
    #[test]
    fn test_pattern_match_cross_phase_relation() {
        let expr1 = Expression::Int(50);
        let expr2 = Expression::Int(100);

        let expr = Expression::CrossPhaseRelation {
            phase1: "validation".to_string(),
            expr1: Box::new(expr1),
            phase2: "execution".to_string(),
            expr2: Box::new(expr2),
            op: BinaryOp::Lt,
        };

        match expr {
            Expression::CrossPhaseRelation {
                phase1: p1,
                expr1: e1,
                phase2: p2,
                expr2: e2,
                op,
            } => {
                assert_eq!(p1, "validation", "Phase1 should be extracted correctly");
                assert_eq!(p2, "execution", "Phase2 should be extracted correctly");
                assert_eq!(op, BinaryOp::Lt, "Operator should be extracted correctly");

                // Test nested pattern matching
                match *e1 {
                    Expression::Int(i) => assert_eq!(i, 50),
                    _ => panic!("Expected Int in expr1"),
                }
                match *e2 {
                    Expression::Int(i) => assert_eq!(i, 100),
                    _ => panic!("Expected Int in expr2"),
                }
            }
            _ => panic!("Expected CrossPhaseRelation variant"),
        }
    }

    // ============================================================================
    // 4. OPERATOR TESTS - BinaryOp and Logical
    // ============================================================================

    /// Test BinaryOp pattern matching with all operator types
    #[test]
    fn test_pattern_match_binary_ops() {
        let operators = vec![
            BinaryOp::Eq,
            BinaryOp::Neq,
            BinaryOp::Lt,
            BinaryOp::Gt,
            BinaryOp::Lte,
            BinaryOp::Gte,
        ];

        let left = Expression::Int(10);
        let right = Expression::Int(20);

        for op in operators {
            let expr = Expression::BinaryOp {
                left: Box::new(left.clone()),
                op,
                right: Box::new(right.clone()),
            };

            match expr {
                Expression::BinaryOp {
                    left: l,
                    op: o,
                    right: r,
                } => {
                    assert_eq!(o, op, "Operator should be extracted correctly");

                    // Test nested pattern matching
                    match *l {
                        Expression::Int(i) => assert_eq!(i, 10),
                        _ => panic!("Expected Int in left"),
                    }
                    match *r {
                        Expression::Int(i) => assert_eq!(i, 20),
                        _ => panic!("Expected Int in right"),
                    }
                }
                _ => panic!("Expected BinaryOp variant for {:?}", op),
            }
        }
    }

    /// Test Logical operation pattern matching
    #[test]
    fn test_pattern_match_logical_ops() {
        let left = Expression::Boolean(true);
        let right = Expression::Boolean(false);

        let operators = vec![LogicalOp::And, LogicalOp::Or];

        for op in operators {
            let expr = Expression::Logical {
                left: Box::new(left.clone()),
                op,
                right: Box::new(right.clone()),
            };

            match expr {
                Expression::Logical {
                    left: l,
                    op: o,
                    right: r,
                } => {
                    assert_eq!(o, op, "Logical operator should be extracted correctly");

                    // Test nested pattern matching
                    match *l {
                        Expression::Boolean(b) => assert!(b),
                        _ => panic!("Expected Boolean in left"),
                    }
                    match *r {
                        Expression::Boolean(b) => assert!(!b),
                        _ => panic!("Expected Boolean in right"),
                    }
                }
                _ => panic!("Expected Logical variant for {:?}", op),
            }
        }
    }

    // ============================================================================
    // 5. UNARY AND COMPOSITE TESTS
    // ============================================================================

    /// Test Not (negation) pattern matching
    #[test]
    fn test_pattern_match_not() {
        let inner = Expression::Boolean(true);
        let expr = Expression::Not(Box::new(inner));

        match expr {
            Expression::Not(inner_expr) => match *inner_expr {
                Expression::Boolean(b) => {
                    assert!(b, "Inner boolean should be extracted correctly");
                }
                _ => panic!("Expected Boolean in Not"),
            },
            _ => panic!("Expected Not variant"),
        }
    }

    /// Test FunctionCall pattern matching
    #[test]
    fn test_pattern_match_function_call() {
        let args = vec![
            Expression::Int(10),
            Expression::Boolean(true),
            Expression::Var("x".to_string()),
        ];

        let expr = Expression::FunctionCall {
            name: "sum_values".to_string(),
            args: args.clone(),
        };

        match expr {
            Expression::FunctionCall { name: n, args: a } => {
                assert_eq!(
                    n, "sum_values",
                    "Function name should be extracted correctly"
                );
                assert_eq!(a.len(), 3, "Should have 3 arguments");

                // Test nested pattern matching on arguments
                match &a[0] {
                    Expression::Int(i) => assert_eq!(*i, 10),
                    _ => panic!("First arg should be Int"),
                }
                match &a[1] {
                    Expression::Boolean(b) => assert!(*b),
                    _ => panic!("Second arg should be Boolean"),
                }
                match &a[2] {
                    Expression::Var(v) => assert_eq!(v, "x"),
                    _ => panic!("Third arg should be Var"),
                }
            }
            _ => panic!("Expected FunctionCall variant"),
        }
    }

    /// Test Tuple pattern matching
    #[test]
    fn test_pattern_match_tuple() {
        let elements = vec![
            Expression::Int(1),
            Expression::Boolean(true),
            Expression::Var("x".to_string()),
        ];

        let expr = Expression::Tuple(elements.clone());

        match expr {
            Expression::Tuple(t) => {
                assert_eq!(t.len(), 3, "Tuple should have 3 elements");

                // Test nested pattern matching
                match &t[0] {
                    Expression::Int(i) => assert_eq!(*i, 1),
                    _ => panic!("First element should be Int"),
                }
                match &t[1] {
                    Expression::Boolean(b) => assert!(*b),
                    _ => panic!("Second element should be Boolean"),
                }
                match &t[2] {
                    Expression::Var(v) => assert_eq!(v, "x"),
                    _ => panic!("Third element should be Var"),
                }
            }
            _ => panic!("Expected Tuple variant"),
        }
    }

    // ============================================================================
    // 6. DEEPLY NESTED EXPRESSION TESTS
    // ============================================================================

    /// Test deeply nested expression pattern matching
    /// Simulates: ((x > 5) && (y < 10)) || true
    #[test]
    fn test_pattern_match_deeply_nested() {
        let left_inner_left = Expression::BinaryOp {
            left: Box::new(Expression::Var("x".to_string())),
            op: BinaryOp::Gt,
            right: Box::new(Expression::Int(5)),
        };

        let left_inner_right = Expression::BinaryOp {
            left: Box::new(Expression::Var("y".to_string())),
            op: BinaryOp::Lt,
            right: Box::new(Expression::Int(10)),
        };

        let left = Expression::Logical {
            left: Box::new(left_inner_left),
            op: LogicalOp::And,
            right: Box::new(left_inner_right),
        };

        let expr = Expression::Logical {
            left: Box::new(left),
            op: LogicalOp::Or,
            right: Box::new(Expression::Boolean(true)),
        };

        // Match and extract all levels
        match expr {
            Expression::Logical {
                left: l,
                op: o1,
                right: r,
            } => {
                assert_eq!(o1, LogicalOp::Or);

                match *l {
                    Expression::Logical {
                        left: l2,
                        op: o2,
                        right: r2,
                    } => {
                        assert_eq!(o2, LogicalOp::And);

                        // Left branch of AND
                        match *l2 {
                            Expression::BinaryOp {
                                left: x_var,
                                op,
                                right: five,
                            } => {
                                assert_eq!(op, BinaryOp::Gt);
                                match *x_var {
                                    Expression::Var(v) => assert_eq!(v, "x"),
                                    _ => panic!("Expected Var"),
                                }
                                match *five {
                                    Expression::Int(i) => assert_eq!(i, 5),
                                    _ => panic!("Expected Int"),
                                }
                            }
                            _ => panic!("Expected BinaryOp in left"),
                        }

                        // Right branch of AND
                        match *r2 {
                            Expression::BinaryOp {
                                left: y_var,
                                op,
                                right: ten,
                            } => {
                                assert_eq!(op, BinaryOp::Lt);
                                match *y_var {
                                    Expression::Var(v) => assert_eq!(v, "y"),
                                    _ => panic!("Expected Var"),
                                }
                                match *ten {
                                    Expression::Int(i) => assert_eq!(i, 10),
                                    _ => panic!("Expected Int"),
                                }
                            }
                            _ => panic!("Expected BinaryOp in right"),
                        }
                    }
                    _ => panic!("Expected Logical in left"),
                }

                // Right branch of outer OR
                match *r {
                    Expression::Boolean(b) => assert!(b),
                    _ => panic!("Expected Boolean in right"),
                }
            }
            _ => panic!("Expected Logical variant"),
        }
    }

    // ============================================================================
    // 7. EXHAUSTIVE PATTERN MATCHING COVERAGE
    // ============================================================================

    /// Helper function to test exhaustive pattern matching
    /// Returns the variant name as a &'static str for verification
    fn get_variant_name(expr: &Expression) -> &'static str {
        match expr {
            Expression::Boolean(_) => "Boolean",
            Expression::Var(_) => "Var",
            Expression::LayerVar { .. } => "LayerVar",
            Expression::PhaseQualifiedVar { .. } => "PhaseQualifiedVar",
            Expression::PhaseConstraint { .. } => "PhaseConstraint",
            Expression::CrossPhaseRelation { .. } => "CrossPhaseRelation",
            Expression::Int(_) => "Int",
            Expression::BinaryOp { .. } => "BinaryOp",
            Expression::Arithmetic { .. } => "Arithmetic",
            Expression::Logical { .. } => "Logical",
            Expression::Not(_) => "Not",
            Expression::FunctionCall { .. } => "FunctionCall",
            Expression::Tuple(_) => "Tuple",
        }
    }

    /// Test exhaustive match covers all variants
    #[test]
    fn test_exhaustive_pattern_coverage() {
        let variants = vec![
            Expression::Boolean(true),
            Expression::Var("test".to_string()),
            Expression::LayerVar {
                layer: "bundler".to_string(),
                var: "nonce".to_string(),
            },
            Expression::PhaseQualifiedVar {
                phase: "validation".to_string(),
                layer: "account".to_string(),
                var: "balance".to_string(),
            },
            Expression::PhaseConstraint {
                phase: "execution".to_string(),
                constraint: Box::new(Expression::Int(1)),
            },
            Expression::CrossPhaseRelation {
                phase1: "validation".to_string(),
                expr1: Box::new(Expression::Int(1)),
                phase2: "execution".to_string(),
                expr2: Box::new(Expression::Int(2)),
                op: BinaryOp::Lt,
            },
            Expression::Int(42),
            Expression::BinaryOp {
                left: Box::new(Expression::Int(1)),
                op: BinaryOp::Eq,
                right: Box::new(Expression::Int(2)),
            },
            Expression::Logical {
                left: Box::new(Expression::Boolean(true)),
                op: LogicalOp::And,
                right: Box::new(Expression::Boolean(false)),
            },
            Expression::Not(Box::new(Expression::Boolean(true))),
            Expression::FunctionCall {
                name: "test_fn".to_string(),
                args: vec![],
            },
            Expression::Tuple(vec![Expression::Int(1)]),
        ];

        let expected_names = vec![
            "Boolean",
            "Var",
            "LayerVar",
            "PhaseQualifiedVar",
            "PhaseConstraint",
            "CrossPhaseRelation",
            "Int",
            "BinaryOp",
            "Logical",
            "Not",
            "FunctionCall",
            "Tuple",
        ];

        for (expr, expected_name) in variants.iter().zip(expected_names.iter()) {
            let actual_name = get_variant_name(expr);
            assert_eq!(
                actual_name, *expected_name,
                "Variant name should match: expected {}, got {}",
                expected_name, actual_name
            );
        }
    }

    // ============================================================================
    // 8. BOOLEAN AND INT LITERAL COMBINATIONS
    // ============================================================================

    /// Test Boolean and Int literals in various contexts
    #[test]
    fn test_literals_in_expressions() {
        // Boolean in BinaryOp should be allowed structurally
        let expr1 = Expression::BinaryOp {
            left: Box::new(Expression::Boolean(true)),
            op: BinaryOp::Eq,
            right: Box::new(Expression::Boolean(false)),
        };

        match expr1 {
            Expression::BinaryOp {
                left: l,
                op,
                right: r,
            } => {
                assert_eq!(op, BinaryOp::Eq);
                match *l {
                    Expression::Boolean(b) => assert!(b),
                    _ => panic!("Expected Boolean"),
                }
                match *r {
                    Expression::Boolean(b) => assert!(!b),
                    _ => panic!("Expected Boolean"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }

        // Int in Var context (Tuple can contain Int)
        let expr2 = Expression::Tuple(vec![
            Expression::Int(100),
            Expression::Int(-50),
            Expression::Int(0),
        ]);

        match expr2 {
            Expression::Tuple(items) => {
                assert_eq!(items.len(), 3);
                let values: Vec<i128> = items
                    .iter()
                    .map(|e| match e {
                        Expression::Int(i) => *i,
                        _ => panic!("Expected Int"),
                    })
                    .collect();

                assert_eq!(values, vec![100, -50, 0]);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    /// Test FunctionCall with various argument types including Int and Boolean
    #[test]
    fn test_function_call_with_various_args() {
        let args = vec![
            Expression::Int(100),
            Expression::Boolean(true),
            Expression::Int(-50),
            Expression::Boolean(false),
        ];

        let expr = Expression::FunctionCall {
            name: "multi_arg_fn".to_string(),
            args,
        };

        match expr {
            Expression::FunctionCall { name: n, args: a } => {
                assert_eq!(n, "multi_arg_fn");
                assert_eq!(a.len(), 4);

                // Verify we can extract Int and Boolean from the same function
                let mut int_count = 0;
                let mut bool_count = 0;

                for arg in &a {
                    match arg {
                        Expression::Int(i) => {
                            int_count += 1;
                            assert!([-50, 100].contains(i));
                        }
                        Expression::Boolean(b) => {
                            bool_count += 1;
                            // Both true and false should be valid
                            let _ = b;
                        }
                        _ => panic!("Unexpected variant in args"),
                    }
                }

                assert_eq!(int_count, 2, "Should have 2 Int arguments");
                assert_eq!(bool_count, 2, "Should have 2 Boolean arguments");
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    // ============================================================================
    // 9. ITERATOR AND COLLECTION TESTS
    // ============================================================================

    /// Test pattern matching in iterators over multiple expressions
    #[test]
    fn test_pattern_match_in_loops() {
        let expressions = vec![
            Expression::Int(1),
            Expression::Boolean(true),
            Expression::Int(2),
            Expression::Boolean(false),
            Expression::Int(3),
        ];

        let mut int_sum = 0i128;
        let mut bool_count = 0;

        for expr in expressions {
            match expr {
                Expression::Int(i) => int_sum += i,
                Expression::Boolean(_) => bool_count += 1,
                _ => panic!("Unexpected variant"),
            }
        }

        assert_eq!(int_sum, 6, "Sum of integers should be 6");
        assert_eq!(bool_count, 2, "Should have 2 booleans");
    }

    /// Test filter-map pattern with expression matching
    #[test]
    fn test_pattern_match_filter_map() {
        #[allow(clippy::useless_vec)]
        let expressions = vec![
            Expression::Int(10),
            Expression::Boolean(true),
            Expression::Var("x".to_string()),
            Expression::Int(20),
            Expression::Not(Box::new(Expression::Boolean(false))),
            Expression::Int(30),
        ];

        let ints: Vec<i128> = expressions
            .iter()
            .filter_map(|expr| match expr {
                Expression::Int(i) => Some(*i),
                _ => None,
            })
            .collect();

        assert_eq!(ints, [10, 20, 30], "Should extract all Int values");
    }

    // ============================================================================
    // 10. CLONING AND EQUALITY TESTS
    // ============================================================================

    /// Test pattern matching after cloning
    #[test]
    fn test_pattern_match_after_clone() {
        let original = Expression::BinaryOp {
            left: Box::new(Expression::Int(5)),
            op: BinaryOp::Gt,
            right: Box::new(Expression::Int(3)),
        };

        let cloned = original.clone();

        // Pattern match on original
        match original {
            Expression::BinaryOp {
                left: l1,
                op: o1,
                right: _r1,
            } => {
                // Pattern match on clone
                match cloned {
                    Expression::BinaryOp {
                        left: l2,
                        op: o2,
                        right: _r2,
                    } => {
                        assert_eq!(o1, o2);
                        // Both should have same values
                        match (*l1, *l2) {
                            (Expression::Int(i1), Expression::Int(i2)) => {
                                assert_eq!(i1, i2, "Int values should match");
                            }
                            _ => panic!("Expected Int values"),
                        }
                    }
                    _ => panic!("Clone should have same variant"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
}

// ============================================================================
// MODULE TESTS - Verify integration
// ============================================================================

#[cfg(test)]
mod module_integration_tests {
    use truent_core::model::Expression;

    /// Verify all Expression variants are accessible and constructible
    #[test]
    fn test_all_variants_constructible() {
        let _bool_expr = Expression::Boolean(true);
        let _var_expr = Expression::Var("x".to_string());
        let _layer_var = Expression::LayerVar {
            layer: "test".to_string(),
            var: "field".to_string(),
        };
        let _phase_var = Expression::PhaseQualifiedVar {
            phase: "test".to_string(),
            layer: "test".to_string(),
            var: "field".to_string(),
        };
        let _phase_constraint = Expression::PhaseConstraint {
            phase: "test".to_string(),
            constraint: Box::new(Expression::Int(1)),
        };
        let _cross_phase = Expression::CrossPhaseRelation {
            phase1: "test".to_string(),
            expr1: Box::new(Expression::Int(1)),
            phase2: "test".to_string(),
            expr2: Box::new(Expression::Int(2)),
            op: truent_core::model::BinaryOp::Lt,
        };
        let _int_expr = Expression::Int(42);
        let _bin_op = Expression::BinaryOp {
            left: Box::new(Expression::Int(1)),
            op: truent_core::model::BinaryOp::Eq,
            right: Box::new(Expression::Int(2)),
        };
        let _logical = Expression::Logical {
            left: Box::new(Expression::Boolean(true)),
            op: truent_core::model::LogicalOp::And,
            right: Box::new(Expression::Boolean(false)),
        };
        let _not = Expression::Not(Box::new(Expression::Boolean(true)));
        let _fn_call = Expression::FunctionCall {
            name: "test".to_string(),
            args: vec![],
        };
        let _tuple = Expression::Tuple(vec![Expression::Int(1)]);
    }
}
