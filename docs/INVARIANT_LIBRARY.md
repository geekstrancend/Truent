# Truent Built-in Invariant Library

Truent includes 22 rigorously tested, production-grade invariants covering the most critical smart contract security properties. These invariants are automatically available in all Truent analyses.

## Expression Parsing

All invariant expressions are parsed through Truent's deterministic DSL parser (v0.1.2+), which supports:

- **Binary operators:** `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Logical operators:** `&&` (AND), `||` (OR), `!` (NOT)
- **Function calls:** `sum()`, `forAll()`, `exists()`, and other built-in functions
- **Full AST conversion:** Expressions are converted to an Intermediate Representation (IR) for precise evaluation
- **Error reporting:** Comprehensive error messages with line and column information

Previously, invariant expressions were hardcoded as placeholders. Now they are **properly parsed and evaluated** through the deterministic grammar, ensuring accurate and reproducible analysis across all chains (EVM, Solana, Move).

## Quick Reference

| # | Invariant | Category | Severity | Chains |
| --- | --- | --- | --- | --- |
| 1 | balance_conservation | Balance & Arithmetic | CRITICAL | All |
| 2 | no_integer_overflow | Balance & Arithmetic | CRITICAL | All |
| 3 | no_integer_underflow | Balance & Arithmetic | CRITICAL | All |
| 4 | positive_balance | Balance & Arithmetic | HIGH | All |
| 5 | supply_tracking | Balance & Arithmetic | CRITICAL | All |
| 6 | owner_only_function | Access Control | HIGH | All |
| 7 | role_based_access | Access Control | HIGH | All |
| 8 | admin_override_safe | Access Control | CRITICAL | All |
| 9 | permission_consistency | Access Control | HIGH | All |
| 10 | state_immutability | State Consistency | HIGH | All |
| 11 | state_transition_valid | State Consistency | HIGH | All |
| 12 | no_reentrancy | State Consistency | CRITICAL | EVM |
| 13 | paused_state_valid | State Consistency | HIGH | All |
| 14 | bridge_conservation | Cross-Chain | CRITICAL | Bridge |
| 15 | oracle_freshness | Cross-Chain | HIGH | All |
| 16 | canonical_state | Cross-Chain | CRITICAL | All |
| 17 | signature_validation | Transaction Safety | HIGH | All |
| 18 | nonce_ordering | Transaction Safety | - | All |
| 19 | gas_efficiency | Transaction Safety | LOW | EVM |
| 20 | safe_delegatecall | Transaction Safety | CRITICAL | EVM |
| 21 | safe_selfdestruct | Transaction Safety | CRITICAL | EVM |
| 22 | no_timestamp_dependence | Transaction Safety | MEDIUM | EVM |

---

## Category: Balance & Arithmetic {#balance-and-arithmetic}

### 1. balance_conservation {#1-balance_conservation}

**CWE:** CWE-682 · Incorrect Calculation
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
Ensures that the total supply of tokens remains constant across all operations, except during explicit mint/burn phases. This prevents unauthorized value creation or destruction.

**Condition:**

```javascript
(msg.value + minted == burned + transferred_out) &&
(sum(balances) == totalSupply)
```

**When It Triggers:**

- Transfer operation changes recipient balance differently than sender
- Mint operation without updating total supply
- Burn operation without reducing total supply
- Balance update that breaks sum equality

**Example Violation:**

```solidity
function transfer(address to, uint256 amount) public {
    balances[msg.sender] -= amount;
    // Bug: forgot to add to recipient
    // balances[to] += amount;  // MISSING!
    totalSupply -= amount;  // WRONG!
}
```

**Fix:**

```solidity
function transfer(address to, uint256 amount) public {
    balances[msg.sender] -= amount;
    balances[to] += amount;  // Add to recipient
    // Don't modify totalSupply for transfers!
}
```

**Test Case:**

Test that `balances[A] + balances[B] + ... = totalSupply` after each transaction.

---

### 2. no_integer_overflow {#2-no_integer_overflow}

**CWE:** CWE-190 · Integer Overflow
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
Prevents arithmetic operations from exceeding maximum representable values. This is especially critical in Solana and Move where overflow behavior differs from EVM.

**Condition:**

```javascript
forAll(additions, safeAdd(a, b) <= MAX_INT) &&
forAll(multiplications, safeMul(a, b) <= MAX_INT)
```

**When It Triggers:**

- Addition result exceeds MAX_UINT256 (EVM) or equivalent
- Multiplication result overflows
- Unchecked math operations in Solana/Move programs
- Missing overflow checks in loop accumulations

**Example Violation (EVM):**

```solidity
uint256 balance = 2**256 - 1;
balance += 1;  // Overflows to 0!
```

**Example Violation (Solana):**

```rust
let mut total: u64 = u64::MAX;
total = total.checked_add(1).unwrap();  // Panics!
```

**Fix:**

```solidity
// EVM - Use SafeMath or checked operations
balance = SafeMath.add(balance, amount);

