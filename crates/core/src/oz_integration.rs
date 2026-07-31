/// OpenZeppelin Integration Module
///
/// Integrates Truent findings with OpenZeppelin Contracts library patterns and audit standards.
/// Provides mapping between Truent detectors and known OZ vulnerabilities, enabling
/// cross-reference with OZ audit recommendations and security standards.
use crate::Finding;
use std::collections::HashMap;

/// OpenZeppelin vulnerability classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OZVulnerabilityType {
    /// Access control bypass
    AccessControl,
    /// Reentrancy attacks
    Reentrancy,
    /// Integer overflow/underflow
    ArithmeticOverflow,
    /// Token transfer vulnerabilities
    TokenTransfer,
    /// Oracle manipulation
    OracleManipulation,
    /// Flash loan attacks
    FlashLoan,
    /// Signature replay
    SignatureReplay,
    /// State management issues
    StateManagement,
    /// Initialization vulnerabilities
    Initialization,
    /// Custom vulnerability
    Custom(String),
}

/// OpenZeppelin audit recommendation
#[derive(Debug, Clone)]
pub struct OZRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Title of recommendation
    pub title: String,
    /// Severity level
    pub severity: String,
    /// OZ best practice code
    pub best_practice: String,
    /// Reference URL or document
    pub reference: String,
}

/// Mapping between Truent detectors and OZ vulnerability types
pub struct OZMappingRegistry {
    detector_to_oz: HashMap<String, OZVulnerabilityType>,
    oz_to_recommendations: HashMap<OZVulnerabilityType, Vec<OZRecommendation>>,
}

impl OZMappingRegistry {
    /// Create new OZ mapping registry
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut registry = Self {
            detector_to_oz: HashMap::new(),
            oz_to_recommendations: HashMap::new(),
        };

        registry.initialize_mappings();
        registry
    }

    /// Initialize detector to OZ vulnerability mappings
    fn initialize_mappings(&mut self) {
        // Phase A detectors
        self.detector_to_oz.insert(
            "health_check".to_string(),
            OZVulnerabilityType::StateManagement,
        );
        self.detector_to_oz.insert(
            "merkle_root".to_string(),
            OZVulnerabilityType::StateManagement,
        );
        self.detector_to_oz.insert(
            "dvn_single_point".to_string(),
            OZVulnerabilityType::AccessControl,
        );
        self.detector_to_oz.insert(
            "synthetic_mint".to_string(),
            OZVulnerabilityType::StateManagement,
        );
        self.detector_to_oz.insert(
            "lst_depeg".to_string(),
            OZVulnerabilityType::OracleManipulation,
        );

        // Phase B detectors
        self.detector_to_oz.insert(
            "oracle_self_trade".to_string(),
            OZVulnerabilityType::OracleManipulation,
        );
        self.detector_to_oz.insert(
            "solana_durable_nonce".to_string(),
            OZVulnerabilityType::SignatureReplay,
        );
        self.detector_to_oz.insert(
            "synthetic_collateral_oracle".to_string(),
            OZVulnerabilityType::OracleManipulation,
        );
        self.detector_to_oz.insert(
            "erc4626_inflation".to_string(),
            OZVulnerabilityType::ArithmeticOverflow,
        );
        self.detector_to_oz.insert(
            "arbitrary_call_msg_value".to_string(),
            OZVulnerabilityType::StateManagement,
        );
        self.detector_to_oz.insert(
            "reentrancy_whitelisted".to_string(),
            OZVulnerabilityType::Reentrancy,
        );
        self.detector_to_oz.insert(
            "proxy_storage_collision".to_string(),
            OZVulnerabilityType::StateManagement,
        );
        self.detector_to_oz.insert(
            "bridge_address_crypto".to_string(),
            OZVulnerabilityType::SignatureReplay,
        );

        // Initialize OZ recommendations
        self.add_oz_recommendations();
    }

    /// Add OpenZeppelin recommendations
    fn add_oz_recommendations(&mut self) {
        // Access control recommendations
        let access_control_recs = vec![
            OZRecommendation {
                id: "OZ-AC-001".to_string(),
                title: "Use onlyOwner modifier for privileged functions".to_string(),
                severity: "High".to_string(),
                best_practice: "Use OpenZeppelin's Ownable contract".to_string(),
                reference: "https://docs.openzeppelin.com/contracts/4.x/access-control".to_string(),
            },
            OZRecommendation {
                id: "OZ-AC-002".to_string(),
                title: "Implement role-based access control".to_string(),
                severity: "High".to_string(),
                best_practice: "Use AccessControl for complex permissions".to_string(),
                reference: "https://docs.openzeppelin.com/contracts/4.x/api/access#AccessControl"
                    .to_string(),
            },
        ];
        self.oz_to_recommendations
            .insert(OZVulnerabilityType::AccessControl, access_control_recs);

        // Reentrancy recommendations
        let reentrancy_recs = vec![OZRecommendation {
            id: "OZ-RE-001".to_string(),
            title: "Apply Checks-Effects-Interactions pattern".to_string(),
            severity: "Critical".to_string(),
            best_practice: "Update state before external calls".to_string(),
            reference: "https://docs.openzeppelin.com/contracts/4.x/api/security#ReentrancyGuard"
                .to_string(),
        }];
        self.oz_to_recommendations
            .insert(OZVulnerabilityType::Reentrancy, reentrancy_recs);

        // Oracle recommendations
        let oracle_recs = vec![OZRecommendation {
            id: "OZ-OR-001".to_string(),
            title: "Use multiple oracle sources".to_string(),
            severity: "High".to_string(),
            best_practice: "Implement oracle aggregation pattern".to_string(),
            reference: "https://docs.openzeppelin.com/contracts/4.x/".to_string(),
        }];
        self.oz_to_recommendations
            .insert(OZVulnerabilityType::OracleManipulation, oracle_recs);
    }

    /// Get OZ vulnerability type for a detector
    pub fn get_oz_type(&self, detector_name: &str) -> Option<OZVulnerabilityType> {
        self.detector_to_oz.get(detector_name).cloned()
    }

    /// Get OZ recommendations for a vulnerability type
    pub fn get_recommendations(
        &self,
        vuln_type: &OZVulnerabilityType,
    ) -> Option<&[OZRecommendation]> {
        self.oz_to_recommendations
            .get(vuln_type)
            .map(|v| v.as_slice())
    }

    /// Enrich a finding with OZ context
    pub fn enrich_finding(&self, finding: &Finding) -> EnrichedFinding {
        let detector_name = finding.invariant_id.to_lowercase();
        let oz_type = self.get_oz_type(&detector_name);

        let recommendations = oz_type
            .as_ref()
            .and_then(|t| self.get_recommendations(t))
            .unwrap_or(&[])
            .to_vec();

        EnrichedFinding {
            original_finding: finding.clone(),
            oz_type,
            oz_recommendations: recommendations,
        }
    }
}

