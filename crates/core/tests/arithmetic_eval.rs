//! Evaluation semantics for arithmetic in invariant expressions.
//!
//! The properties here are safety-relevant: a wrapped product or a
//! width-mismatched comparison would report a violated invariant as holding,
//! which is the one failure mode a security tool must not have.

use truent_core::evaluator::{EvaluationError, Evaluator, Value};
use truent_core::model::{ArithOp, BinaryOp, Expression};

fn ctx(vars: &[(&str, Value)]) -> truent_core::evaluator::ExecutionContext {
    let mut c = truent_core::evaluator::ExecutionContext::new();
    for (k, v) in vars {
        c.state_vars.insert((*k).to_string(), v.clone());
    }
    c
}

fn arith(l: Expression, op: ArithOp, r: Expression) -> Expression {
    Expression::Arithmetic {
        left: Box::new(l),
        op,
        right: Box::new(r),
    }
}

fn var(n: &str) -> Expression {
    Expression::Var(n.to_string())
}

#[test]
fn evaluates_the_four_operators() {
    let e = Evaluator::new(ctx(&[("a", Value::U64(10)), ("b", Value::U64(3))]));
    for (op, want) in [
        (ArithOp::Add, 13u64),
        (ArithOp::Sub, 7),
        (ArithOp::Mul, 30),
        (ArithOp::Div, 3), // truncating, matching EVM
    ] {
        let got = e.evaluate(&arith(var("a"), op, var("b"))).expect("eval");
        assert_eq!(got, Value::U64(want), "for {op:?}");
    }
}

/// Two realistic token balances multiply past u64. Wrapping here would make
/// `reserve_a * reserve_b >= k` spuriously true.
#[test]
fn multiplication_widens_instead_of_wrapping() {
    let big = 10_000_000_000_000_000_000u64; // ~1e19, fits u64
    let e = Evaluator::new(ctx(&[("a", Value::U64(big)), ("b", Value::U64(3))]));
    let got = e
        .evaluate(&arith(var("a"), ArithOp::Mul, var("b")))
        .expect("eval");
    assert_eq!(got, Value::U128(big as u128 * 3));
}

#[test]
fn overflow_is_an_error_not_a_wrap() {
    let e = Evaluator::new(ctx(&[("a", Value::U128(u128::MAX)), ("b", Value::U128(2))]));
    assert!(matches!(
        e.evaluate(&arith(var("a"), ArithOp::Mul, var("b"))),
        Err(EvaluationError::Overflow)
    ));
}

/// Unsigned subtraction going negative is a revert on the EVM, so it must
/// surface rather than wrap to a huge number.
#[test]
fn unsigned_underflow_is_an_error() {
    let e = Evaluator::new(ctx(&[("a", Value::U64(1)), ("b", Value::U64(2))]));
    assert!(matches!(
        e.evaluate(&arith(var("a"), ArithOp::Sub, var("b"))),
        Err(EvaluationError::Underflow)
    ));
}

#[test]
fn division_by_zero_is_an_error() {
    let e = Evaluator::new(ctx(&[("a", Value::U64(1)), ("b", Value::U64(0))]));
    assert!(matches!(
        e.evaluate(&arith(var("a"), ArithOp::Div, var("b"))),
        Err(EvaluationError::DivisionByZero)
    ));
}

/// A widened product compared against a plain u64 bound is the common shape of
/// a conservation invariant. Before, differing variants were a TypeError.
#[test]
fn comparisons_work_across_numeric_widths() {
    let e = Evaluator::new(ctx(&[("wide", Value::U128(5)), ("narrow", Value::U64(6))]));
    let lt = Expression::BinaryOp {
        left: Box::new(var("wide")),
        op: BinaryOp::Lt,
        right: Box::new(var("narrow")),
    };
    assert_eq!(e.evaluate(&lt).expect("eval"), Value::Bool(true));

    let eq = Expression::BinaryOp {
        left: Box::new(var("wide")),
        op: BinaryOp::Eq,
        right: Box::new(Expression::Int(5)),
    };
    assert_eq!(e.evaluate(&eq).expect("eval"), Value::Bool(true));
}

/// End-to-end: the constant-product property, evaluated.
#[test]
fn constant_product_invariant_evaluates() {
    // (r_yes * r_no) >= k
    let expr = Expression::BinaryOp {
        left: Box::new(arith(var("r_yes"), ArithOp::Mul, var("r_no"))),
        op: BinaryOp::Gte,
        right: Box::new(var("k")),
    };

    let holds = Evaluator::new(ctx(&[
        ("r_yes", Value::U64(1_000)),
        ("r_no", Value::U64(1_000)),
        ("k", Value::U64(1_000_000)),
    ]));
    assert_eq!(holds.evaluate(&expr).expect("eval"), Value::Bool(true));

    let violated = Evaluator::new(ctx(&[
        ("r_yes", Value::U64(900)),
        ("r_no", Value::U64(1_000)),
        ("k", Value::U64(1_000_000)),
    ]));
    assert_eq!(violated.evaluate(&expr).expect("eval"), Value::Bool(false));
}
