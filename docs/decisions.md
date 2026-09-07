# Decision Log (ADR — Architecture Decision Records)

This file records significant design decisions made during the development of Rayos Wallet Contracts. Each entry explains the **context**, the **decision**, and the **consequences** so future contributors understand *why* things are the way they are.

---

## ADR-001 — Use a separate Policy contract instead of embedding rules in the Wallet

**Date**: 2026-09  
**Status**: Accepted

### Context
We needed to support spend limits, session keys, and guardian recovery. The question was whether these rules should live inside the Wallet contract or a separate Policy contract.

### Decision
Rules are in a separate `Policy` contract. The Wallet stores a `PolicyAddress` and delegates rule enforcement to it.

### Consequences
- ✅ Policy can be swapped without redeploying the Wallet
- ✅ Multiple wallets can share a single Policy (e.g. team accounts)
- ✅ Wallet contract stays small and focused on identity
- ⚠️ Cross-contract calls add latency and gas cost
- ⚠️ Wallet must trust the Policy — if Policy is malicious, funds are at risk (mitigated: only owner can call `set_policy`)

---

## ADR-002 — Store all signers in a single Map in instance storage

**Date**: 2026-09  
**Status**: Accepted

### Context
Soroban persistent storage does not support native iteration. We needed to support `get_signers()` as a list, and fast lookup during `__check_auth`.

### Decision
All signers are stored as a single `Map<Bytes, Bytes>` (credential_id → public_key) in **instance storage** under the key `DataKey::Signers`.

### Consequences
- ✅ `get_signers()` is trivial — return the full map
- ✅ Auth lookup is O(map_size) — fine for 1–10 devices
- ✅ Single TTL bump covers all signers
- ⚠️ Instance storage has a size cap — impractical for >~50 signers. Out-of-scope for consumer wallets.
- ⚠️ Any mutation (add/remove) rewrites the full map entry

---

## ADR-003 — Use `wasm32v1-none` target (not `wasm32-unknown-unknown`)

**Date**: 2026-09  
**Status**: Accepted

### Context
Soroban SDK 27 with Rust 1.82+ requires `wasm32v1-none`. `wasm32-unknown-unknown` on newer Rust versions enables WASM features (`reference-types`, `multi-value`) not yet supported by the Soroban host.

### Decision
Pin the WASM build target to `wasm32v1-none` in `rust-toolchain.toml` and `.cargo/config.toml`.

### Consequences
- ✅ Builds are compatible with Soroban Protocol 27 host
- ✅ CI enforces this via the `wasm-size-check` job
- ⚠️ Requires Rust ≥ 1.84 (available since early 2025)

---

## ADR-004 — `Hash<32>` type for `__check_auth` payload

**Date**: 2026-09  
**Status**: Accepted

### Context
We initially tried exposing `check_session` in the Policy with a `Hash<32>` parameter so the wallet could forward the payload. The Soroban SDK refuses this — `Hash<32>` can only come from host-provided sources.

### Decision
The Policy's `get_session` returns the `SessionKey` metadata. The Wallet handles `secp256r1_verify` directly using its own `Hash<32>` from the host. The Policy is not involved in signature verification at auth time.

### Consequences
- ✅ Stronger security boundary — Policy cannot forge verification payloads
- ✅ Wallet retains full cryptographic control
- ⚠️ Policy is not called from inside `__check_auth` (planned for CAP-0071 integration in v2)

---

## ADR-005 — Configurable recovery threshold with a 2-of-3 default

**Date**: 2026-09  
**Status**: Accepted

### Context
Guardian recovery needs to balance security (enough approvals to prevent single-guardian attacks) and usability (not so many approvals that recovery becomes impossible).

### Decision
- Default threshold: `2` (2-of-M)
- Default configuration: `2-of-3` (owner sets up 3 guardians, 2 must approve)
- Configurable: owner can set any threshold via `set_recovery_threshold`
- No enforced minimum on-chain (the contract trusts the owner to set a sensible value)

### Consequences
- ✅ Common-case (2-of-3) is safe and practical
- ✅ Power users can configure more guardians / higher thresholds
- ⚠️ No enforced floor means an owner could set threshold=1 (single-guardian attack surface). Documented in threat model.

---

## ADR-006 — 48-hour timelock on testnet, 7-day on mainnet

**Date**: 2026-09  
**Status**: Accepted

### Context
A recovery timelock gives the real owner time to notice and cancel a malicious recovery attempt. The optimal duration trades security (longer = more time to react) vs. usability (shorter = faster recovery after genuine loss).

### Decision
- Testnet default: `172800` seconds (48 hours) — fast enough for testing
- Mainnet recommendation: `604800` seconds (7 days) — documented in `MIGRATION.md`
- The value is configurable per-policy via `set_recovery_timelock`

### Consequences
- ✅ Industry standard 7-day delay for mainnet matches leading social recovery wallets (Argent, Safe)
- ✅ Short testnet delay allows rapid iteration
- ⚠️ Must be documented clearly so users understand they have 7 days to object

---

## ADR-007 — Deterministic wallet addresses via `deployer.with_current_contract(salt)`

**Date**: 2026-09  
**Status**: Accepted

### Context
Users need to know their wallet address before they deploy it (so they can receive funds, share it in advance, etc.). Traditional factories require a deployment transaction first.

### Decision
Use Soroban's `env.deployer().with_current_contract(salt).deployed_address()` which computes `address = SHA-256(factory_id || salt)` deterministically.

### Consequences
- ✅ `predict_address(salt)` is a pure read-only call — no fees, no state
- ✅ "Fund before activate" UX is trivially supported
- ✅ No on-chain registry needed
- ⚠️ The salt must be unique per wallet. We recommend using a hash of the user's WebAuthn credential ID as the salt.
