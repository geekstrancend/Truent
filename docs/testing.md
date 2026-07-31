# Testing Guide

## Overview

This guide explains how to run, write, and maintain tests in Truent.

**Quick Start:**

```bash
# Run all tests
cargo test

# Run specific test category
cargo test --test unit
cargo test --test property
cargo test --test cli
cargo test --test integration
cargo test --test security

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Test Categories

### 1. Unit Tests

**Location:** `/tests/unit/mod.rs`

**Purpose:** Test individual components in isolation

**Components Tested:**
- Parser (DSL parsing correctness)
- Type checker (type inference and validation)
- Evaluator (expression evaluation)
- AST construction

**Run:**
```bash
cargo test --test unit
```

**Add Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parser_simple_invariant() {
        let dsl = "invariant: test\nglobal: x > 0";
        let ast = parse(dsl).unwrap();
        assert_eq!(ast.name, "test");
    }
}
```

**Best Practices:**
- Test one thing per test
- Use descriptive names: `test_parser_handles_empty_context`
- Arrange-Act-Assert pattern
- No global state

### 2. Property-Based Tests

**Location:** `/tests/property/mod.rs`

**Purpose:** Verify invariants hold across large input spaces

**Properties Tested:**
- Parser never panics
- Evaluation is deterministic
- Type checking is consistent
- Arithmetic follows laws

**Run:**
```bash
cargo test --test property

# With verbose output
cargo test --test property -- --nocapture
```

**Add Tests:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_parser_deterministic(dsl in ".*") {
        let result1 = parse(&dsl);
        let result2 = parse(&dsl);
        prop_assert_eq!(result1, result2);
    }
}
```

**Shrinking:** proptest automatically finds minimal failing case:

```
Falsified after 42 tries and 18 shrinks, smallest example:
dsl = "\n"
```

### 3. CLI Tests

**Location:** `/tests/cli/mod.rs`

**Purpose:** Validate command-line interface behavior

**Tested:**
- Exit codes (0=success, 1=violation, 2=config error, 3=internal)
- Output formats (JSON, Markdown, text)
- Error messages
- Help and version strings

**Run:**
```bash
cargo test --test cli

# With output capture
cargo test --test cli -- --nocapture
```

**Add Tests:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("truent")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));
}
```

**Common Predicates:**
- `predicate::str::contains("text")`
- `predicate::str::contains_regex("pattern")`
- `predicate::path::exists()`
- `predicate::path::is_file()`

### 4. Integration Tests

**Location:** `/tests/integration/mod.rs`

**Purpose:** End-to-end workflows with real project structures

**Scenarios:**
- Run analysis on multi-chain project
- Validate invariants against real examples
- Test categorization and reporting
- Verify output accuracy

**Run:**
```bash
cargo test --test integration

# With temp directory debugging
RUST_LOG=debug cargo test --test integration -- --nocapture
```

**Add Tests:**
```rust
#[test]
fn test_solana_vault_analysis() {
    let project = create_test_project(TestProject::SolanaVault);
    let output = run_analysis(&project).unwrap();
    
    assert!(output.contains("vault_conservation"));
    assert_eq!(output_format(&output), Format::Json);
}
```

**Test Fixtures:** Use `tempfile::TempDir` for temporary projects:

```rust
let dir = TempDir::new().unwrap();
create_invariant(&dir, "invariant: test\nglobal: x > 0")?;
let output = run_truent(&dir)?;
```

### 5. Security Tests

**Location:** `/tests/security/mod.rs`

**Purpose:** Validate security constraints and hardening

**Tested:**
- DSL injection prevention
- Type confusion prevention
- Buffer overflow detection
- Path traversal prevention
- Integer overflow catching
- Thread safety
- Exit code integrity

**Run:**
```bash
cargo test --test security

# Memory safety validation
cargo +nightly miri test --test security
```

**Add Tests:**
```rust
#[test]
fn test_injection_prevention() {
    let injection = "; system('rm -rf /');";
    let dsl = format!("invariant: test\n{}", injection);
    
    // Must reject at parse time
    assert!(parse(&dsl).is_err());
}

#[test]
fn test_overflow_detection() {
    let large = u64::MAX;
    let result = checked_add(large, 1);
    assert!(result.is_none());
}
```

## Performance Benchmarking

**Location:** `/benches/benchmarks.rs`

**Purpose:** Measure performance on different inputs

**Benchmarks:**
- Parser speed (simple/complex DSL)
- Type checker speed
- Evaluator speed
- Memory usage
- Scaling characteristics

**Run:**
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench parser

# Compare to baseline
cargo bench -- --baseline main

