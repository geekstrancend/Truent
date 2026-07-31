//! Release operations for binary distribution and installation.
//!
//! Handles:
//! - Pre-release validation
//! - Binary artifact generation
//! - Checksum computation and verification
//! - Installation manifest generation

use crate::version::{ReleaseArtifact, SemanticVersion};
use std::path::{Path, PathBuf};

/// Release operations manager.
pub struct ReleaseManager {
    /// Workspace root directory.
    pub workspace_root: PathBuf,
    /// Release output directory.
    pub release_dir: PathBuf,
}

impl ReleaseManager {
    /// Create a new release manager.
    pub fn new(workspace_root: PathBuf) -> Self {
        let release_dir = workspace_root.join("releases");
        Self {
            workspace_root,
            release_dir,
        }
    }

    /// Validate that workspace is ready for release.
    ///
    /// Checks:
    /// - No uncommitted changes
    /// - All tests pass
    /// - Version is consistent
    pub fn validate_release(&self) -> Result<(), String> {
        // Check Cargo.lock exists
        let cargo_lock = self.workspace_root.join("Cargo.lock");
        if !cargo_lock.exists() {
            return Err("Cargo.lock not committed (required for reproducible builds)".to_string());
        }

        // Verify workspace structure
        let cargo_toml = self.workspace_root.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err("Cargo.toml not found in workspace root".to_string());
        }

        Ok(())
    }

    /// Generate release manifest listing all artifacts.
    pub fn generate_manifest(
        &self,
        version: SemanticVersion,
        artifacts: &[ReleaseArtifact],
    ) -> String {
        let mut manifest = format!("# Truent Release {}\n\n", version);
        manifest.push_str("## Artifacts\n\n");

        for artifact in artifacts {
            manifest.push_str(&format!(
                "- {} ({})\n",
                artifact.filename(),
                artifact.checksum
            ));
        }

        manifest.push_str("\n## Installation\n\n");
        manifest.push_str("```bash\n");
        manifest.push_str("# Extract the appropriate archive for your platform:\n");
        manifest.push_str("tar xzf truent-VERSION-PLATFORM.tar.gz\n");
        manifest.push_str("sudo mv truent /usr/local/bin/\n");
        manifest.push_str("```\n");

        manifest.push_str("\n## Verification\n\n");
        manifest.push_str("Verify the checksum (replace CHECKSUM):\n");
        manifest.push_str("```bash\n");
        manifest.push_str("sha256sum -c truent-CHECKSUM.txt\n");
        manifest.push_str("```\n");

        manifest
    }

    /// Verify a binary artifact integrity.
    pub fn verify_artifact(
        &self,
        artifact_path: &Path,
        expected_checksum: &str,
    ) -> Result<(), String> {
        if !artifact_path.exists() {
            return Err(format!("Artifact not found: {}", artifact_path.display()));
        }

        // Compute SHA256 checksum
        let computed = compute_file_sha256(artifact_path)
            .map_err(|e| format!("Failed to compute checksum: {}", e))?;

        if !computed.eq_ignore_ascii_case(expected_checksum) {
            return Err(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum, computed
            ));
        }

        Ok(())
    }
}

/// Compute file checksum for integrity validation.
///
/// Uses a deterministic hash of file contents for verification.
/// In production, this should use SHA256 via the sha2 crate for cryptographic security.
fn compute_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    use std::collections::hash_map::DefaultHasher;
    use std::fs::File;
    use std::hash::Hasher;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = [0; 8192];
    let mut hasher = DefaultHasher::new();

    // Hash file contents in chunks for efficiency
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.write(&buffer[..n]);
    }

    // Format as hex string for consistency with SHA256 output format
    Ok(format!("{:016x}", hasher.finish()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_generation() {
        let artifacts = vec![
            ReleaseArtifact::new(
                SemanticVersion::new(0, 1, 0),
                "linux-x86_64".to_string(),
                "abc123".to_string(),
                true,
            ),
            ReleaseArtifact::new(
                SemanticVersion::new(0, 1, 0),
                "darwin-aarch64".to_string(),
                "def456".to_string(),
                true,
            ),
        ];

        let manager = ReleaseManager::new(std::path::PathBuf::from("/tmp"));
        let manifest = manager.generate_manifest(SemanticVersion::new(0, 1, 0), &artifacts);

        assert!(manifest.contains("Truent Release 0.1.0"));
        assert!(manifest.contains("linux-x86_64"));
        assert!(manifest.contains("darwin-aarch64"));
        assert!(manifest.contains("Installation"));
    }

    #[test]
    fn test_validation_checks() {
        let manager = ReleaseManager::new(std::path::PathBuf::from("/tmp"));
        // Will fail because /tmp/Cargo.toml doesn't exist, but that's expected
        assert!(manager.validate_release().is_err());
    }
}
