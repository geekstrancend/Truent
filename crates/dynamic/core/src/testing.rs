//! A pure-Rust, in-memory [`ExecutionBackend`] simulating two tiny synthetic
//! contracts. This exists so the fuzzing engine itself (generation,
//! execution, shrinking, invariant checking) is provably correct without
//! depending on any real VM — `truent-dynamic-evm` provides the real
//! `revm`-backed implementation for actual Solidity bytecode.
//!
//! Test-only (`cfg(test)`): shared by this crate's own unit tests
//! (`shrink.rs`, and the end-to-end proof in `lib.rs`) so there's exactly
//! one mock implementation instead of a bespoke one per test file.

use crate::backend::{CallOutcome, EncodedCall, ExecutionBackend, FunctionSpec, ParamKind};
use crate::trace::TraceEvent;
use std::collections::HashMap;

pub const INCR: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
/// Decrements the counter unconditionally — stands in for "a state
/// transition that shouldn't be reachable but is", regardless of the
/// specific real-world reason (missing access control, a stray public
/// setter, etc.). The point is that *something* breaks monotonicity and the
/// invariant must catch it.
pub const DECR: [u8; 4] = [0x02, 0x00, 0x00, 0x00];
pub const VALUE: [u8; 4] = [0x03, 0x00, 0x00, 0x00];

pub const MINT: [u8; 4] = [0x10, 0x00, 0x00, 0x00];
/// Credits a balance without a matching debit or totalSupply update — the
/// "value created out of thin air" bug shape that
/// [`crate::invariant::ConservationInvariant`] exists to catch.
pub const BUGGY_CREDIT: [u8; 4] = [0x11, 0x00, 0x00, 0x00];
pub const TOTAL_SUPPLY: [u8; 4] = [0x12, 0x00, 0x00, 0x00];
pub const BALANCE_OF: [u8; 4] = [0x13, 0x00, 0x00, 0x00];

pub const OWNER: [u8; 4] = [0x20, 0x00, 0x00, 0x00];
/// Reassigns ownership with no caller check at all — the "missing
/// onlyOwner modifier" bug shape that [`crate::invariant::AccessControlInvariant`]
/// exists to catch.
pub const TRANSFER_OWNERSHIP: [u8; 4] = [0x21, 0x00, 0x00, 0x00];
/// The correctly-guarded counterpart: reverts unless the caller is the
/// current owner, used to prove the invariant doesn't false-positive on a
/// contract that actually enforces access control.
pub const SAFE_TRANSFER_OWNERSHIP: [u8; 4] = [0x22, 0x00, 0x00, 0x00];
/// The deployer-equivalent address `MockState` initializes `owner` to —
/// distinct from the `[1u8;20]..[4u8;20]` actor addresses tests use as
/// "attacker" candidates, so "did a non-owner succeed" is unambiguous.
pub const GENESIS_OWNER: [u8; 20] = [0xAA; 20];

/// A vault `withdraw` that sends ETH to the caller *before* zeroing their
/// balance — the classic reentrancy bug. Its synthetic trace shows the
/// vault re-entered by a non-reverting nested call, with the balance write
/// happening after the re-entry.
pub const WITHDRAW_VULN: [u8; 4] = [0x30, 0x00, 0x00, 0x00];
/// A `withdraw` protected by a `nonReentrant` guard: its trace shows the
/// re-entrant inner call reverting.
pub const WITHDRAW_GUARDED: [u8; 4] = [0x31, 0x00, 0x00, 0x00];
/// A checks-effects-interactions-correct `withdraw`: its trace writes state
/// before the external call, so the re-entry finds nothing to exploit.
pub const WITHDRAW_CEI: [u8; 4] = [0x32, 0x00, 0x00, 0x00];

/// The vault contract's own address in the synthetic reentrancy traces.
pub const VAULT_ADDR: [u8; 20] = [0xC0; 20];
/// The attacker contract's address in the synthetic reentrancy traces.
pub const ATTACKER_ADDR: [u8; 20] = [0xA7; 20];

pub fn counter_functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new("increment", INCR, vec![], true),
        FunctionSpec::new("decrement", DECR, vec![], true),
        FunctionSpec::new("value", VALUE, vec![], false),
    ]
}