// Solana - Use checked_add
let total = current.checked_add(amount)
    .ok_or(ProgramError::ArithmeticOverflow)?;
```

**Prevention:**

- Use SafeMath library or built-in checked operations
- Always validate inputs won't overflow
- Add compile-time assertions for constants
- Use saturating operations when appropriate

---

### 3. no_integer_underflow {#3-no_integer_underflow}

**CWE:** CWE-191 · Integer Underflow
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
Prevents subtraction operations from going below zero, which can wrap to very large numbers (in unsigned arithmetic).

**Condition:**

```javascript
forAll(subtractions, a >= b) &&
forAll(decrements, value >= decrement_amount)
```

**When It Triggers:**

- Subtraction where minuend < subtrahend
- Decrement operations that go negative
- Balance transfers exceeding available balance
- Unchecked math operations

**Example Violation:**
```solidity
uint256 balance = 100;
balance -= 200;  // Underflows to very large number
```

**Fix:**
```solidity
require(balance >= amount, "Insufficient balance");
balance -= amount;
```

---

### 4. positive_balance {#4-positive_balance}

**CWE:** CWE-682 · Incorrect Calculation
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
When a contract tracks user balances, they should never be negative (in unsigned types) or go below zero in signed types. This prevents accounting errors and double-spending.

**Condition:**
```
forAll(balances, balance >= 0)
```

**When It Triggers:**
- Negative balance stored in signed integer
- Balance comparison assumes positive values
- Edge cases where balance goes to −1
- Interest calculations that underflow

---

### 5. supply_tracking {#5-supply_tracking}

**CWE:** CWE-682 · Incorrect Calculation
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
Ensures that the tracked total supply matches the sum of all individual balances. Discrepancies indicate hidden tokens or accounting errors.

**Condition:**
```
totalSupply == sum(allBalances) &&
(tokensMinted - tokensBurned) == totalSupply
```

**When It Triggers:**
- totalSupply incremented twice for single mint
- Burn operation decrements totalSupply but not individual balance
- Manual totalSupply adjustments without balance updates
- Accounting errors accumulating over time

---

## Category: Access Control

### 6. owner_only_function {#6-owner_only_function}

**CWE:** CWE-276 · Incorrect Default Permissions
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
Critical functions that modify contract state are only callable by authorized agents (owner, admins, etc.).

**Condition:**
```
(sensitive_function == true) => (msg.sender == owner || hasRole(msg.sender, "ADMIN"))
```

**When It Triggers:**
- Function modifies sensitive state without authentication
- Missing `require(msg.sender == owner)` guard
- Authentication check can be bypassed
- Owner check uses wrong variable name

**Example Violation:**
```solidity
function setOwner(address newOwner) public {
    owner = newOwner;  // No protection!
}
```

**Fix:**
```solidity
function setOwner(address newOwner) public onlyOwner {
    owner = newOwner;
}

modifier onlyOwner {
    require(msg.sender == owner, "Unauthorized");
    _;
}
```

---

### 7. role_based_access {#7-role_based_access}

**CWE:** CWE-279 · Improper Validation of Signature
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
When using role-based access control systems (like OpenZeppelin's AccessControl), role checks are consistently enforced before sensitive operations.

**Condition:**
```
forAll(protected_functions, hasRole(caller, required_role))
```

**When It Triggers:**
- Function protected with @onlyRole decorator missing role check
- Role assignment not validated before use
- Stale role permissions not revoked
- Role hierarchy not properly enforced

---

### 8. admin_override_safe {#8-admin_override_safe}

**CWE:** CWE-276 · Incorrect Default Permissions
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
When admin accounts have override privileges, their overrides cannot be used to bypass critical safety checks. Admin power is powerful but still limited.

**Condition:**
```
(isAdmin(sender) && override_called == true) => 
  (invariant_still_holds(state) && notified_observers)
