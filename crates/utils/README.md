# truent-utils

Shared utilities used across the Truent workspace: logging setup, `solc`
invocation, release/versioning helpers.

## Usage

```toml
[dependencies]
truent-utils = "0.3.0"
```

## Key Modules

- `logging::setup_tracing()` — initialize `tracing`-based logging
- `solc::SolcManager` — locates and invokes the `solc` compiler, parses its `--combined-json` output into `SolcOutput`/`SourceData` (used by the EVM analyzer's AST-based path; gracefully unavailable if `solc` isn't installed)
- `release::ReleaseManager` — release artifact/manifest helpers used by the CLI's release tooling
- `version::{Platform, ReleaseArtifact, SemanticVersion, ReproducibleBuildConfig}` — platform target triples and semver parsing shared by the CLI and npm packaging

## Example

```rust
use truent_utils::{setup_tracing, SolcManager};

setup_tracing();

match SolcManager::new() {
    Ok(solc) => {
        let output = solc.get_ast_for_source(source, "Vault.sol")?;
        // ...
    }
    Err(_) => {
        // solc not installed - callers fall back to pattern-based analysis
    }
}
```

## License

MIT
