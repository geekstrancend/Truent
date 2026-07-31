# truent-generator-move

Code generator for Move invariant enforcement.

## Usage

```toml
[dependencies]
truent-generator-move = "0.3.0"
truent-core = "0.3.0"
```

## Current State

`MoveGenerator` implements the `CodeGenerator` trait: given a `ProgramModel`
and a list of `Invariant`s, it emits one `assert!(<expression>,
E_INVARIANT_<NAME>);` string per invariant. This is straightforward string
templating, not a validating code generator - it doesn't check that the
expression is well-formed Move, and it isn't currently invoked by the CLI.

## Example

```rust
use truent_core::traits::CodeGenerator;
use truent_generator_move::MoveGenerator;

let generator = MoveGenerator;
let output = generator.generate(&program_model, &invariants)?;
println!("{}", output.code);
for assertion in &output.assertions {
    println!("{assertion}");
}
```

## License

MIT
