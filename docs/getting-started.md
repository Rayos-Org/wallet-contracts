# Developer Guide — Getting Started

Everything you need to go from zero to running tests locally, deploying to testnet, and writing a dApp integration.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development](#local-development)
- [Running Tests](#running-tests)
- [Building WASM](#building-wasm)
- [Testnet Deployment](#testnet-deployment)
- [Calling Contracts Manually](#calling-contracts-manually)
- [Generating TypeScript Bindings](#generating-typescript-bindings)
- [Writing a dApp Integration](#writing-a-dapp-integration)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| **Rust** | stable (≥ 1.84) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **wasm32v1-none target** | — | `rustup target add wasm32v1-none` |
| **Stellar CLI** | latest | `cargo install stellar-cli --locked` |
| **Git** | any | system package manager |

> **Windows users**: The deployment scripts are bash. Use Git Bash or WSL2 to run them. All `cargo` commands work natively in PowerShell.

---

## Local Development

```bash
# Clone the repo
git clone https://github.com/Rayos-Org/wallet-contracts.git
cd wallet-contracts

# Verify toolchain (should show stable + wasm32v1-none)
rustup show

# Build for the host (used for tests)
cargo build

# Type-check without building
cargo check
```

---

## Running Tests

```bash
# All contracts
cargo test

# A single contract
cargo test -p wallet
cargo test -p policy
cargo test -p factory

# Run a specific test by name
cargo test -p policy test_spend_limits_fuzz

# Show println! output during tests
cargo test -- --nocapture
```

### Test architecture

| Test file | What it covers |
|---|---|
| `contracts/wallet/src/test.rs` | Wallet init, signer add/remove |
| `contracts/factory/src/test.rs` | Factory init, address prediction |
| `contracts/policy/src/test.rs` | Spend limits, session keys, allow-lists, guardian recovery + proptest fuzzing |
| `tests/integration_test.rs` | Full multi-contract flow: deploy → set policy → guardian recovery |

---

## Building WASM

Contracts must be compiled to `wasm32v1-none` (not `wasm32-unknown-unknown`) for Soroban SDK 27+.

```bash
# Build all three contracts to WASM
cargo wasm-build

# Outputs are in:
# target/wasm32v1-none/release/wallet.wasm
# target/wasm32v1-none/release/factory.wasm
# target/wasm32v1-none/release/policy.wasm
```

> **Size check**: Each contract must stay under 128 KB. The CI enforces this automatically.

---

## Testnet Deployment

### One-shot script (recommended)

```bash
bash scripts/deploy_testnet.sh
```

This script:
1. Builds all WASM files
2. Creates a `deployer` identity and funds it from the testnet friendbot
3. Uploads the Wallet WASM (`stellar contract install`)
4. Deploys and initialises the Factory
5. Deploys and initialises the Policy
6. Saves addresses to `.testnet-addresses`

### Manual deployment

```bash
# 1. Fund your identity
stellar keys generate my-key --network testnet

# 2. Upload the wallet WASM
WALLET_HASH=$(stellar contract install \
  --wasm target/wasm32v1-none/release/wallet.wasm \
  --source my-key --network testnet)

# 3. Deploy the factory
FACTORY=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/factory.wasm \
  --source my-key --network testnet)

# 4. Initialise the factory
stellar contract invoke --id $FACTORY --source my-key --network testnet \
  -- init --wasm_hash "$WALLET_HASH"

# 5. Deploy the policy
POLICY=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/policy.wasm \
  --source my-key --network testnet)

# 6. Initialise the policy
OWNER=$(stellar keys address my-key)
stellar contract invoke --id $POLICY --source my-key --network testnet \
  -- init --owner "$OWNER"
```

---

## Calling Contracts Manually

### Create a wallet via Factory

```bash
# salt: 32 bytes as hex
stellar contract invoke --id $FACTORY --source my-key --network testnet \
  -- deploy_wallet \
     --salt "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20" \
     --credential_id "74657374637265640000" \
     --public_key "04<X_32_BYTES><Y_32_BYTES>"
```

### Set a spend limit

```bash
stellar contract invoke --id $POLICY --source my-key --network testnet \
  -- set_spend_limit \
     --token "<USDC_CONTRACT_ADDRESS>" \
     --amount 1000000000 \
     --window_secs 3600
```

### Propose a recovery

```bash
stellar contract invoke --id $POLICY --source guardian1 --network testnet \
  -- propose_recovery \
     --caller "<GUARDIAN1_ADDRESS>" \
     --new_credential_id "6e657763726564656e7469616c" \
     --new_public_key "04<NEW_X><NEW_Y>"
```

---

## Generating TypeScript Bindings

After deploying, generate strongly-typed TypeScript clients:

```bash
bash scripts/generate_bindings.sh

# Outputs to:
# bindings/factory/   — TypeScript client for the factory
# bindings/policy/    — TypeScript client for the policy
# bindings/wallet/    — TypeScript client for the wallet
```

The bindings are generated by `stellar contract bindings typescript` and produce:
- A typed `Contract` class per contract
- Method signatures matching the Rust function signatures
- XDR serialisation/deserialisation handled automatically

---

## Writing a dApp Integration

### Minimal example (TypeScript/JavaScript)

```typescript
import { FactoryContract } from './bindings/factory';
import { PolicyContract } from './bindings/policy';

const factory = new FactoryContract({ contractId: FACTORY_ADDRESS, networkPassphrase, rpcUrl });
const policy  = new PolicyContract({ contractId: POLICY_ADDRESS,  networkPassphrase, rpcUrl });

// 1. Predict where the wallet will live (no fees)
const salt = crypto.getRandomValues(new Uint8Array(32));
const walletAddress = await factory.predict_address({ salt });
console.log("Your future wallet address:", walletAddress);

// 2. Create a passkey with WebAuthn
const credential = await navigator.credentials.create({ publicKey: { ... } });

// 3. Deploy the wallet (triggers when user funds it)
await factory.deploy_wallet({
  salt,
  credential_id: Buffer.from(credential.rawId),
  public_key: extractPublicKey(credential),
});

// 4. Set a spend limit of 100 USDC/hour
await policy.set_spend_limit({
  token: USDC_CONTRACT_ADDRESS,
  amount: BigInt(100_000_000), // 100 USDC in stroops
  window_secs: BigInt(3600),
});
```

### Signing a transaction with a passkey

When the user signs a Stellar transaction:
1. The Soroban host calls `wallet.__check_auth(txHash, WebAuthnSignature)`.
2. Your client builds the `WebAuthnSignature = { credential_id, secp256r1_sig }`.
3. The signature over `txHash` is produced by the WebAuthn authenticator (`navigator.credentials.get()`).
4. The 64-byte compact signature must be extracted from the DER-encoded authenticator response.

> **Reference**: See [`kalepail/passkey-kit`](https://github.com/kalepail/passkey-kit) for a production-ready WebAuthn ↔ Stellar signing flow.

---

## Troubleshooting

### `wasm32-unknown-unknown` target build failure

```
Rust compiler 1.82+ with target 'wasm32-unknown-unknown' is unsupported
```

**Fix**: Use `wasm32v1-none` (already configured in `.cargo/config.toml`).

```bash
rustup target add wasm32v1-none
cargo wasm-build   # uses the alias which targets wasm32v1-none
```

---

### `typenum` / `hybrid-array` version conflict

This was a known issue with older Cargo.lock states. The workspace pins:

```toml
# Cargo.toml
[patch.crates-io]
typenum = { version = "=1.17.0" }
```

If you see it again: `cargo update -p typenum --precise 1.17.0`.

---

### Tests failing with `os error 2` on Windows

This usually means the Cargo registry cache was corrupted (e.g. from a disk-full event).

```powershell
Remove-Item "$env:USERPROFILE\.cargo\registry\src" -Recurse -Force
cargo build   # re-downloads cleanly
```

---

### `CannotRemoveLastSigner` error

You are trying to remove the only registered credential on a wallet. Add a second passkey first:

```bash
stellar contract invoke --id $WALLET -- add_signer --credential_id "..." --public_key "..."
```

Then remove the old one.

---

### `TimelockNotExpired` during recovery

The recovery timelock has not elapsed. Check `execute_after` on the proposal:

```bash
stellar contract invoke --id $POLICY -- get_session  # or check proposal via explorer
```

Wait for the ledger timestamp to exceed `execute_after`, then retry `execute_recovery`.
