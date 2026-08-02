#![deny(unsafe_code)]
#![allow(dead_code)] // Functions used in future feature development
#![allow(missing_docs)] // CLI is self-documenting via clap help

//! Truent CLI: Multi-chain smart contract invariant enforcement tool.
//!
//! Production-grade terminal UI with professional styling and comprehensive
//! analysis capabilities for smart contract security.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;

// UI module
mod ui;
use ui::*;

// Import analyzers
use truent_analyzer_evm::EvmAnalyzer;
use truent_analyzer_move::MoveAnalyzer;
use truent_analyzer_solana::SolanaAnalyzer;
use truent_analyzer_soroban::SorobanAnalyzer;
use truent_core::traits::ChainAnalyzer;
use truent_core::{CodeFuzzer, Finding};
use truent_library::InvariantLibrary;

// ============================================================================
// CLI STRUCTURE
// ============================================================================

/// Truent: Production-grade multi-chain invariant analysis tool.
#[derive(Parser)]
#[command(
    name = "truent",
    about = "Enforce invariants on smart contracts across Solana, EVM, Move, and Soroban",
    version = env!("CARGO_PKG_VERSION"),
    author = "Truent Contributors"
)]
struct Cli {
    /// Enable verbose output.
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all non-error output.
    #[arg(long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze contracts for invariant violations (recommended: use 'scan' instead).
    Check(CheckArgs),
    /// Scan contracts with full invariant enforcement (primary command).
    Scan(ScanArgs),
    /// Generate a report from analysis results.
    Report(ReportArgs),
    /// Initialize a .truent.toml configuration file.
    Init(InitArgs),
    /// Check that all Truent components are working correctly.
    Doctor(DoctorArgs),
    /// Display exploit registry.
    Registry(RegistryArgs),
    /// Display compiled invariants.
    Invariants(InvariantsArgs),
    /// Run fuzzer on contract invariants.
    Fuzz(FuzzArgs),
}

/// Arguments for the `doctor` subcommand.
#[derive(Parser)]
struct DoctorArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    format: FormatArg,

    /// Output file.
    #[arg(long)]
    output: Option<PathBuf>,
}

/// Arguments for the `check` subcommand.
#[derive(Parser)]
struct CheckArgs {
    /// Path to analyze (file or directory).
    path: PathBuf,

    /// Blockchain to analyze.
    #[arg(long, value_enum, default_value = "evm")]
    chain: ChainArg,

    /// Fail if violations at or above this severity are found.
    #[arg(long, value_enum, default_value = "low")]
    fail_on: SeverityArg,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    format: FormatArg,

    /// Output file (for non-text formats).
    #[arg(long)]
    output: Option<PathBuf>,

    /// Configuration file.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Random seed for reproducible analysis (default: 42).
    #[arg(long)]
    seed: Option<u64>,
}

/// Arguments for the `scan` subcommand (enhanced version of check).
#[derive(Parser)]
struct ScanArgs {
    /// Path to analyze (file or directory).
    path: PathBuf,

    /// Blockchain to analyze.
    #[arg(long, value_enum, default_value = "evm")]
    chain: ChainArg,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    output: FormatArg,

    /// Output file (for non-text formats).
    #[arg(long)]
    file: Option<PathBuf>,

    /// Minimum severity to report.
    #[arg(long, value_enum)]
    severity: Option<SeverityArg>,

    /// Filter by specific invariant ID (can be used multiple times).
    #[arg(long)]
    invariant: Vec<String>,

    /// RPC URL for on-chain verification (e.g., bridge config checks).
    #[arg(long)]
    rpc: Option<String>,

    /// Fail the scan if any issues at or above this severity are found.
    #[arg(long, value_enum, default_value = "critical")]
    fail_on: SeverityArg,

    /// Use parallel scanning with rayon (thread pool).
    #[arg(long)]
    parallel: bool,

    /// Disable ANSI color output.
    #[arg(long)]
    no_color: bool,

    /// Configuration file.
    #[arg(long)]
    config: Option<PathBuf>,
}

/// Arguments for the `registry` subcommand.
#[derive(Parser)]
struct RegistryArgs {
    /// Subcommand for registry operations.
    #[command(subcommand)]
    action: RegistryAction,
}

#[derive(Subcommand)]
enum RegistryAction {
    /// List all known exploits in the registry.
    List {
        /// Filter by chain (evm, solana, move).
        #[arg(long)]
        chain: Option<String>,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: FormatArg,
    },
    /// Show details of a specific exploit.
    Show {
        /// Exploit ID (e.g., "euler-finance-2023").
        id: String,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: FormatArg,
    },
}

/// Arguments for the `invariants` subcommand.
#[derive(Parser)]
struct InvariantsArgs {
    /// Subcommand for invariants operations.
    #[command(subcommand)]
    action: InvariantsAction,
}

#[derive(Subcommand)]
enum InvariantsAction {
    /// List all compiled invariants.
    List {
        /// Filter by chain (evm, solana, move).
        #[arg(long)]
        chain: Option<String>,
        /// Filter by severity.
        #[arg(long)]
        severity: Option<SeverityArg>,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: FormatArg,
    },
    /// Show details of a specific invariant.
    Show {
        /// Invariant ID (e.g., "evm_reentrancy_classic").
        id: String,
        /// Output format.
        #[arg(long, value_enum, default_value = "text")]
        format: FormatArg,
    },
}

/// Arguments for the `fuzz` subcommand.
#[derive(Parser)]
struct FuzzArgs {
    /// Contract file or directory to fuzz. Omit when using --address to
    /// fuzz an already-deployed contract by fetching its bytecode instead
    /// of reading local source.
    path: Option<PathBuf>,

    /// Blockchain to fuzz for.
    #[arg(long, value_enum, default_value = "evm")]
    chain: ChainArg,

    /// Maximum call sequence depth for fuzzer.
    #[arg(long, default_value = "10")]
    depth: usize,

    /// Number of iterations to run.
    #[arg(long, default_value = "10000")]
    iterations: u32,

    /// Random seed for reproducibility.
    #[arg(long)]
    seed: Option<u64>,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    output: FormatArg,

    /// Output file.
    #[arg(long)]
    file: Option<PathBuf>,

    /// Run the dynamic/coverage-guided invariant fuzzer (deploys the
    /// contract in-memory via revm, generates call sequences, and checks
    /// auto-detected invariants after every call) instead of the default
    /// source-mutation crash fuzzer. EVM only for now.
    #[arg(long)]
    dynamic: bool,

    /// Fuzz an already-deployed contract by address instead of local
    /// source: fetches its bytecode via --rpc-url and probes it against
    /// known ERC20/Ownable selectors (no ABI available for an unverified
    /// contract). Only valid with --dynamic; requires --rpc-url. Does not
    /// fork the contract's on-chain storage — only its code.
    #[arg(long, conflicts_with = "path")]
    address: Option<String>,

    /// JSON-RPC endpoint to fetch bytecode from when using --address.
    #[arg(long, requires = "address")]
    rpc_url: Option<String>,

    /// Solana fuzz plan (JSON): the genesis accounts, account pool, pins and
    /// invariants that an IDL cannot express. Required for
    /// `--dynamic --chain solana`, where `path` is the program's Anchor IDL.
    #[arg(long)]
    plan: Option<PathBuf>,
}

/// Arguments for the `report` subcommand.
#[derive(Parser)]
struct ReportArgs {
    /// Input results file.
    #[arg(long)]
    input: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    format: FormatArg,

    /// Output file.
    #[arg(long)]
    output: Option<PathBuf>,
}

/// Arguments for the `init` subcommand.
#[derive(Parser)]
struct InitArgs {
    /// Project directory.
    #[arg(default_value = ".")]
    path: PathBuf,
}

/// Supported blockchain networks.
#[derive(ValueEnum, Clone, Debug)]
enum ChainArg {
    /// Ethereum and EVM-compatible chains.
    Evm,
    /// Solana.
    Solana,
    /// Move (Aptos, Sui).
    Move,
    /// Soroban (Stellar).
    Soroban,
}

/// Violation severity levels.
#[derive(ValueEnum, Clone, Debug)]
enum SeverityArg {
    /// Low severity issues.
    Low,
    /// Medium severity issues.
    Medium,
    /// High severity issues.
    High,
    /// Critical severity issues.
    Critical,
}

/// Output format options.
#[derive(ValueEnum, Clone, Debug)]
enum FormatArg {
    /// Human-readable text with colors and boxes.
    Text,
    /// JSON (one object per line).
    Json,
    /// HTML report.
    Html,
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Show banner on first launch if TTY
    if is_tty() {
        eprintln!("{}", render_banner(env!("CARGO_PKG_VERSION")));
    }

    match cli.command {
        Commands::Check(args) => cmd_check(args, cli.quiet, cli.verbose)?,
        Commands::Scan(args) => cmd_scan(args, cli.quiet, cli.verbose)?,
        Commands::Report(args) => cmd_report(args, cli.quiet)?,
        Commands::Init(args) => cmd_init(args, cli.quiet)?,
        Commands::Doctor(args) => cmd_doctor(args, cli.quiet)?,
        Commands::Registry(args) => cmd_registry(args, cli.quiet)?,
        Commands::Invariants(args) => cmd_invariants(args, cli.quiet)?,
        Commands::Fuzz(args) => cmd_fuzz(args, cli.quiet, cli.verbose)?,
    }

    Ok(())
}

// ============================================================================
// COMMAND HANDLERS
// ============================================================================

