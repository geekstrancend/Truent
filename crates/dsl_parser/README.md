# truent-dsl-parser

Parser for Truent's invariant DSL, built on `pest`.

Compiles a single invariant definition string into a `truent_ir::Invariant`
(name, expression, severity, category) — this is what backs
`truent-library`'s built-in invariants and any user-defined invariants.

## Usage

```toml
[dependencies]
truent-dsl-parser = "0.3.0"
truent-ir = "0.3.0"
```

## Parsing an Invariant

```rust
use truent_dsl_parser::parse_invariant;

let invariant = parse_invariant("invariant BalancePositive { balance >= 0 }")?;
println!("{}: {}", invariant.name, invariant.expression);
```

## DSL Syntax

```
invariant BalancePositive { balance >= 0 }

invariant NoReentrancy {
  call_order_respected AND no_recursive_calls
}
```

An invariant is a name followed by a `{ ... }` block containing a boolean
expression built from comparisons (`>=`, `<=`, `==`, ...), logical operators
(`AND`, `OR`, `NOT`), and identifiers referring to state/predicates the
target chain analyzer understands.

## License

MIT
