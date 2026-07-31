# CLI Testing Guide

## Overview

CLI tests validate that the command-line interface behaves correctly, produces correct exit codes, outputs valid formats, and handles errors gracefully.

## Exit Code Semantics

Truent uses standard Unix exit codes:

| Code | Meaning | When Used |
|------|---------|-----------|
| **0** | Success | Invariants passed, no violations |
| **1** | Invariant Violation | One or more invariants failed |
| **2** | Configuration Error | Invalid arguments, missing files, bad config |
| **3** | Internal Error | Parser panic, internal inconsistency, unrecoverable state |

## Test Structure

```
tests/cli/
├── mod.rs                    # Main CLI test module
├── exit_codes.rs             # Exit code validation
├── output_formats.rs         # Output format tests
└── configuration.rs          # Config file tests
```

## Test Categories

### 1. **Help and Version Output**

```rust
#[test]
fn test_cli_help_output() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Truent"));
}

#[test]
fn test_cli_version_output() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("--version");
    cmd.assert().success();
}
```

### 2. **Exit Code Validation**

```rust
#[test]
fn test_exit_code_success_is_zero() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("--help");
    let output = cmd.output()?;
    
    assert_eq!(
        output.status.code(),
        Some(0),
        "Success should exit with 0"
    );
}

#[test]
fn test_exit_code_error_is_nonzero() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("nonexistent_command");
    let output = cmd.output()?;
    
    assert_ne!(
        output.status.code(),
        Some(0),
        "Error should exit with non-zero"
    );
}
```

### 3. **Output Format Validation**

#### JSON Output
```rust
#[test]
fn test_json_output_is_valid() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("report")
        .arg("--format").arg("json");
    
    let output = cmd.output()?;
    assert!(output.status.success());
    
    // Validate JSON
    let json: serde_json::Value = 
        serde_json::from_slice(&output.stdout)?;
    assert!(json["invariants"].is_array());
}
```

#### Markdown Output
```rust
#[test]
fn test_markdown_output_format() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("report")
        .arg("--format").arg("markdown");
    
    let output = cmd.output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    // Markdown should contain headers
    assert!(stdout.contains("#") || stdout.is_empty());
}
```

### 4. **Error Handling**

```rust
#[test]
fn test_missing_required_file() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("build")
        .arg("--source").arg("/nonexistent/file.rs");
    
    cmd.assert().failure();
}

#[test]
fn test_invalid_argument_value() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("--log-level").arg("invalid_level");
    
    cmd.assert().failure();
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("nonexistent_command");
    
    cmd.assert().failure();
}
```

### 5. **Determinism**

```rust
#[test]
fn test_same_input_produces_same_output() {
    // Run 1
    let mut cmd1 = Command::cargo_bin("truent")?;
    cmd1.arg("list");
    let output1 = cmd1.output()?;

    // Run 2
    let mut cmd2 = Command::cargo_bin("truent")?;
    cmd2.arg("list");
    let output2 = cmd2.output()?;

    // Must be identical
    assert_eq!(output1.stdout, output2.stdout);
    assert_eq!(output1.stderr, output2.stderr);
    assert_eq!(output1.status.code(), output2.status.code());
}
```

## Running CLI Tests

```bash
# Run all CLI tests
cargo test --test cli

# Run specific CLI test
cargo test test_json_output_is_valid

# Run with output display
cargo test --test cli -- --nocapture
```

## Snapshot Testing with Insta

For output that's long or complex, use snapshot testing:

```rust
use insta::assert_snapshot;

#[test]
fn test_report_output() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("report")
        .arg("--format").arg("json");
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    assert_snapshot!(stdout);  // Compares against stored snapshot
}
```

On first run, creates `tests/cli/__snapshots__/test_report_output.snap`:
```
assert_snapshot!(stdout)
@@ -1,5 +1,5 @@
 {
   "status": "pass",
   "invariants": [
     {
       "name": "vault_conservation",
       "passed": true
     }
   ]
 }
```

## Testing Subcommands

### init command
```rust
#[test]
fn test_init_creates_project() {
    let temp = TempDir::new()?;
    let project_path = temp.path().join("new_project");

    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("init")
        .arg(&project_path);
    
    cmd.assert().success();
    assert!(project_path.exists());
}
```

### build command
```rust
#[test]
fn test_build_with_valid_args() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("build")
        .arg("--source").arg("test.rs")
        .arg("--chain").arg("solana");
    
    // Command should parse args correctly
}
```

### check command
```rust
#[test]
fn test_check_invariant_file() {
    let temp = TempDir::new()?;
    let invariant_file = temp.path().join("test.invar");
    // Write test file...

    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("check")
        .arg(&invariant_file);
    
    cmd.assert().success();
}
```

## Best Practices

### 1. Use Temporary Directories
```rust
#[test]
fn test_with_temp_files() {
    let temp = TempDir::new()?;
    let file_path = temp.path().join("test.txt");
    
    // File is cleaned up automatically when temp is dropped
    // Use of .path() is safe
}
```

### 2. Assert on Multiple Properties
```rust
#[test]
fn test_comprehensive() {
    let mut cmd = Command::cargo_bin("truent")?;
    cmd.arg("check").arg("test.invar");
    
    let assert = cmd.assert();
    assert
        .success()
        .stdout(predicate::str::contains("passed"))
        .stderr("");  // No errors
}
```

### 3. Predicate Matchers

```rust
use predicates::prelude::*;

cmd.assert()
    .success()
    .stdout(predicate::str::contains("invariants"))
    .stdout(predicate::str::contains("passed").or(predicate::str::contains("failed")))
    .stderr(predicate::str::is_empty().or(
        predicate::str::contains("warning")
    ));
```

### 4. JSON Validation
```rust
fn validate_json_report(output: &str) -> Result<()> {
    let json: serde_json::Value = serde_json::from_str(output)?;
    
    assert!(json["status"].is_string());
    assert!(json["invariants"].is_array());
    
    Ok(())
}

#[test]
fn test_json_structure() -> Result<()> {
    let output = run_command_get_output("report", &["--format", "json"])?;
    validate_json_report(&output)?;
    Ok(())
}
```

## Continuous Integration

CLI tests run in CI to ensure binary works correctly:

```yaml
cli-tests:
  name: CLI Behavior Tests
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Build CLI
      run: cargo build --release
    - name: Run CLI tests
      run: cargo test --test cli
```

## Troubleshooting

**Test passes locally but fails in CI:**
- Check PATH differences
- Verify file paths are platform-independent
- Don't assume specific working directory

**Output comparison failures:**
- Platform line endings (CRLF vs LF)
- Floating point formatting might differ
- Use `predicate::str::contains()` for partial matching

**Intermittent failures:**
- Could indicate race condition in CLI
- Run with `--test-threads=1` to debug
- Check for timing-dependent output