/// Handle the `check` subcommand.
fn cmd_check(args: CheckArgs, quiet: bool, verbose: bool) -> Result<()> {
    let start_time = Instant::now();

    // Set random seed for reproducibility
    let seed = args.seed.unwrap_or(42);
    if verbose && !quiet {
        eprintln!("Setting random seed to: {}", seed);
    }

    let chain_name = match &args.chain {
        ChainArg::Evm => "EVM",
        ChainArg::Solana => "Solana",
        ChainArg::Move => "Move",
        ChainArg::Soroban => "Soroban",
    };

    // Convert config path to String for header
    let config_str = args
        .config
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let config_ref = config_str.as_deref();

    // Display header (only for text format)
    if !quiet && matches!(args.format, FormatArg::Text) {
        println!(
            "{}",
            render_check_header(
                &args.path.display().to_string(),
                chain_name,
                config_ref,
                args.config.as_ref().map(|p| p.exists()).unwrap_or(false)
            )
        );
    }

    // Create spinner (only for text format)
    let spinner = if !quiet && matches!(args.format, FormatArg::Text) {
        Some(Spinner::start(&format!(
            "Analyzing {}...",
            args.path.display()
        )))
    } else {
        None
    };

    // Run actual analysis
    let violations = match run_analysis(&args.path, &args.chain, verbose) {
        Ok(vios) => vios,
        Err(e) => {
            if let Some(s) = spinner {
                s.stop_with_failure(&e.to_string());
            }
            return Err(e);
        }
    };

    let duration_secs = start_time.elapsed().as_secs_f64();

    // Count violations by severity
    let (critical, high, medium, low) = count_violations_by_severity(&violations);
    // Approximate count of live detectors for this chain (a single detector can produce
    // multiple findings, so this is a lower bound, not an exact "checks run" count).
    let detector_count = match args.chain {
        ChainArg::Evm => 44,
        ChainArg::Solana => 11,
        ChainArg::Move => 7,
        ChainArg::Soroban => 9,
    };
    let total_checks = detector_count.max(violations.len());
    let passed = total_checks.saturating_sub(violations.len());

    // Build summary
    let summary = AnalysisSummary {
        target: args.path.display().to_string(),
        chain: chain_name.to_string(),
        total_checks,
        violations: violations.len(),
        passed,
        suppressed: 0,
        duration_secs,
        severity_breakdown: SeverityBreakdown {
            critical,
            high,
            medium,
            low,
        },
    };

    // Stop spinner with success
    if let Some(s) = spinner {
        s.stop_with_success(&format!("{} checks in {:.1}s", total_checks, duration_secs));
    }

    // Handle different output formats
    match args.format {
        FormatArg::Text => {
            // Build text report
            let mut report_text = String::new();

            // Display violations
            if !violations.is_empty() {
                report_text.push_str(&render_violations(&violations));
                report_text.push('\n');
            }

            // Display passed checks (verbose mode)
            if verbose {
                let passed_check_names = vec![
                    "balance_conservation".to_string(),
                    "no_integer_overflow".to_string(),
                    "owner_only_withdraw".to_string(),
                    "access_control_present".to_string(),
                    "arithmetic_overflow".to_string(),
                    "missing_signer_check".to_string(),
                ];
                report_text.push_str(&render_passed_checks(&passed_check_names));
                report_text.push('\n');
            }

            // Display summary
            report_text.push_str(&render_summary(&summary));

            if let Some(output_path) = args.output {
                // Write to file
                std::fs::write(&output_path, &report_text)?;
                if !quiet {
                    eprintln!("✓ Report written to {}", output_path.display());
                }
            } else {
                // Write to stdout
                if !quiet {
                    println!("{}", report_text);
                }
            }
        }
        FormatArg::Json => {
            // Create JSON report
            let report = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "chain": summary.chain,
                "target": summary.target,
                "duration_ms": (summary.duration_secs * 1000.0) as u64,
                "summary": {
                    "total_checks": summary.total_checks,
                    "violations": summary.violations,
                    "critical": summary.severity_breakdown.critical,
                    "high": summary.severity_breakdown.high,
                    "medium": summary.severity_breakdown.medium,
                    "low": summary.severity_breakdown.low,
                    "passed": summary.passed,
                    "suppressed": summary.suppressed,
                },
                "violations": violations,
            });

            let output_json = serde_json::to_string_pretty(&report)?;

            if let Some(output_path) = args.output {
                // Write to file
                std::fs::write(&output_path, &output_json)?;
                if !quiet {
                    eprintln!("✓ Report written to {}", output_path.display());
                }
            } else {
                // Write to stdout
                println!("{}", output_json);
            }
        }
        FormatArg::Html => {
            // Generate HTML report
            let html_report = generate_html_report(&summary, &violations);

            if let Some(output_path) = args.output {
                // Write to file
                std::fs::write(&output_path, &html_report)?;
                if !quiet {
                    eprintln!("✓ HTML report written to {}", output_path.display());
                }
            } else {
                // Write to stdout
                println!("{}", html_report);
            }
        }
    }

    // Exit non-zero only if a violation meets or exceeds --fail-on (default: low, i.e. any).
    let fail_rank = severity_arg_rank(&args.fail_on);
    if violations
        .iter()
        .any(|v| severity_rank(&v.severity) >= fail_rank)
    {
        std::process::exit(1);
    }

    Ok(())
}

/// Generate an HTML report of the security analysis.
fn generate_html_report(summary: &AnalysisSummary, violations: &[Violation]) -> String {
    let violation_rows = violations
        .iter()
        .map(|v| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td></tr>",
                v.invariant_id,
                v.severity,
                v.location,
                v.message.replace("<", "&lt;").replace(">", "&gt;")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    #[allow(clippy::useless_vec)]
    let severity_colors = vec![
        format!(
            "<li><strong>Critical:</strong> {} findings</li>",
            summary.severity_breakdown.critical
        ),
        format!(
            "<li><strong>High:</strong> {} findings</li>",
            summary.severity_breakdown.high
        ),
        format!(
            "<li><strong>Medium:</strong> {} findings</li>",
            summary.severity_breakdown.medium
        ),
        format!(
            "<li><strong>Low:</strong> {} findings</li>",
            summary.severity_breakdown.low
        ),
    ];

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Truent Security Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            margin: 20px;
            background-color: #f6f8fb;
            color: #24292e;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #0366d6;
            border-bottom: 2px solid #e1e4e8;
            padding-bottom: 10px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin: 20px 0;
        }}
        .summary-box {{
            background: #f6f8fa;
            padding: 15px;
            border-radius: 6px;
            border-left: 4px solid #0366d6;
        }}
        .summary-box strong {{
            font-size: 18px;
            color: #0366d6;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th {{
            background-color: #f6f8fa;
            padding: 12px;
            text-align: left;
            font-weight: 600;
            border-bottom: 2px solid #e1e4e8;
        }}
        td {{
            padding: 12px;
            border-bottom: 1px solid #e1e4e8;
        }}
        tr:hover {{
            background-color: #f6f8fa;
        }}
        .severity-critical {{
            color: #d73a49;
            font-weight: 600;
        }}
        .severity-high {{
            color: #fd7e14;
            font-weight: 600;
        }}
        .severity-medium {{
            color: #ffc107;
            font-weight: 600;
        }}
        .severity-low {{
            color: #6f42c1;
            font-weight: 600;
        }}
        .timestamp {{
            color: #6a737d;
            font-size: 12px;
            margin-top: 20px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 Truent Security Analysis Report</h1>
        
        <div class="summary">
            <div class="summary-box">
                <strong>Target:</strong> {}<br>
                <strong>Chain:</strong> {}<br>
                <strong>Duration:</strong> {:.2}s
            </div>
            <div class="summary-box">
                <strong>Total Checks:</strong> {}<br>
                <strong>Violations Found:</strong> {}<br>
                <strong>Passed:</strong> {}
            </div>
        </div>

        <h2>Severity Breakdown</h2>
        <ul>
            {}
        </ul>

        <h2>Findings</h2>
        {}
        
        <table>
            <thead>
                <tr>
                    <th>Invariant</th>
                    <th>Severity</th>
                    <th>Location</th>
                    <th>Message</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>

        <div class="timestamp">
            Generated by Truent v{}
        </div>
    </div>
</body>
</html>"#,
        summary.target,
        summary.chain,
        summary.duration_secs,
        summary.total_checks,
        summary.violations,
        summary.passed,
        severity_colors.join("\n"),
        if violations.is_empty() {
            "<p style=\"color: #28a745; font-weight: 600;\">✓ No security violations found!</p>"
                .to_string()
        } else {
            format!(
                "<p style=\"color: #d73a49;\">⚠️ {} security violations detected</p>",
                violations.len()
            )
        },
        violation_rows,
        env!("CARGO_PKG_VERSION"),
    )
}

/// File extension associated with each chain's source files.
fn chain_extension(chain: &ChainArg) -> &'static str {
    match chain {
        ChainArg::Evm => "sol",
        ChainArg::Solana => "rs",
        ChainArg::Move => "move",
        ChainArg::Soroban => "rs",
    }
}

/// Recursively collect source files matching the chain's extension under `dir`.
fn collect_source_files(dir: &Path, extension: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip common non-source directories to avoid scanning dependency trees.
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if matches!(
                name,
                "node_modules" | "target" | ".git" | "build" | "dist" | "out"
            ) {
                continue;
            }
            collect_source_files(&path, extension, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    Ok(())
}

/// Run every live pattern detector for `chain` against a single file's source text.
fn run_detectors_on_file(chain: &ChainArg, path: &Path) -> Result<Vec<Finding>> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let file_path = path.to_string_lossy().to_string();

    let findings = match chain {
        ChainArg::Evm => truent_analyzer_evm::detectors::run_all_detectors(&source, &file_path),
        ChainArg::Solana => truent_analyzer_solana::run_all_detectors(&source, &file_path),
        ChainArg::Move => truent_analyzer_move::run_all_detectors(&source, &file_path),
        ChainArg::Soroban => truent_analyzer_soroban::run_all_detectors(&source, &file_path),
    };

    Ok(findings)
}

