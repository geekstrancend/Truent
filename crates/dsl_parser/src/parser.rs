//! Parser for invariant DSL expressions.

use crate::grammar::{Grammar, Rule};
use pest::Parser;
use truent_core::model::{ArithOp, BinaryOp, Expression, Invariant};
use truent_core::Result;

/// Parser for invariant DSL.
pub struct InvariantParser;

impl InvariantParser {
    /// Parse a single invariant definition.
    pub fn parse_invariant(input: &str) -> Result<Invariant> {
        let parsed = Grammar::parse(Rule::invariant_def, input)
            .map_err(|e| truent_core::InvarError::ConfigError(e.to_string()))?;

        let invariant_rule = parsed.into_iter().next().ok_or_else(|| {
            truent_core::InvarError::ConfigError("No invariant found".to_string())
        })?;

        let inner = invariant_rule.into_inner();
        let inner_items: Vec<_> = inner.collect();

        if inner_items.is_empty() {
            return Err(truent_core::InvarError::ConfigError(
                "Expected invariant name and expression".to_string(),
            ));
        }

        let name = inner_items[0].as_str().to_string();

        // Parse optional layer specifications and expression
        let (layers, expr_idx) = if inner_items.len() > 2 {
            // Check if second item is layer specification
            let second_item = &inner_items[1];
            if second_item.as_rule() == Rule::layer_name
                || (second_item.as_str().starts_with('(') && second_item.as_str().contains(','))
            {
                // This is a layer specification, parse all layer names
                let mut layer_specs = Vec::new();
                for item in inner_items.iter().take(inner_items.len() - 1).skip(1) {
                    let item_str = item
                        .as_str()
                        .trim_matches(|c| c == '(' || c == ')' || c == ',')
                        .trim();
                    if !item_str.is_empty() {
                        layer_specs.push(item_str.to_string());
                    }
                }
                (layer_specs, inner_items.len() - 1)
            } else {
                (vec![], 1)
            }
        } else {
            (vec![], 1)
        };

        let expression = Self::parse_expr(inner_items[expr_idx].clone())?;

        Ok(Invariant {
            name,
            description: None,
            expression,
            severity: "medium".to_string(),
            category: "general".to_string(),
            is_always_true: true,
            layers,
            phases: vec![],
        })
    }

