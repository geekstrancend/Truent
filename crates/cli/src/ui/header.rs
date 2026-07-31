//! Header component shown at the start of analysis.

use crate::ui::constants::{color_dim, color_header, color_label, color_value};
use crate::ui::utils::{divider, term_width};

/// Render the check command header.
///
/// Displays a divider-based header with tool name, target, chain, and config info.
///
/// # Arguments
/// * `target` - The file or path being analyzed
/// * `chain` - The blockchain (EVM, Solana, Move)
/// * `config_path` - Optional path to the configuration file
/// * `config_found` - Whether the config file was found
///
/// # Returns
/// The formatted header as a string
pub fn render_check_header(
    target: &str,
    chain: &str,
    config_path: Option<&str>,
    config_found: bool,
) -> String {
    let width = term_width();
    let mut output = String::new();

    // Top divider
    output.push_str(&format!("{}\n", color_dim(&divider(width))));

    // Title line
    output.push_str(&format!(
        "{}  {}  {}  {}\n",
        color_header("Truent"),
        color_dim("·"),
        color_dim("Multi-chain Invariant Checker"),
        color_dim("·"),
    ));
    output.push_str(&format!(
        "{}\n",
        color_dim(&format!("v{}", env!("CARGO_PKG_VERSION")))
    ));

    // Middle divider
    output.push_str(&format!("{}\n", color_dim(&divider(width))));
    output.push('\n');

    // Target line
    output.push_str(&format!(
        "{}  {}\n",
        color_label("Target"),
        color_value(target)
    ));

    // Chain line
    output.push_str(&format!(
        "{}  {}\n",
        color_label("Chain"),
        color_value(chain)
    ));

    // Config line
    if let Some(config) = config_path {
        let config_status = if config_found {
            "(found)"
        } else {
            "(not found)"
        };
        output.push_str(&format!(
            "{}  {} {}\n",
            color_label("Config"),
            color_value(config),
            color_dim(config_status)
        ));
    }

    output.push('\n');

    // Bottom divider
    output.push_str(&format!("{}\n", color_dim(&divider(width))));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_contains_expected_parts() {
        let header =
            render_check_header("./contracts/Token.sol", "EVM", Some(".truent.toml"), true);

        assert!(header.contains("Truent"));
        assert!(header.contains("./contracts/Token.sol"));
        assert!(header.contains("EVM"));
        assert!(header.contains(".truent.toml"));
        assert!(header.contains("found"));
    }

    #[test]
    fn test_header_without_config() {
        let header = render_check_header("./src", "Solana", None, false);

        assert!(header.contains("Truent"));
        assert!(header.contains("./src"));
        assert!(header.contains("Solana"));
    }
}
