//! Well-known ERC20/Ownable function selectors, for probing contracts that
//! have no ABI at all — the common case for a real bug-bounty target,
//! where the deployed bytecode may not be verified on any block explorer.
//!
//! Selectors are computed via [`crate::solc_bridge::selector_for`] (the
//! same keccak256 signature-hashing already cross-checked against WETH9's
//! real constants in `solc_bridge`'s test suite) rather than hardcoded as
//! hex literals — a memorized/mistyped constant would silently probe the
//! wrong function forever, so deriving them from source-of-truth signature
//! strings through already-verified code is the safer choice.

use crate::solc_bridge::selector_for;
use truent_dynamic_core::{FunctionSpec, ParamKind};

/// The standard ERC20 read surface, used to probe whether unverified
/// bytecode looks like a token contract at all.
pub fn erc20_probe_functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new(
            "totalSupply",
            selector_for("totalSupply", &[]),
            vec![],
            false,
        ),
        FunctionSpec::new(
            "balanceOf",
            selector_for("balanceOf", &["address"]),
            vec![ParamKind::Address],
            false,
        ),
        FunctionSpec::new("decimals", selector_for("decimals", &[]), vec![], false),
    ]
}

/// The mutating ERC20 surface a dynamic fuzzer would drive calls through,
/// once probing confirms the read surface above is actually present.
pub fn erc20_mutator_functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new(
            "transfer",
            selector_for("transfer", &["address", "uint256"]),
            vec![ParamKind::Address, ParamKind::Uint256],
            true,
        ),
        FunctionSpec::new(
            "approve",
            selector_for("approve", &["address", "uint256"]),
            vec![ParamKind::Address, ParamKind::Uint256],
            true,
        ),
        FunctionSpec::new(
            "transferFrom",
            selector_for("transferFrom", &["address", "address", "uint256"]),
            vec![ParamKind::Address, ParamKind::Address, ParamKind::Uint256],
            true,
        ),
    ]
}

/// The OpenZeppelin-Ownable surface, for probing access-control targets
/// the same way.
pub fn ownable_probe_functions() -> Vec<FunctionSpec> {
    vec![FunctionSpec::new(
        "owner",
        selector_for("owner", &[]),
        vec![],
        false,
    )]
}

pub fn ownable_mutator_functions() -> Vec<FunctionSpec> {
    vec![FunctionSpec::new(
        "transferOwnership",
        selector_for("transferOwnership", &["address"]),
        vec![ParamKind::Address],
        true,
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-checks against independently well-known, publicly documented
    /// selector constants (the same ones any block explorer or 4byte
    /// signature database would show) — a second, unrelated ground truth
    /// beyond the WETH9 check already in `solc_bridge`, covering the
    /// specific signatures this module relies on.
    #[test]
    fn selectors_match_independently_known_constants() {
        assert_eq!(selector_for("totalSupply", &[]), [0x18, 0x16, 0x0d, 0xdd]);
        assert_eq!(
            selector_for("balanceOf", &["address"]),
            [0x70, 0xa0, 0x82, 0x31]
        );
        assert_eq!(selector_for("decimals", &[]), [0x31, 0x3c, 0xe5, 0x67]);
        assert_eq!(
            selector_for("transfer", &["address", "uint256"]),
            [0xa9, 0x05, 0x9c, 0xbb]
        );
        assert_eq!(
            selector_for("approve", &["address", "uint256"]),
            [0x09, 0x5e, 0xa7, 0xb3]
        );
        assert_eq!(
            selector_for("transferFrom", &["address", "address", "uint256"]),
            [0x23, 0xb8, 0x72, 0xdd]
        );
        assert_eq!(selector_for("owner", &[]), [0x8d, 0xa5, 0xcb, 0x5b]);
        assert_eq!(
            selector_for("transferOwnership", &["address"]),
            [0xf2, 0xfd, 0xe3, 0x8b]
        );
    }

    #[test]
    fn probe_function_lists_match_their_computed_selectors() {
        let erc20 = erc20_probe_functions();
        assert_eq!(erc20[0].selector, selector_for("totalSupply", &[]));
        let ownable = ownable_probe_functions();
        assert_eq!(ownable[0].selector, selector_for("owner", &[]));
    }
}
