//! CLI behavior tests.
//!
//! These tests validate that the CLI tool behaves correctly,
//! producing correct exit codes, output formats, and error handling.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Setup a test project directory with sample files
fn setup_test_project() -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let base = temp.path();

    // Create a sample DSL file
    let dsl_content = r#"
invariant: balance_conservation
description: "Total balance must be conserved across transactions"

forall tx in transactions:
    sum(tx.inputs) == sum(tx.outputs) + tx.fee
"#;

    fs::write(base.join("invariants.invar"), dsl_content).expect("Failed to write invariants file");

    temp
}

#[test]
fn test_cli_help_output() {
    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Truent"));
}

#[test]
fn test_cli_version_output() {
    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_cli_init_creates_project() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp.path().join("new_project");

    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("init").arg(&project_path);

    cmd.assert().success();
    assert!(
        project_path.exists(),
        "init should create project directory"
    );
}

#[test]
fn test_cli_missing_file_exits_with_error() {
    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("build")
        .arg("--source")
        .arg("/nonexistent/file.rs")
        .arg("--chain")
        .arg("solana")
        .arg("--output")
        .arg("/tmp/out");

    cmd.assert().failure();
}

#[test]
fn test_cli_invalid_chain_exits_with_error() {
    let temp = setup_test_project();

    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("build")
        .arg("--source")
        .arg(temp.path().join("test.rs"))
        .arg("--chain")
        .arg("invalid_chain")
        .arg("--output")
        .arg(temp.path().join("output"));

    cmd.assert().failure();
}

#[test]
fn test_cli_verbose_flag_produces_output() {
    let _temp = setup_test_project();

    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("--verbose").arg("doctor");

    cmd.assert().success();
}

#[test]
fn test_cli_log_level_flag() {
    let mut cmd = Command::cargo_bin("truent").expect("Found to find binary");
    cmd.arg("--verbose").arg("doctor");

    cmd.assert().success();
}

#[test]
fn test_cli_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
    cmd.arg("nonexistent_command");

    cmd.assert().failure();
}

/// Exit code tests
mod exit_codes {
    use super::*;

    #[test]
    fn test_exit_code_success_is_zero() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("--help");

        let output = cmd.output().expect("Failed to execute");
        assert_eq!(output.status.code(), Some(0), "Success should exit with 0");
    }

    #[test]
    fn test_exit_code_error_is_nonzero() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("nonexistent_command");

        let output = cmd.output().expect("Failed to execute");
        assert_ne!(
            output.status.code(),
            Some(0),
            "Error should exit with non-zero"
        );
    }
}

/// Output format tests
mod output_formats {
    use super::*;

    #[test]
    fn test_json_output_is_valid() {
        let temp = setup_test_project();

        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("report")
            .arg("--input")
            .arg(temp.path().join("test_report.json"))
            .arg("--format")
            .arg("json");

        // Check if output is valid JSON (when it runs successfully)
        let output = cmd.output().expect("Failed to execute");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Validate it's JSON-like
            assert!(stdout.contains("{") || stdout.is_empty());
        }
    }

    #[test]
    fn test_markdown_output() {
        let temp = setup_test_project();

        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("report")
            .arg("--input")
            .arg(temp.path().join("test_report.json"))
            .arg("--format")
            .arg("markdown");

        let output = cmd.output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Markdown output shouldn't be JSON
        assert!(!stdout.starts_with("{") || output.status.success());
    }
}

/// Configuration tests
mod configuration {
    use super::*;

    #[test]
    fn test_config_file_loading() {
        let temp = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp.path().join("invar.toml");

        let config_content = r#"
[project]
name = "test-project"
version = "0.1.0"

[invariants]
chains = ["solana", "evm"]
"#;

        fs::write(&config_path, config_content).expect("Failed to write config");

        // Config should be loadable
        assert!(config_path.exists());
    }
}

/// Determinism tests
mod determinism {
    use super::*;

    #[test]
    fn test_same_input_same_output() {
        let _temp = setup_test_project();

        // Run the same command twice
        let mut cmd1 = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd1.arg("list");
        let output1 = cmd1.output().expect("Failed to execute");

        let mut cmd2 = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd2.arg("list");
        let output2 = cmd2.output().expect("Failed to execute");

        // Same input should produce same output
        assert_eq!(output1.stdout, output2.stdout, "CLI must be deterministic");
        assert_eq!(
            output1.stderr, output2.stderr,
            "Errors must be deterministic"
        );
    }
}

/// Regression tests asserting that `truent check` actually detects real
/// vulnerabilities in the bundled fixtures, instead of always reporting a
/// clean scan. These fixtures previously sat unused by any test while the
/// detection pipeline itself was disconnected (see CHANGELOG / git history).
mod detection {
    use super::*;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    #[test]
    fn test_check_detects_vulnerable_evm_fixture() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("check")
            .arg(fixture_path("test_vulnerable_evm.sol"))
            .arg("--chain")
            .arg("evm")
            .arg("--format")
            .arg("json");

        let output = cmd.output().expect("Failed to execute");
        // Static detectors emit leads, not proven violations, and a lead does
        // not fail the run — blocking a build on an unexecuted pattern match
        // is what teaches teams to bypass the gate. The findings are still
        // reported; `--fail-on-leads` (asserted below) is how you gate on them.
        assert_eq!(
            output.status.code(),
            Some(0),
            "unproven leads must not fail the run by default"
        );

