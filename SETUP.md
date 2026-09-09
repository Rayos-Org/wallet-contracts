# Environment Setup & Developer Runbook

This guide covers complete instructions for onboarding, local environment configuration, compilation, running tests, local simulation, testnet deployment, and generating typed clients for **Rayos Wallet Contracts**.

---

## 📋 Table of Contents

- [System Requirements & Prerequisites](#-system-requirements--prerequisites)
- [Repository Setup](#-repository-setup)
- [Rust Toolchain & Target Verification](#-rust-toolchain--target-verification)
- [Building the Contracts](#-building-the-contracts)
- [Running Tests & Quality Assurance](#-running-tests--quality-assurance)
- [Testnet Deployment](#-testnet-deployment)
- [Client SDK & TypeScript Bindings](#-client-sdk--typescript-bindings)
- [Troubleshooting & Common Issues](#-troubleshooting--common-issues)
- [Further Documentation](#-further-documentation)

---

## 🛠 System Requirements & Prerequisites

Ensure the following tools are installed before proceeding:

| Tool | Recommended Version | Purpose | Installation |
|---|---|---|---|
| **Rust** | Stable (≥ 1.84.0) | Contract development & host execution | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **`wasm32v1-none` target** | Latest | Soroban Protocol 27+ WASM compilation | `rustup target add wasm32v1-none` |
| **Stellar CLI** | `v22.0.0` or higher | Contract deployment, invocation & inspection | `cargo install stellar-cli --locked` |
| **Git** | ≥ 2.30.0 | Version control | [git-scm.com](https://git-scm.com/) |
| **GitHub CLI (`gh`)** | Latest (optional) | CI and repository administration | [cli.github.com](https://cli.github.com/) |
| **Node.js & npm** | ≥ 18.0.0 (optional) | Consuming generated TypeScript bindings | [nodejs.org](https://nodejs.org/) |

> [!IMPORTANT]
> **Rust 1.84+ and `wasm32v1-none`**: Soroban SDK v27 requires the `wasm32v1-none` target. Do not use `wasm32-unknown-unknown` on newer Rust compilers, as it activates unsupported WebAssembly features.

---

## 📂 Repository Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/Rayos-Org/wallet-contracts.git
   cd wallet-contracts
   ```

2. **Verify directory structure:**

   ```text
   wallet-contracts/
   ├── .cargo/               # Cargo aliases (wasm-build)
   ├── .github/              # CI workflows & issue/PR templates
   ├── contracts/
   │   ├── factory/          # Deterministic wallet deployer
   │   ├── policy/           # Spend limits, session keys, allow-lists, recovery
   │   └── wallet/           # Passkey WebAuthn smart account
   ├── docs/                 # In-depth architectural & API references
   ├── scripts/              # Automated deployment and binding scripts
   ├── tests/                # Cross-contract integration tests
   ├── Cargo.toml            # Workspace root manifest
   ├── rust-toolchain.toml   # Pinned toolchain definition
   └── SETUP.md              # This runbook
   ```

---

## 🦀 Rust Toolchain & Target Verification

Check that your active Rust toolchain matches `rust-toolchain.toml`:

```bash
# Verify toolchain status
rustup show

# Ensure the target is installed
rustup target add wasm32v1-none
```

Verify Cargo aliases:
The repository includes `.cargo/config.toml` which defines `cargo wasm-build` as a shortcut for:
`cargo build --target wasm32v1-none --release`

---

## 🔨 Building the Contracts

### 1. Build for Host (Fast type-checking and testing)

```bash
cargo build
cargo check
```

### 2. Compile WebAssembly Binaries

Compile all three contracts to optimized WASM:

```bash
cargo wasm-build
```

Compiled WASM artifacts will be produced at:
- `target/wasm32v1-none/release/wallet.wasm`
- `target/wasm32v1-none/release/factory.wasm`
- `target/wasm32v1-none/release/policy.wasm`

> [!NOTE]
> All contracts are optimized to remain well under the 128 KB Soroban bytecode budget.

---

## 🧪 Running Tests & Quality Assurance

### Run Unit & Integration Tests

```bash
# Run all workspace unit tests
cargo test

# Run tests for a specific contract
cargo test -p wallet
cargo test -p policy
cargo test -p factory

# Run property-based fuzz tests (100 runs)
cargo test -p policy test_spend_limits_fuzz

# Display test stdout / debug output
cargo test -- --nocapture
```

### Code Formatting & Linting

Enforce codebase conventions before committing or submitting a PR:

```bash
# Check formatting
cargo fmt --all -- --check

# Format codebase
cargo fmt --all

# Run Clippy with strict warnings check (matches CI gate)
cargo clippy --all-targets -- -D warnings
```

---

## 🌐 Testnet Deployment

### Prerequisites for Deployment

Configure the Stellar testnet and create a funded deployer identity:

```bash
# Verify networks
stellar network ls

# Generate and fund an identity via Friendbot
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet
```

### Option A: Automated Script (Recommended)

Run the automated testnet setup script:

```bash
# On Linux/macOS or Windows Git Bash / WSL
bash scripts/deploy_testnet.sh
```

This script:
1. Compiles contracts via `cargo wasm-build`.
2. Verifies and funds the `deployer` key on testnet.
3. Installs `wallet.wasm` and captures the WASM hash.
4. Deploys and initializes the **Factory Contract**.
5. Deploys and initializes the **Policy Contract**.
6. Writes deployed contract addresses to `.testnet-addresses`.

### Option B: Manual Step-by-Step Deployment

```bash
# 1. Install Wallet WASM
WALLET_HASH=$(stellar contract upload \
  --wasm target/wasm32v1-none/release/wallet.wasm \
  --source deployer \
  --network testnet)

# 2. Deploy Factory Contract
FACTORY_ADDRESS=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/factory.wasm \
  --source deployer \
  --network testnet)

# 3. Initialize Factory with the Wallet WASM hash
stellar contract invoke \
  --id $FACTORY_ADDRESS \
  --source deployer \
  --network testnet \
  -- init --wasm_hash "$WALLET_HASH"

# 4. Deploy Policy Contract
POLICY_ADDRESS=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/policy.wasm \
  --source deployer \
  --network testnet)

# 5. Initialize Policy with deployer as initial owner
OWNER_ADDR=$(stellar keys address deployer)
stellar contract invoke \
  --id $POLICY_ADDRESS \
  --source deployer \
  --network testnet \
  -- init --owner "$OWNER_ADDR"

# 6. Deploy a User Smart Wallet through the Factory
# Salt: 32 bytes hex, Credential ID: Hex, Public Key: 65 bytes uncompressed secp256r1
WALLET_ADDR=$(stellar contract invoke \
  --id $FACTORY_ADDRESS \
  --source deployer \
  --network testnet \
  -- deploy_wallet \
     --salt "0000000000000000000000000000000000000000000000000000000000000001" \
     --credential_id "64656d6f63726564" \
     --public_key "0400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
```

### Reference Addresses (Stellar Testnet)

| Contract | Address / Hash |
|---|---|
| **Factory** | `CCCAMWJOF7IYTVCU7SR6HFTNH5XRMDMWPYN464NY5BCKUPMUM64RZ5CH` |
| **Policy** | `CCDM3O2SXX3E24MCWLRK5YBVQHJCA4OQKJFF6KWCK6FHZS65DGMT6DOY` |
| **Demo Wallet** | `CAIIUPVI5VO2BCXKPUIRZX4BW4YQ6G6FWPJEWVKIVRN4H3KWJTUV3VXM` |
| **Wallet WASM Hash** | `8c2e77ad251a8e32590280c95627fd24864f1ff1917d781e44510f450445d7c6` |

---

## 📦 Client SDK & TypeScript Bindings

Generate strongly-typed TypeScript SDK bindings directly from deployed contract interfaces:

```bash
# Execute binding generation script
bash scripts/generate_bindings.sh
```

Or manually:

```bash
# Generate Factory client
stellar contract bindings typescript \
  --network testnet \
  --contract-id CCCAMWJOF7IYTVCU7SR6HFTNH5XRMDMWPYN464NY5BCKUPMUM64RZ5CH \
  --output-dir bindings/factory

# Generate Policy client
stellar contract bindings typescript \
  --network testnet \
  --contract-id CCDM3O2SXX3E24MCWLRK5YBVQHJCA4OQKJFF6KWCK6FHZS65DGMT6DOY \
  --output-dir bindings/policy

# Generate Wallet client (from bytecode)
stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/wallet.wasm \
  --output-dir bindings/wallet
```

---

## 🔍 Troubleshooting & Common Issues

### 1. `unsupported target 'wasm32-unknown-unknown'`
- **Cause**: Newer Rust compilers enable WebAssembly features (such as reference types) unsupported on Soroban.
- **Fix**: Always build with `--target wasm32v1-none` or run `cargo wasm-build`.

### 2. `error setting certificate file` on Windows Git Push
- **Cause**: Windows Git installation points to a missing `ca-bundle.crt`.
- **Fix**: Switch Git to the native Windows certificate store:
  ```powershell
  git config --global http.sslbackend schannel
  ```

### 3. GitHub CLI Auth in Git operations
- If `git push` prompts for credentials or fails with `401 Unauthorized`:
  ```bash
  gh auth setup-git
  ```

### 4. `CannotRemoveLastSigner` (Error code `102`)
- The wallet contract strictly forbids removing the final registered passkey to prevent permanently bricking accounts. Add a replacement signer before revoking an existing one.

---

## 📖 Further Documentation

- **[Main README](./README.md)** — Architectural summary & protocol overview
- **[Docs Index](./docs/index.md)** — Master guide to deep documentation
- **[Architecture Deep Dive](./docs/architecture.md)** — Contract boundaries, TTL and auth flows
- **[API Reference](./docs/api-reference.md)** — Comprehensive interface specifications
- **[Threat Model](./docs/threat-model.md)** — Security boundaries & attack mitigations
- **[Architecture Decision Records (ADR)](./docs/decisions.md)** — Design rationale log
- **[Contributing Guide](./CONTRIBUTING.md)** — Contributor onboarding & PR conventions
