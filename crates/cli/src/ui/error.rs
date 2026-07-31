//! Error display component for Truent CLI.

use crate::ui::constants::{color_accent, color_dim, color_failure};

/// Render an error message in rustc style.
///
/// Displays errors with consistent formatting:
/// ```text
/// error: could not read file './contracts/Token.sol'
///        No such file or directory (os error 2)
///
/// hint: Check that the path exists and Truent has read permission.
///       Run 'truent doctor' to verify your installation.
/// ```
///
/// # Arguments
/// * `title` - The error title/message
/// * `detail` - Additional error details
/// * `hint` - Optional hint text for resolution
///
/// # Returns
/// The formatted error message
#[allow(dead_code)]
pub fn render_error(title: &str, detail: &str, hint: Option<&str>) -> String {
    let mut output = String::new();

    // Error line
    output.push_str(&format!("{} {}\n", color_failure("error:"), title));

    // Detail line (indented)
    if !detail.is_empty() {
        output.push_str(&format!("       {}\n", color_dim(detail)));
    }

    // Hint section
    if let Some(hint_text) = hint {
        output.push('\n');
        output.push_str(&format!("{} {}\n", color_accent("hint:"), hint_text));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_basic() {
        let msg = render_error("file not found", "No such file", None);
        assert!(msg.contains("error:"));
        assert!(msg.contains("file not found"));
        assert!(msg.contains("No such file"));
    }

    #[test]
    fn test_error_with_hint() {
        let msg = render_error(
            "invalid config",
            "syntax error on line 5",
            Some("Check your TOML syntax"),
        );
        assert!(msg.contains("error:"));
        assert!(msg.contains("hint:"));
        assert!(msg.contains("Check your TOML syntax"));
    }

    #[test]
    fn test_error_without_detail() {
        let msg = render_error("something went wrong", "", None);
        assert!(msg.contains("something went wrong"));
    }
}
