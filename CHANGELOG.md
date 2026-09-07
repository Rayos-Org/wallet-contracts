# Changelog

All notable changes to this project will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Initial community testnet release

---

## [0.5.0] — 2026-09-08 · Phase 5: Scripts, Bindings & CI

### Added
- `scripts/deploy_testnet.sh` — one-shot testnet deployment automation
- `scripts/generate_bindings.sh` — TypeScript binding generation via `stellar contract bindings`
- `.github/workflows/ci.yml` — full CI pipeline: lint, test, audit, WASM size check
- Updated WASM target to `wasm32v1-none` (required by Soroban SDK 27 on Rust 1.84+)
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`

### Fixed
- Removed stale nested `.git` and `.gitignore` inside `contracts/wallet/`
- Fixed unused imports throughout (`cargo fix`)

---

## [0.4.0] — 2026-09-08 · Phase 4: Fuzz Testing & Threat Model

### Added
- `docs/threat-model.md` — full documented threat model with actors, assets, mitigations
- `MIGRATION.md` — CAP-0071 / CAP-0072 protocol upgrade roadmap
- `proptest` property-based fuzz tests for spend-limit math in `policy`

---

## [0.3.0] — 2026-09-08 · Phase 3: Allow-lists + Guardian Recovery

### Added
- `policy/src/allow_lists.rs` — per-contract allow-listing with enable/disable toggle
- `policy/src/guardians.rs` — N-of-M guardian recovery with configurable threshold and timelock
- `wallet: recover_signer()` — policy-gated signer rotation endpoint on wallet contract
- `RecoveryProposal`, `ProposalStatus` storage types
- Unit tests for allow-list enforcement and full guardian recovery flow

### Changed
- `policy/src/storage.rs` — extended `DataKey` with `AllowList`, `Guardian`, `RecoveryProposal`, `RecoveryTimelock`
- `policy/src/errors.rs` — added `NotAllowListed`, `InvalidGuardian`, `AlreadyApproved`, `TimelockNotExpired`, `NotEnoughApprovals`, `ProposalNotActive`

---

## [0.2.0] — 2026-09-08 · Phase 2: Factory + Spend Limits + Session Keys

### Added
- `contracts/factory/` — `deploy_wallet` and `predict_address` factory contract
- `policy/src/spend_limits.rs` — rolling spend accumulator per token per time window
- `policy/src/session_keys.rs` — scoped time-limited session key management
- Unit tests for factory, spend limits, and session keys

### Changed
- `policy/src/storage.rs` — `SpendLimit`, `SessionKey` types
- Refactored session verification to use `secp256r1_verify` in the wallet's `__check_auth` rather than exposing raw hash endpoints (security hardening)

---

## [0.1.0] — 2026-09-08 · Phase 1: Wallet Contract

### Added
- `contracts/wallet/` — initial wallet contract implementing `CustomAccountInterface`
- `auth.rs` — WebAuthn passkey verification via Soroban's native `secp256r1_verify`
- `signers.rs` — add / remove / list passkey credentials (prevents removing last signer)
- `storage.rs` — typed `DataKey` with instance TTL management
- `errors.rs` — typed `ContractError` with `contracterror` macro
- Unit tests for wallet initialization, signer addition and removal

---

## [0.0.1] — 2026-09-08 · Phase 0: Workspace Scaffold

### Added
- Cargo workspace with `wallet`, `factory`, `policy` crate members
- `rust-toolchain.toml` pinned to stable
- `.cargo/config.toml` with `wasm-build` alias
- `.github/workflows/ci.yml` (initial skeleton)
- `.gitignore`