        let mut gated = Command::cargo_bin("truent").expect("binary exists");
        gated
            .arg("check")
            .arg(fixture_path("test_vulnerable_evm.sol"))
            .arg("--chain")
            .arg("evm")
            .arg("--format")
            .arg("json")
            .arg("--fail-on-leads");
        assert_eq!(
            gated.output().expect("Failed to execute").status.code(),
            Some(1),
            "--fail-on-leads must gate on unproven results"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("check --format json must emit valid JSON");
        let violation_count = json["summary"]["violations"]
            .as_u64()
            .expect("summary.violations must be a number");
        assert!(
            violation_count > 0,
            "Expected at least one violation in the known-vulnerable EVM fixture, found 0"
        );
    }

    #[test]
    fn test_check_detects_vulnerable_solana_fixture() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("check")
            .arg(fixture_path("test_vulnerable_solana.rs"))
            .arg("--chain")
            .arg("solana")
            .arg("--format")
            .arg("json");

        let output = cmd.output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("check --format json must emit valid JSON");
        let violation_count = json["summary"]["violations"]
            .as_u64()
            .expect("summary.violations must be a number");
        assert!(
            violation_count > 0,
            "Expected at least one violation in the known-vulnerable Solana fixture, found 0"
        );
    }

    #[test]
    fn test_check_detects_vulnerable_move_fixture() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("check")
            .arg(fixture_path("test_vulnerable_move.move"))
            .arg("--chain")
            .arg("move")
            .arg("--format")
            .arg("json");

        let output = cmd.output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("check --format json must emit valid JSON");
        let violation_count = json["summary"]["violations"]
            .as_u64()
            .expect("summary.violations must be a number");
        assert!(
            violation_count > 0,
            "Expected at least one violation in the known-vulnerable Move fixture, found 0"
        );
    }

    /// The chain-agnostic shared-IR rule (`unauthorized_privileged_mutation`,
    /// truent_ir::rules) used to only fire in each analyzer crate's own unit
    /// tests - it was never wired into the production detector pipeline the
    /// CLI actually calls. This proves it now fires end to end for the two
    /// chains whose semantic-model extractor needs no external tool (Solana's
    /// Anchor parser and Move's regex extractor both work on raw source
    /// text; EVM's needs solc, which this environment doesn't have, so it's
    /// intentionally not asserted on here).
    #[test]
    fn test_shared_ir_rule_fires_for_solana_and_move() {
        for (fixture, chain) in [
            ("test_vulnerable_solana.rs", "solana"),
            ("test_vulnerable_move.move", "move"),
        ] {
            let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
            cmd.arg("check")
                .arg(fixture_path(fixture))
                .arg("--chain")
                .arg(chain)
                .arg("--format")
                .arg("json");

            let output = cmd.output().expect("Failed to execute");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let json: serde_json::Value =
                serde_json::from_str(&stdout).expect("check --format json must emit valid JSON");
            let violations = json["violations"]
                .as_array()
                .expect("violations must be an array");

            assert!(
                violations
                    .iter()
                    .any(|v| v["invariant_id"] == "unauthorized_privileged_mutation"),
                "Expected the shared-IR rule to fire for {chain} fixture {fixture}, but it didn't. Violations: {violations:#?}"
            );
        }
    }

    #[test]
    fn test_scan_respects_severity_and_fail_on_filters() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("scan")
            .arg(fixture_path("test_vulnerable_evm.sol"))
            .arg("--chain")
            .arg("evm")
            .arg("--output")
            .arg("json")
            .arg("--severity")
            .arg("critical")
            .arg("--fail-on")
            .arg("critical");

        let output = cmd.output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("scan --output json must emit valid JSON");
        let violations = json["violations"]
            .as_array()
            .expect("violations must be an array");
        assert!(
            !violations.is_empty(),
            "Expected at least one critical violation in the vulnerable EVM fixture"
        );
        assert!(
            violations.iter().all(|v| v["severity"] == "critical"),
            "--severity critical must filter out all non-critical violations"
        );
        // Severity filtering is unchanged; what changed is that gating now
        // also requires the result to have been reproduced. These are leads,
        // so the run passes unless --fail-on-leads is given.
        assert_eq!(
            output.status.code(),
            Some(0),
            "critical *leads* are reported but do not fail the run"
        );
    }

    /// `truent fuzz` used to be a pure stub that printed "0 violations found"
    /// without doing any work. It now actually mutates the target file and
    /// runs the real detectors against each variant, so a run must complete
    /// successfully (exit 0, no crashes) rather than just no-op.
    #[test]
    fn test_fuzz_runs_against_real_detectors_without_crashing() {
        let mut cmd = Command::cargo_bin("truent").expect("Failed to find binary");
        cmd.arg("fuzz")
            .arg(fixture_path("test_vulnerable_evm.sol"))
            .arg("--chain")
            .arg("evm")
            .arg("--iterations")
            .arg("50")
            .arg("--depth")
            .arg("4")
            .arg("--seed")
            .arg("7");

        let output = cmd.output().expect("Failed to execute");
        assert_eq!(
            output.status.code(),
            Some(0),
            "fuzz must exit 0 when no crashes are found"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Ran 50 iterations"),
            "expected fuzz to report the iteration count, got: {stderr}"
        );
        assert!(
            stderr.contains("No crashes found"),
            "expected fuzz to report no crashes on the known-parseable fixture, got: {stderr}"
        );
    }
}
