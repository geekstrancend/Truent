# truent-generator-solana

Code generator for Solana (Anchor/native Rust) invariant enforcement.

## Usage

```toml
[dependencies]
truent-generator-solana = "0.3.0"
truent-core = "0.3.0"
```

## Current State

`SolanaGenerator` implements the `CodeGenerator` trait: given a
`ProgramModel` and a list of `Invariant`s, it emits one
`assert!(<expression>, "Invariant <name> violated");` string per invariant.
This is straightforward string templating, not a validating code
generator - it doesn't check that the expression is well-formed Rust, and
it isn't currently invoked by the CLI.

## Example

```rust
use truent_core::traits::CodeGenerator;
use truent_generator_solana::SolanaGenerator;

let generator = SolanaGenerator;
let output = generator.generate(&program_model, &invariants)?;
println!("{}", output.code);
for assertion in &output.assertions {
    println!("{assertion}");
}
```

## License

MIT