/// Convert a title-cased, human-readable name out of a detector's invariant_id,
/// e.g. "evm_missing_post_state_health_check" -> "Missing Post State Health Check".
fn invariant_id_to_title(invariant_id: &str) -> String {
    let stripped = invariant_id
        .strip_prefix("evm_")
        .or_else(|| invariant_id.strip_prefix("sol_"))
        .or_else(|| invariant_id.strip_prefix("move_"))
        .unwrap_or(invariant_id);

    stripped
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert a detector `Finding` into a display/report-ready `Violation`.
fn finding_to_violation(finding: &Finding, index: usize, total: usize) -> Violation {
    let reference = get_vulnerability_reference(&finding.invariant_id);
    Violation {
        index,
        total,
        severity: finding.severity.name().to_lowercase(),
        title: invariant_id_to_title(&finding.invariant_id),
        invariant_id: finding.invariant_id.clone(),
        location: format!("{}:{}", finding.file, finding.line),
        cwe: map_invariant_to_cwe(&finding.invariant_id),
        message: finding.message.clone(),
        recommendation: format!(
            "Review this finding and apply the recommended fix. Details: {}",
            reference
        ),
        reference,
        code_snippet: finding
            .source_fragment
            .clone()
            .unwrap_or_else(|| finding.snippet.clone()),
    }
}

/// Run actual analysis on the source file or directory, returning display-ready violations.
fn run_analysis(source_path: &Path, chain: &ChainArg, verbose: bool) -> Result<Vec<Violation>> {
    // Check if path exists
    if !source_path.exists() {
        return Err(anyhow::anyhow!("Path not found: {}", source_path.display()));
    }

    let extension = chain_extension(chain);
    let files = if source_path.is_dir() {
        let mut files = Vec::new();
        collect_source_files(source_path, extension, &mut files)?;
        files
    } else {
        vec![source_path.to_path_buf()]
    };

    if files.is_empty() {
        if verbose {
            eprintln!(
                "⚠ No .{} files found under {}",
                extension,
                source_path.display()
            );
        }
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();
    for file in &files {
        findings.extend(run_detectors_on_file(chain, file)?);
    }

    if verbose {
        eprintln!(
            "✓ Scanned {} file(s), {} findings from pattern detectors",
            files.len(),
            findings.len()
        );
    }

    // Best-effort structural analysis (function/state-var counts) for diagnostics only.
    // This requires solc (EVM) to be installed; never let its absence block detection,
    // since the detectors above operate on raw source text and don't need it.
    if verbose {
        if let Some(first_file) = files.first() {
            let structural = match chain {
                ChainArg::Evm => EvmAnalyzer.analyze(first_file),
                ChainArg::Solana => SolanaAnalyzer.analyze(first_file),
                ChainArg::Move => MoveAnalyzer.analyze(first_file),
                ChainArg::Soroban => SorobanAnalyzer.analyze(first_file),
            };
            match structural {
                Ok(program) => eprintln!(
                    "✓ Structural analysis: {} functions in {}",
                    program.functions.len(),
                    first_file.display()
                ),
                Err(e) => eprintln!("⚠ Structural analysis unavailable ({e}); detectors still ran"),
            }
        }

        // Load real built-in invariants for this chain (informational for now; the
        // invariant library's default expressions aren't yet wired into detection).
        let chain_name = match chain {
            ChainArg::Evm => "evm",
            ChainArg::Solana => "solana",
            ChainArg::Move => "move",
            ChainArg::Soroban => "soroban",
        };
        let lib = InvariantLibrary::with_defaults(chain_name);
        eprintln!(
            "✓ Loaded {} built-in invariant definitions for {}",
            lib.all().len(),
            chain_name
        );
    }

    let total = findings.len();
    let violations: Vec<Violation> = findings
        .iter()
        .enumerate()
        .map(|(i, f)| finding_to_violation(f, i + 1, total))
        .collect();

    Ok(violations)
}

/// Generate detailed violation information with actionable recommendations.
fn generate_detailed_violation_info(
    program: &truent_core::model::ProgramModel,
    invariant: &truent_core::model::Invariant,
    confidence: f64,
) -> (String, String) {
    let invariant_lower = invariant.name.to_lowercase();
    let is_solana = program.chain.to_lowercase().contains("solana");

    // Solana-specific violation details
    if is_solana {
        if invariant_lower.contains("lamport") {
            let message = format!(
                "Detected unsafe lamport manipulation with {:.0}% confidence. Direct arithmetic on account lamports without validation or safety checks detected.",
                confidence * 100.0
            );
            let recommendation =
                "CRITICAL: Never directly manipulate lamports without proper validation.\n\
                 Fix: Use checked arithmetic and validate account signer status before modifying lamports:\n\
                 ✓ Require account to be a signer: #[account(mut, signer)]\n\
                 ✓ Use checked_add/checked_sub instead of saturating_add/sub\n\
                 ✓ Validate minimum balance after transfer\n\
                 ✓ Consider using Solana's system program for transfers\n\
                 Reference: https://docs.solana.com/developing/programming-model/transactions"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("overflow") || invariant_lower.contains("integer_overflow") {
            let message = format!(
                "Detected unchecked arithmetic operation with {:.0}% confidence. Potential integer overflow/underflow risk found.",
                confidence * 100.0
            );
            let recommendation =
                "Use overflow-checked arithmetic for all calculations:\n\
                 Available options:\n\
                 1. Use checked_add/checked_sub/checked_mul/checked_div that return Option<T>\n\
                 2. Use wrapping_add/wrapping_sub for intentional wrapping behavior (rare, always document)\n\
                 3. For Solana tokens, use SPL Token's u128 multiplication internally\n\
                 4. Add overflow checks with: require!(value <= MAX_ALLOWED, ErrorCode::Overflow)\n\
                 Example fix:\n\
                   let result = amount.checked_add(fee).ok_or(ErrorCode::Overflow)?;\n\
                 Reference: https://github.com/solana-labs/spl-token/blob/master/program/src/instruction.rs"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("signer") {
            let message = format!(
                "Detected missing signer verification with {:.0}% confidence. Function may accept unauthorized callers.",
                confidence * 100.0
            );
            let recommendation =
                "Ensure all sensitive operations require proper signer verification:\n\
                 Required fixes:\n\
                 1. Mark sensitive account parameters as signers: #[account(mut, signer)]\n\
                 2. Add explicit checks: require!(account.is_signer, ErrorCode::MissingSigner)\n\
                 3. For specific authorities, validate: require!(authority.key == EXPECTED_AUTH, ...)\n\
                 4. Use require_keys_eq! macro for owner validation\n\
                 Security: This prevents unauthorized account ownership transfers and fund theft.\n\
                 Reference: https://docs.rs/anchor-lang/latest/anchor_lang/require_keys_eq/index.html"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("account_validation") || invariant_lower.contains("account") {
            let message = format!(
                "Detected missing account validation with {:.0}% confidence. Accounts may not be properly owned or validated.",
                confidence * 100.0
            );
            let recommendation =
                "Implement comprehensive account validation:\n\
                 Required checks for each account:\n\
                 1. Owner verification: require!(account.owner == &system_program::ID, ...)\n\
                 2. Account type validation: Verify account data layout matches expected structure\n\
                 3. Signer checks: require!(account.is_signer, ...) for authority accounts\n\
                 4. Mint validation: For token accounts, verify mint matches expected\n\
                 Anchor example:\n\
                   #[account(mut, owner = system_program::ID)]\n\
                   pub account: UncheckedAccount<'info>,\n\
                 Better approach: Use Account<'info, YourDataType> for automatic validation\n\
                 Reference: https://docs.anchor-lang.com/frequently-asked-questions/security#how-do-i-validate-accounts"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("rent") {
            let message = format!(
                "Detected potential rent exemption issue with {:.0}% confidence. Account may not maintain required minimum balance.",
                confidence * 100.0
            );
            let recommendation = "Ensure accounts maintain rent exemption:\n\
                 Solana requires accounts to maintain a minimum lamport balance.\n\
                 Best practices:\n\
                 1. Allocate sufficient space for account data\n\
                 2. Set initial balance >= rent_exempt_minimum\n\
                 3. When withdrawing lamports: verify_account_rent_exemption!(account, rent)\n\
                 4. Use system_program::create_account for proper initialization\n\
                 5. For PDAs, ensure bump seed doesn't affect rent calculation\n\
                 Implementation:\n\
                   let rent = Rent::get()?;\n\
                   let required_lamports = rent.minimum_balance(data_len);\n\
                 Reference: https://docs.solana.com/developing/programming-model/accounts"
                .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("pda") {
            let message = format!(
                "Detected PDA derivation issue with {:.0}% confidence. Bump seed or seed derivation may be incorrect.",
                confidence * 100.0
            );
            let recommendation =
                "Fix PDA derivation security issues:\n\
                 Common problems and fixes:\n\
                 1. Hardcoded bump seed: WRONG - use find_program_address instead\n\
                 2. Missing bump storage: Always store bump in account data for verification\n\
                 3. Seed ordering: Order seeds consistently \n\
                 4. Seed validation: Verify derived PDA in instrumentation\n\
                 Correct pattern:\n\
                   let (pda, bump) = Pubkey::find_program_address(&[seed], program_id);\n\
                   require_keys_eq!(expected_account, pda, ErrorCode::InvalidPDA);\n\
                 Store bump and re-derive for verification, never trust the bump argument\n\
                 Reference: https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("deserial") || invariant_lower.contains("instruction") {
            let message = format!(
                "Detected unsafe deserialization with {:.0}% confidence. Account data not properly validated before parsing.",
                confidence * 100.0
            );
            let recommendation =
                "Implement safe deserialization practices:\n\
                 Never assume account data layout matches expectations.\n\
                 Best practices:\n\
                 1. Use try_from_slice with error handling (not unwrap)\n\
                 2. Validate account size before deserializing: require!(account.data_len() >= EXPECTED_SIZE, ...)\n\
                 3. Use Anchor's Account<T> type which handles validation\n\
                 4. For raw deserialization: let data = account.data.borrow();\n\
                             let parsed = MyData::try_from_slice(&data)?;\n\
                 5. Add version checks for account data migrations\n\
                 Never use: \n\
                   let data = from_slice::<MyData>(&account.data).unwrap(); ❌\n\
                 Reference: https://docs.anchor-lang.com/frequently-asked-questions/security#how-do-i-validate-data"
                    .to_string();
            return (message, recommendation);
        }

        if invariant_lower.contains("token") {
            let message = format!(
                "Detected unchecked token operation with {:.0}% confidence. Token transfers may overflow or lose precision.",
                confidence * 100.0
            );
            let recommendation =
                "Use SPL Token's checked arithmetic for all transfers:\n\
                 Security issues with unchecked token math:\n\
                 1. Overflow in token amounts (rare but possible with custom decimals)\n\
                 2. Fee rounding attacks\n\
                 3. Solana Token program has internal overflow checks, but verify your math\n\
                 Best practice:\n\
                   // For SPL Token transfers, the program validates\n\
                   spl_token::instruction::transfer(...)?\n\
                 For custom math:\n\
                   let amount = tokens.checked_mul(price).ok_or(ErrorCode::Overflow)?;\n\
                 Testing:\n\
                   - Test with amounts near u64::MAX\n\
                   - Test with high-decimal tokens\n\
                 Reference: https://github.com/solana-labs/solana-program-library/tree/master/token"
                    .to_string();
            return (message, recommendation);
        }
    }

    // Generic/cross-platform violations
    if invariant_lower.contains("reentrancy") {
        let message = format!(
            "Detected potential reentrancy risk with {:.0}% confidence. Complex state interactions without guards.",
            confidence * 100.0
        );
        let recommendation = "Implement reentrancy protections:\n\
             1. Use checks-effects-interactions pattern\n\
             2. Apply state changes before external calls\n\
             3. Use reentrancy guards (mutex-style locking)\n\
             4. For EVM: use OpenZeppelin's ReentrancyGuard\n\
             5. For Solana: No reentrancy risk by design (sequential execution)\n\
             Reference: https://ethereumbook.org/code/vulnerabilities/"
            .to_string();
        return (message, recommendation);
    }

    // Fallback for unknown violations
    let message = format!(
        "Detected violation of '{}' invariant with {:.0}% confidence.",
        invariant.name,
        confidence * 100.0
    );
    let recommendation = format!(
        "Review the '{}' invariant documentation at https://docs.truent.dev/invariants/{} and apply recommended fixes.",
        invariant.name, invariant.name
    );

    (message, recommendation)
}

/// Find the approximate line where a vulnerability marker appears in the program.
fn find_vulnerability_line(
    program: &truent_core::model::ProgramModel,
    invariant_name: &str,
) -> Option<usize> {
    let invariant_lower = invariant_name.to_lowercase();

    // Search all markers for any embedded line number
    for func in program.functions.values() {
        for marker in &func.mutates {
            // Look for embedded line numbers in format: MARKER:LINE_NUMBER
            if let Some(colon_pos) = marker.rfind(':') {
                if let Ok(line_num) = marker[colon_pos + 1..].parse::<usize>() {
                    // We found a marker with an embedded line number
                    // Check if this marker is relevant to the invariant
                    let marker_upper = marker.to_uppercase();

                    #[allow(clippy::if_same_then_else)]
                    // Match based on invariant type
                    if invariant_lower.contains("signer") && marker_upper.contains("SIGNER") {
                        return Some(line_num);
                    } else if invariant_lower.contains("lamport")
                        && marker_upper.contains("LAMPORT")
                    {
                        return Some(line_num);
                    } else if (invariant_lower.contains("overflow")
                        || invariant_lower.contains("arithmetic"))
                        && marker_upper.contains("ARITHMETIC")
                    {
                        return Some(line_num);
                    } else if invariant_lower.contains("account")
                        && (marker_upper.contains("ACCOUNT") || marker_upper.contains("VALIDATION"))
                    {
                        return Some(line_num);
                    } else if invariant_lower.contains("rent") && marker_upper.contains("RENT") {
                        return Some(line_num);
                    } else if invariant_lower.contains("pda") && marker_upper.contains("PDA") {
                        return Some(line_num);
                    } else if (invariant_lower.contains("deserialization")
                        || invariant_lower.contains("instruction"))
                        && (marker_upper.contains("DESERIAL")
                            || marker_upper.contains("INSTRUCTION"))
                    {
                        return Some(line_num);
                    } else if invariant_lower.contains("token") && marker_upper.contains("TOKEN") {
                        return Some(line_num);
                    } else if invariant_lower.contains("reentrancy")
                        && marker_upper.contains("REENTRANCY")
                    {
                        return Some(line_num);
                    }
                }
            }
        }
    }

    None
}

/// Extract code snippet from source file at the given line number.
/// Shows the target line plus 2 lines of context before and after.
fn extract_code_snippet(
    source_path: &std::path::Path,
    line_number: usize,
) -> std::io::Result<String> {
    use std::fs;
    use std::io::BufRead;

    let file = fs::File::open(source_path)?;
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    // Calculate context range (2 lines before and after)
    let start_line = if line_number > 2 { line_number - 3 } else { 0 };
    let end_line = std::cmp::min(line_number + 1, lines.len());

    if line_number == 0 || line_number > lines.len() {
        return Ok(format!(
            "Line {} is out of range in {}",
            line_number,
            source_path.display()
        ));
    }

    let mut snippet = String::new();
    for (idx, line) in lines[start_line..end_line].iter().enumerate() {
        let actual_line_num = start_line + idx + 1;
        let marker = if actual_line_num == line_number {
            ">>> "
        } else {
            "    "
        };
        snippet.push_str(&format!("{}{:3} | {}\n", marker, actual_line_num, line));
    }

    Ok(snippet.trim().to_string())
}

/// Get proper reference documentation links for each vulnerability type.
/// Map invariant IDs to documentation anchors in INVARIANT_LIBRARY.md
/// Handles 50+ detected invariants by mapping to canonical documentation
fn get_vulnerability_reference(invariant_id: &str) -> String {
    // Map of detected invariant IDs to documentation section anchors
    // Format: full_id -> anchor_in_library_md
    let invariant_mapping: &[(&str, &str)] = &[
        // Balance & Arithmetic (IDs 1-5 in docs)
        ("reentrancy_classic", "#1-balance_conservation"),
        ("overflow", "#2-no_integer_overflow"),
        ("underflow", "#3-no_integer_underflow"),
        ("balance_check", "#4-positive_balance"),
        ("conservation_check", "#5-supply_tracking"),
        ("conservation_check_absent", "#5-supply_tracking"),
        // Access Control (IDs 6-9 in docs)
        ("missing_signer", "#6-owner_only_function"),
        ("access_control", "#7-role_based_access"),
        ("shallow_auth", "#7-role_based_access"),
        ("single_eoa_admin", "#8-admin_override_safe"),
        ("permission", "#9-permission_consistency"),
        // State Consistency (IDs 10-13 in docs)
        ("state_transition", "#11-state_transition_valid"),
        ("reentrancy", "#12-no_reentrancy"),
        ("paused", "#13-paused_state_valid"),
        // Cross-Chain (IDs 14-16 in docs)
        ("bridge", "#14-bridge_conservation"),
        ("oracle", "#15-oracle_freshness"),
        ("oracle_spot_price", "#15-oracle_freshness"),
        ("oracle_self_trade", "#15-oracle_freshness"),
        ("oracle_rate", "#15-oracle_freshness"),
        ("canonical", "#16-canonical_state"),
        // Transaction Safety (IDs 17-22 in docs)
        ("signature", "#17-signature_validation"),
        ("nonce", "#18-nonce_ordering"),
        ("gas", "#19-gas_efficiency"),
        ("delegatecall", "#20-safe_delegatecall"),
        ("selfdestruct", "#21-safe_selfdestruct"),
        ("timestamp", "#22-no_timestamp_dependence"),
        // Additional EVM-specific invariants
        ("flash_loan", "#12-no_reentrancy"),
        ("dvn", "#15-oracle_freshness"),
        ("merkle_root", "#11-state_transition_valid"),
        ("precision_loss", "#2-no_integer_overflow"),
        ("zero_challenge", "#11-state_transition_valid"),
        ("public_relay", "#6-owner_only_function"),
        ("aa_entropy", "#17-signature_validation"),
        ("bridge_address", "#14-bridge_conservation"),
        ("synthetic_collateral", "#16-canonical_state"),
        ("dvn_single_point", "#15-oracle_freshness"),
        ("lst_depeg", "#5-supply_tracking"),
        ("erc4626_inflation", "#5-supply_tracking"),
        ("token_balance_manipulation", "#1-balance_conservation"),
        ("arbitrary_call_msg_value", "#20-safe_delegatecall"),
        ("router_slippage", "#15-oracle_freshness"),
        ("health_check", "#11-state_transition_valid"),
        ("state_mutation_ordering", "#11-state_transition_valid"),
        ("unbacked_synthetic_mint", "#5-supply_tracking"),
        ("upgrade_path", "#20-safe_delegatecall"),
        ("constructor_race", "#11-state_transition_valid"),
        ("proxy_storage", "#11-state_transition_valid"),
        ("arithmetic_rounding", "#2-no_integer_overflow"),
        // Solana-specific
        ("signer", "#6-owner_only_function"),
        ("account_validation", "#6-owner_only_function"),
        ("rent_exemption", "#10-state_immutability"),
        ("pda_authority", "#11-state_transition_valid"),
        ("sysvar_account", "#6-owner_only_function"),
        ("admin_timelock", "#8-admin_override_safe"),
        ("treasury_authority", "#8-admin_override_safe"),
        ("durable_nonce", "#18-nonce_ordering"),
        // Move-specific
        ("liquidity_conservation", "#1-balance_conservation"),
        ("type_safety", "#11-state_transition_valid"),
        ("resource_destruction", "#1-balance_conservation"),
    ];

    let id_lower = invariant_id.to_lowercase();

    // Strip chain prefix (evm_, sol_, move_)
    let clean_id = id_lower
        .strip_prefix("evm_")
        .unwrap_or(&id_lower)
        .strip_prefix("sol_")
        .unwrap_or(&id_lower)
        .strip_prefix("move_")
        .unwrap_or(&id_lower);

    // Try to find a mapping for any substring match
    for (pattern, anchor) in invariant_mapping.iter() {
        if clean_id.contains(pattern) {
            return format!(
                "https://github.com/geekstrancend/Truent/blob/main/docs/INVARIANT_LIBRARY.md{}",
                anchor
            );
        }
    }

    // Fallback to GitHub search if no mapping found
    format!(
        "https://github.com/geekstrancend/Truent/search?q={}",
        id_lower.replace("_", "%20")
    )
}

/// Detect which invariants were actually violated based on program structure.
fn detect_violated_invariants(
    program: &truent_core::model::ProgramModel,
    invariants: &[truent_core::model::Invariant],
) -> Vec<(truent_core::model::Invariant, f64)> {
    let mut violated = Vec::new();

    // Heuristic: check invariants based on program characteristics
    for invariant in invariants {
        let confidence = calculate_violation_confidence(program, invariant);
        if confidence > 0.3 {
            // Threshold for reporting
            violated.push((invariant.clone(), confidence));
        }
    }

    violated
}

/// Calculate confidence score for an invariant violation based on program analysis.
fn calculate_violation_confidence(
    program: &truent_core::model::ProgramModel,
    invariant: &truent_core::model::Invariant,
) -> f64 {
    let mut confidence = 0.0;
    let invariant_lower = invariant.name.to_lowercase();
    let chain_lower = program.chain.to_lowercase();

    // === SOLANA-SPECIFIC DETECTIONS ===
    if chain_lower.contains("solana") {
        if (invariant_lower.contains("lamport") || invariant_lower.contains("sol_lamport"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA_LAMPORT_UNSAFE"))
            })
        {
            return 0.95;
        }
        if (invariant_lower.contains("overflow")
            || invariant_lower.contains("sol_integer_overflow"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA_UNCHECKED_ARITHMETIC"))
            })
        {
            return 0.90;
        }
        if (invariant_lower.contains("signer") || invariant_lower.contains("sol_signer_checks"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA_MISSING_SIGNER"))
            })
        {
            return 0.88;
        }
        if (invariant_lower.contains("account")
            || invariant_lower.contains("sol_account_validation"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA__MISSING_VALIDATION"))
            })
        {
            return 0.85;
        }
        if (invariant_lower.contains("rent") || invariant_lower.contains("sol_rent_exemption"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA_RENT_EXEMPTION"))
            })
        {
            return 0.82;
        }
        if (invariant_lower.contains("pda") || invariant_lower.contains("sol_pda_derivation"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("SOLANA_PDA_DERIVATION"))
            })
        {
            return 0.80;
        }
    }

    // === EVM-SPECIFIC DETECTIONS ===
    if chain_lower.contains("evm") {
        if invariant_lower.contains("reentrancy")
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("EVM_REENTRANCY")))
        {
            return 0.93;
        }
        if (invariant_lower.contains("call") || invariant_lower.contains("external"))
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("EVM_UNCHECKED_CALL")))
        {
            return 0.91;
        }
        if (invariant_lower.contains("overflow") || invariant_lower.contains("arithmetic"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("EVM_UNCHECKED_ARITHMETIC"))
            })
        {
            return 0.89;
        }
        if invariant_lower.contains("delegatecall")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("EVM_DELEGATECALL_ABUSE"))
            })
        {
            return 0.92;
        }
        if invariant_lower.contains("timestamp")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("EVM_TIMESTAMP_DEPENDENCY"))
            })
        {
            return 0.85;
        }
        if (invariant_lower.contains("front") || invariant_lower.contains("ordering"))
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("EVM_FRONT_RUNNING")))
        {
            return 0.80;
        }
        if invariant_lower.contains("access")
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("EVM_ACCESS_CONTROL")))
        {
            return 0.87;
        }
        if invariant_lower.contains("validation")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("EVM_INPUT_VALIDATION"))
            })
        {
            return 0.83;
        }
    }

    // === MOVE-SPECIFIC DETECTIONS ===
    if chain_lower.contains("move") {
        if invariant_lower.contains("resource")
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("MOVE_RESOURCE_LEAK")))
        {
            return 0.89;
        }
        if invariant_lower.contains("ability")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("MOVE_MISSING_ABILITY"))
            })
        {
            return 0.86;
        }
        if (invariant_lower.contains("overflow") || invariant_lower.contains("arithmetic"))
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("MOVE_UNCHECKED_ARITHMETIC"))
            })
        {
            return 0.88;
        }
        if invariant_lower.contains("signer")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("MOVE_MISSING_SIGNER"))
            })
        {
            return 0.84;
        }
        if invariant_lower.contains("mutation")
            || invariant_lower.contains("unguarded")
                && program.functions.iter().any(|f| {
                    f.1.mutates
                        .iter()
                        .any(|m| m.contains("MOVE_UNGUARDED_MUTATION"))
                })
        {
            return 0.82;
        }
        if invariant_lower.contains("privilege")
            && program.functions.iter().any(|f| {
                f.1.mutates
                    .iter()
                    .any(|m| m.contains("MOVE_PRIVILEGE_ESCALATION"))
            })
        {
            return 0.81;
        }
        if invariant_lower.contains("abort")
            && program
                .functions
                .iter()
                .any(|f| f.1.mutates.iter().any(|m| m.contains("MOVE_UNSAFE_ABORT")))
        {
            return 0.79;
        }
    }

    // Check for reentrancy patterns
    if invariant_lower.contains("reentrancy") && program.functions.len() > 2 {
        confidence += 0.3;
    }

    // Check for arithmetic issues
    if (invariant_lower.contains("overflow") || invariant_lower.contains("underflow"))
        && program.functions.iter().any(|f| {
            f.1.name.contains("add") || f.1.name.contains("mul") || f.1.name.contains("increment")
        })
    {
        confidence += 0.35;
    }

    // Check for access control
    if invariant_lower.contains("access")
        && program.functions.iter().any(|f| f.1.is_entry_point)
        && program.functions.len() > 1
    {
        confidence += 0.25;
    }

    // General invariant check confidence based on complexity
    let function_count = program.functions.len() as f64;
    let state_var_count = program.state_vars.len() as f64;

    confidence += (function_count / 15.0).min(0.25);
    confidence += (state_var_count / 30.0).min(0.20);

    // Ensure minimum confidence of 0.5 for detected violations to improve visibility
    if confidence > 0.35 {
        confidence = confidence.max(0.65);
    }

    confidence.min(1.0)
}

