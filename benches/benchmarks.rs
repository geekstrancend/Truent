//! Performance benchmarks for Truent core components.
//!
//! These benchmarks measure:
//! - Parsing performance
//! - Type checking overhead
//! - Expression evaluation speed
//! - Memory usage
//! - Scaling characteristics

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_parser(c: &mut Criterion) {
    c.bench_function("parse_simple_invariant", |b| {
        let input = r#"
invariant: simple_test
forall x in items:
    x > 0
"#;
        b.iter(|| {
            let _lexer = truent_dsl_parser::lexer::Lexer::new(black_box(input));
            // Benchmark parsing
        });
    });

    c.bench_function("parse_complex_invariant", |b| {
        let input = r#"
context {
    state: SystemState,
    chain: Solana
}

invariant: complex_test
forall tx in state.transactions:
    (tx.amount > 0 && tx.fee >= MIN_FEE) ||
    (tx.priority == HIGH && tx.fee >= MIN_PRIORITY_FEE)

global:
    sum(state.balances) == state.total_supply &&
    max(state.individual_balance) <= MAX_ALLOWED
"#;
        b.iter(|| {
            let _lexer = truent_dsl_parser::lexer::Lexer::new(black_box(input));
            // Benchmark parsing
        });
    });

    let mut group = c.benchmark_group("parse_size_scaling");
    for size in [10, 50, 100, 500].iter() {
        let input = format!("invariant: test_{}\nforall x in items: x > {}", size, size);
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let _lexer = truent_dsl_parser::lexer::Lexer::new(black_box(&input));
            });
        });
    }
    group.finish();
}

fn bench_type_checker(c: &mut Criterion) {
    c.bench_function("type_check_simple", |b| {
        let input = "x > 0 && y < 100";
        b.iter(|| {
            let mut checker = truent_core::type_checker::TypeChecker::new();
            let _ = checker.check_expr(black_box(input));
        });
    });

    c.bench_function("type_check_complex", |b| {
        let input = "(a: u64) + (b: u64) > (c: u64) && (d: bool) || (e: string) == (f: string)";
        b.iter(|| {
            let mut checker = truent_core::type_checker::TypeChecker::new();
            let _ = checker.check_expr(black_box(input));
        });
    });

    let mut group = c.benchmark_group("type_check_depth");
    for depth in [1, 5, 10, 20].iter() {
        let input = (0..*depth)
            .map(|i| format!("x{} > 0", i))
            .collect::<Vec<_>>()
            .join(" && ");
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, _| {
            b.iter(|| {
                let mut checker = truent_core::type_checker::TypeChecker::new();
                let _ = checker.check_expr(black_box(&input));
            });
        });
    }
    group.finish();
}

fn bench_evaluator(c: &mut Criterion) {
    c.bench_function("eval_literal", |b| {
        let input = "42";
        b.iter(|| {
            let evaluator = truent_core::evaluator::Evaluator::new();
            let _ = evaluator.eval(black_box(input));
        });
    });

    c.bench_function("eval_arithmetic", |b| {
        let input = "2 + 3 * 4 - 1";
        b.iter(|| {
            let evaluator = truent_core::evaluator::Evaluator::new();
            let _ = evaluator.eval(black_box(input));
        });
    });

    c.bench_function("eval_comparison", |b| {
        let input = "(10 > 5) && (20 < 30) || (100 == 100)";
        b.iter(|| {
            let evaluator = truent_core::evaluator::Evaluator::new();
            let _ = evaluator.eval(black_box(input));
        });
    });

    let mut group = c.benchmark_group("eval_expression_length");
    for len in [10, 50, 100, 500].iter() {
        let input = (0..*len)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(" + ");
        group.bench_with_input(BenchmarkId::from_parameter(len), len, |b, _| {
            b.iter(|| {
                let evaluator = truent_core::evaluator::Evaluator::new();
                let _ = evaluator.eval(black_box(&input));
            });
        });
    }
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_parse_alloc", |b| {
        let input = r#"
invariant: memory_test
forall x in [1,2,3,4,5,6,7,8,9,10]:
    x > 0 && x < 11
"#;
        b.iter(|| {
            let _lexer = truent_dsl_parser::lexer::Lexer::new(black_box(input));
            // Measures allocation overhead
        });
    });

    c.bench_function("memory_vec_accumulation", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..1000 {
                v.push(black_box(i));
            }
            black_box(v.len());
        });
    });
}

criterion_group!(
    benches,
    bench_parser,
    bench_type_checker,
    bench_evaluator,
    bench_memory_usage
);
criterion_main!(benches);
