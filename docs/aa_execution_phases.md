# Account Abstraction Execution Phase Invariants

## Overview

The Truent framework now supports **execution phase-qualified invariants** for Account Abstraction (AA) systems. This extension addresses a critical gap in AA security modeling: most subtle failures in AA systems come from **phase misalignment** rather than contract-level state violations.

## The Problem: Actor Boundaries vs. Execution Phases

**Before:** The model only tracked actor boundaries (bundler, account, paymaster, entrypoint). An invariant like `bundler::nonce <= account::nonce` checks relationships *between* layers but cannot express *when* those relationships should hold.

**Example of a phase-misalignment bug:**
```
// This was impossible to express before:
// "Paymaster's deposit balance can ONLY be checked AFTER validation phase"
// Checking it during validation might return incorrect results
```

**After:** The model now supports three execution phases:

1. **Validation Phase** (`validation::`)
   - `validateUserOp()` executes
   - Signature checks occur
   - Paymaster verification runs
   - Account balance is NOT yet finalized
   - Gas is NOT yet deducted

2. **Execution Phase** (`execution::`)
   - Account code executes
   - State mutations are allowed
   - Call data is executed
   - Nonce is incremented
   - Gas usage is finalized

3. **Settlement Phase** (`settlement::`)
   - Operation bundled with others
   - Balances transferred
   - Paymaster reimbursement processed
   - Final state is canonical

## Expression Types for Phase-Based Invariants

### 1. Phase-Qualified Variables

Access state at a specific execution phase:

```rust
validation::account::balance    // Account balance at validation
execution::account::balance     // Account balance at execution
settlement::account::balance    // Account balance at settlement
```

### 2. Phase Constraints

Ensure an expression holds during a specific phase:

```rust
invariant ValidateFunds {
    expression: (account::balance >= gas_cost) @ validation
    // Reads as: "During validation, ensure balance >= gas_cost"
}
```

### 3. Cross-Phase Relations

Relate variable values across two or more phases:

```rust
invariant BalanceDecreases {
    expression: 
        validation::account::balance >= execution::account::balance &&
        execution::account::balance >= settlement::account::balance
    // "Balance can only decrease across phases"
}
```

## Implementation Details

### AAContext with Phase Tracking

The `AAContext` structure now includes:

```rust
pub struct AAContext {
    /// Current execution phase
    pub current_phase: Option<ExecutionPhase>,
    
    /// Layer-specific state at current moment
    pub layer_state: BTreeMap<String, BTreeMap<String, Value>>,
    
    /// Phase snapshots for cross-phase analysis
    pub phase_snapshots: BTreeMap<String, BTreeMap<String, Value>>,
    
    // ... other fields
}

impl AAContext {
    pub fn set_phase(&mut self, phase: ExecutionPhase) { ... }
    pub fn snapshot_phase(&mut self, phase: ExecutionPhase) { ... }
    pub fn get_layer_var_at_phase(&self, phase: ExecutionPhase, layer: &str, var: &str) 
        -> Option<&Value> { ... }
}
```

### Invariant Metadata

The `Invariant` struct now includes phase information:

```rust
pub struct Invariant {
    pub name: String,
    pub expression: Expression,
    pub layers: Vec<String>,        // Which layers (bundler, account, paymaster, etc.)
    pub phases: Vec<String>,        // Which phases (validation, execution, settlement)
    // ... other fields
}
```

If `phases: []` is empty, the invariant applies to all phases.

### Expression Extensions

New expression variants to the DSL:

```rust
pub enum Expression {
    // Existing variants...
    LayerVar { layer: String, var: String },
    
    // NEW: Phase-qualified variable
    PhaseQualifiedVar { phase: String, layer: String, var: String },
    
    // NEW: Constraint within a phase
    PhaseConstraint { phase: String, constraint: Box<Expression> },
    
    // NEW: Relation across phases
    CrossPhaseRelation {
        phase1: String,
        expr1: Box<Expression>,
        phase2: String,
        expr2: Box<Expression>,
        op: BinaryOp,
    },
    
    // ... other variants
}
```

## Usage Examples

### Example 1: Validation-Only Balance Check

```invar
invariant PaymasterSponsorship {
    description: "Paymaster can only sponsor in validation phase"
    severity: "critical"
    category: "account-abstraction"
    phases: ["validation"]
    expression: validation::paymaster::deposit >= validation::paymaster::required_amount
}
```

### Example 2: Nonce Monotonicity Across Phases

```invar
invariant NonceIncrement {
    description: "Nonce strictly increases during execution, stable after"
    severity: "high"
    category: "account-abstraction"
    phases: ["validation", "execution", "settlement"]
    expression: (
        validation::account::nonce < execution::account::nonce &&
        execution::account::nonce == settlement::account::nonce
    )
}
```

### Example 3: State Mutation Confinement