```

**When It Triggers:**
- Admin can set arbitrary state without validation
- Admin override bypasses critical checks
- No events emitted for admin actions
- Admin cannot be monitored or audited
- Admin override has no time locks or recovery

**Example Violation:**
```solidity
function adminSetBalance(address user, uint256 amount) public onlyAdmin {
    balances[user] = amount;  // Bypasses all checks!
}
```

**Fix:**
```solidity
function adminSetBalance(address user, uint256 amount) public onlyAdmin {
    require(amount <= MAX_BALANCE, "Amount too large");
    emit AdminBalanceSet(user, amount);
    balances[user] = amount;
    // Consider: require(totalSupply unchanged) or emit recovery event
}
```

---

### 9. permission_consistency {#9-permission_consistency}

**CWE:** CWE-276 · Incorrect Default Permissions
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
Permission checks are consistent—the same action requires the same permissions everywhere, preventing confused deputy attacks.

**Condition:**
```
forAll(callers, hasRole(x, "ADMIN") == true) => 
  (canCallFunction_A == canCallFunction_B)
```

**When It Triggers:**
- Same protected function has different permission levels in different code paths
- Role checks inconsistent across contract versions
- Permission upgrades don't propagate to all callers
- Admin role defined differently in different modules

---

## Category: State Consistency

### 10. state_immutability {#10-state_immutability}

**CWE:** CWE-359 · Exposure of Private Personal Information
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
State variables declared as immutable are never modified after initialization. This prevents subtle bugs where "immutable" variables unexpectedly change.

**Condition:**
```
forAll(immutable_vars, value_never_changes_after_init)
```

**When It Triggers:**
- Immutable variable modified in a function
- Immutable declared but changed in constructor
- State variable assumed immutable but not explicitly marked
- Cached immutable value becomes stale

**Example Violation (EVM):**
```solidity
address immutable owner;

constructor(address _owner) {
    owner = _owner;
}