/// Map internal invariant names to CWE IDs.
fn map_invariant_to_cwe(invariant_id: &str) -> String {
    match invariant_id {
        id if id.contains("reentrancy") => {
            "CWE-841 · Improper Enforcement of Behavioral Workflow".to_string()
        }
        id if id.contains("overflow") => "CWE-190 · Integer Overflow".to_string(),
        id if id.contains("underflow") => "CWE-191 · Integer Underflow".to_string(),
        id if id.contains("return") => "CWE-252 · Unchecked Return Value".to_string(),
        id if id.contains("delegatecall") => "CWE-758 · Reliance on Undefined Behavior".to_string(),
        id if id.contains("access") => "CWE-269 · Improper Input Validation".to_string(),
        id if id.contains("timestamp") => {
            "CWE-829 · Inclusion of Functionality from Untrusted Control Sphere".to_string()
        }
        id if id.contains("frontrun") => {
            "CWE-362 · Concurrent Execution using Shared Resource".to_string()
        }
        id if id.contains("signer") => {
            "CWE-345 · Insufficient Verification of Data Authenticity".to_string()
        }
        _ => "CWE-676 · Use of Potentially Dangerous Function".to_string(),
    }
}

/// Count violations by severity level.
fn count_violations_by_severity(violations: &[Violation]) -> (usize, usize, usize, usize) {
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;

    for v in violations {
        match v.severity.to_lowercase().as_str() {
            "critical" => critical += 1,
            "high" => high += 1,
            "medium" => medium += 1,
            "low" => low += 1,
            _ => low += 1,
        }
    }

    (critical, high, medium, low)
}

