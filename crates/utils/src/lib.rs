#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Utilities for Truent: logging, path handling, version management, and Solidity compiler integration.

pub mod logging;
pub mod path_utils;
pub mod release;
pub mod solc;
pub mod version;

pub use logging::setup_tracing;
pub use release::ReleaseManager;
pub use solc::{SolcManager, SolcOutput, SourceData};
pub use version::{Platform, ReleaseArtifact, ReproducibleBuildConfig, SemanticVersion};
