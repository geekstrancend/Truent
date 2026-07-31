# truent-generator-evm

Code generator for EVM (Solidity) invariant enforcement.

## Usage

```toml
[dependencies]
truent-generator-evm = "0.3.0"
truent-core = "0.3.0"
```

## Current State

`EvmGenerator` implements the `CodeGenerator` trait: given a `ProgramModel`
and a list of `Invariant`s, it emits one `require(<expression>, "Invariant:
<name>");` assertion string per invariant. This is straightforward string
templating, not a validating code generator - it doesn't check that the
expression is well-formed Solidity, and it isn't currently invoked by the
CLI (`cargo tree`-reachable from `truent-cli`, but no command wires it up
yet).

## Example

```rust
use truent_core::traits::CodeGenerator;
use truent_generator_evm::EvmGenerator;

let generator = EvmGenerator;
let output = generator.generate(&program_model, &invariants)?;
println!("{}", output.code);
for assertion in &output.assertions {
    println!("{assertion}");
}
```

## License

MIT