# Generate HTML report
# Opens in target/criterion/report/index.html
```

**Add Benchmarks:**
```rust
fn bench_parser(c: &mut Criterion) {
    c.bench_function("parse_simple", |b| {
        b.iter(|| parse_invariant("invariant: test\nglobal: x > 0"))
    });
}
```

**Scaling Tests:**
```rust
let mut group = c.benchmark_group("parser_scaling");
for size in [10, 50, 100, 500].iter() {
    group.bench_with_input(
        BenchmarkId::from_parameter(size),
        size,
        |b, &size| {
            let dsl = generate_large_dsl(size);
            b.iter(|| parse(&dsl));
        },
    );
}
```

## Coverage

**Location:** `target/coverage/` (generated)

**Purpose:** Measure test code coverage

**Run:**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

**Coverage Goals:**
- Core engine: 90%+
- Utilities: 85%+
- CLI: 80%+

**Improve Coverage:**
1. Run tarpaulin to identify gaps
2. Write tests for uncovered paths
3. Check error handling paths
4. Test edge cases

## Determinism

**Purpose:** Ensure tests produce identical results across runs

**Validation:**
```bash
# Run tests 3 times
cargo test
cargo clean
cargo test
cargo test
```

**CI Check:** Pipeline runs determinism check:
```bash
cargo test --test unit
cargo test --test unit  # Again
cargo test --test unit  # Third time
# All three must produce identical output
```

**Common Non-Determinism Sources:**
- HashMap iteration order (use BTreeMap)
- System time (mock in tests)
- Floating point (use fixed precision)
- File system ordering (sort before comparison)

## Debugging Tests

### Verbose Output

```bash
# Show all println! output
cargo test -- --nocapture

# Show test names
cargo test -- --nocapture --test-threads=1

# Show backtraces
RUST_BACKTRACE=1 cargo test
```

### Debug Single Test

```bash
# Run only one test
cargo test test_parser_simple -- --nocapture

# Debug with debugger
rust-gdb --args target/debug/deps/truent-<hash> test_parser_simple

# Debug in IDE
# Set breakpoint and run via code-debug extension
```

### Introspection

Add print debugging:
```rust
println!("AST: {:#?}", ast);
eprintln!("Error: {:?}", error);
dbg!(&value);  // Prints and returns value
```

## Test Organization

### Directory Structure

```
/tests/
  common.rs           # Shared utilities
  unit/
    mod.rs           # Unit tests
  property/
    mod.rs           # Property tests
  cli/
    mod.rs           # CLI tests
  integration/
    mod.rs           # Integration tests
  security/
    mod.rs           # Security tests
  fixtures/
    *.invar          # Test DSL files
    *.sol            # Test smart contracts
    *.rs             # Test Rust files
```

### Running Subsets

```bash
# All parser tests
cargo test parser

# All integration tests
cargo test --test integration

# Specific module
cargo test unit::parser

# Matching regex
cargo test test_parser_
```

## Continuous Integration

### Local Pre-Commit

```bash
#!/bin/bash
# .git/hooks/pre-commit

cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Install:
```bash
chmod +x .git/hooks/pre-commit
```

### CI Pipeline

Tests in GitHub Actions:
- **Every commit**: Format check, lint, unit tests
- **Every PR**: All tests, coverage report
- **Before release**: Full matrix (OS × Rust version)

See [CI Pipeline Documentation](./ci/ci-pipeline.md) for details.

## Best Practices

### Test Structure

```rust
#[test]
fn test_descriptive_name() {
    // Arrange: Set up test data
    let input = create_test_invariant();
    
    // Act: Perform operation
    let result = parse(&input).unwrap();
    
    // Assert: Verify result
    assert_eq!(result.name, "test");
}
```

### Test Names

Good names describe what is tested and what's expected:
- `test_parser_rejects_invalid_syntax` - Good
- `test_type_checker_infers_int_from_arithmetic` - Good
- `test_evaluator_handles_empty_context` - Good
- `test_works` - Bad
- `test1` - Bad

### Dependencies

Test only public interfaces:
```rust
// BAD - Testing private implementation
#[test]
fn test_internal_buffer_size() {
    assert_eq!(internal_buffer::SIZE, 1024);
}

// GOOD - Testing public behavior
#[test]
fn test_large_input_handling() {
    let large_input = "x".repeat(10000);
    assert!(parse(&large_input).is_ok());
}
```

### Isolation

Each test should be independent:
```rust
// BAD - Test depends on previous test
static mut COUNTER: u32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_value() {
    assert_eq!(unsafe { COUNTER }, 1);  // Depends on ordering!
}

// GOOD - Isolated tests
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

### Assertions

Be specific:
```rust
// BAD - Vague assertion
assert!(result.len() > 0);

// GOOD - Specific assertion
assert_eq!(result.len(), 3, "Expected 3 results but got {}", result.len());
```

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [proptest Guide](https://docs.rs/proptest/latest/proptest/)
- [assert_cmd Documentation](https://docs.rs/assert_cmd/)
- [criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)

## Troubleshooting

### Test Hangs

```bash
# Run with timeout
timeout 30 cargo test test_name

# Run with single thread to see which test
cargo test -- --test-threads=1 -- --nocapture
```

### Flaky Tests

**Signs:** Test passes sometimes, fails sometimes

**Causes:**
- Time-dependent logic
- Randomness (not seeded)
- Async race conditions
- Global state

**Fixes:**
- Use simulated time
- Seed RNG: `let mut rng = StdRng::seed_from_u64(42)`
- Add `#[tokio::test(flavor = "multi_thread")]`
- Use synchronization primitives

### Memory Leaks

```bash
# Run under valgrind
valgrind --leak-check=full cargo test test_name

# Or use miri (for undefined behavior)
cargo +nightly miri test test_name
```

## Summary

- **Unit tests** → Component correctness
- **Property tests** → Invariant validation
- **CLI tests** → Interface specification
- **Integration tests** → End-to-end workflows
- **Security tests** → Hardening verification
- **Benchmarks** → Performance measurement

All together: **Developer trust through measurable quality**
