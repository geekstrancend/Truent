//! Library loader for TOML-based invariants.

use truent_core::model::Invariant;
use truent_core::Result;
use truent_dsl_parser::parse_invariant;
use std::path::Path;
use tracing::info;

/// Loads invariants from TOML files.
pub struct LibraryLoader;

impl LibraryLoader {
    /// Load invariants from a TOML file.
    ///
    /// Expects TOML structure like:
    /// ```toml
    /// [[invariants]]
    /// name = "balance_conservation"
    /// expression = "sum_balances == total_supply"
    /// severity = "critical"
    /// ```
    pub fn load_from_toml(path: &Path) -> Result<Vec<Invariant>> {
        info!("Loading invariants from {:?}", path);

        let content = std::fs::read_to_string(path).map_err(truent_core::InvarError::IoError)?;

        // Parse TOML
        let table: toml::Table = toml::from_str(&content)
            .map_err(|e| truent_core::InvarError::ConfigError(e.to_string()))?;

        let mut invariants = Vec::new();

        // Extract invariants from table
        if let Some(inv_array) = table.get("invariants").and_then(|v| v.as_array()) {
            for (idx, inv_table) in inv_array.iter().enumerate() {
                match parse_invariant_table(inv_table) {
                    Ok(inv) => {
                        info!("Loaded invariant: {}", inv.name);
                        invariants.push(inv);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse invariant at index {}: {}", idx, e);
                    }
                }
            }
        }

        info!(
            "Loaded {} invariants from {}",
            invariants.len(),
            path.display()
        );
        Ok(invariants)
    }

    /// Load all invariants from a directory.
    pub fn load_from_dir(dir: &Path) -> Result<Vec<Invariant>> {
        let mut all_invariants = Vec::new();

        // Read all .toml files in directory
        let entries = std::fs::read_dir(dir).map_err(truent_core::InvarError::IoError)?;

        for entry in entries {
            let entry = entry.map_err(truent_core::InvarError::IoError)?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "toml") {
                let invariants = Self::load_from_toml(&path)?;
                all_invariants.extend(invariants);
            }
        }

        Ok(all_invariants)
    }
}

/// Parse an invariant from a TOML table value.
///
/// Creates a complete invariant by parsing the expression string using the DSL parser.
fn parse_invariant_table(table: &toml::Value) -> Result<Invariant> {
    let table = table.as_table().ok_or_else(|| {
        truent_core::InvarError::ConfigError("Invariant must be a table".to_string())
    })?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            truent_core::InvarError::ConfigError("Invariant must have a 'name' field".to_string())
        })?
        .to_string();

    let expression_str = table
        .get("expression")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            truent_core::InvarError::ConfigError(
                "Invariant must have an 'expression' field".to_string(),
            )
        })?;

    let severity = table
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .to_string();

    let category = table
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("general")
        .to_string();

    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse the expression string using the DSL parser
    // Construct full invariant format for the parser
    let full_invariant_str = format!(r#"invariant {} {{ {} }}"#, name, expression_str);

    let mut parsed_invariant = parse_invariant(&full_invariant_str)?;

    // Override with TOML-provided severity and category
    parsed_invariant.severity = severity;
    parsed_invariant.category = category;
    if description.is_some() {
        parsed_invariant.description = description;
    }

    info!(
        "Parsed invariant '{}' with expression '{}' (severity: {}, category: {})",
        name, expression_str, parsed_invariant.severity, parsed_invariant.category
    );

    Ok(parsed_invariant)
}