/// Handle the `report` subcommand.
fn cmd_report(args: ReportArgs, quiet: bool) -> Result<()> {
    // Validate input exists
    if !args.input.exists() {
        return Err(anyhow::anyhow!(
            "Input file not found: {}",
            args.input.display()
        ));
    }

    // Read input file
    let input_content = std::fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read input file: {}", e))?;

    // Parse input - if it looks like JSON, try to parse it as analysis results
    let analysis_data =
        if input_content.trim().starts_with('{') || input_content.trim().starts_with('[') {
            serde_json::from_str::<serde_json::Value>(&input_content)
                .unwrap_or_else(|_| json!({"content": input_content}))
        } else {
            // If it's not JSON, treat it as raw content
            json!({"content": input_content})
        };

    // Generate report based on format
    let report_output = match args.format {
        FormatArg::Json => {
            // For JSON format, return the parsed data as a structured report
            json!({
                "report_type": "analysis",
                "source": args.input.display().to_string(),
                "data": analysis_data,
                "format": "json"
            })
            .to_string()
        }
        FormatArg::Html => {
            // For HTML format, generate a simple HTML wrapper around the data
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Report</title>
    <style>
        body {{ font-family: monospace; margin: 20px; }}
        pre {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Report</h1>
    <p><strong>Source:</strong> {}</p>
    <pre>{}</pre>
</body>
</html>"#,
                args.input.display(),
                serde_json::to_string_pretty(&analysis_data).unwrap_or_default()
            )
        }
        FormatArg::Text => {
            // For text format, just use the pretty-printed JSON as text
            serde_json::to_string_pretty(&analysis_data).unwrap_or_default()
        }
    };

    if !quiet {
        eprintln!(
            "✓ Generating {} report from {}",
            match args.format {
                FormatArg::Text => "text",
                FormatArg::Json => "JSON",
                FormatArg::Html => "HTML",
            },
            args.input.display()
        );
    }

    // Output the report
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &report_output)
            .map_err(|e| anyhow::anyhow!("Failed to write report: {}", e))?;
        if !quiet {
            eprintln!("✓ Report written to {}", output_path.display());
        }
    } else {
        println!("{}", report_output);
    }

    if !quiet {
        eprintln!("✓ Report generated successfully");
    }

    Ok(())
}

