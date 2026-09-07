<div align="center">

# Wallet Contracts

**Smart contract wallet infrastructure on Soroban — WebAuthn passkeys, guardian recovery, session keys & spend policies**

[![CI](https://img.shields.io/github/actions/workflow/status/Rayos-Org/wallet-contracts/ci.yml?branch=main&style=for-the-badge&logo=github&label=CI)](https://github.com/Rayos-Org/wallet-contracts/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Soroban](https://img.shields.io/badge/soroban-SDK%2027-blueviolet?style=for-the-badge)](https://soroban.stellar.org)
[![Protocol](https://img.shields.io/badge/stellar-protocol%2027-blue?style=for-the-badge&logo=stellar)](https://developers.stellar.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-green?style=for-the-badge)](./LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](./CONTRIBUTING.md)

</div>

---

## 🧭 Overview

Rayos Wallet Contracts is a **production-grade, auditable smart contract wallet system** built on the [Soroban](https://soroban.stellar.org) smart contract platform on the Stellar network.

We are solving **one of the most fundamental UX and security problems in Web3**: wallets should protect users by default — not by asking them to memorize seed phrases, manage hardware keys, or lose everything when a single device breaks.

Our contracts give every user a self-custodial wallet that:

- ✅ **Signs with passkeys** (WebAuthn/secp256r1 device biometrics) — no seed phrases
- ✅ **Recovers via social guardians** — N-of-M multisig with a challenge timelock
- ✅ **Enforces spend policies on-chain** — rolling spend caps, scoped session keys, allow-lists
- ✅ **Deterministic addresses** — wallets have predictable addresses before first deployment (great for "fund before activate" UX)
- ✅ **Zero dependency on centralised relayers** for authentication

---

## 🔴 The Problem We Are Solving

| Problem | Industry Status Quo | Rayos Approach |
|---|---|---|
| **Seed phrase custody** | Users write down 24 words and pray | WebAuthn passkey — device biometrics sign every transaction |
| **Account recovery** | "You lose it, you lose everything" | N-of-M guardian recovery with anti-abuse timelock |
| **Session authorisation** | Apps get full wallet control | Scoped, time-limited session keys per dApp |
| **Spend safety** | No on-chain limits | Rolling spend limits per token per time window |
| **Contract interoperability** | Wallets are opaque | Allow-lists restrict which contracts can be called |
| **Predictable addresses** | Address only known after deploy | `predict_address(salt)` gives the address upfront |

---

## 🏛️ Architecture

The system is composed of three Soroban contracts, each with a clear responsibility:

```
┌────────────────────────────────────────────────────────────┐
│                        Client / dApp                       │
└─────────────────────────┬──────────────────────────────────┘
                          │
           ┌──────────────▼──────────────┐
           │       Factory Contract       │
           │  deploy_wallet(salt, cred)   │
           │  predict_address(salt)       │
           └──────────────┬──────────────┘
                          │ deploys
           ┌──────────────▼──────────────┐
           │       Wallet Contract        │◄──── WebAuthn passkey (secp256r1)
           │  __check_auth()              │
           │  add_signer / remove_signer  │
           │  recover_signer              │
           │  set_policy                  │
           └──────────────┬──────────────┘
                          │ calls into
           ┌──────────────▼──────────────┐
           │       Policy Contract        │
           │  Spend Limits                │
           │  Session Keys                │
           │  Allow-Lists                 │
           │  Guardian Recovery           │
           └─────────────────────────────┘
```

### Contract Responsibilities

#### 🏭 Factory Contract
Deploys new wallet instances at **deterministic addresses** using Soroban's `deployer` host function. The address can be predicted before paying any fees — critical for "receive XLM before you've activated your wallet" UX flows.

#### 👛 Wallet Contract
The core account contract. Implements Soroban's `CustomAccountInterface` (`__check_auth`), verifying WebAuthn passkeys via the host's native `secp256r1_verify`. Acts as an on-chain identity anchor. Delegates spend and session checks to the active policy contract.

#### 🛡️ Policy Contract
A decoupled rules engine attached to the wallet. Enforces:
- **Spend limits** — rolling accumulator per token per time window
- **Session keys** — scoped, time-expiring sub-keys (e.g. for dApp auto-approval)
- **Allow-lists** — restrict which target contracts the wallet may interact with
- **Guardian recovery** — N-of-M recovery flow with configurable timelock

---

## 🔁 Key Flows

### Wallet Creation

```
Client                      Factory               Wallet
  │                            │                    │
  │── predict_address(salt) ──►│                    │
  │◄── address ───────────────│                    │
  │                            │                    │
  │ (user funds address)       │                    │
  │                            │                    │
  │── deploy_wallet(salt,cred)►│                    │
  │                            │── deploys ────────►│
  │                            │── init(cred,pubkey)►│
  │◄────────────────────────── wallet_address ──────┤
```

### Signing a Transaction

```
Soroban Host            Wallet Contract          Policy Contract
     │                        │                        │
     │── __check_auth(payload)►│                       │
     │                        │── secp256r1_verify()   │
     │                        │── check_allow_list() ─►│
     │                        │── check_spend() ──────►│
     │                        │                        │── update accumulator
     │◄── OK ────────────────│◄── OK ─────────────────│
     │                        │                        │
     │ (transaction executes) │                        │
```

### Guardian Recovery

```
Guardian G1              Policy Contract          Wallet Contract
    │                          │                       │
    │── propose_recovery() ───►│                       │
    │                          │ (stores proposal,     │
    │                          │  starts timelock)     │
    │                          │                       │
Guardian G2                    │                       │
    │── approve_recovery() ───►│                       │
    │                          │ (threshold met)       │
    │                          │                       │
    │  (timelock expires)      │                       │
    │                          │                       │
Anyone                         │                       │
    │── execute_recovery() ───►│                       │
    │                          │── recover_signer() ──►│
    │                          │                       │── registers new passkey
    │                          │◄── OK ────────────────│
```

---

## 📦 Repository Structure

```
wallet-contracts/
├── contracts/
│   ├── wallet/
│   │   └── src/
│   │       ├── lib.rs          # Contract entrypoints & CustomAccountInterface
│   │       ├── auth.rs         # WebAuthn / secp256r1 signature verification
│   │       ├── signers.rs      # Add / remove / list registered passkeys
│   │       ├── storage.rs      # Typed storage keys & TTL management
│   │       ├── errors.rs       # Typed error codes
│   │       └── test.rs         # Unit tests
│   ├── factory/
│   │   └── src/
│   │       ├── lib.rs          # deploy_wallet, predict_address
│   │       ├── storage.rs      # WasmHash persistence
│   │       └── test.rs         # Unit tests
│   └── policy/
│       └── src/
│           ├── lib.rs          # All policy entrypoints
│           ├── spend_limits.rs # Rolling spend cap logic
│           ├── session_keys.rs # Scoped session key management
│           ├── allow_lists.rs  # Contract allow-list enforcement
│           ├── guardians.rs    # N-of-M recovery flow
│           ├── storage.rs      # All policy storage types & keys
│           ├── errors.rs       # Typed error codes
│           └── test.rs         # Unit & property-based fuzz tests
├── tests/
│   └── integration_test.rs     # Full multi-contract flow tests
├── scripts/
│   ├── deploy_testnet.sh       # One-shot testnet deployment
│   └── generate_bindings.sh    # TypeScript binding generation
├── docs/
│   └── threat-model.md         # Security threat model & mitigations
├── MIGRATION.md                # CAP-0071 / CAP-0072 upgrade roadmap
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── Cargo.toml                  # Workspace root
└── rust-toolchain.toml         # Pinned Rust stable toolchain
```

---

## 🌐 Deployed Contracts (Testnet)

> Testnet contracts are deployed on Stellar Testnet. Addresses will be updated after the initial community deployment.

| Contract | Testnet Address | Explorer |
|---|---|---|
| **Factory** | `CCCAMWJOF7IYTVCU7SR6HFTNH5XRMDMWPYN464NY5BCKUPMUM64RZ5CH` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCCAMWJOF7IYTVCU7SR6HFTNH5XRMDMWPYN464NY5BCKUPMUM64RZ5CH) |
| **Policy** | `CCDM3O2SXX3E24MCWLRK5YBVQHJCA4OQKJFF6KWCK6FHZS65DGMT6DOY` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCDM3O2SXX3E24MCWLRK5YBVQHJCA4OQKJFF6KWCK6FHZS65DGMT6DOY) |
| **Wallet (WASM hash)** | `8c2e77ad251a8e32590280c95627fd24864f1ff1917d781e44510f450445d7c6` | [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/8c2e77ad251a8e32590280c95627fd24864f1ff1917d781e44510f450445d7c6) |

> **Mainnet**: Not yet deployed. This project is currently in open testnet. See our [community process](#-community--roadmap).

---

## 🔭 Protocol & Standards

| Standard | Status | Notes |
|---|---|---|
| **Soroban Protocol 27** | ✅ Targeting | SDK v27, `secp256r1_verify` native host function |
| **WebAuthn / FIDO2** | ✅ Implemented | secp256r1 passkey credential verification |
| **CAP-0071 (Delegated Auth)** | 🔜 Planned v2 | Native delegation reduces tx size & complexity |
| **CAP-0072 (G-Account Delegation)** | 🔭 Evaluating | May replace contract wallet for simple use cases |
| **`wasm32v1-none` target** | ✅ Required | Rust 1.84+ mandatory target for Soroban SDK 27 |

---

## 📚 Documentation

| Document | Description |
|---|---|
| [Documentation Index](./docs/index.md) | **Start Here:** Directory of all project documentation |
| [Getting Started](./docs/getting-started.md) | Setup, building, running tests, deploying, TypeScript bindings |
| [Architecture](./docs/architecture.md) | Design decisions, module breakdown, storage layout, auth model |
| [API Reference](./docs/api-reference.md) | Full function signatures, parameters, errors for all contracts |
| [Threat Model](./docs/threat-model.md) | Security actors, threat vectors, and mitigations |
| [Architecture Decisions](./docs/decisions.md) | Why things are designed the way they are (ADR log) |
| [Migration Guide](./MIGRATION.md) | CAP-0071 / CAP-0072 upgrade path for Protocol 27+ |

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the Soroban WASM target
rustup target add wasm32v1-none

# Install Stellar CLI
cargo install stellar-cli --locked
```

### Build

```bash
# Compile all contracts to WASM
cargo wasm-build

# Or build normally (for tests)
cargo build
```

### Test

```bash
# Run all unit tests + property-based fuzz tests
cargo test

# Run tests for a specific contract
cargo test -p policy
cargo test -p wallet
cargo test -p factory
```

### Deploy to Testnet

```bash
# One-shot deployment (builds, creates identity, funds, deploys & inits all contracts)
bash scripts/deploy_testnet.sh

# Generate TypeScript bindings after deploying
bash scripts/generate_bindings.sh
```

### Lint & Security Audit

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo audit
```

---

## 🔐 Security

We take security seriously. Before any mainnet deployment:

- **Threat model documented** — see [`docs/threat-model.md`](./docs/threat-model.md)
- **Property-based fuzz testing** — `proptest` over all spend-limit math
- **`cargo audit`** in CI on every PR
- **No privileged upgradeability** — contracts are immutable after deployment

To report a vulnerability, see [SECURITY.md](./SECURITY.md).

---

## 🤝 Community & Roadmap

This project is being released to the open-source community **first on testnet** to gather feedback before any mainnet deployment.

| Phase | Status | Description |
|---|---|---|
| ✅ Phase 0 | Complete | Workspace scaffold, CI, tooling |
| ✅ Phase 1 | Complete | Wallet contract + WebAuthn auth |
| ✅ Phase 2 | Complete | Factory + Spend limits + Session keys |
| ✅ Phase 3 | Complete | Allow-lists + Guardian recovery |
| ✅ Phase 4 | Complete | Fuzz testing + Threat model |
| ✅ Phase 5 | Complete | Deployment scripts + TypeScript bindings |
| 🔜 Phase 6 | In Progress | Community testnet rollout |
| 🔭 Phase 7 | Planned | Mainnet deployment (pending community audit) |
| 🔭 Phase 8 | Planned | CAP-0071 v2 wallet (native delegated auth) |

---

## 🧑‍💻 Contributing

We welcome contributions of all kinds — bug reports, feature requests, documentation improvements, and code!

- Please read [CONTRIBUTING.md](./CONTRIBUTING.md) before opening a PR.
- Please review our [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) to ensure a welcoming environment.
- Check the [CHANGELOG.md](./CHANGELOG.md) for version history.

**Quick start:**

```bash
git clone https://github.com/Rayos-Org/wallet-contracts.git
cd wallet-contracts
cargo test
```

---

## 📜 License

This project is licensed under the [Apache 2.0 License](./LICENSE).

---

<div align="center">

Built with ❤️ by the [Rayos team](https://github.com/Rayos-Org) on [Stellar](https://stellar.org)

</div>
