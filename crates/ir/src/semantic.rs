//! Chain-agnostic semantic IR for authorization/privileged-mutation analysis.
//!
//! Each chain analyzer builds a [`SemanticModel`] by extracting these facts
//! from chain-native source (Solidity, Anchor/Rust, Move, ...). Detection
//! rules (see [`crate::rules`]) operate only on this IR, so a rule written
//! once applies unmodified to every chain that populates the model. This is
//! the minimal slice of the shared IR needed to prove that pattern end to
//! end (Truent PRD Epic 6.1) — it is expected to grow incrementally as more
//! rules are ported onto it, not to be a complete cross-chain abstraction.

use serde::{Deserialize, Serialize};

/// The kind of authorization check guarding a privileged mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthCheckKind {
    /// A cryptographic signer/ownership check (Solidity `onlyOwner`, Anchor
    /// `Signer<'info>`, Move `&signer` bound to a checked address).
    Signer,
    /// A role/capability-based check (RBAC, Move capability types, Anchor
    /// `has_one`/`owner`/`address` constraints).
    RoleOrCapability,
    /// A multisig/governance-gated check.
    Multisig,
    /// Some other explicit guard the extractor recognized but couldn't
    /// classify further.
    Other(String),
}

/// A single authorization guard found protecting a mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationCheck {
    /// The classification of this guard.
    pub kind: AuthCheckKind,
    /// Human-readable source of the check (modifier name, account field,
    /// capability parameter, etc), for citation in findings.
    pub source: String,
}

/// The category of a privileged mutation — a state change that moves value
/// or changes control, and therefore needs an authorization check reaching
/// it before it executes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationKind {
    /// Moves funds/tokens/value out of the program's control.
    FundTransfer,
    /// Changes who controls an account, role, or the program itself.
    AuthorityChange,
    /// Upgrades the program/contract's code.
    Upgrade,
    /// Closes/destroys an account and reclaims its resources.
    AccountClose,
    /// A privileged mutation that doesn't fit the categories above.
    Other(String),
}

/// A privileged mutation site: an entry point that performs a sensitive
/// state change, along with whatever authorization checks were found
/// guarding it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegedMutation {
    /// Name of the function/instruction handler/entry point.
    pub entry_point: String,
    /// What kind of privileged mutation this is.
    pub kind: MutationKind,
    /// 1-indexed source line, for citation in findings.
    pub line: usize,
    /// Authorization checks found guarding this mutation. Empty means the
    /// extractor found no guard reaching this entry point.
    pub guards: Vec<AuthorizationCheck>,
}

impl PrivilegedMutation {
    /// Whether at least one authorization check guards this mutation.
    pub fn is_guarded(&self) -> bool {
        !self.guards.is_empty()
    }
}

/// Chain-agnostic semantic model: the set of privileged mutation sites
/// extracted from one source file, independent of the chain's native
/// syntax.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticModel {
    /// Chain tag: "evm", "solana", "move", ...
    pub chain: String,
    /// Source file path, for citation in findings.
    pub source_path: String,
    /// Extracted privileged mutation sites.
    pub mutations: Vec<PrivilegedMutation>,
}

impl SemanticModel {
    /// Create a new, empty semantic model for the given chain/source.
    pub fn new(chain: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            chain: chain.into(),
            source_path: source_path.into(),
            mutations: Vec::new(),
        }
    }
}
