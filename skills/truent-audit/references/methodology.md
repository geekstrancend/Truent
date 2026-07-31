# Auditor methodology (how the LLM lenses must think)

Pattern-matching finds the bugs a detector already encodes — Truent's engine
already ran those in Phase 1. Your job in Phase 2 is the *residual*: the bugs
that only exist in this protocol's specific logic. Those come from **how you
reason**, not from a checklist. Three disciplines, applied continuously — not
once, not in order. Reach for the one the moment its trigger fires.

## 1. Feynman — explain it in plain words (trigger: opening any function)

The moment you open a function, stop and explain what it does to someone who
has never seen Solidity — no jargon, no `transferFrom`, no `mload`. If your
explanation stays crisp, you understand it. The instant your wording goes fuzzy
or you reach for a technical term to stay accurate, **that fuzzy spot is an
unexamined assumption** — and unexamined assumptions are where bugs live.

> `_handleFee(token, fee)` — not "it transfers the fee" (that's jargon hiding
> the logic) but "it takes the protocol's cut off the user's payment and moves
> it to the treasury." Now: what if the payment was in ETH and this uses an
> ERC-20 method? Your plain-English story breaks. That break is the bug.

## 2. Socratic — drill every "why" to the assumption (trigger: a line whose purpose isn't obvious)

For each line: *why is this here? what does it assume? what happens if the
assumption is false?* Never accept "because that's how it's written" or "the
function name says so." The first answer is a restatement; the real assumption
is two or three "whys" deeper.

> `if (token != ETH) IERC20(token).transferFrom(msg.sender, address(this), amt);`
> — Why the `!= ETH` guard? Because ETH can't move via `transferFrom`. Why no
> `else`? Because the dev assumed ETH arrives via `msg.value`. Where is
> `msg.value == amt` enforced on the ETH path? **Nowhere.** Bug.

## 3. Inversion — attack every clean path (trigger: a path/guard that looks correct)

After you understand what the code is *supposed* to do, turn around and ask how
to make it *not* do that. Same code, attacker's eye. Read every check and ask
"what value slips past it?" Read every state write and ask "what state am I in
just before this?" A senior auditor never reads code only forward.

## The Truent turn — reason to a *property*, then let the engine break it

This is where you diverge from a prompt-only auditor. When your reasoning lands
on a suspected bug, don't stop at prose. Ask: **what invariant does this
violate?** State it as something a machine can check —

- a conservation law (`sum(balanceOf) == totalSupply`),
- a monotone quantity (a share price / index that must never decrease),
- an access rule (only `owner()` may change `owner()`),
- a round-trip that must not profit (`deposit(x); withdraw(all) ≤ x`).

Then hand it to the engine (see `verification.md`). If the fuzzer reproduces a
violation, your `REASONED` lead becomes a `VERIFIED` finding with a runnable
PoC. If the fuzzer can't break it after a real search, **that is signal** — you
were probably wrong; downgrade or drop. The machine is the tie-breaker, not
your confidence.

## Rules of engagement

- **You are an attacker, never a defender.** When you find a bug, deepen it —
  chain it, find more victims, lower the precondition cost. Do not argue
  yourself out of a real bug.
- **Proof or it's a lead.** A finding needs concrete values, a trace, or (best)
  an engine PoC. Without that, it's a `REASONED` lead — say so honestly. Leads
  are calibration, not failures.
- **Weaponize across the codebase.** A bug found in one contract → search every
  other contract for the same shape by name *and* by pattern. Missing a repeat
  is an audit failure.
- **Trust your discomfort.** When a path feels too clean or a conclusion comes
  too fast, that's the trigger. Reach for the tool. Don't stop until the
  discomfort has a name.
