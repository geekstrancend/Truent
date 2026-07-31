//! Exploit registry: Historical DeFi hacks mapped to Truent invariants.
//!
//! This module contains 21 documented exploits from major DeFi protocols,
//! each mapped to the Truent invariants they violated.
//! Data is embedded at compile time via include_bytes! and loaded lazily.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An historical exploit record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exploit {
    /// Unique identifier (slug format).
    pub id: String,
    /// Protocol name.
    pub protocol: String,
    /// Date of exploit (YYYY-MM-DD).
    pub date: String,
    /// Estimated loss in USD.
    pub loss_usd: u64,
    /// Chain affected.
    pub chain: String,
    /// Invariants that would have detected this.
    pub invariant_ids: Vec<String>,
    /// Summary of the attack.
    pub attack_summary: String,
    /// Transaction hash or block number.
    pub tx_hash: String,
    /// Link to postmortem or analysis.
    pub postmortem_url: String,
}

/// Registry of all known exploits.
static EXPLOIT_REGISTRY_JSON: &[u8] = include_bytes!("exploits.json");

/// Loaded exploit registry (lazy singleton).
pub static EXPLOIT_REGISTRY: Lazy<ExploitRegistry> =
    Lazy::new(|| ExploitRegistry::load().expect("Failed to parse exploit registry"));

/// The exploit registry container.
#[derive(Debug, Clone)]
pub struct ExploitRegistry {
    exploits: BTreeMap<String, Exploit>,
}

impl ExploitRegistry {
    /// Load the registry from embedded JSON.
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let exploits: Vec<Exploit> = serde_json::from_slice(EXPLOIT_REGISTRY_JSON)?;
        let mut map = BTreeMap::new();
        for exploit in exploits {
            map.insert(exploit.id.clone(), exploit);
        }
        Ok(Self { exploits: map })
    }

    /// Get an exploit by ID.
    pub fn get(&self, id: &str) -> Option<&Exploit> {
        self.exploits.get(id)
    }

    /// Get all exploits.
    pub fn all(&self) -> Vec<&Exploit> {
        self.exploits.values().collect()
    }

    /// Get exploits for a specific chain.
    pub fn by_chain(&self, chain: &str) -> Vec<&Exploit> {
        self.exploits
            .values()
            .filter(|e| e.chain.eq_ignore_ascii_case(chain))
            .collect()
    }

    /// Get exploits that violated a specific invariant.
    pub fn by_invariant(&self, invariant_id: &str) -> Vec<&Exploit> {
        self.exploits
            .values()
            .filter(|e| e.invariant_ids.iter().any(|id| id == invariant_id))
            .collect()
    }

    /// Get total loss across all exploits.
    pub fn total_loss(&self) -> u64 {
        self.exploits.values().map(|e| e.loss_usd).sum()
    }

    /// Count total exploits.
    pub fn count(&self) -> usize {
        self.exploits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads() {
        let registry = &*EXPLOIT_REGISTRY;
        assert!(registry.count() > 0, "Registry should have exploits");
    }

    #[test]
    fn test_registry_access() {
        let registry = &*EXPLOIT_REGISTRY;
        let all = registry.all();
        assert!(!all.is_empty());

        // Each exploit should have required fields
        for exploit in all {
            assert!(!exploit.id.is_empty());
            assert!(!exploit.protocol.is_empty());
            assert!(!exploit.chain.is_empty());
        }
    }
}
