//! Design system constants for Truent CLI.
//!
//! Defines the color palette, icons, and typography hierarchy used throughout
//! the CLI interface to maintain a consistent, professional appearance.

use colored::Colorize;

// ============================================================================
// ICONS
// ============================================================================

/// Critical severity icon (bright red).
pub const ICON_CRITICAL: &str = "✗";

/// High severity icon (red).
pub const ICON_HIGH: &str = "✗";

/// Medium severity icon (yellow).
pub const ICON_MEDIUM: &str = "⚠";

/// Low severity icon (cyan).
pub const ICON_LOW: &str = "ℹ";

/// Pass/success icon (bright green).
pub const ICON_PASS: &str = "✓";

/// Suppressed check icon (dim).
#[allow(dead_code)]
pub const ICON_SUPPRESS: &str = "○";

/// Arrow for recommendations (cyan).
pub const ICON_ARROW: &str = "→";

/// Bullet for list items (dim).
#[allow(dead_code)]
pub const ICON_BULLET: &str = "·";

// Box drawing characters
#[allow(dead_code)]
pub const ICON_BOX_H: &str = "─";
#[allow(dead_code)]
pub const ICON_BOX_V: &str = "│";
#[allow(dead_code)]
pub const ICON_BOX_TL: &str = "╭";
#[allow(dead_code)]
pub const ICON_BOX_TR: &str = "╮";
#[allow(dead_code)]
pub const ICON_BOX_BL: &str = "╰";
#[allow(dead_code)]
pub const ICON_BOX_BR: &str = "╯";
#[allow(dead_code)]
pub const ICON_BOX_ML: &str = "├";
#[allow(dead_code)]
pub const ICON_BOX_MR: &str = "┤";

/// Spinner animation frames.
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

// ============================================================================
// COLOR CODES
// ============================================================================

/// Truent brand green (#34D399), shared with the web `--acc-text` token so the
/// banner and the site read as the same colour.
const BRAND_R: u8 = 0x34;
const BRAND_G: u8 = 0xD3;
const BRAND_B: u8 = 0x99;

/// Whether the terminal advertises 24-bit colour.
///
/// `colored` silently degrades `truecolor` to the nearest of the 16 ANSI
/// colours, and for the brand green that nearest match is *cyan* — the blue
/// family the palette deliberately moved away from. So the fallback is chosen
/// here instead: brand hex where it renders faithfully, ANSI green everywhere
/// else. Either way the mark is green.
fn supports_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false)
}

/// Apply the brand green, falling back to ANSI green on 16-colour terminals.
fn brand(s: &str) -> colored::ColoredString {
    if supports_truecolor() {
        s.truecolor(BRAND_R, BRAND_G, BRAND_B)
    } else {
        s.green()
    }
}

/// Apply critical severity styling (bright red bold).
pub fn color_critical(s: &str) -> String {
    s.bright_red().bold().to_string()
}

/// Apply high severity styling (red bold).
pub fn color_high(s: &str) -> String {
    s.red().bold().to_string()
}

/// Apply medium severity styling (yellow bold).
pub fn color_medium(s: &str) -> String {
    s.yellow().bold().to_string()
}

/// Apply low severity styling (cyan bold).
pub fn color_low(s: &str) -> String {
    s.cyan().bold().to_string()
}

/// Apply success styling (bright green bold).
pub fn color_success(s: &str) -> String {
    s.bright_green().bold().to_string()
}

/// Apply failure styling (bright red bold).
pub fn color_failure(s: &str) -> String {
    s.bright_red().bold().to_string()
}

/// Apply warning styling (yellow).
#[allow(dead_code)]
pub fn color_warning(s: &str) -> String {
    s.yellow().to_string()
}

/// Apply skipped styling (dim white).
#[allow(dead_code)]
pub fn color_skipped(s: &str) -> String {
    s.white().dimmed().to_string()
}

/// Apply border styling (dim white).
pub fn color_border(s: &str) -> String {
    s.white().dimmed().to_string()
}

/// Apply label styling (white bold).
pub fn color_label(s: &str) -> String {
    s.white().bold().to_string()
}

/// Apply value styling (white).
pub fn color_value(s: &str) -> String {
    s.white().to_string()
}

/// Apply secondary info styling (dim white).
pub fn color_dim(s: &str) -> String {
    s.dimmed().to_string()
}

/// Apply accent styling (Truent green).
#[allow(dead_code)]
pub fn color_accent(s: &str) -> String {
    brand(s).to_string()
}

/// Apply recommendation styling (italic cyan).
pub fn color_recommendation(s: &str) -> String {
    s.italic().cyan().to_string()
}

/// Apply header styling (Truent green, bold).
pub fn color_header(s: &str) -> String {
    brand(s).bold().to_string()
}
