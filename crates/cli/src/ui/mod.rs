//! UI component library for Truent CLI.
//!
//! This module provides a comprehensive design system and component library
//! for building beautiful, consistent terminal interfaces. All components are
//! functions that return `String` values, making them fully testable without
//! side effects.
//!
//! # Design Philosophy
//!
//! - **Trustworthy**: Security-focused styling matches high-quality tools like Rust's rustc
//! - **Scannable**: Users understand results in under 3 seconds
//! - **Informative**: Every line of output earns its place
//! - **Consistent**: Same visual language across all commands
//!
//! # Components
//!
//! - `constants`: Color codes, icons, and typography tokens
//! - `utils`: Terminal utilities (width detection, text wrapping, boxes)
//! - `banner`: Truent splash screen
//! - `progress`: Spinner for long operations
//! - `violation`: Bordered violation panels
//! - `summary`: Analysis dashboard with charts
//! - `header`: Command header with metadata
//! - `doctor`: Health check display
//! - `passed`: Passed checks list
//! - `init`: Initialization success message
//! - `error`: Error message formatting

pub mod banner;
pub mod constants;
pub mod doctor;
pub mod error;
pub mod header;
pub mod init;
pub mod passed;
pub mod progress;
pub mod summary;
pub mod utils;
pub mod violation;

// Re-export commonly used items
pub use banner::render_banner;
pub use doctor::{render_doctor_results, HealthCheck};
pub use header::render_check_header;
pub use init::render_init_success;
pub use passed::render_passed_checks;
pub use progress::Spinner;
pub use summary::{render_summary, AnalysisSummary, SeverityBreakdown};
pub use utils::*;
pub use violation::{render_violations, Violation};
