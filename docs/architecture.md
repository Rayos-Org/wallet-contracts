# Architecture Deep Dive

A developer-oriented guide to the design decisions, module boundaries, cross-contract communication patterns, and storage layout of the Rayos Wallet Contracts.

---

## Table of Contents

- [Design Philosophy](#design-philosophy)
- [Contract Boundaries](#contract-boundaries)
- [Module Breakdown](#module-breakdown)
- [Cross-Contract Communication](#cross-contract-communication)
- [Storage Layout](#storage-layout)
- [Authentication Model](#authentication-model)
- [TTL & State Persistence](#ttl--state-persistence)
- [Key Design Decisions](#key-design-decisions)

---

## Design Philosophy

The core design principle is **separation of concerns**:

```
Wallet   = Identity (who can sign)
Policy   = Rules   (what they can do)
Factory  = Registry (how wallets are created)
```

This means:
- The Wallet never encodes business logic — it delegates to Policy.
- The Policy is upgradeable independently (swap it with `wallet.set_policy(new_policy)`).
- The Factory is the only contract that can deploy Wallets, ensuring consistent initialisation.

---

## Contract Boundaries

```
┌─────────────────────────────────────────────────────┐
│                     Soroban Host                    │
│  (secp256r1_verify, invoker auth, ledger timestamp) │
└───────────────────┬─────────────────────────────────┘
                    │
         ┌──────────▼──────────┐
         │   Factory Contract   │
         │                      │
         │  Holds: WasmHash     │
         │  Deploys: Wallet     │
         │  Predicts: Address   │
         └──────────┬───────────┘
                    │ deploys + calls init()
         ┌──────────▼──────────┐
         │   Wallet Contract    │◄── CustomAccountInterface
         │                      │
         │  Holds: Signers map  │
         │  Holds: PolicyAddr   │
         │  Verifies: passkeys  │
         └──────────┬───────────┘
                    │ calls check_spend / check_allow_list / get_session
         ┌──────────▼──────────┐
         │   Policy Contract    │
         │                      │
         │  Spend limits        │
         │  Session keys        │
         │  Allow-lists         │
         │  Guardian recovery   │
         └─────────────────────┘
```

### Isolation invariants

| Invariant | Enforcement |
|---|---|
| Factory cannot call Wallet after init | No stored reference back to Factory |
| Only Policy can call `recover_signer` | `policy_address.require_auth()` check |
| Only owner can mutate Policy settings | `owner.require_auth()` in every setter |
| Wallet cannot brick itself | `CannotRemoveLastSigner` guard |
| Policy errors propagate cleanly to Wallet | Typed `ContractError` via `contracterror` macro |

---

## Module Breakdown

### Wallet Contract (`contracts/wallet/src/`)

```
lib.rs        — Public entrypoints & CustomAccountInterface impl
├── auth.rs   — WebAuthnSignature struct + secp256r1_verify wrapper
├── signers.rs— CRUD for Map<credential_id → public_key>
├── storage.rs— DataKey enum, TTL constants
├── errors.rs — ContractError enum
└── test.rs   — Unit tests
```

**Data flow for `__check_auth`**:

```
Host calls __check_auth(payload, WebAuthnSignature{cred_id, sig})
  → auth::verify_signature(env, payload, signature)
      → signers::get_signers(env)           // load Map from instance storage
      → look up cred_id → public_key
      → env.crypto().secp256r1_verify(pk, payload, sig)
  → Ok(()) or ContractError
```

### Factory Contract (`contracts/factory/src/`)

```
lib.rs        — deploy_wallet, predict_address
├── storage.rs— DataKey::WasmHash
└── test.rs   — Unit tests
```

**Deterministic address derivation**:  
`address = SHA-256(factory_contract_id || salt)` — computed entirely by the Soroban host via `env.deployer().with_current_contract(salt).deployed_address()`. No randomness, no nonces.

### Policy Contract (`contracts/policy/src/`)

```
lib.rs            — All public entrypoints + owner auth guard
├── spend_limits.rs— Rolling accumulator logic
├── session_keys.rs— Session CRUD + expiry check
├── allow_lists.rs — Per-target allow/deny flags
├── guardians.rs   — N-of-M proposal/approve/execute
├── storage.rs     — DataKey, SpendLimit, SessionKey, RecoveryProposal
├── errors.rs      — ContractError enum
└── test.rs        — Unit tests + proptest fuzz suite
```

---

## Cross-Contract Communication

### Factory → Wallet

```rust
// deploy_wallet in factory/src/lib.rs
let wallet_address = env.deployer()
    .with_current_contract(salt)
    .deploy(wasm_hash);

env.invoke_contract::<()>(
    &wallet_address,
    &Symbol::new(&env, "init"),
    (credential_id, public_key).into_val(&env),
);
```

The Factory deploys and immediately calls `init`. Since `init` only allows being called once, this is safe and atomic.

### Policy → Wallet (Recovery)

```rust
// execute_recovery in policy/src/guardians.rs
env.invoke_contract::<()>(
    &wallet,
    &Symbol::new(env, "recover_signer"),
    (proposal.new_credential_id, proposal.new_public_key).into_val(env),
);
```

`recover_signer` on the Wallet enforces `policy_address.require_auth()`. Because this call originates from the Policy contract, the Soroban auth system considers the Policy as the invoker and the check passes.

### Wallet → Policy (Auth-time checks)

Currently the `__check_auth` in the Wallet does not yet call into the Policy at auth time (the full integration is planned in the CAP-0071 v2 refactor). The Policy is called explicitly by the dApp or relayer to pre-check limits before constructing transactions.

---

## Storage Layout

### Wallet — Instance Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::Signers` | `Map<Bytes, Bytes>` | All registered `credential_id → public_key` entries |
| `DataKey::PolicyAddress` | `Address` | The active Policy contract for this wallet |

All stored in **instance storage** — lives and dies with the contract instance. TTL is bumped to 30 days on every mutation.

### Factory — Instance Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::WasmHash` | `BytesN<32>` | SHA-256 hash of the Wallet WASM to deploy |

### Policy — Instance Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::Owner` | `Address` | Owner of this policy |
| `DataKey::AllowListEnabled` | `bool` | Whether allow-list filtering is active |
| `DataKey::RecoveryThreshold` | `u32` | N in N-of-M |
| `DataKey::RecoveryTimelock` | `u64` | Delay in seconds |
| `DataKey::ProposalCounter` | `u64` | Auto-incrementing proposal ID |

### Policy — Persistent Storage

| Key | Type | Description |
|---|---|---|
| `DataKey::SpendLimit(token)` | `SpendLimit` | Rolling cap state per token |
| `DataKey::SessionKey(session_id)` | `SessionKey` | Session key metadata |
| `DataKey::AllowList(address)` | `bool` | Per-contract allow/deny flag |
| `DataKey::Guardian(address)` | `bool` | Guardian membership flag |
| `DataKey::RecoveryProposal(id)` | `RecoveryProposal` | Full proposal state |

Persistent storage entries are also TTL-bumped on every access (30-day rolling window).

---

## Authentication Model

### Native Passkey Auth (`__check_auth`)

Soroban's `CustomAccountInterface` replaces the traditional private key signature check. When a transaction invokes the wallet, the host calls `__check_auth` with:

- `signature_payload: Hash<32>` — host-provided, tamper-proof hash of the transaction
- `signature: WebAuthnSignature` — client-provided `{ credential_id, secp256r1_sig }`

The contract looks up the public key for the `credential_id` and calls the native `secp256r1_verify` host function. If the signature is valid → transaction proceeds.

```
Hash<32> type is enforced by the SDK: only the host can produce a valid Hash<32>.
This prevents callers from replaying arbitrary payloads.
```

### Owner Auth in Policy

Every Policy mutation uses `owner.require_auth()`. In practice, `owner` is the Wallet contract address itself, meaning the only way to change policy settings is via a transaction signed by a registered passkey on that wallet.

### Guardian Auth in Recovery

```
propose_recovery / approve_recovery: caller.require_auth()
```

Each guardian must individually sign their call. The contract also validates they are in the registered guardian set (`Guardian(caller)` key exists in persistent storage).

---

## TTL & State Persistence

Soroban contracts have no persistent state by default — entries expire after a ledger-based TTL. We manage this explicitly.

| Storage type | Used for | TTL strategy |
|---|---|---|
| Instance | Small, always-needed config (signers map, owner, policy) | Bumped 30 days on every call |
| Persistent | Large or rarely-accessed data (spend limits, sessions, proposals) | Bumped 30 days on access |
| Temporary | Not used | — |

```rust
// TTL constants (wallet/src/storage.rs)
const DAY_IN_LEDGERS: u32 = 17280;        // ~5 sec per ledger
const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
```

`extend_ttl` is called at the start of every mutating function — not just at creation. This keeps active wallets alive indefinitely without any user intervention.

---

## Key Design Decisions

### Why a separate Policy contract?

Putting limits directly in the Wallet would make the wallet hard to upgrade and impossible to share across multiple wallets (e.g. a team wallet with shared limits). The Policy contract can be:
- **Swapped** — change the entire rule set without touching the wallet
- **Shared** — a single policy for a multi-device setup
- **Extended** — add new rule types without redeploying the wallet

### Why store signers in a `Map` in instance storage?

Soroban doesn't support native iteration over persistent storage keys. By keeping all signers in a single `Map<Bytes, Bytes>` in instance storage, we get:
- O(1) lookup during `__check_auth`
- `get_signers()` returns the full map for display
- All entries bump TTL together in a single `instance().extend_ttl` call

Trade-off: Instance storage has size limits. A wallet with thousands of credentials would hit these. In practice, wallets have 1–5 devices, so this is fine.

### Why `predict_address` instead of a registry?

Traditional wallet factories keep an on-chain registry of deployed wallets. In Soroban, deterministic deployment means we can compute the address from `(factory_address, salt)` off-chain or on-chain with zero state. No registry entry to maintain, no extra storage fees.

### Why not inline the Policy into `__check_auth` now?

Until CAP-0071 native auth delegation is available, calling another contract from inside `__check_auth` has gas cost implications and requires careful re-entrancy reasoning. The current design is conservative: `__check_auth` only verifies the passkey signature. Policy checks happen separately. This will be tightened in the v2 wallet that targets Protocol 27+.
