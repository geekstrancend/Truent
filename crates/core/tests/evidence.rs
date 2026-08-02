//! A result must not claim proof it does not have.
//!
//! Truent's premise — and its documentation — is that a finding is something
//! an engine made happen, and anything it could not reproduce is a lead. That
//! only holds if claiming proof is opt-in.

use truent_core::{Evidence, Finding, Severity};

fn finding() -> Finding {
    Finding::new(
        "evm_test".into(),
        Severity::Critical,
        "t.sol".into(),
        1,
        0,
        "msg".into(),
        "code".into(),
    )
}

#[test]
fn findings_default_to_lead() {
    let f = finding();
    assert_eq!(f.evidence, Evidence::Lead);
    assert!(!f.is_proven(), "a pattern match has not been demonstrated");
}

#[test]
fn proof_is_opt_in() {
    let f = finding().proven();
    assert_eq!(f.evidence, Evidence::Proven);
    assert!(f.is_proven());
}

#[test]
fn evidence_survives_serialisation() {
    let f = finding().proven();
    let json = serde_json::to_string(&f).expect("serialise");
    assert!(json.contains("\"evidence\":\"proven\""), "got {json}");
    let back: Finding = serde_json::from_str(&json).expect("deserialise");
    assert!(back.is_proven());
}

/// Older reports without the field must not be read back as proven.
#[test]
fn missing_evidence_field_defaults_to_lead() {
    let json = r#"{
        "invariant_id":"x","severity":"Critical","file":"t.sol","line":1,"col":0,
        "message":"m","snippet":"s","source_fragment":null,"transaction_hash":null,
        "metadata":{}
    }"#;
    let f: Finding = serde_json::from_str(json).expect("deserialise legacy");
    assert_eq!(f.evidence, Evidence::Lead);
}
