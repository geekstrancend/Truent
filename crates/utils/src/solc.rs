//! Solidity compiler (solc) management and execution.
//!
//! This module handles finding, downloading, and running the solc compiler
//! to generate JSON AST output for Solidity smart contracts.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Minimum supported solc version
pub const MIN_SOLC_VERSION: &str = "0.6.0";

/// Represents the solc compiler manager
#[derive(Debug, Clone)]
pub struct SolcManager {
    solc_path: PathBuf,
    version: String,
}

/// Solc combined JSON output
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolcOutput {
    /// Contract data keyed by fully qualified name
    pub contracts: std::collections::HashMap<String, serde_json::Value>,
    /// Source files with AST
    pub sources: std::collections::HashMap<String, SourceData>,
}

/// Source file data with AST
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceData {
    #[serde(rename = "AST")]
    /// Raw AST JSON value from solc output
    pub ast: serde_json::Value,
}

impl SolcManager {
    /// Find or download a compatible solc binary
    ///
    /// Search order:
    /// 1. SOLC_PATH environment variable
    /// 2. solc on system PATH
    /// 3. ~/.truent/solc/solc (cached download)
    /// 4. Download latest stable from binaries.soliditylang.org
    pub fn new() -> Result<Self> {
        // Try SOLC_PATH environment variable
        if let Ok(path) = std::env::var("SOLC_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                let version = Self::get_version(&path)?;
                info!(
                    "Using solc from SOLC_PATH: {} (v{})",
                    path.display(),
                    version
                );
                return Ok(Self {
                    solc_path: path,
                    version,
                });
            }
        }

        // Try solc on system PATH
        if let Ok(output) = Command::new("solc").arg("--version").output() {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                let version = Self::parse_version(&version_str)?;
                info!("Using system solc (v{})", version);
                return Ok(Self {
                    solc_path: PathBuf::from("solc"),
                    version,
                });
            }
        }

        // Try cached download
        let cache_path = Self::cache_path();
        if cache_path.exists() {
            let version = Self::get_version(&cache_path)?;
            info!(
                "Using cached solc from {} (v{})",
                cache_path.display(),
                version
            );
            return Ok(Self {
                solc_path: cache_path,
                version,
            });
        }

        // Download latest stable
        info!("solc not found locally, downloading...");
        Self::download_solc()
    }

    /// Get the full AST JSON output for a Solidity file
    pub fn get_ast_json(&self, source_path: &Path) -> Result<SolcOutput> {
        let output = Command::new(&self.solc_path)
            .args([
                "--combined-json",
                "ast,abi,bin,bin-runtime,srcmap,srcmap-runtime",
                "--allow-paths",
                ".",
            ])
            .arg(source_path)
            .output()
            .with_context(|| format!("Failed to run solc on: {}", source_path.display()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Only fail on fatal errors, not warnings
            if Self::has_fatal_errors(&stderr) {
                bail!("solc failed on {}: {}", source_path.display(), stderr);
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout).with_context(|| {
            format!(
                "Failed to parse solc JSON output for: {}",
                source_path.display()
            )
        })
    }

    /// Get AST for source code string (writes to temp file)
    pub fn get_ast_for_source(&self, source: &str, _filename: &str) -> Result<SolcOutput> {
        let tmp = tempfile::Builder::new().suffix(".sol").tempfile()?;
        std::fs::write(tmp.path(), source)?;
        self.get_ast_json(tmp.path())
    }

    fn cache_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".truent")
            .join("solc")
            .join("solc")
    }

    fn get_version(path: &Path) -> Result<String> {
        let output = Command::new(path).arg("--version").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_version(&stdout)
    }

    fn parse_version(output: &str) -> Result<String> {
        // Extract version from "solc, the solidity compiler programm
        // Version: 0.8.21+commit.d9974bed"
        output
            .lines()
            .find(|l| l.contains("Version:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .map(|v| v.split('+').next().unwrap_or(v).to_string())
            .ok_or_else(|| anyhow!("Could not parse solc version"))
    }

    fn has_fatal_errors(stderr: &str) -> bool {
        stderr
            .lines()
            .any(|l| l.to_lowercase().contains("error:") && !l.contains("Warning:"))
    }

    fn download_solc() -> Result<Self> {
        let cache_path = Self::cache_path();
        std::fs::create_dir_all(cache_path.parent().unwrap())?;

        let platform = if cfg!(target_os = "macos") {
            "macosx-amd64"
        } else if cfg!(target_os = "windows") {
            "windows-amd64"
        } else {
            "linux-amd64"
        };

        let version = "0.8.21";
        let url =
            format!("https://binaries.soliditylang.org/{platform}/solc-{platform}-v{version}");

        info!("Downloading solc v{} for {}...", version, platform);

        let response = ureq::get(&url)
            .call()
            .map_err(|e| anyhow!("Failed to download solc from {}: {}", url, e))?;

        let mut file = std::fs::File::create(&cache_path)?;
        std::io::copy(&mut response.into_reader(), &mut file)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&cache_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&cache_path, perms)?;
        }

        info!("Downloaded solc to {}", cache_path.display());

        Ok(Self {
            solc_path: cache_path,
            version: version.to_string(),
        })
    }

    /// Get the version of the managed solc binary
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get the path to the solc binary
    pub fn path(&self) -> &Path {
        &self.solc_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let output = "solc, the solidity compiler programm\nVersion: 0.8.21+commit.d9974bed";
        let version = SolcManager::parse_version(output).unwrap();
        assert_eq!(version, "0.8.21");
    }

    #[test]
    fn test_has_fatal_errors() {
        assert!(SolcManager::has_fatal_errors("Error: Something went wrong"));
        assert!(!SolcManager::has_fatal_errors("Warning: Be careful"));
        assert!(SolcManager::has_fatal_errors(
            "Warning: Be careful\nError: Failed"
        ));
    }
}