pub fn token_functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new(
            "mint",
            MINT,
            vec![ParamKind::Address, ParamKind::Uint256],
            true,
        ),
        FunctionSpec::new(
            "buggyCredit",
            BUGGY_CREDIT,
            vec![ParamKind::Address, ParamKind::Uint256],
            true,
        ),
        FunctionSpec::new("totalSupply", TOTAL_SUPPLY, vec![], false),
        FunctionSpec::new("balanceOf", BALANCE_OF, vec![ParamKind::Address], false),
    ]
}

pub fn ownable_functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new("owner", OWNER, vec![], false),
        FunctionSpec::new(
            "transferOwnership",
            TRANSFER_OWNERSHIP,
            vec![ParamKind::Address],
            true,
        ),
    ]
}

pub fn ownable_functions_safe() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new("owner", OWNER, vec![], false),
        FunctionSpec::new(
            "transferOwnership",
            SAFE_TRANSFER_OWNERSHIP,
            vec![ParamKind::Address],
            true,
        ),
    ]
}

/// A vault whose only mutating entry point is the reentrancy-vulnerable
/// `withdraw`.
pub fn vault_functions_vulnerable() -> Vec<FunctionSpec> {
    vec![FunctionSpec::new("withdraw", WITHDRAW_VULN, vec![], true)]
}

/// The two safe vault variants (guarded and CEI-correct), for proving the
/// reentrancy invariant doesn't false-positive.
pub fn vault_functions_safe() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec::new("withdrawGuarded", WITHDRAW_GUARDED, vec![], true),
        FunctionSpec::new("withdrawCei", WITHDRAW_CEI, vec![], true),
    ]
}

#[derive(Clone)]
struct MockState {
    counter: u128,
    balances: HashMap<[u8; 20], u128>,
    total_supply: u128,
    owner: [u8; 20],
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            counter: 0,
            balances: HashMap::new(),
            total_supply: 0,
            owner: GENESIS_OWNER,
        }
    }
}

#[derive(Default)]
pub struct MockBackend {
    state: MockState,
    snapshots: Vec<MockState>,
    last_trace: Vec<TraceEvent>,
}

/// The synthetic execution trace for a reentrancy-vulnerable withdraw:
/// vault re-entered by a non-reverting nested call, balance written after.
fn vulnerable_withdraw_trace() -> Vec<TraceEvent> {
    vec![
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::CallBegin {
            address: ATTACKER_ADDR,
        },
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::StorageWrite {
            address: VAULT_ADDR,
        },
        TraceEvent::CallEnd { reverted: false },
        TraceEvent::CallEnd { reverted: false },
        TraceEvent::StorageWrite {
            address: VAULT_ADDR,
        },
        TraceEvent::CallEnd { reverted: false },
    ]
}

/// A guarded withdraw: the re-entrant inner frame reverts.
fn guarded_withdraw_trace() -> Vec<TraceEvent> {
    vec![
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::CallBegin {
            address: ATTACKER_ADDR,
        },
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::CallEnd { reverted: true },
        TraceEvent::CallEnd { reverted: false },
        TraceEvent::StorageWrite {
            address: VAULT_ADDR,
        },
        TraceEvent::CallEnd { reverted: false },
    ]
}

/// A CEI-correct withdraw: state is written before the external call.
fn cei_withdraw_trace() -> Vec<TraceEvent> {
    vec![
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::StorageWrite {
            address: VAULT_ADDR,
        },
        TraceEvent::CallBegin {
            address: ATTACKER_ADDR,
        },
        TraceEvent::CallBegin {
            address: VAULT_ADDR,
        },
        TraceEvent::CallEnd { reverted: false },
        TraceEvent::CallEnd { reverted: false },
        TraceEvent::CallEnd { reverted: false },
    ]
}

fn word_to_u128(word: &[u8]) -> u128 {
    let mut buf = [0u8; 16];
    let start = word.len().saturating_sub(16);
    let src = &word[start..];
    buf[16 - src.len()..].copy_from_slice(src);
    u128::from_be_bytes(buf)
}

fn u128_to_word(value: u128) -> Vec<u8> {
    let mut out = vec![0u8; 32];
    out[16..32].copy_from_slice(&value.to_be_bytes());
    out
}

fn addr_from_calldata(calldata: &[u8]) -> [u8; 20] {
    let word = &calldata[4..36];
    word[12..32].try_into().unwrap()
}

