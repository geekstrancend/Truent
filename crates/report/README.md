# truent-report

Report generation for Truent findings.

## Usage

```toml
[dependencies]
truent-report = "0.3.0"
truent-core = "0.3.0"
```

## Generating a Report

```rust
use truent_report::{SecurityReport, ReportFormat};

let report = SecurityReport::new(
    "Vault.sol Security Audit".to_string(),
    vec!["contracts/Vault.sol".to_string()],
    findings, // Vec<truent_core::Finding>
    "Automated static analysis found 3 findings.".to_string(),
);

let json = report.generate(ReportFormat::Json);
let html = report.generate(ReportFormat::Html);
let csv = report.generate(ReportFormat::Csv);
let markdown = report.generate(ReportFormat::Markdown);
```

`SeverityStats::from_findings(&findings)` (used internally by `SecurityReport::new`)
gives the critical/high/medium/low counts if you just need the breakdown
without a full report.

## Terminal / CI formats

For direct terminal output, NDJSON, or SARIF (GitHub code-scanning format),
use the free functions in `formatter_ansi`:

```rust
use truent_report::{format_terminal, format_ndjson, format_sarif};

let colored = format_terminal(&findings, true);
let ndjson = format_ndjson(&findings);
let sarif = format_sarif(&findings, env!("CARGO_PKG_VERSION"));
```

## License

MIT
