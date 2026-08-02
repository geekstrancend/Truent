//! Arithmetic in the invariant DSL.
//!
//! Conservation properties — `sum(balances) == totalSupply`,
//! `reserve_a * reserve_b >= k` — are the core of what an invariant language
//! is for, and none of them parsed before arithmetic existed. The examples in
//! `examples/invariants.invar` did not parse either, so they are pinned here.

use truent_core::model::Expression;
use truent_dsl_parser::parse_invariant;

fn expr(src: &str) -> Expression {
    parse_invariant(src)
        .unwrap_or_else(|e| panic!("failed to parse `{src}`: {e:?}"))
        .expression
}

#[test]
fn parses_arithmetic_on_either_side_of_a_comparison() {
    assert_eq!(
        expr("invariant A { collateral >= user_yes + user_no }").to_string(),
        "(collateral >= (user_yes + user_no))"
    );
    assert_eq!(
        expr("invariant A { r_yes + r_no >= total_yes }").to_string(),
        "((r_yes + r_no) >= total_yes)"
    );
}

#[test]
fn all_four_operators_are_supported() {
    for (src, want) in [
        ("invariant A { a + b >= c }", "((a + b) >= c)"),
        ("invariant A { a - b >= c }", "((a - b) >= c)"),
        ("invariant A { a * b >= c }", "((a * b) >= c)"),
        ("invariant A { a / b >= c }", "((a / b) >= c)"),
    ] {
        assert_eq!(expr(src).to_string(), want, "for {src}");
    }
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    // `a * b + c` must be `(a*b)+c`, never `a*(b+c)` — the two differ for
    // every non-trivial conservation check.
    assert_eq!(
        expr("invariant A { a * b + c >= d }").to_string(),
        "(((a * b) + c) >= d)"
    );
    assert_eq!(
        expr("invariant A { a + b * c >= d }").to_string(),
        "((a + (b * c)) >= d)"
    );
}

#[test]
fn parentheses_override_precedence() {
    assert_eq!(
        expr("invariant A { (a + b) * c >= d }").to_string(),
        "(((a + b) * c) >= d)"
    );
}

#[test]
fn arithmetic_composes_with_logical_operators() {
    assert_eq!(
        expr("invariant A { (a * b >= k) && (c > 0) }").to_string(),
        "(((a * b) >= k) && (c > 0))"
    );
}

/// The two examples shipped in `examples/invariants.invar` that demonstrate
/// conservation. Both failed to parse before arithmetic was added.
#[test]
fn shipped_examples_parse() {
    expr("invariant TokenConservation { (balance_alice + balance_bob + balance_charlie) == total_supply }");
    expr("invariant LiquidityPreservation { (reserve_a > 0) && (reserve_b > 0) && ((reserve_a * reserve_b) >= constant_product) }");
}

#[test]
fn every_invariant_in_the_examples_file_parses() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/invariants.invar"
    );
    let src = std::fs::read_to_string(path).expect("examples/invariants.invar");

    let mut count = 0;
    let mut depth = 0usize;
    let mut current = String::new();
    for line in src.lines() {
        let line = line.split("//").next().unwrap_or("");
        if line.trim().is_empty() && depth == 0 {
            continue;
        }
        if current.is_empty() && !line.trim_start().starts_with("invariant") {
            continue;
        }
        current.push_str(line);
        current.push('\n');
        depth += line.matches('{').count();
        depth -= line.matches('}').count().min(depth);
        if depth == 0 && !current.trim().is_empty() {
            parse_invariant(current.trim())
                .unwrap_or_else(|e| panic!("example failed to parse:\n{current}\n{e:?}"));
            count += 1;
            current.clear();
        }
    }
    assert!(
        count >= 8,
        "expected to exercise the examples file, got {count}"
    );
}