```invar
invariant StateChangesDuringExecution {
    description: "State only changes during execution phase"
    severity: "critical"
    category: "account-abstraction"
    phases: ["validation", "execution", "settlement"]
    expression: (
        validation::account::state_hash == execution::account::state_hash_before &&
        execution::account::state_hash != execution::account::state_hash_after &&
        execution::account::state_hash_after == settlement::account::state_hash
    )
}
```

### Example 4: Gas Accounting

```invar
invariant GasUsageFinalized {
    description: "Gas usage finalized during execution, immutable after"
    severity: "high"
    category: "account-abstraction"
    expression: (
        validation::user_op::pre_op_gas == execution::user_op::pre_op_gas &&
        execution::user_op::call_gas_limit <= settlement::user_op::call_gas_limit
    )
}
```

## Evaluation Strategy

When evaluating phase-qualified invariants:

1. **Initialize phase tracking** in `AAContext`
2. **Snapshot state** at each phase transition using `snapshot_phase()`
3. **Evaluate constraints** where phase-qualified variables look up from snapshots
4. **Report violations** with phase context for debugging

Example evaluation flow:

```rust
let mut ctx = AAContext::default();

// Validation phase
ctx.set_phase(ExecutionPhase::Validation);
ctx.set_layer_var("account".to_string(), "balance".to_string(), json!(1000));
ctx.snapshot_phase(ExecutionPhase::Validation);

// Execution phase
ctx.set_phase(ExecutionPhase::Execution);
ctx.set_layer_var("account".to_string(), "balance".to_string(), json!(900));
ctx.snapshot_phase(ExecutionPhase::Execution);

// Settlement phase
ctx.set_phase(ExecutionPhase::Settlement);
ctx.set_layer_var("account".to_string(), "balance".to_string(), json!(850));
ctx.snapshot_phase(ExecutionPhase::Settlement);

// Now check cross-phase invariants
// validation::account::balance (1000) >= execution::account::balance (900) ✓
// execution::account::balance (900) >= settlement::account::balance (850) ✓
```

## Real-World Vulnerability Examples

### Vulnerability 1: Premature Settlement Assumption

**Bug:** Checking gas cost against balance during validation without re-checking during settlement.

```invar
invariant GasPaidCorrectly {
    description: "Gas cost validated during validation, balance confirmed during settlement"
    severity: "critical"
    phases: ["validation", "settlement"]
    expression: (
        validation::account::balance >= validation::gas_cost &&
        settlement::paymaster::deposit >= settlement::gas_reimbursement
    )
}
```

### Vulnerability 2: Reentrancy Across Phases

**Bug:** Reentrancy guard checked in validation but account enters execution without re-validating.

```invar
invariant ReentrancyConfinement {
    description: "Reentrancy state locked during validation and execution"
    severity: "critical"
    phases: ["validation", "execution"]
    expression: (
        validation::account::reentrancy_locked == true &&
        execution::account::reentrancy_locked == true
    )
}
```

### Vulnerability 3: Nonce Exhaustion Attack

**Bug:** Nonce not properly monotonic across phases, allowing replay.

```invar
invariant NonceExhaustionPrevention {
    description: "Nonce strictly monotonically increases"
    severity: "critical"
    phases: ["validation", "execution", "settlement"]
    expression: (
        validation::account::nonce < execution::account::nonce &&
        execution::account::nonce <= settlement::account::nonce
    )
}
```

## Integration with Analyzers

Phase-qualified invariants integrate with all Truent analyzers:

- **DSL Parser** (`dsl_parser`): Parses phase syntax `phase::layer::var`
- **Type Checker** (`type_checker`): Validates phase expression types
- **Evaluator** (`evaluator`): Evaluates against `AAContext` with snapshots
- **Threat Model** (`threat_model`): Validates phase names and forbidden patterns
- **Generators** (EVM, Move, Solana): Generate phase-aware checks in target language

## Testing Phase Invariants

The framework includes a test for phase tracking:

```rust
#[test]
fn test_phase_tracking() {
    let mut ctx = AAContext::default();
    ctx.set_phase(ExecutionPhase::Validation);
    // ... assertions ...
    ctx.snapshot_phase(ExecutionPhase::Validation);
    // ... continue testing ...
}
```

Run tests:
```bash
cargo test -p truent-core test_phase_tracking -- --nocapture
```

## Migration from Actor-Only to Phase-Based Invariants

If you have existing actor-boundary invariants:

**Before (actor boundaries only):**
```invar
bundler::balance >= account::required_balance
```

**After (with phase context):**
```invar
validation::bundler::balance >= execution::account::required_balance
```

This makes the temporal relationship explicit: "Bundler's balance (at validation) must cover account's requirements (by execution)."

## Future Enhancements

Potential extensions to the phase model:

1. **Sub-phases**: Finer granularity (e.g., `validation::pre` vs `validation::post`)
2. **Phase duration constraints**: Time-based relationships
3. **Probabilistic phases**: For rollup sequencing invariants
4. **Dynamic phase detection**: Automatically infer phases from call traces
5. **Phase-dependent complexity**: Different bounds per phase

---

For more examples, see [aa_phase_invariants.invar](../examples/aa_phase_invariants.invar).
