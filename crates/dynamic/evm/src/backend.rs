//! `revm`-backed [`ExecutionBackend`] for real Solidity bytecode.
//!
//! Every call runs under the [`ReentrancyInspector`], so `last_call_trace`
//! returns the real call/storage structure of the execution — that's what
//! makes [`truent_dynamic_core::ReentrancyInvariant`] fire on actual
//! compiled contracts, not just the synthetic-trace mock proof in
//! `truent-dynamic-core`.
//!
//! UNVERIFIED OFFLINE: this crate is written where `revm` can't be fetched,
//! so the exact `revm = 14` API used here (the inspector wiring via
//! `with_external_context`/`inspector_handle_register`, `CallInputs`/
//! `Interpreter` field names) is validated by CI, not locally.

use crate::reentrancy_inspector::ReentrancyInspector;
use crate::types::CompiledContract;
use revm::db::InMemoryDB;
use revm::primitives::{
    AccountInfo, Address, Bytecode, Bytes, ExecutionResult, Output, TransactTo, U256,
};
use revm::{inspector_handle_register, Evm};
use truent_dynamic_core::{CallOutcome, EncodedCall, ExecutionBackend, TraceEvent};

/// A deployed contract instance plus the in-memory state it lives in.
/// `snapshot`/`revert_to` clone the whole DB rather than diffing it — state
/// for a single fuzzed contract under test is small, and correctness here
/// matters far more than shaving allocations off a debug tool.
pub struct RevmBackend {
    db: InMemoryDB,
    contract_address: Address,
    snapshots: Vec<InMemoryDB>,
    /// The execution trace of the most recent [`RevmBackend::call`],
    /// produced by the [`ReentrancyInspector`], for
    /// [`truent_dynamic_core::detect_reentrancy`].
    last_trace: Vec<TraceEvent>,
}

impl RevmBackend {
    /// Deploy `init_code` (Solidity creation bytecode) and return a backend
    /// ready to receive calls against the resulting contract address.
    pub fn deploy(init_code: Vec<u8>, deployer: [u8; 20]) -> anyhow::Result<Self> {
        let deployer_addr = Address::from(deployer);
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            deployer_addr,
            AccountInfo {
                balance: U256::from(u128::MAX),
                ..Default::default()
            },
        );

        let mut evm = Evm::builder()
            .with_db(db)
            .modify_tx_env(|tx| {
                tx.caller = deployer_addr;
                tx.transact_to = TransactTo::Create;
                tx.data = Bytes::from(init_code);
                tx.value = U256::ZERO;
            })
            .build();

        let result = evm
            .transact_commit()
            .map_err(|e| anyhow::anyhow!("deployment transaction failed: {e:?}"))?;

        let contract_address = match result {
            ExecutionResult::Success {
                output: Output::Create(_, Some(addr)),
                ..
            } => addr,
            ExecutionResult::Success { .. } => {
                anyhow::bail!("deployment succeeded but returned no contract address")
            }
            ExecutionResult::Revert { output, .. } => {
                anyhow::bail!("constructor reverted: 0x{}", hex::encode(output))
            }
            ExecutionResult::Halt { reason, .. } => {
                anyhow::bail!("constructor halted: {reason:?}")
            }
        };

        // Fund every actor the harness might use as a caller so `payable`
        // calls with a nonzero value don't spuriously revert on
        // insufficient balance — that's not the class of bug this engine
        // is trying to find.
        let db = evm.context.evm.db.clone();