/// Finding enriched with OZ context
#[derive(Debug, Clone)]
pub struct EnrichedFinding {
    /// Original Truent finding
    pub original_finding: Finding,
    /// Mapped OZ vulnerability type
    pub oz_type: Option<OZVulnerabilityType>,
    /// OZ recommendations
    pub oz_recommendations: Vec<OZRecommendation>,
}

impl EnrichedFinding {
    /// Generate OZ audit report section
    pub fn to_audit_report(&self) -> String {
        let mut report = format!("## Finding: {}\n\n", self.original_finding.message);

        report.push_str(&format!(
            "**Severity:** {:?}\n",
            self.original_finding.severity
        ));

        if let Some(ref oz_type) = self.oz_type {
            report.push_str(&format!("**OZ Classification:** {:?}\n\n", oz_type));
        }

        if !self.oz_recommendations.is_empty() {
            report.push_str("### OpenZeppelin Recommendations\n\n");
            for rec in &self.oz_recommendations {
                report.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    rec.id, rec.severity, rec.title
                ));
                report.push_str(&format!("  - Best Practice: {}\n", rec.best_practice));
                report.push_str(&format!("  - Reference: {}\n\n", rec.reference));
            }
        }

        report.push_str(&format!(
            "**Location:** {}:{}\n",
            self.original_finding.file, self.original_finding.line
        ));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oz_registry_maps_detectors() {
        let registry = OZMappingRegistry::new();

        assert_eq!(
            registry.get_oz_type("health_check"),
            Some(OZVulnerabilityType::StateManagement)
        );

        assert_eq!(
            registry.get_oz_type("oracle_self_trade"),
            Some(OZVulnerabilityType::OracleManipulation)
        );
    }

    #[test]
    fn oz_registry_provides_recommendations() {
        let registry = OZMappingRegistry::new();

        let recs = registry.get_recommendations(&OZVulnerabilityType::AccessControl);
        assert!(recs.is_some());
        assert!(!recs.unwrap().is_empty());
    }

    #[test]
    fn enriched_finding_generates_audit_report() {
        let finding = Finding::new(
            "test_detector".to_string(),
            crate::Severity::High,
            "test.sol".to_string(),
            42,
            0,
            "Test vulnerability".to_string(),
            "test code".to_string(),
        );

        let registry = OZMappingRegistry::new();
        let enriched = registry.enrich_finding(&finding);

        let report = enriched.to_audit_report();
        assert!(report.contains("test.sol:42"));
    }

    #[test]
    fn oz_type_equality() {
        let t1 = OZVulnerabilityType::Reentrancy;
        let t2 = OZVulnerabilityType::Reentrancy;
        assert_eq!(t1, t2);
    }
}
