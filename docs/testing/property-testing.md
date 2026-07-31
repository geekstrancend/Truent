# Property-Based Testing Guide

## Overview

Property-based testing uses **generative techniques** to validate invariants over large input spaces. Rather than writing specific test cases, we define properties that must hold for **all** valid inputs.

**Key Insight:** Properties are statements about program behavior that should be true universally.

## Why Property-Based Testing?

### Traditional Testing
```rust
#[test]
fn test_parser() {
    assert!(parse("invariant: test\ntrue").is_ok());
    assert!(parse("invariant: test2\ntrue").is_ok());
    // What about 1000 other valid inputs?
}
```

### Property-Based Testing
```rust
proptest! {
    #[test]
    fn prop_parser_never_panics(input in arb_dsl_snippet()) {
        // Guaranteed to test thousands of inputs
        // Will find edge cases we didn't think of
        let result = parse(&input);
        // Must return Ok or Err, never panic
    }
}
```

## Test Structure

```
tests/property/
├── mod.rs                          # Main property test module
├── parser_properties.rs            # DSL parser properties
├── evaluator_properties.rs         # Evaluator properties
├── type_checker_properties.rs      # Type system properties
├── invariant_properties.rs         # Engine properties
└── collection_properties.rs        # Collection operation properties
```

## Core Properties to Test

### 1. **No Panic Property**
> The program should never panic on any input

```rust
proptest! {
    #[test]
    fn prop_parser_never_panics(input in r"[a-zA-Z0-9]{0,1024}") {
        // Parser must return Ok or Err, not panic
        let _ = parse(&input);
        // If test completes, property held
    }
}
```

### 2. **Determinism Property**
> Same input always produces same output

```rust
proptest! {
    #[test]
    fn prop_evaluation_deterministic(
        input in r"[0-9+\-*()]{1,200}"
    ) {
        let result1 = evaluator.eval(&input);
        let result2 = evaluator.eval(&input);
        
        prop_assert_eq!(
            format!("{:?}", result1),
            format!("{:?}", result2),
            "Evaluation must be deterministic"
        );
    }
}
```

### 3. **Idempotency Property**
> Applying operation twice = applying once

```rust
proptest! {
    #[test]
    fn prop_evaluation_idempotent(expr in arb_expression()) {
        let eval_once = evaluator.eval(&expr);
        let eval_twice = evaluator
            .eval(&expr)
            .and_then(|first| {
                evaluator.eval(&format!("{}", first))
            });
        
        // Might not be exactly equal, but must be stable
    }
}
```

### 4. **Commutativity Property**
> Order shouldn't matter for commutative operations

```rust
proptest! {
    #[test]
    fn prop_addition_commutative(
        a in 1i64..1000,
        b in 1i64..1000,
    ) {
        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);
        
        let result1 = evaluator.eval(&expr1);
        let result2 = evaluator.eval(&expr2);
        
        prop_assert_eq!(result1, result2);
    }
}
```

### 5. **Invariant Property**
> Certain properties must always be true

```rust
proptest! {
    #[test]
    fn prop_balance_conservation(
        deposits in prop::collection::vec(0u64..1_000_000, 1..100)
    ) {
        let total_before = deposits.iter().sum::<u64>();
        
        // Simulate withdrawal
        let withdrawn = if !deposits.is_empty() {
            deposits[0] / 2
        } else {
            0
        };
        
        let total_after = deposits.iter().sum::<u64>() - withdrawn;
        
        // Conservation property
        prop_assert!(total_after == total_before - withdrawn);
    }
}
```

## Running Property Tests

```bash
# Run all property tests
cargo test --test property

# Run specific property
cargo test prop_parser_never_panics

# Configure shrinking (smaller failing examples)
PROPTEST_VERBOSE=1 cargo test

# Save failing examples for regression
cargo test -- --nocapture
```

## Property Strategies (Generators)

### Pre-defined Strategies

```rust
use proptest::prelude::*;

// Numeric ranges
any::<u32>()             // Any u32
0usize..100              // Range from 0 to 100
1i64..i64::MAX/2         // Safe range to prevent overflow

// Collections
prop::collection::vec(any::<u32>(), 0..100)
prop::collection::hash_set(any::<u32>(), 0..100)

// Strings
"[a-z]+"                 // Regex pattern
"[a-zA-Z0-9_]{1,50}"     // Alphanumeric with length bounds

// Boolean combinations
prop_oneof![Just(true), Just(false)]
```

### Custom Strategies

```rust
prop_compose! {
    fn arb_valid_invariant()(
        name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        condition in "[a-z]+ > 0"
    ) -> String {
        format!("invariant: {}\n{}", name, condition)
    }
}

proptest! {
    #[test]
    fn prop_parse_custom(inv in arb_valid_invariant()) {
        assert!(parse(&inv).is_ok());
    }
}
```

## Shrinking

When a property fails, proptest automatically finds the **minimal failing example**.

```rust
proptest! {
    #[test]
    fn prop_example(x in 0u32..1000) {
        assert!(x < 500);  // Will fail for some inputs
    }
}

// When fails, finds minimal example: x = 500
// Saved in proptest-regressions/ for replay
```

## Properties We Test

### Parser
- ✅ Never panics on any string input
- ✅ Deterministic output
- ✅ Accepts all valid DSL
- ✅ Rejects all invalid DSL

### Type Checker
- ✅ Never panics
- ✅ Type consistency holds
- ✅ Type errors are explicit
- ✅ Well-typed expressions have matching types

### Evaluator
- ✅ Never panics
- ✅ Deterministic results
- ✅ Arithmetic operators commute where applicable
- ✅ No overflow without detection

### Collections
- ✅ Ordered access is deterministic
- ✅ Set operations are commutative
- ✅ Sum operations are associative

## Regression Testing

When a property fails, proptest saves the failing input:

```
proptest-regressions/
├── my_test.rs          # Generated regression tests
```

These are replayed automatically in future runs:
```bash
cargo test prop_example  # Replays all saved failures
```

## Best Practices

1. **Start with "never panic"** - The foundational property
2. **Test invariants** - Properties you depend on
3. **Use reasonable bounds** - Generator output shouldn't take forever
4. **Document properties** - Explain *why* the property should hold
5. **Check against known properties** - Commutativity, associativity, distributivity
6. **Combine with unit tests** - Properties find edge cases, units document intent

## When to Use Property-Based Testing

✅ **Good use cases:**
- Parser robustness
- Arithmetic correctness
- Serialization round-trips
- Collection operations
- Type system soundness

❌ **Not ideal for:**
- Complex state machines
- User interaction flows
- Performance characteristics
- Platform-specific behavior

## Debugging Failing Properties

```bash
# Verbose output
PROPTEST_VERBOSE=1 cargo test prop_example

# Single thread for determinism check
cargo test prop_example -- --test-threads=1

# Check saved regressions
cat proptest-regressions/my_test.rs
```

## Integration with CI

Properties run automatically in CI:

```yaml
property-tests:
  name: Property Tests
  runs-on: ubuntu-latest
  - name: Run property tests
    run: cargo test --test property --verbose
```

Failed properties block merges, ensuring quality.
