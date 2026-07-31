//! Semantic versioning and release management for Truent.
//!
//! Provides:
//! - Semantic versioning with validation
//! - Release artifact generation
//! - Reproducible build metadata
//! - Security checksums (SHA256)

use std::fmt;

/// Semantic version following SemVer 2.0.0.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct SemanticVersion {
    /// Major version (breaking changes).
    pub major: u32,
    /// Minor version (feature additions).
    pub minor: u32,
    /// Patch version (bug fixes).
    pub patch: u32,
}

impl SemanticVersion {
    /// Create a new semantic version.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse semantic version from string (e.g., "0.1.0").
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return Err("Invalid version format, expected MAJOR.MINOR.PATCH".to_string());
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| "Major version must be a number")?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| "Minor version must be a number")?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| "Patch version must be a number")?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    /// Check if this version is compatible with a minimum required version.
    pub fn is_compatible_with(&self, minimum: SemanticVersion) -> bool {
        if self.major != minimum.major {
            return self.major > minimum.major;
        }
        if self.minor != minimum.minor {
            return self.minor > minimum.minor;
        }
        self.patch >= minimum.patch
    }

    /// Increment major version (reset minor and patch).
    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    /// Increment minor version (reset patch).
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Increment patch version.
    pub fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Release artifact metadata.
#[derive(Debug, Clone)]
pub struct ReleaseArtifact {
    /// Semantic version of the release.
    pub version: SemanticVersion,
    /// Target platform (e.g., "linux-x86_64", "darwin-aarch64", "windows-x86_64").
    pub target: String,
    /// SHA256 checksum of the binary.
    pub checksum: String,
    /// Whether this is a reproducible build.
    pub reproducible: bool,
}

impl ReleaseArtifact {
    /// Create a new release artifact.
    pub fn new(
        version: SemanticVersion,
        target: String,
        checksum: String,
        reproducible: bool,
    ) -> Self {
        Self {
            version,
            target,
            checksum,
            reproducible,
        }
    }

    /// Compute expected artifact filename.
    pub fn filename(&self) -> String {
        format!("truent-{}-{}", self.version, self.target)
    }

    /// Verify that artifact checksum matches expected value.
    pub fn verify_checksum(&self, actual_checksum: &str) -> bool {
        self.checksum.eq_ignore_ascii_case(actual_checksum)
    }
}

impl fmt::Display for ReleaseArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "truent {} ({}) [{}]",
            self.version, self.target, self.checksum
        )
    }
}

/// Reproducible build configuration.
#[derive(Debug, Clone)]
pub struct ReproducibleBuildConfig {
    /// Enable LTO (Link Time Optimization).
    pub lto: bool,
    /// Optimization level (0-3).
    pub opt_level: u32,
    /// Strip debug symbols.
    pub strip: bool,
    /// Pin Rust version.
    pub rust_version: String,
}

impl ReproducibleBuildConfig {
    /// Create default reproducible build configuration.
    pub fn default_release() -> Self {
        Self {
            lto: true,
            opt_level: 3,
            strip: false, // Keep debug symbols for crash analysis
            rust_version: "1.70.0".to_string(),
        }
    }

    /// Verify that build environment matches configuration.
    pub fn verify_environment(&self, current_rust_version: &str) -> Result<(), String> {
        if current_rust_version != self.rust_version {
            return Err(format!(
                "Rust version mismatch: expected {}, got {}",
                self.rust_version, current_rust_version
            ));
        }
        Ok(())
    }
}

/// Supported platforms for binary releases.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Platform {
    /// Linux x86_64.
    LinuxX86_64,
    /// Linux aarch64 (ARM64).
    LinuxAarch64,
    /// macOS x86_64 (Intel).
    MacOSX86_64,
    /// macOS aarch64 (Apple Silicon).
    MacOSAarch64,
    /// Windows x86_64.
    WindowsX86_64,
}

impl Platform {
    /// Get the target triple for this platform.
    pub fn target_triple(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Self::LinuxAarch64 => "aarch64-unknown-linux-gnu",
            Self::MacOSX86_64 => "x86_64-apple-darwin",
            Self::MacOSAarch64 => "aarch64-apple-darwin",
            Self::WindowsX86_64 => "x86_64-pc-windows-msvc",
        }
    }

    /// Get the artifact filename suffix for this platform.
    pub fn artifact_suffix(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "linux-x86_64",
            Self::LinuxAarch64 => "linux-aarch64",
            Self::MacOSX86_64 => "darwin-x86_64",
            Self::MacOSAarch64 => "darwin-aarch64",
            Self::WindowsX86_64 => "windows-x86_64",
        }
    }

    /// Get all supported platforms.
    pub fn all() -> &'static [Self] {
        &[
            Self::LinuxX86_64,
            Self::LinuxAarch64,
            Self::MacOSX86_64,
            Self::MacOSAarch64,
            Self::WindowsX86_64,
        ]
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.artifact_suffix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemanticVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_semver_parse_invalid() {
        assert!(SemanticVersion::parse("1.2").is_err());
        assert!(SemanticVersion::parse("1.2.a").is_err());
    }

    #[test]
    fn test_semver_display() {
        let v = SemanticVersion::new(0, 1, 0);
        assert_eq!(v.to_string(), "0.1.0");
    }

    #[test]
    fn test_semver_bump() {
        let mut v = SemanticVersion::new(0, 1, 0);
        v.bump_patch();
        assert_eq!(v, SemanticVersion::new(0, 1, 1));

        v.bump_minor();
        assert_eq!(v, SemanticVersion::new(0, 2, 0));

        v.bump_major();
        assert_eq!(v, SemanticVersion::new(1, 0, 0));
    }

    #[test]
    fn test_semver_compatibility() {
        let v1 = SemanticVersion::new(1, 2, 3);
        let v2 = SemanticVersion::new(1, 2, 0);
        let v3 = SemanticVersion::new(0, 5, 0);

        assert!(v1.is_compatible_with(v2)); // 1.2.3 >= 1.2.0
        assert!(!v2.is_compatible_with(v1)); // 1.2.0 < 1.2.3
        assert!(!v3.is_compatible_with(v1)); // 0.5.0 < 1.0.0
    }

    #[test]
    fn test_release_artifact_filename() {
        let artifact = ReleaseArtifact::new(
            SemanticVersion::new(0, 1, 0),
            "linux-x86_64".to_string(),
            "abc123".to_string(),
            true,
        );
        assert_eq!(artifact.filename(), "truent-0.1.0-linux-x86_64");
    }

    #[test]
    fn test_release_artifact_verify_checksum() {
        let artifact = ReleaseArtifact::new(
            SemanticVersion::new(0, 1, 0),
            "linux-x86_64".to_string(),
            "ABC123".to_string(),
            true,
        );
        assert!(artifact.verify_checksum("abc123")); // Case-insensitive
        assert!(!artifact.verify_checksum("xyz789"));
    }

    #[test]
    fn test_platform_target_triples() {
        assert_eq!(
            Platform::LinuxX86_64.target_triple(),
            "x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            Platform::MacOSAarch64.target_triple(),
            "aarch64-apple-darwin"
        );
        assert_eq!(
            Platform::WindowsX86_64.target_triple(),
            "x86_64-pc-windows-msvc"
        );
    }

    #[test]
    fn test_platform_all() {
        assert_eq!(Platform::all().len(), 5);
    }
}
