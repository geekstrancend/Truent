//! Error types for Truent core operations.

use thiserror::Error;

/// The result type for Truent core operations.
pub type Result<T> = std::result::Result<T, InvarError>;

/// Errors that can occur during invariant analysis and generation.
#[derive(Error, Debug)]
pub enum InvarError {
    /// IO error occurred during file operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid invariant syntax or structure.
    #[error("Invalid invariant: {0}")]
    InvalidInvariant(String),

    /// Undefined identifier in invariant expression.
    #[error("Undefined identifier: {0}")]
    UndefinedIdentifier(String),

    /// Type mismatch in expression.
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    /// Unsupported chain or pattern.
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Analysis failed with details.
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    /// Generation failed with details.
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// Simulation failed with details.
    #[error("Simulation failed: {0}")]
    SimulationFailed(String),

    /// Configuration or parsing error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Custom error message.
    #[error("{0}")]
    Custom(String),
}

impl InvarError {
    /// Create a custom error with a message.
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        Self::Custom(msg.into())
    }

    /// Create an invalid invariant error.
    pub fn invalid_invariant<S: Into<String>>(msg: S) -> Self {
        Self::InvalidInvariant(msg.into())
    }

    /// Create an undefined identifier error.
    pub fn undefined_identifier<S: Into<String>>(name: S) -> Self {
        Self::UndefinedIdentifier(name.into())
    }

    /// Create a type mismatch error.
    pub fn type_mismatch<S: Into<String>>(msg: S) -> Self {
        Self::TypeMismatch(msg.into())
    }

    /// Create an unsupported pattern error.
    pub fn unsupported<S: Into<String>>(msg: S) -> Self {
        Self::Unsupported(msg.into())
    }
}