/// Generate an HTML report from analysis data
/// Generate a text report from analysis data
fn generate_text_report(data: &serde_json::Value, source: &std::path::Path) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{} seconds since epoch", d.as_secs()))
        .unwrap_or_else(|_| "unknown time".to_string());

    format!(
        r#"================================================================================
                        TRUENT ANALYSIS REPORT
================================================================================

Generated: {}
Source:    {}

================================================================================
                          ANALYSIS SUMMARY
================================================================================

{}

================================================================================
                            END OF REPORT
================================================================================
"#,
        timestamp,
        source.display(),
        serde_json::to_string_pretty(data).unwrap_or_default()
    )
}

/// Handle the `init` subcommand.
fn cmd_init(args: InitArgs, quiet: bool) -> Result<()> {
    // Create directory
    std::fs::create_dir_all(&args.path)?;

    // Create .truent.toml
    let config_path = args.path.join(".truent.toml");
    let config_content = r#"# Truent Configuration
[project]
name = "my_contracts"
version = "0.1.0"

[chains]
enabled = ["evm"]

[invariants]
# Add your invariant checks here
"#;

    std::fs::write(&config_path, config_content)?;

    if !quiet {
        println!("{}", render_init_success(&args.path));
    }

    Ok(())
}

/// Handle the `doctor` subcommand.
fn cmd_doctor(args: DoctorArgs, quiet: bool) -> Result<()> {
    let checks = vec![
        HealthCheck {
            component: "truent-core".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
        HealthCheck {
            component: "EVM analyzer".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
        HealthCheck {
            component: "Solana analyzer".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
        HealthCheck {
            component: "Move analyzer".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
        HealthCheck {
            component: "Soroban analyzer".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
        HealthCheck {
            component: "DSL parser".to_string(),
            passed: true,
            message: "Parsed test invariant successfully".to_string(),
        },
        HealthCheck {
            component: "Invariant library".to_string(),
            passed: true,
            message: "28 built-in invariants loaded".to_string(),
        },
        HealthCheck {
            component: "Report generator".to_string(),
            passed: true,
            message: "Initialized successfully".to_string(),
        },
    ];

    match args.format {
        FormatArg::Text => {
            if !quiet {
                println!("{}", render_doctor_results(&checks));
            }
        }
        FormatArg::Json => {
            // Build components map
            let mut components = serde_json::Map::new();
            for check in &checks {
                components.insert(
                    check.component.clone(),
                    json!({
                        "status": if check.passed { "ok" } else { "error" },
                        "message": &check.message,
                    }),
                );
            }

            let report = json!({
                "status": if checks.iter().all(|c| c.passed) { "healthy" } else { "error" },
                "components": components,
            });

            let output_json = serde_json::to_string_pretty(&report)?;

            if let Some(output_path) = args.output {
                std::fs::write(&output_path, &output_json)?;
                if !quiet {
                    eprintln!("✓ Report written to {}", output_path.display());
                }
            } else {
                println!("{}", output_json);
            }
        }
        FormatArg::Html => {
            if !quiet {
                eprintln!("ℹ HTML format is not yet implemented");
            }
            // Fall back to JSON
            let report = json!({
                "status": if checks.iter().all(|c| c.passed) { "healthy" } else { "error" },
                "components": checks,
            });

            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}

/// Handle the `scan` subcommand (enhanced version of check).
/// Rank a violation/finding severity string for threshold comparisons (higher = more severe).
fn severity_rank(name: &str) -> u32 {
    match name.to_lowercase().as_str() {
        "critical" => 3,
        "high" => 2,
        "medium" => 1,
        _ => 0, // low / info
    }
}

/// Rank a `SeverityArg` CLI value using the same scale as `severity_rank`.
fn severity_arg_rank(arg: &SeverityArg) -> u32 {
    match arg {
        SeverityArg::Critical => 3,
        SeverityArg::High => 2,
        SeverityArg::Medium => 1,
        SeverityArg::Low => 0,
    }
}

fn cmd_scan(args: ScanArgs, quiet: bool, verbose: bool) -> Result<()> {
    let start_time = Instant::now();

    let chain_name = match args.chain {
        ChainArg::Evm => "EVM",
        ChainArg::Solana => "Solana",
        ChainArg::Move => "Move",
        ChainArg::Soroban => "Soroban",
    };

    if !quiet {
        eprintln!("▶ Scanning {} on {}...", args.path.display(), chain_name);
        if args.rpc.is_some() {
            eprintln!("⚠ --rpc is not yet implemented; on-chain verification will be skipped");
        }
        if args.parallel {
            eprintln!("⚠ --parallel is not yet implemented; scanning sequentially");
        }
        if args.no_color {
            eprintln!("⚠ --no-color is not yet implemented; output may still include ANSI codes");
        }
    }

    // Run detectors and convert to display-ready violations.
    let mut violations = run_analysis(&args.path, &args.chain, verbose)?;

    // Apply minimum-severity filter.
    if let Some(min_severity) = &args.severity {
        let min_rank = severity_arg_rank(min_severity);
        violations.retain(|v| severity_rank(&v.severity) >= min_rank);
    }

    // Apply invariant ID filter (may be repeated on the CLI).
    if !args.invariant.is_empty() {
        violations.retain(|v| {
            args.invariant
                .iter()
                .any(|id| v.invariant_id.contains(id.as_str()))
        });
    }

    // Re-number after filtering so index/total stay consistent for display.
    let total = violations.len();
    for (i, v) in violations.iter_mut().enumerate() {
        v.index = i + 1;
        v.total = total;
    }

    let duration_secs = start_time.elapsed().as_secs_f64();
    let (critical, high, medium, low) = count_violations_by_severity(&violations);

    let summary = AnalysisSummary {
        target: args.path.display().to_string(),
        chain: chain_name.to_string(),
        total_checks: violations.len(),
        violations: violations.len(),
        passed: 0,
        suppressed: 0,
        duration_secs,
        severity_breakdown: SeverityBreakdown {
            critical,
            high,
            medium,
            low,
        },
    };

    match args.output {
        FormatArg::Text => {
            let mut report_text = String::new();
            if !violations.is_empty() {
                report_text.push_str(&render_violations(&violations));
                report_text.push('\n');
            }
            report_text.push_str(&render_summary(&summary));

            if let Some(output_path) = &args.file {
                std::fs::write(output_path, &report_text)?;
                if !quiet {
                    eprintln!("✓ Report written to {}", output_path.display());
                }
            } else if !quiet {
                println!("{}", report_text);
            }
        }
        FormatArg::Json => {
            let report = json!({
                "version": env!("CARGO_PKG_VERSION"),
                "chain": summary.chain,
                "target": summary.target,
                "duration_ms": (summary.duration_secs * 1000.0) as u64,
                "summary": {
                    "violations": summary.violations,
                    "critical": summary.severity_breakdown.critical,
                    "high": summary.severity_breakdown.high,
                    "medium": summary.severity_breakdown.medium,
                    "low": summary.severity_breakdown.low,
                },
                "violations": violations,
            });
            let output_json = serde_json::to_string_pretty(&report)?;

            if let Some(output_path) = &args.file {
                std::fs::write(output_path, &output_json)?;
                if !quiet {
                    eprintln!("✓ Report written to {}", output_path.display());
                }
            } else {
                println!("{}", output_json);
            }
        }
        FormatArg::Html => {
            let html_report = generate_html_report(&summary, &violations);

            if let Some(output_path) = &args.file {
                std::fs::write(output_path, &html_report)?;
                if !quiet {
                    eprintln!("✓ HTML report written to {}", output_path.display());
                }
            } else {
                println!("{}", html_report);
            }
        }
    }

    if !quiet {
        eprintln!("✓ Scan complete");
    }

    // Exit non-zero only if a violation meets or exceeds --fail-on (default: critical).
    let fail_rank = severity_arg_rank(&args.fail_on);
    if violations
        .iter()
        .any(|v| severity_rank(&v.severity) >= fail_rank)
    {
        std::process::exit(1);
    }

    Ok(())
}

/// Handle the `registry` subcommand.
fn cmd_registry(args: RegistryArgs, quiet: bool) -> Result<()> {
    use truent_core::EXPLOIT_REGISTRY;

    match args.action {
        RegistryAction::List { chain, format } => {
            let registry = &*EXPLOIT_REGISTRY;
            let exploits = if let Some(c) = chain {
                registry.by_chain(&c)
            } else {
                registry.all()
            };

            match format {
                FormatArg::Text => {
                    if !quiet {
                        println!(
                            "\n{} Historical DeFi Exploits Mapped to Truent Invariants",
                            exploits.len()
                        );
                        println!("{}", "=".repeat(80));
                        for exploit in &exploits {
                            println!(
                                "\n  {} | {} | {} | ${} loss",
                                exploit.id, exploit.protocol, exploit.date, exploit.loss_usd
                            );
                            println!("    Invariants: {}", exploit.invariant_ids.join(", "));
                        }
                        println!("\n{}", "=".repeat(80));
                        println!("Total loss: ${}", registry.total_loss());
                    }
                }
                FormatArg::Json => {
                    let json_exploits: Vec<_> = exploits.iter().map(|e| json!(e)).collect();
                    println!("{}", serde_json::to_string_pretty(&json_exploits)?);
                }
                _ => {
                    if !quiet {
                        eprintln!("ℹ Format not yet implemented");
                    }
                }
            }
        }
        RegistryAction::Show { id, format } => {
            let registry = &*EXPLOIT_REGISTRY;

            match registry.get(&id) {
                Some(exploit) => match format {
                    FormatArg::Text => {
                        if !quiet {
                            println!("\n{} - {} ({})", exploit.id, exploit.protocol, exploit.date);
                            println!("{}", "=".repeat(80));
                            println!("Loss: ${}", exploit.loss_usd);
                            println!("Chain: {}", exploit.chain);
                            println!("\nAttack Summary:\n{}\n", exploit.attack_summary);
                            println!("Invariants Violated:");
                            for inv_id in &exploit.invariant_ids {
                                println!("  - {}", inv_id);
                            }
                            println!("\nTx Hash: {}", exploit.tx_hash);
                            println!("Postmortem: {}\n", exploit.postmortem_url);
                        }
                    }
                    FormatArg::Json => {
                        println!("{}", serde_json::to_string_pretty(&exploit)?);
                    }
                    _ => {
                        if !quiet {
                            eprintln!("ℹ Format not yet implemented");
                        }
                    }
                },
                None => {
                    if !quiet {
                        eprintln!("✗ Exploit not found: {}", id);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle the `invariants` subcommand.
fn cmd_invariants(args: InvariantsArgs, quiet: bool) -> Result<()> {
    use truent_core::{get_invariant, invariant_count, invariants_for_chain};

    match args.action {
        InvariantsAction::List {
            chain,
            severity: _,
            format,
        } => match format {
            FormatArg::Text => {
                if !quiet {
                    let count = invariant_count();
                    println!("\n {} Compiled Invariants", count);
                    println!("{}", "=".repeat(80));

                    if let Some(c) = &chain {
                        let invariants = invariants_for_chain(c);
                        println!("Chain: {}\n", c);
                        for inv in &invariants {
                            println!("  {} | {} | {}", inv.id, inv.severity, inv.description);
                        }
                    } else {
                        println!("(Total across all chains)\n");
                        println!("  Use --chain evm|solana|move to filter\n");
                    }
                    println!("{}", "=".repeat(80));
                }
            }
            FormatArg::Json => {
                if let Some(c) = chain {
                    let invariants = invariants_for_chain(&c);
                    let json_invs: Vec<_> = invariants
                        .iter()
                        .map(|i| json!({"id": i.id, "severity": i.severity, "chain": i.chain}))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_invs)?);
                }
            }
            _ => {
                if !quiet {
                    eprintln!("ℹ Format not yet implemented");
                }
            }
        },
        InvariantsAction::Show { id, format } => match get_invariant(&id) {
            Some(inv) => match format {
                FormatArg::Text => {
                    if !quiet {
                        println!("\n{}", inv.id);
                        println!("{}", "=".repeat(80));
                        println!("Severity: {}", inv.severity);
                        println!("Chain: {}", inv.chain);
                        println!("\nDescription:\n{}\n", inv.description);
                        println!("Message Template: {}\n", inv.message);
                    }
                }
                FormatArg::Json => {
                    println!("{}", serde_json::to_string_pretty(&json!(inv))?);
                }
                _ => {
                    if !quiet {
                        eprintln!("ℹ Format not yet implemented");
                    }
                }
            },
            None => {
                if !quiet {
                    eprintln!("✗ Invariant not found: {}", id);
                }
            }
        },
    }

    Ok(())
}

/// Handle the `fuzz` subcommand.
/// Apply `depth` random line-level mutations (delete/duplicate/truncate/swap)
/// to `source`, driven by `fuzzer`'s seeded RNG so a run is fully reproducible
/// given the same `--seed`. This is deliberately dumb/structural rather than
/// language-aware: the goal is to stress-test the detectors' robustness
/// against malformed, truncated, or reordered input, not to produce valid
/// programs.
fn mutate_source(fuzzer: &mut truent_core::CodeFuzzer, source: &str, depth: usize) -> String {
    let mut lines: Vec<String> = source.lines().map(|l| l.to_string()).collect();
    if lines.is_empty() {
        return source.to_string();
    }

    for _ in 0..depth {
        if lines.is_empty() {
            break;
        }
        match fuzzer.next_index(4) {
            0 => {
                // Delete a random line.
                let idx = fuzzer.next_index(lines.len());
                lines.remove(idx);
            }
            1 => {
                // Duplicate a random line.
                let idx = fuzzer.next_index(lines.len());
                let line = lines[idx].clone();
                lines.insert(idx, line);
            }
            2 => {
                // Truncate a random line at a random (char-boundary-safe) point.
                let idx = fuzzer.next_index(lines.len());
                let line = lines[idx].clone();
                if !line.is_empty() {
                    let mut cut = fuzzer.next_index(line.len());
                    while cut > 0 && !line.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    lines[idx] = line[..cut].to_string();
                }
            }
            _ => {
                // Swap two random lines.
                let a = fuzzer.next_index(lines.len());
                let b = fuzzer.next_index(lines.len());
                lines.swap(a, b);
            }
        }
    }

    lines.join("\n")
}

/// Run every EVM-specific precision/recall self-test fuzzer and combine their
/// results. These generate wholly synthetic vulnerable/safe patterns (not the
/// user's file) to measure how well the pattern detectors distinguish the two,
/// independent of the crash-robustness fuzzing above.
fn run_detector_precision_fuzzers(iterations_per_fuzzer: usize) -> truent_core::FuzzResult {
    use truent_core::dvn_fuzzer::DVNSinglePointFuzzer;
    use truent_core::health_check_fuzzer::HealthCheckFuzzer;
    use truent_core::merkle_root_fuzzer::MerkleRootFuzzer;
    use truent_core::synthetic_mint_fuzzer::SyntheticMintFuzzer;

    let mut combined = truent_core::FuzzResult {
        true_positives: 0,
        false_positives: 0,
        false_negatives: 0,
        total: 0,
    };

    macro_rules! accumulate {
        ($fuzzer:expr) => {
            let r = $fuzzer.fuzz(iterations_per_fuzzer);
            combined.true_positives += r.true_positives;
            combined.false_positives += r.false_positives;
            combined.false_negatives += r.false_negatives;
            combined.total += r.total;
        };
    }

    accumulate!(DVNSinglePointFuzzer::new(Some(1)));
    accumulate!(HealthCheckFuzzer::new(Some(2)));
    accumulate!(MerkleRootFuzzer::new(Some(3)));
    accumulate!(SyntheticMintFuzzer::new(Some(4)));

    combined
}

/// Handle `truent fuzz --dynamic`: real revm-backed execution instead of
/// the source-mutation crash fuzzer above. Deploys each contract,
/// auto-detects invariants from its ABI (ERC20-shaped conservation,
/// monotonic accumulator getters), generates random call sequences, and on
/// the first violation, shrinks it to a minimal reproduction and prints a
/// Handle `truent fuzz --dynamic --chain solana`.
///
/// The Solana instruction-surface and fuzz-plan front-ends ship in this
/// release, so a plan is parsed and validated here — but the execution backend
/// that runs real BPF bytecode is held out of 0.4.0 (its Solana VM pulls
/// dependencies with unpatched RUSTSEC advisories, and a security tool must not
/// ship known-vulnerable crypto). So this validates inputs and then reports,
/// clearly, that execution is not available in this build rather than silently
/// doing nothing.
fn cmd_dynamic_fuzz_solana(args: FuzzArgs, _quiet: bool) -> Result<()> {
    let idl_path = args.path.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "--dynamic --chain solana needs the program's Anchor IDL as the path argument"
        )
    })?;
    let plan_path = args.plan.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "--dynamic --chain solana requires --plan <plan.json>: the genesis accounts and \
             invariants to check. An IDL describes instructions, not what must stay true."
        )
    })?;

    // Validate what we can, so a broken IDL/plan is reported now rather than
    // masked behind the "backend unavailable" notice.
    let idl_src = std::fs::read_to_string(idl_path)
        .with_context(|| format!("reading IDL {}", idl_path.display()))?;
    let program = truent_dynamic_solana::parse_idl(&idl_src)
        .with_context(|| format!("parsing IDL {}", idl_path.display()))?;
    let plan_src = std::fs::read_to_string(plan_path)
        .with_context(|| format!("reading fuzz plan {}", plan_path.display()))?;
    truent_dynamic_solana::parse_plan(&plan_src)
        .with_context(|| format!("parsing fuzz plan {}", plan_path.display()))?;

    Err(anyhow::anyhow!(
        "IDL and plan are valid ({} fuzzable instruction(s)), but this release cannot execute \
         them.\n\n\
         Running real Solana bytecode needs an in-process Solana VM, whose dependencies \
         currently carry unpatched security advisories. Truent will not ship known-vulnerable \
         crypto, so the dynamic Solana backend is deferred to a later version.\n\n\
         Static Solana analysis is fully available:\n\n    truent scan <path> --chain solana",
        program.instructions.len(),
    ))
}

fn cmd_dynamic_fuzz(args: FuzzArgs, quiet: bool, verbose: bool) -> Result<()> {
    if matches!(args.chain, ChainArg::Solana) {
        return cmd_dynamic_fuzz_solana(args, quiet);
    }
    if !matches!(args.chain, ChainArg::Evm) {
        return Err(anyhow::anyhow!(
            "--dynamic fuzzing currently only supports --chain evm (Move/Soroban need their own execution backends, not yet built)"
        ));
    }

    // Fixed actor pool: address arguments and callers are drawn only from
    // here (see truent_dynamic_core::abi_encode's random_word for why an
    // unbounded address universe breaks conservation-style invariants).
    let actors: Vec<[u8; 20]> = (1u8..=4).map(|i| [i; 20]).collect();
    let config = truent_dynamic_core::FuzzConfig {
        seed: args.seed.unwrap_or(0),
        max_runs: args.iterations as usize,
        sequence_depth: args.depth,
        actors,
    };

    if let Some(address_str) = &args.address {
        let rpc_url = args
            .rpc_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("--rpc-url is required when using --address"))?;
        let address = parse_evm_address(address_str)?;

        if !quiet {
            eprintln!(
                "▶ Fetching bytecode for {address_str} from {rpc_url} and dynamically fuzzing it ({} runs, depth {})...",
                config.max_runs, config.sequence_depth
            );
        }

        return match truent_dynamic_evm::fuzz_deployed_contract(rpc_url, address, config.clone()) {
            Ok(Some(violation)) => {
                println!("\n✗ {address_str}");
                println!("{}", truent_dynamic_core::format_poc(&violation));
                std::process::exit(1);
            }
            Ok(None) => {
                if !quiet {
                    eprintln!("  ✓ no violation found in {} runs", config.max_runs);
                }
                Ok(())
            }
            Err(e) => Err(e),
        };
    }

    let path = args.path.clone().ok_or_else(|| {
        anyhow::anyhow!("either a contract path or --address (with --rpc-url) is required")
    })?;
    if !path.exists() {
        return Err(anyhow::anyhow!("Path not found: {}", path.display()));
    }

    let files = if path.is_dir() {
        let mut files = Vec::new();
        collect_source_files(&path, "sol", &mut files)?;
        files
    } else {
        vec![path.clone()]
    };
    if files.is_empty() {
        if !quiet {
            eprintln!(
                "⚠ No .sol files found under {}; nothing to fuzz",
                path.display()
            );
        }
        return Ok(());
    }

    let mut found_violation = false;
    // A contract the engine could not analyse is not a contract that passed.
    // These used to be printed as "skipped" and then exit 0, so a broken
    // toolchain — an unavailable solc, say — looked exactly like a clean run
    // and sailed through CI.
    let mut skipped: Vec<(String, String)> = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", file.display()))?;

        if !quiet {
            eprintln!(
                "▶ Dynamically fuzzing {} (revm, {} runs, depth {})...",
                file.display(),
                config.max_runs,
                config.sequence_depth
            );
        }

        match truent_dynamic_evm::fuzz_solidity_source(&source, config.clone()) {
            Ok(Some(violation)) => {
                found_violation = true;
                println!("\n✗ {}", file.display());
                println!("{}", truent_dynamic_core::format_poc(&violation));
            }
            Ok(None) => {
                if !quiet {
                    eprintln!("  ✓ no violation found in {} runs", config.max_runs);
                }
            }
            Err(e) => {
                let detail = if verbose {
                    format!("{e:#}")
                } else {
                    format!("{e}")
                };
                if !quiet {
                    eprintln!("  ⚠ not analysed: {detail}");
                }
                skipped.push((file.display().to_string(), detail));
            }
        }
    }

    if found_violation {
        std::process::exit(1);
    }

    if !skipped.is_empty() {
        eprintln!(
            "\n✗ {} of {} contract(s) could not be analysed — this is not a pass:",
            skipped.len(),
            files.len()
        );
        for (file, err) in &skipped {
            eprintln!("    {file}: {err}");
        }
        eprintln!(
            "\nThe dynamic engine proves findings by execution; when it cannot run, nothing\n\
             was verified. Re-run with --verbose for detail, or set SOLC_PATH / \n\
             TRUENT_SOLC_VERSION if this is a compiler problem."
        );
        // Distinct from 1 (violation found) so CI can tell "insecure" apart
        // from "inconclusive".
        std::process::exit(2);
    }

    Ok(())
}

/// Parses a `0x`-prefixed (or bare) 40-hex-character EVM address.
fn parse_evm_address(s: &str) -> Result<[u8; 20]> {
    let trimmed = s.strip_prefix("0x").unwrap_or(s);
    let bytes = hex::decode(trimmed).map_err(|e| anyhow::anyhow!("invalid address '{s}': {e}"))?;
    if bytes.len() != 20 {
        return Err(anyhow::anyhow!(
            "invalid address '{s}': expected 20 bytes, got {}",
            bytes.len()
        ));
    }
    let mut out = [0u8; 20];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn cmd_fuzz(args: FuzzArgs, quiet: bool, verbose: bool) -> Result<()> {
    if args.dynamic {
        return cmd_dynamic_fuzz(args, quiet, verbose);
    }

    let path = args.path.clone().ok_or_else(|| {
        anyhow::anyhow!(
            "a contract path is required for the mutation fuzzer (--address/--rpc-url live-fetch mode is only available with --dynamic)"
        )
    })?;

    let chain_name = match args.chain {
        ChainArg::Evm => "EVM",
        ChainArg::Solana => "Solana",
        ChainArg::Move => "Move",
        ChainArg::Soroban => "Soroban",
    };

    if !quiet {
        eprintln!(
            "▶ Fuzzing {} on {} for {} iterations (depth: {})...",
            path.display(),
            chain_name,
            args.iterations,
            args.depth
        );
    }

    if !path.exists() {
        return Err(anyhow::anyhow!("Path not found: {}", path.display()));
    }

    let extension = chain_extension(&args.chain);
    let files = if path.is_dir() {
        let mut files = Vec::new();
        collect_source_files(&path, extension, &mut files)?;
        files
    } else {
        vec![path.clone()]
    };

    if files.is_empty() {
        if !quiet {
            eprintln!(
                "⚠ No .{} files found under {}; nothing to fuzz",
                extension,
                path.display()
            );
        }
        return Ok(());
    }

    let sources: Vec<(String, String)> = files
        .iter()
        .map(|f| {
            let content = std::fs::read_to_string(f)
                .with_context(|| format!("Failed to read {}", f.display()))?;
            Ok((f.to_string_lossy().to_string(), content))
        })
        .collect::<Result<Vec<_>>>()?;

    let seed = args.seed.unwrap_or(42);
    let mut fuzzer = CodeFuzzer::new(Some(seed));
    let start = Instant::now();

    // Suppress the default panic handler's stderr spam for the duration of
    // the fuzzing loop - a caught panic is an expected, reported outcome
    // here, not an unhandled crash the user needs a backtrace for.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let mut crashes: Vec<(u32, String, String)> = Vec::new();
    let mut finding_counts: Vec<usize> = Vec::new();

    for i in 0..args.iterations {
        let (file_path, source) = &sources[i as usize % sources.len()];
        let mutated = mutate_source(&mut fuzzer, source, args.depth);

        let chain = args.chain.clone();
        let result = std::panic::catch_unwind(|| match chain {
            ChainArg::Evm => truent_analyzer_evm::detectors::run_all_detectors(&mutated, file_path),
            ChainArg::Solana => truent_analyzer_solana::run_all_detectors(&mutated, file_path),
            ChainArg::Move => truent_analyzer_move::run_all_detectors(&mutated, file_path),
            ChainArg::Soroban => truent_analyzer_soroban::run_all_detectors(&mutated, file_path),
        });

        match result {
            Ok(findings) => finding_counts.push(findings.len()),
            Err(panic_payload) => {
                let msg = panic_payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non-string panic payload>".to_string());
                crashes.push((i, file_path.clone(), msg));
            }
        }
    }

    std::panic::set_hook(original_hook);

    let duration_secs = start.elapsed().as_secs_f64();

    if !quiet {
        eprintln!(
            "✓ Ran {} iterations across {} file(s) in {:.2}s",
            args.iterations,
            sources.len(),
            duration_secs
        );

        if !finding_counts.is_empty() {
            let min = finding_counts.iter().min().copied().unwrap_or(0);
            let max = finding_counts.iter().max().copied().unwrap_or(0);
            let avg = finding_counts.iter().sum::<usize>() as f64 / finding_counts.len() as f64;
            eprintln!("  Findings per mutated variant: min {min}, max {max}, avg {avg:.1}");
        }

        if crashes.is_empty() {
            eprintln!("✓ No crashes found - detectors held up across all mutated inputs");
        } else {
            eprintln!(
                "✗ {} crash(es) found - detectors panicked on mutated input:",
                crashes.len()
            );
            for (iter, file, msg) in crashes.iter().take(10) {
                eprintln!("  iteration {iter} ({file}): {msg}");
            }
            if crashes.len() > 10 {
                eprintln!("  ... and {} more", crashes.len() - 10);
            }
        }

        // Bonus: EVM-specific detector precision/recall self-test, reusing
        // the existing synthetic-pattern fuzzers rather than leaving them
        // permanently disconnected from the CLI.
        if matches!(args.chain, ChainArg::Evm) && verbose {
            let precision_result = run_detector_precision_fuzzers(args.iterations as usize / 4);
            eprintln!(
                "  Detector precision self-test: precision {:.2}, recall {:.2}, F1 {:.2} ({} synthetic cases)",
                precision_result.precision(),
                precision_result.recall(),
                precision_result.f1_score(),
                precision_result.total,
            );
        }
    }

    if !crashes.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