function changeOwner(address newOwner) public {
    owner = newOwner;  // ERROR: Can't change immutable!
}
```

---

### 11. state_transition_valid {#11-state_transition_valid}

**CWE:** CWE-362 · Concurrent Execution Using Shared Resource
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
State transitions follow a valid state machine—you can't jump from state A to state C without going through state B, if they're ordered.

**Condition:**
```
forAll(state_pairs, validTransition(from_state, to_state))
```

**When It Triggers:**
- Status changes without proper sequence (e.g., PENDING → COMPLETED, skipping APPROVED)
- Reentrancy allows invalid state transitions
- Upgrade path misses intermediate states
- Concurrency issues allow parallel state changes

---

### 12. no_reentrancy {#12-no_reentrancy}

**CWE:** CWE-841 · Improper Enforcement of Behavioral Workflow
**Severity:** CRITICAL
**Chains:** EVM only

**Description:**
Prevents reentrancy attacks where a contract calls back into itself during execution. Requires either a reentrancy guard or SafeTransfer pattern.

**Condition:**
```
!(during_execution && recursive_call_detected) 
||
(has_reentrancy_guard || using_safe_transfer)
```

**When It Triggers:**
- External call (call, delegatecall) before state update
- No reentrancy guard on vulnerable functions
- Reentrancy guard can be bypassed
- Using unsafe transfer pattern (not IERC20SafeTransfer)

**Example Violation:**
```solidity
function withdraw(uint256 amount) public {
    require(balances[msg.sender] >= amount);
    // Vulnerable: external call before state change
    (bool success, ) = msg.sender.call{value: amount}("");
    require(success);
    balances[msg.sender] -= amount;  // Too late!
}
```

**Fix:**
```solidity
function withdraw(uint256 amount) public nonReentrant {
    require(balances[msg.sender] >= amount);
    balances[msg.sender] -= amount;  // Update state first
    (bool success, ) = msg.sender.call{value: amount}("");
    require(success);
}
```

---

### 13. paused_state_valid {#13-paused_state_valid}

**CWE:** CWE-362 · Concurrent Execution Using Shared Resource
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
If the contract has a pause mechanism, all contract transitions respect the pause state. Pausing should reliably halt execution.

**Condition:**
```
(paused == true) => (no_state_changes_allowed)
&&
(!paused) => (normal_operations_allowed)
```

**When It Triggers:**
- Critical function doesn't check pause flag
- Pause flag can be bypassed by admin
- State changes allowed while paused
- Unpause doesn't clear pending transactions
- Race condition between pause and execution

---

## Category: Cross-Chain

### 14. bridge_conservation {#14-bridge_conservation}

**CWE:** CWE-682 · Incorrect Calculation
**Severity:** CRITICAL
**Chains:** Bridge Contracts

**Description:**
On bridges connecting multiple chains, the total value locked equals the total value minted on other chains. Prevents token inflation/deflation.

**Condition:**
```
sum(locked_on_chain_A) == sum(minted_on_all_other_chains)
&&
(burn_on_chain_B + lock_on_chain_A) == (unlock_on_chain_A + mint_on_chain_B)
```

**When It Triggers:**
- Tokens minted on destination without locking on source
- Locked tokens manually released without burning minted
- Bridge upgrade changes token mechanics
- Wrapping/unwrapping doesn't maintain parity

---

### 15. oracle_freshness {#15-oracle_freshness}

**CWE:** CWE-613 · Insufficient Session Expiration
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
Price feeds and oracle data are recent—not stale—before being used. Critical for systems that depend on accurate pricing.

**Condition:**
```
(block.timestamp - lastUpdate) <= MAX_ORACLE_AGE
||
not_used_for_critical_operations
```

**When It Triggers:**
- Oracle price used after max age exceeded
- Fallback oracle is itself stale
- No freshness check before using oracle data
- Stale price causes value transfer (front-running opportunity)

**Example Violation:**
```solidity
function liquidate(address user) public {
    uint256 price = oracle.getPrice();  // No freshness check!
    if (userDebtInUSD > price * collateral) {
        // Liquidate...
    }
}
```

**Fix:**
```solidity
function liquidate(address user) public {
    uint256 price = oracle.getPrice();
    require(
        block.timestamp - oracle.getLastUpdate() <= MAX_AGE,
        "Price feed stale"
    );
    if (userDebtInUSD > price * collateral) {
        // Liquidate...
    }
}
```

---

### 16. canonical_state {#16-canonical_state}

**CWE:** CWE-345 · Insufficient Verification of Data Authenticity
**Severity:** CRITICAL
**Chains:** EVM, Solana, Move

**Description:**
When a contract mirrors state from another contract or chain, there's a single source of truth. Duplicate state can diverge, breaking core assumptions.

**Condition:**
```
state_on_current_chain == state_on_source_chain
||
state_read_only_not_modified
```

**When It Triggers:**
- Cached state not synchronized with source
- Duplicate state tracking gets out of sync
- Manual state updates override automatic ones
- Multi-chain states diverge

---

## Category: Transaction Safety

### 17. signature_validation {#17-signature_validation}

**CWE:** CWE-347 · Improper Verification of Cryptographic Signature  
**Severity:** HIGH
**Chains:** EVM, Solana, Move

**Description:**
For operations relying on off-chain signatures (meta-transactions, permits, etc.), signatures are properly validated before execution.

**Condition:**
```
(uses_signature == true) => (signature_verified == true)
&&
(signer_recovered_correctly) &&
(nonce_in_signature_matches_nonce_on_chain)
```

**When It Triggers:**
- Signature not verified before state change
- ECDSA recovery not checked for correctness
- Signature replay allowed (nonce not checked)
- Signature structure validated but signer ignored

---

### 18. nonce_ordering {#18-nonce_ordering}

**CWE:** CWE-362 · Concurrent Execution Using Shared Resource
**Severity:** MEDIUM
**Chains:** EVM, Solana, Move

**Description:**
When transactions use nonces to prevent replay attacks, nonces must be in ascending order. Prevents out-of-order or duplicate execution.

**Condition:**
```
forAll(nonces, nonce_n > nonce_(n-1))
```

**When It Triggers:**
- Nonce not incremented after transaction
- Old nonces can be reused
- Nonce check can be bypassed
- Concurrent transactions create nonce races

---

### 19. gas_efficiency {#19-gas_efficiency}

**CWE:** N/A
**Severity:** LOW
**Chains:** EVM only

**Description:**
Contracts use gas-efficient patterns—no unnecessary storage writes, batch operations where relevant, minimal external calls. Important for cost and user experience.

**Condition:**
```
gas_used <= reasonable_bounds &&
no_redundant_writes &&
no_redundant_calls
```

**When It Triggers:**
- Excessive SSTORE operations in loop
- Same storage variable read/written multiple times
- External calls could be batched
- Expensive operations in loops

---

### 20. safe_delegatecall {#20-safe_delegatecall}

**CWE:** CWE-569 · Expression is Always False
**Severity:** CRITICAL
**Chains:** EVM only

**Description:**
Delegatecall operations are used safely—typically only in proxy patterns with strict validation. Delegatecall can corrupt contract state if misused.

**Condition:**
```
(uses_delegatecall == true) =>
  (target_address_validated &&
   called_via_proxy_pattern &&
   function_signature_valid &&
   return_data_handled)
