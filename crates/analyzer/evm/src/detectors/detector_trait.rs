//! Detector trait for unified vulnerability detection.
//!
//! All EVM detectors implement this trait to provide a consistent interface
//! for discovering invariant violations.

use truent_core::Finding;

/// Trait for EVM vulnerability detectors.
pub trait Detector {
    /// Run detection analysis on source code.
    ///
    /// # Arguments
    /// * `source` - The source code to analyze
    /// * `file_path` - Path to the source file (for reporting)
    ///
    /// # Returns
    /// Vector of findings, one per violation detected
    fn detect(&self, source: &str, file_path: &str) -> Vec<Finding>;
}