impl ExecutionBackend for MockBackend {
    fn call(&mut self, call: &EncodedCall) -> CallOutcome {
        let selector: [u8; 4] = call.calldata[0..4].try_into().unwrap_or([0; 4]);
        // Default: a trivial single-frame trace, so a stale reentrancy
        // trace from an earlier vault call never lingers into an unrelated
        // (e.g. read-only) call.
        self.last_trace = vec![
            TraceEvent::CallBegin {
                address: VAULT_ADDR,
            },
            TraceEvent::CallEnd { reverted: false },
        ];
        match selector {
            INCR => {
                self.state.counter = self.state.counter.saturating_add(1);
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            DECR => {
                self.state.counter = self.state.counter.saturating_sub(1);
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            VALUE => CallOutcome {
                reverted: false,
                return_data: u128_to_word(self.state.counter),
            },
            MINT => {
                let addr = addr_from_calldata(&call.calldata);
                let amount = word_to_u128(&call.calldata[36..68]);
                // Mirror real checked-arithmetic Solidity (0.8+): an
                // overflowing mint reverts rather than wrapping/panicking,
                // so the edge-value-biased generator (which deliberately
                // probes u256::MAX) exercises a revert path here instead of
                // crashing the mock.
                let current = *self.state.balances.get(&addr).unwrap_or(&0);
                let (new_balance, new_supply) = match (
                    current.checked_add(amount),
                    self.state.total_supply.checked_add(amount),
                ) {
                    (Some(b), Some(s)) => (b, s),
                    _ => {
                        return CallOutcome {
                            reverted: true,
                            return_data: vec![],
                        }
                    }
                };
                self.state.balances.insert(addr, new_balance);
                self.state.total_supply = new_supply;
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            BUGGY_CREDIT => {
                let addr = addr_from_calldata(&call.calldata);
                let amount = word_to_u128(&call.calldata[36..68]);
                // Bug: credits the recipient without updating totalSupply,
                // breaking sum(balanceOf) == totalSupply().
                let current = *self.state.balances.get(&addr).unwrap_or(&0);
                let Some(new_balance) = current.checked_add(amount) else {
                    return CallOutcome {
                        reverted: true,
                        return_data: vec![],
                    };
                };
                self.state.balances.insert(addr, new_balance);
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            TOTAL_SUPPLY => CallOutcome {
                reverted: false,
                return_data: u128_to_word(self.state.total_supply),
            },
            BALANCE_OF => {
                let addr = addr_from_calldata(&call.calldata);
                let bal = *self.state.balances.get(&addr).unwrap_or(&0);
                CallOutcome {
                    reverted: false,
                    return_data: u128_to_word(bal),
                }
            }
            OWNER => {
                let mut word = vec![0u8; 32];
                word[12..32].copy_from_slice(&self.state.owner);
                CallOutcome {
                    reverted: false,
                    return_data: word,
                }
            }
            TRANSFER_OWNERSHIP => {
                // Bug: no check that call.caller == self.state.owner.
                self.state.owner = addr_from_calldata(&call.calldata);
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            SAFE_TRANSFER_OWNERSHIP => {
                if call.caller != self.state.owner {
                    return CallOutcome {
                        reverted: true,
                        return_data: vec![],
                    };
                }
                self.state.owner = addr_from_calldata(&call.calldata);
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            WITHDRAW_VULN => {
                self.last_trace = vulnerable_withdraw_trace();
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            WITHDRAW_GUARDED => {
                self.last_trace = guarded_withdraw_trace();
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            WITHDRAW_CEI => {
                self.last_trace = cei_withdraw_trace();
                CallOutcome {
                    reverted: false,
                    return_data: vec![],
                }
            }
            _ => CallOutcome {
                reverted: true,
                return_data: vec![],
            },
        }
    }

    fn snapshot(&mut self) -> u64 {
        self.snapshots.push(self.state.clone());
        (self.snapshots.len() - 1) as u64
    }

    fn revert_to(&mut self, snapshot: u64) {
        if let Some(state) = self.snapshots.get(snapshot as usize) {
            self.state = state.clone();
        }
    }

    fn last_call_trace(&self) -> &[TraceEvent] {
        &self.last_trace
    }
}