    fn parse_expr(rule: pest::iterators::Pair<Rule>) -> Result<Expression> {
        use pest::iterators::Pair;

        fn parse_pair(pair: Pair<Rule>) -> Result<Expression> {
            match pair.as_rule() {
                Rule::expr
                | Rule::logical_or
                | Rule::logical_and
                | Rule::comparison
                | Rule::sum
                | Rule::term
                | Rule::unary => {
                    let items: Vec<_> = pair.into_inner().collect();
                    if items.is_empty() {
                        return Err(truent_core::InvarError::ConfigError(
                            "Expected expression".to_string(),
                        ));
                    }

                    let mut left = parse_pair(items[0].clone())?;
                    let mut i = 1;

                    while i < items.len() {
                        let operator = &items[i];
                        i += 1;

                        if i >= items.len() {
                            return Err(truent_core::InvarError::ConfigError(
                                "Expected operand after operator".to_string(),
                            ));
                        }

                        let right = parse_pair(items[i].clone())?;
                        i += 1;

                        match operator.as_rule() {
                            Rule::and => {
                                left = Expression::Logical {
                                    left: Box::new(left),
                                    op: truent_core::model::LogicalOp::And,
                                    right: Box::new(right),
                                };
                            }
                            Rule::or => {
                                left = Expression::Logical {
                                    left: Box::new(left),
                                    op: truent_core::model::LogicalOp::Or,
                                    right: Box::new(right),
                                };
                            }
                            Rule::eq => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Eq,
                                    right: Box::new(right),
                                };
                            }
                            Rule::neq => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Neq,
                                    right: Box::new(right),
                                };
                            }
                            Rule::lt => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Lt,
                                    right: Box::new(right),
                                };
                            }
                            Rule::gt => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Gt,
                                    right: Box::new(right),
                                };
                            }
                            Rule::lte => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Lte,
                                    right: Box::new(right),
                                };
                            }
                            Rule::gte => {
                                left = Expression::BinaryOp {
                                    left: Box::new(left),
                                    op: BinaryOp::Gte,
                                    right: Box::new(right),
                                };
                            }
                            Rule::add => {
                                left = Expression::Arithmetic {
                                    left: Box::new(left),
                                    op: ArithOp::Add,
                                    right: Box::new(right),
                                };
                            }
                            Rule::sub => {
                                left = Expression::Arithmetic {
                                    left: Box::new(left),
                                    op: ArithOp::Sub,
                                    right: Box::new(right),
                                };
                            }
                            Rule::mul => {
                                left = Expression::Arithmetic {
                                    left: Box::new(left),
                                    op: ArithOp::Mul,
                                    right: Box::new(right),
                                };
                            }
                            Rule::div => {
                                left = Expression::Arithmetic {
                                    left: Box::new(left),
                                    op: ArithOp::Div,
                                    right: Box::new(right),
                                };
                            }
                            // Previously a silent no-op, which dropped the
                            // operand and produced a wrong expression rather
                            // than reporting the problem.
                            other => {
                                return Err(truent_core::InvarError::ConfigError(format!(
                                    "Unsupported operator in expression: {:?}",
                                    other
                                )));
                            }
                        }
                    }
                    Ok(left)
                }
                Rule::primary => {
                    let mut inner = pair.into_inner();
                    let next = inner.next();
                    if let Some(inner_pair) = next {
                        parse_pair(inner_pair)
                    } else {
                        // Empty primary - should not happen in well-formed grammar
                        Err(truent_core::InvarError::ConfigError(
                            "Unexpected empty primary expression".to_string(),
                        ))
                    }
                }
                Rule::function_call => {
                    let items: Vec<_> = pair.into_inner().collect();
                    if items.is_empty() {
                        return Err(truent_core::InvarError::ConfigError(
                            "Expected function name".to_string(),
                        ));
                    }
                    let name = items[0].as_str().to_string();
                    let args: Result<Vec<_>> = items[1..]
                        .iter()
                        .map(|arg| parse_pair(arg.clone()))
                        .collect();
                    Ok(Expression::FunctionCall { name, args: args? })
                }
                Rule::boolean => {
                    let val = pair.as_str() == "true";
                    Ok(Expression::Boolean(val))
                }
                Rule::integer => {
                    let val = pair.as_str().parse::<i128>().map_err(|_| {
                        truent_core::InvarError::ConfigError("Invalid integer".to_string())
                    })?;
                    Ok(Expression::Int(val))
                }
                Rule::identifier => Ok(Expression::Var(pair.as_str().to_string())),
                Rule::qualified_id => {
                    let items: Vec<_> = pair.into_inner().collect();
                    if items.len() != 2 {
                        return Err(truent_core::InvarError::ConfigError(
                            "Expected layer::identifier".to_string(),
                        ));
                    }
                    let layer = items[0].as_str().to_string();
                    let var = items[1].as_str().to_string();
                    Ok(Expression::LayerVar { layer, var })
                }
                Rule::var_id => {
                    let mut inner = pair.into_inner();
                    if let Some(first) = inner.next() {
                        if first.as_rule() == Rule::qualified_id {
                            let items: Vec<_> = first.into_inner().collect();
                            if items.len() == 2 {
                                let layer = items[0].as_str().to_string();
                                let var = items[1].as_str().to_string();
                                return Ok(Expression::LayerVar { layer, var });
                            }
                        } else if first.as_rule() == Rule::simple_id {
                            return Ok(Expression::Var(first.as_str().to_string()));
                        } else {
                            return parse_pair(first);
                        }
                    }
                    Err(truent_core::InvarError::ConfigError(
                        "Expected identifier or layer::identifier".to_string(),
                    ))
                }
                _ => Err(truent_core::InvarError::ConfigError(format!(
                    "Unexpected rule: {:?}",
                    pair.as_rule()
                ))),
            }
        }

        parse_pair(rule)
    }
}

/// Parse a complete invariant definition string.
pub fn parse_invariant(input: &str) -> Result<Invariant> {
    InvariantParser::parse_invariant(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_invariant() {
        let input = r#"invariant BalancePositive { balance >= 0 }"#;
        let result = parse_invariant(input);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
        let inv = result.unwrap();
        assert_eq!(inv.name, "BalancePositive");
    }

    #[test]
    fn test_parse_invariant_with_and() {
        let input = r#"invariant MultiCondition { balance >= 0 && total_supply > 0 }"#;
        let result = parse_invariant(input);
        if let Err(ref e) = result {
            eprintln!("Parse error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_invariant_no_expression() {
        let input = r#"invariant Empty { }"#;
        let result = parse_invariant(input);
        assert!(result.is_err());
    }
}