        Ok(Self {
            db,
            contract_address,
            snapshots: Vec::new(),
            last_trace: Vec::new(),
        })
    }

    /// Deploys already-compiled *runtime* bytecode directly, without
    /// running a constructor. For contracts fetched from a live chain via
    /// `eth_getCode` — which returns runtime bytecode, not creation/init
    /// code — there is no constructor to run; the code is simply placed at
    /// the target address with an otherwise-empty account (see
    /// `truent_dynamic_evm::fuzz_deployed_contract`'s doc comment for what
    /// that does and doesn't reproduce faithfully versus the real chain).
    pub fn from_runtime_bytecode(bytecode: Vec<u8>, contract_address: [u8; 20]) -> Self {
        let address = Address::from(contract_address);
        let mut db = InMemoryDB::default();
        let mut info = AccountInfo::from_bytecode(Bytecode::new_raw(Bytes::from(bytecode)));
        info.balance = U256::from(u128::MAX);
        db.insert_account_info(address, info);

        Self {
            db,
            contract_address: address,
            snapshots: Vec::new(),
            last_trace: Vec::new(),
        }
    }

    /// Pre-fund a set of addresses so they can be used as callers/`value`
    /// senders without running out of ETH mid-fuzz.
    pub fn fund_actors(&mut self, actors: &[[u8; 20]]) {
        for actor in actors {
            self.credit(*actor, U256::from(u128::MAX));
        }
    }

    /// Set an address's balance while preserving everything else about the
    /// account.
    ///
    /// Writing a fresh `AccountInfo` also clears the nonce *and the code hash*,
    /// so funding an address that holds a contract silently deleted that
    /// contract. Any scenario needing a pre-funded contract — a bank holding
    /// other depositors' ETH, which is what makes a reentrancy drain possible
    /// — was unstageable as a result.
    pub fn credit(&mut self, address: [u8; 20], balance: U256) {
        let addr = Address::from(address);
        let mut info = revm::DatabaseRef::basic_ref(&self.db, addr)
            .ok()
            .flatten()
            .unwrap_or_default();
        info.balance = balance;
        self.db.insert_account_info(addr, info);
    }

    pub fn contract_address(&self) -> [u8; 20] {
        self.contract_address.into_array()
    }

    /// Deploy a second contract into the *same* state as the primary one
    /// (e.g. an attacker contract that needs to interact with the contract
    /// under test), returning its address. `init_code` should already have
    /// any ABI-encoded constructor arguments appended.
    pub fn deploy_secondary(
        &mut self,
        init_code: Vec<u8>,
        deployer: [u8; 20],
    ) -> anyhow::Result<[u8; 20]> {
        let deployer_addr = Address::from(deployer);

        // Top up the balance but keep the existing nonce. Overwriting the whole
        // account reset the nonce to 0, and CREATE derives the new address from
        // (deployer, nonce) — so deploying a second contract from the same
        // deployer landed on the address the first one already occupied and
        // halted with CreateCollision. That made every multi-contract scenario
        // — victim plus attacker, which is exactly the reentrancy setup —
        // impossible to stage.
        let existing_nonce = revm::DatabaseRef::basic_ref(&self.db, deployer_addr)
            .ok()
            .flatten()
            .map(|acc| acc.nonce)
            .unwrap_or(0);
        self.db.insert_account_info(
            deployer_addr,
            AccountInfo {
                balance: U256::from(u128::MAX),
                nonce: existing_nonce,
                ..Default::default()
            },
        );

        let db = std::mem::take(&mut self.db);
        let mut evm = Evm::builder()
            .with_db(db)
            .modify_tx_env(|tx| {
                tx.caller = deployer_addr;
                tx.transact_to = TransactTo::Create;
                tx.data = Bytes::from(init_code);
                tx.value = U256::ZERO;
            })
            .build();

        let result = evm
            .transact_commit()
            .map_err(|e| anyhow::anyhow!("secondary deployment failed: {e:?}"));
        self.db = evm.context.evm.db.clone();

        match result? {
            ExecutionResult::Success {
                output: Output::Create(_, Some(addr)),
                ..
            } => Ok(addr.into_array()),
            ExecutionResult::Success { .. } => {
                anyhow::bail!("secondary deployment returned no contract address")
            }
            ExecutionResult::Revert { output, .. } => {
                anyhow::bail!("secondary constructor reverted: 0x{}", hex::encode(output))
            }
            ExecutionResult::Halt { reason, .. } => {
                anyhow::bail!("secondary constructor halted: {reason:?}")
            }
        }
    }

    /// Execute a call to an arbitrary `target` address (not just the primary
    /// contract), recording its trace. The trait-level
    /// [`ExecutionBackend::call`] is this against the primary contract.
    pub fn call_address(
        &mut self,
        target: [u8; 20],
        calldata: Vec<u8>,
        caller: [u8; 20],
        value: u128,
    ) -> CallOutcome {
        let db = std::mem::take(&mut self.db);
        // Run with the reentrancy inspector as the external context so this
        // call's full call/storage structure is recorded. `transact_commit`
        // requires the inspector handler to be registered, which
        // `inspector_handle_register` does.
        let mut evm = Evm::builder()
            .with_db(db)
            .with_external_context(ReentrancyInspector::default())
            .append_handler_register(inspector_handle_register)
            .modify_tx_env(|tx| {
                tx.caller = Address::from(caller);
                tx.transact_to = TransactTo::Call(Address::from(target));
                tx.data = Bytes::from(calldata);
                tx.value = U256::from(value);
            })
            .build();

        let outcome = match evm.transact_commit() {
            Ok(ExecutionResult::Success {
                output: Output::Call(data),
                ..
            }) => CallOutcome {
                reverted: false,
                return_data: data.to_vec(),
            },
            Ok(ExecutionResult::Success { .. })
            | Ok(ExecutionResult::Revert { .. })
            | Ok(ExecutionResult::Halt { .. }) => CallOutcome {
                reverted: true,
                return_data: Vec::new(),
            },
            Err(_) => CallOutcome {
                reverted: true,
                return_data: Vec::new(),
            },
        };

        self.last_trace = evm.context.external.take_events();
        self.db = evm.context.evm.db.clone();
        outcome
    }
}

impl ExecutionBackend for RevmBackend {
    fn call(&mut self, call: &EncodedCall) -> CallOutcome {
        self.call_address(
            self.contract_address.into_array(),
            call.calldata.clone(),
            call.caller,
            call.value,
        )
    }

    fn snapshot(&mut self) -> u64 {
        self.snapshots.push(self.db.clone());
        (self.snapshots.len() - 1) as u64
    }

    fn revert_to(&mut self, snapshot: u64) {
        if let Some(state) = self.snapshots.get(snapshot as usize) {
            self.db = state.clone();
        }
    }

    fn last_call_trace(&self) -> &[TraceEvent] {
        &self.last_trace
    }
}

/// Builds a fresh, already-deployed [`RevmBackend`] for one fuzz attempt.
/// `truent_dynamic_core::fuzz` calls this many times (once per attempt,
/// more during shrinking) — each call redeploys from the same creation
/// bytecode so every attempt starts from an identical genesis state.
pub fn backend_factory<'a>(
    contract: &'a CompiledContract,
    actors: &'a [[u8; 20]],
) -> impl Fn() -> Box<dyn ExecutionBackend> + 'a {
    move || {
        let deployer = actors.first().copied().unwrap_or([0xAAu8; 20]);
        let mut backend = RevmBackend::deploy(contract.init_code.clone(), deployer)
            .expect("deployment must succeed against known-good bytecode compiled moments earlier");
        backend.fund_actors(actors);
        Box::new(backend)
    }
}
