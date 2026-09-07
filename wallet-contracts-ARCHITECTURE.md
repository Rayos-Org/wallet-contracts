# `wallet-contracts` — Architecture

## 1. Purpose & Scope

This repo owns all on-chain logic: the per-user wallet contract, the factory that deploys
wallets deterministically, and the policy contract that enforces spend limits, session keys,
allow-lists, and guardian recovery. Everything here is Rust compiled to WASM for Soroban.

This is the **source of truth** for the wallet's authorization behavior. If it isn't enforced
here, it isn't actually secure — frontend/backend checks are UX conveniences only, never the
security boundary.

## 2. Tech Stack

All tooling here is open source and part of the public Stellar/Soroban toolchain — no
proprietary compilers, closed audit tools, or paid SaaS dependencies in the build path:

- Rust (stable toolchain, pinned via `rust-toolchain.toml`)
- `soroban-sdk` (Apache-2.0, pinned version, matching current Protocol 26 host functions)
- `stellar-cli` (open source, SDF) for build/deploy/invoke
- `proptest` (Apache-2.0/MIT) for property-based fuzz testing
- `cargo-audit` (open source, RustSec) for dependency vulnerability scanning in CI
- Reference implementations studied/interoperated with: `kalepail/passkey-kit` and
  `kalepail/smart-account-kit` (both open source) — used as design references and,
  where sensible, as actual dependencies rather than reimplementing security-critical
  primitives from scratch

## 3. Directory Structure

```
wallet-contracts/
├── contracts/
│   ├── wallet/
│   │   ├── src/
│   │   │   ├── lib.rs           # contract entrypoints
│   │   │   ├── auth.rs          # __check_auth, WebAuthn signature verification
│   │   │   ├── signers.rs       # add/remove/list registered credentials
│   │   │   └── storage.rs       # typed storage keys/accessors
│   │   └── Cargo.toml
│   ├── factory/
│   │   ├── src/
│   │   │   ├── lib.rs           # deploy_wallet, deterministic address derivation
│   │   │   └── storage.rs
│   │   └── Cargo.toml
│   └── policy/
│       ├── src/
│       │   ├── lib.rs           # policy contract entrypoints
│       │   ├── spend_limits.rs
│       │   ├── session_keys.rs
│       │   ├── allow_lists.rs
│       │   └── guardians.rs     # N-of-M recovery logic
│       └── Cargo.toml
├── tests/
│   ├── wallet_test.rs
│   ├── factory_test.rs
│   ├── policy_test.rs
│   └── integration_test.rs      # full flow: deploy → sign → spend → recover
├── fuzz/
│   └── policy_fuzz.rs           # proptest targets for spend/session edge cases
├── scripts/
│   ├── deploy_testnet.sh
│   └── generate_bindings.sh     # runs `stellar contract bindings typescript`
├── docs/
│   └── threat-model.md
├── Cargo.toml                   # workspace root
├── rust-toolchain.toml
└── ARCHITECTURE.md              # this file
```

## 4. Contract Interfaces

### Wallet contract

| Function | Description |
|---|---|
| `__check_auth(payload, context)` | Verifies a secp256r1 WebAuthn signature against a registered credential; delegates spend/session checks to the active policy contract |
| `add_signer(credential_id, public_key)` | Registers an additional device/passkey |
| `remove_signer(credential_id)` | Revokes a device/passkey — requires existing auth |
| `set_policy(policy_address)` | Points the wallet at a policy contract instance |
| `get_signers()` | Read-only: list registered credentials |

### Factory contract

| Function | Description |
|---|---|
| `deploy_wallet(credential_id, public_key, salt)` | Deploys a new wallet contract at a deterministic address |
| `predict_address(salt)` | Read-only: compute the address before deployment (needed for "receive before first sign-in" UX) |

### Policy contract

| Function | Description |
|---|---|
| `set_spend_limit(token, amount, window)` | Configure a rolling spend cap per token |
| `create_session_key(scope, expiry)` | Issue a scoped, time-limited signer |
| `revoke_session_key(session_id)` | Immediately invalidate a session key |
| `set_allow_list(addresses)` | Restrict which contracts the wallet may call |
| `add_guardian(address)` / `remove_guardian(address)` | Manage recovery guardians |
| `propose_recovery(new_signer)` / `approve_recovery(proposal_id)` | N-of-M guardian recovery flow |

Error codes are defined once in `contracts/wallet/src/errors.rs` (and equivalents per
contract) using `soroban_sdk::contracterror` so the SDK can generate typed error handling
rather than parsing strings.

## 5. Key Flows

**Wallet creation**
1. Client generates a WebAuthn credential (passkey) locally.
2. Client calls `factory.predict_address(salt)` to show the user their future address.
3. On first funding or first explicit "activate," client calls `factory.deploy_wallet(...)`.

**Spending with a policy check**
1. Client builds a transaction invoking the target contract (e.g., token `transfer`).
2. Wallet's `__check_auth` runs, verifies the WebAuthn signature, then calls into the
   active policy contract to confirm the spend is within limits/allow-list.
3. If approved, the transaction executes; policy contract updates its rolling spend counters.

**Guardian recovery**
1. User loses their device. A guardian (or the user from a new device, per your chosen UX)
   calls `propose_recovery(new_signer)`.
2. Other guardians call `approve_recovery(proposal_id)` until the N-of-M threshold is met.
3. Policy contract calls back into the wallet to rotate the registered signer.
4. A timelock (configurable) delays execution to give the real owner a window to object if
   the device wasn't actually lost — this is a critical anti-abuse control, document the
   chosen delay explicitly in `docs/threat-model.md`.

## 6. Testing Strategy

- **Unit tests** per contract function, using the Soroban test framework's mock host.
- **Integration tests** covering full multi-contract flows (deploy → sign → spend → recover)
  against a local Soroban test network.
- **Property-based fuzzing** (`proptest`) specifically targeting policy math: overflow on
  spend accumulation, expired-session edge cases, replay of stale recovery proposals.
- **Coverage target**: 80%+ on `policy` and `wallet` contracts before any mainnet deployment
  is considered.
- All tests run against the **currently pinned Protocol version** in CI; bump the pin
  deliberately when moving to a new protocol, never silently.

## 7. CI/CD

- `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit` on every PR
- WASM build size check (Soroban has contract size limits — fail CI if a contract exceeds
  a configured threshold with headroom)
- On tag: build release WASM, publish the `.wasm` as a GitHub release artifact with its
  hash recorded, and trigger `wallet-sdk`'s binding-regeneration workflow

## 8. Migration Notes (CAP-0071 / CAP-0072)

Track protocol evolution explicitly here rather than in scattered comments:

- **Today (Protocol 26)**: standard `Address` credentials, app-level policy contract as
  designed above.
- **Protocol 27 (CAP-0071, testnet now)**: native delegated-auth host functions become
  available. Plan a v2 wallet contract that can optionally use native delegation instead of
  the current custom `__check_auth` plumbing — smaller transactions, simpler simulation.
- **Protocol 28+ (CAP-0072)**: G-accounts become delegatable too. Revisit whether a
  contract-based wallet is still the right primitive for all use cases, or whether some
  users are better served by a delegated G-account.

Keep a running `MIGRATION.md` in this repo updated as these land on mainnet — don't let this
knowledge live only in your head or old PR descriptions.

## 9. Dependencies on Other Repos

- None inbound. This repo is the foundation; nothing here should import from `wallet-sdk`,
  `relay-backend`, or any app.
- Outbound: `wallet-sdk` consumes generated bindings; `relay-backend` consumes contract
  event schemas for indexing.
