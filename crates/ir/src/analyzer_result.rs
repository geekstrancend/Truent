//! Analysis context tracking.

use truent_core::model::ProgramModel;

/// Context information from analysis phase.
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// The analyzed program model.
    pub program: ProgramModel,

    /// Whether analysis completed successfully.
    pub is_valid: bool,

    /// Any warnings encountered during analysis.
    pub warnings: Vec<String>,
}

impl AnalysisContext {
    /// Create a new analysis context.
    pub fn new(program: ProgramModel) -> Self {
        Self {
            program,
            is_valid: true,
            warnings: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Mark analysis as invalid.
    pub fn mark_invalid(&mut self) {
        self.is_valid = false;
    }
}