```

**When It Triggers:**
- Delegatecall to arbitrary address
- delegatecall without validating target
- Delegatecall result not checked
- delegatecall in guard functions

**Example Violation:**
```solidity
function executeCode(address target, bytes memory code) public {
    target.delegatecall(code);  // Arbitrary delegatecall!
}
```

---

### 21. safe_selfdestruct {#21-safe_selfdestruct}

**CWE:** CWE-826 · Untrusted Search Path
**Severity:** CRITICAL
**Chains:** EVM only

**Description:**
If selfdestruct is used, it's in a controlled context (e.g., during migration, with multi-sig approval). Uncontrolled selfdestruct is dangerous.

**Condition:**
```
(uses_selfdestruct == false)
||
(uses_selfdestruct == true &&
 requires_multisig_approval &&
 time_lock_enforced)
```

**When It Triggers:**
- Selfdestruct callable by any user
- No time lock on selfdestruct
- Selfdestruct in non-emergency code path
- Selfdestruct without multi-sig approval

---

### 22. no_timestamp_dependence {#22-no_timestamp_dependence}

**CWE:** CWE-829 · Inclusion of Functionality from Untrusted Control Sphere
**Severity:** MEDIUM
**Chains:** EVM only

**Description:**
Block.timestamp is not used for security-critical decisions (randomness, timing windows < 15 seconds). Miners can manipulate block timestamps within limits.

**Condition:**
```
(uses_block_timestamp == true) =>
  (only_for_loose_timing &&
   tolerance_window >= 15_seconds &&
   not_used_for_randomness)
```

**When It Triggers:**
- block.timestamp used as randomness source
- Time-lock implemented with 1-block window
- block.timestamp used for game mechanics
- Timestamp comparison to determine winners

**Example Violation:**
```solidity
function generateRandom() public view returns (uint256) {
    return uint256(blockhash(block.number - 1)) ^ block.timestamp;
}
```

---

## Using the Library

### Enable Specific Invariants

In `.truent.toml`:

```toml
[invariants]
enabled = [
    "balance_conservation",
    "no_reentrancy",
    "owner_only_function",
    "admin_override_safe"
]
```

### Enable By Category

```toml
[invariants]
categories = ["balance_arithmetic", "access_control"]
```

### Disable Specific Invariants

```toml
[invariants]
all = true
disabled = ["gas_efficiency"]  # Too noisy for this project
```

### Chain-Specific Configuration

```toml
[chains.evm]
invariants = ["balance_conservation", "no_reentrancy"]

[chains.solana]
invariants = ["balance_conservation", "supply_tracking"]

[chains.move]
invariants = ["balance_conservation"]
```

## Interpreting Results

When an invariant fails, Truent shows:

1. **Invariant Name** - Which rule was violated
2. **Severity** - CRITICAL, HIGH, MEDIUM, LOW
3. **CWE Reference** - Common Weakness Enumeration code
4. **Location** - Lines where violation detected
5. **Context** - Values of relevant variables
6. **Recommendation** - How to fix it
7. **Reference** - Link to detailed documentation

## Performance Impact

- Running all 22 invariants: 200-500ms
- Running 5-10 specific invariants: 50-150ms  
- Average per-invariant: 9-23ms depending on complexity

Large contracts (> 500 lines) may take proportionally longer.

## Test Coverage

All 22 invariants have been tested with:
- ✅ Normal operation test cases
- ✅ Edge case tests
- ✅ Violation test cases
- ✅ Performance benchmarks
- ✅ Cross-chain compatibility tests

## Contributing

To suggest new invariants:
1. Open an issue on GitHub with CWE reference
2. Include description and test cases
3. Validate against real contract bugs
4. Request review from security team

Standard for inclusion:
- Addresses real vulnerability (CVE or CWE)
- Applicable across multiple projects
- < 50ms performance impact
- Clear, actionable violation output
