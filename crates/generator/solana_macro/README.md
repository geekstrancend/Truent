# truent-solana-macro

Procedural attribute macro scaffolding for Solana invariant enforcement.

## Usage

```toml
[dependencies]
truent-solana-macro = "0.3.0"
```

## Current State

`#[invariant_enforced(...)]` parses its `checks` argument list and the
annotated function's signature, and computes a deterministic hash of the
parsed checks (`generate_check_statements`, `compute_check_hash`) - but the
statements it currently injects are comments (`// Invariant: <check>`)
followed by `let _ = ();`, not real assertions. **It does not currently
enforce anything at compile time or runtime** - applying it to a function
changes nothing about that function's behavior. Treat it as parsing/hashing
scaffolding for a real enforcement mechanism that hasn't been built yet,
not as a working invariant-enforcement macro.

## Example

```rust
use truent_solana_macro::invariant_enforced;

#[invariant_enforced("balance >= 0", "supply == sum_of_balances")]
pub fn transfer(from: &mut Account, to: &mut Account, amount: u64) -> ProgramResult {
    from.balance = from.balance.checked_sub(amount)?;
    to.balance = to.balance.checked_add(amount)?;
    Ok(())
    // No invariant check is actually injected here yet.
}
```

## License

MIT
