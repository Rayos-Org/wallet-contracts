# Contract API Reference

Complete function signatures, parameters, return types, and error codes for every Soroban contract in this workspace.

---

## Table of Contents

- [Wallet Contract](#-wallet-contract)
- [Factory Contract](#-factory-contract)
- [Policy Contract](#-policy-contract)
- [Error Code Reference](#-error-code-reference)

---

## 👛 Wallet Contract

**Crate**: `wallet`  
**Source**: [`contracts/wallet/src/lib.rs`](../contracts/wallet/src/lib.rs)  
**Interface**: Implements Soroban's `CustomAccountInterface` (`__check_auth`).  

The wallet is the user's on-chain identity anchor. It holds the registered WebAuthn passkey credentials and delegates policy enforcement to a connected Policy contract.

---

### `init(credential_id, public_key) → Result<()>`

Registers the first passkey credential. Can only be called once. Typically invoked by the Factory immediately after deployment.

| Parameter | Type | Description |
|---|---|---|
| `credential_id` | `Bytes` | Unique identifier for the WebAuthn credential (e.g. a CBOR-encoded ID from the authenticator) |
| `public_key` | `Bytes` | Uncompressed secp256r1 public key (65 bytes: `04 || X || Y`) |

**Errors**: `AlreadyInitialized`

---

### `add_signer(credential_id, public_key) → Result<()>`

Registers an additional passkey credential (e.g. a second device). Requires existing wallet auth.

| Parameter | Type | Description |
|---|---|---|
| `credential_id` | `Bytes` | New credential identifier |
| `public_key` | `Bytes` | Uncompressed secp256r1 public key (65 bytes) |

**Auth**: `env.current_contract_address().require_auth()` — must be signed by an existing passkey.  
**Errors**: `SignerAlreadyExists`

---

### `remove_signer(credential_id) → Result<()>`

Revokes a registered credential. Refuses to remove the last one (would brick the wallet).

| Parameter | Type | Description |
|---|---|---|
| `credential_id` | `Bytes` | Identifier of the credential to revoke |

**Auth**: `env.current_contract_address().require_auth()`  
**Errors**: `SignerNotFound`, `CannotRemoveLastSigner`

---

### `recover_signer(credential_id, public_key) → Result<()>`

Guardian-recovery pathway to register a new passkey after device loss. Only callable by the registered Policy contract.

| Parameter | Type | Description |
|---|---|---|
| `credential_id` | `Bytes` | New credential identifier |
| `public_key` | `Bytes` | New uncompressed secp256r1 public key (65 bytes) |

**Auth**: `policy_address.require_auth()` — only the wallet's active Policy contract can call this.  
**Errors**: `Unauthorized` (no policy set), `SignerAlreadyExists`

---

### `set_policy(policy) → Result<()>`

Points the wallet at a Policy contract instance.

| Parameter | Type | Description |
|---|---|---|
| `policy` | `Address` | Address of the deployed Policy contract |

**Auth**: `env.current_contract_address().require_auth()`

---

### `get_signers() → Map<Bytes, Bytes>`

Read-only. Returns all registered credentials as a map of `credential_id → public_key`.

---

### `__check_auth(signature_payload, signature, auth_contexts) → Result<()>`

Called by the Soroban host for every transaction this account authorises. Verifies the WebAuthn signature against a registered passkey.

| Parameter | Type | Description |
|---|---|---|
| `signature_payload` | `Hash<32>` | SHA-256 hash of the transaction envelope, provided by the host |
| `signature` | `WebAuthnSignature` | `{ credential_id: Bytes, signature: BytesN<64> }` |
| `auth_contexts` | `Vec<Context>` | Transaction contexts (used for future policy checks) |

**Errors**: `SignerNotFound`, `InvalidSignature`

> **Security note**: The `signature_payload` is sourced entirely from the Soroban host — it cannot be forged by callers. The SDK enforces this via the `Hash<32>` type restriction.

---

## 🏭 Factory Contract

**Crate**: `factory`  
**Source**: [`contracts/factory/src/lib.rs`](../contracts/factory/src/lib.rs)

Deploys new Wallet contract instances at deterministic, predictable addresses using Soroban's built-in `deployer` host object.

---

### `init(wasm_hash)`

Stores the WASM hash of the wallet contract that this factory will deploy.

| Parameter | Type | Description |
|---|---|---|
| `wasm_hash` | `BytesN<32>` | SHA-256 hash of the uploaded wallet WASM binary |

**Note**: Call `stellar contract install --wasm wallet.wasm` first to upload and get the hash.

---

### `deploy_wallet(salt, credential_id, public_key) → Address`

Deploys a new wallet contract at a deterministic address and initialises it with the first passkey in a single atomic transaction.

| Parameter | Type | Description |
|---|---|---|
| `salt` | `BytesN<32>` | 32-byte salt — unique per wallet. Use a hash of the user's identifier. |
| `credential_id` | `Bytes` | First passkey credential identifier |
| `public_key` | `Bytes` | Uncompressed secp256r1 public key (65 bytes) |

**Returns**: The `Address` of the newly deployed wallet contract.

**Internal flow**:
1. `env.deployer().with_current_contract(salt).deploy(wasm_hash)` — deploys the WASM
2. `env.invoke_contract(&wallet_address, "init", (credential_id, public_key))` — initialises the wallet

---

### `predict_address(salt) → Address`

Pure read-only computation of the wallet address that **would** be deployed for the given salt. No state change, no fees beyond the read cost.

| Parameter | Type | Description |
|---|---|---|
| `salt` | `BytesN<32>` | Same salt you will pass to `deploy_wallet` |

**Returns**: The deterministic `Address`.

> **UX use case**: Call this before deployment so the user can fund their wallet address before they've ever interacted with the network.

---

## 🛡️ Policy Contract

**Crate**: `policy`  
**Source**: [`contracts/policy/src/lib.rs`](../contracts/policy/src/lib.rs)

A decoupled, composable rules engine that enforces spend limits, session keys, contract allow-lists, and guardian-based recovery. Attached to a wallet via `wallet.set_policy(policy_address)`.

---

### `init(owner) → Result<()>`

Initialises the policy with an owner (the wallet contract or a multisig). Can only be called once.

| Parameter | Type | Description |
|---|---|---|
| `owner` | `Address` | The address authorised to configure all policy settings |

**Errors**: `Unauthorized` (already initialised)

---

### Spend Limits

#### `set_spend_limit(token, amount, window_secs) → Result<()>`

Configures a rolling spend cap for a specific token.

| Parameter | Type | Description |
|---|---|---|
| `token` | `Address` | The Stellar asset contract address |
| `amount` | `i128` | Maximum total spend allowed within the window |
| `window_secs` | `u64` | Rolling window duration in seconds (e.g. `3600` = 1 hour) |

**Auth**: owner  
**Errors**: — (always succeeds if authorised)

#### `check_spend(token, spend_amount) → Result<()>`

Verifies and records a spend. Called by the wallet's `__check_auth` before authorising a token transfer.

| Parameter | Type | Description |
|---|---|---|
| `token` | `Address` | Token being spent |
| `spend_amount` | `i128` | Amount being spent in this transaction |

**Errors**: `SpendLimitExceeded`

> **Rolling window logic**: When `current_time >= window_start + window_secs`, the accumulator resets. Otherwise amounts are added and checked against the limit.

---

### Session Keys

#### `create_session_key(session_id, public_key, scope, expiry) → Result<()>`

Issues a time-limited, scope-restricted sub-key for a specific dApp interaction.

| Parameter | Type | Description |
|---|---|---|
| `session_id` | `Bytes` | Unique identifier for this session (client-generated) |
| `public_key` | `Bytes` | secp256r1 public key of the session key (65 bytes) |
| `scope` | `Vec<Address>` | List of contracts this key is allowed to interact with. Empty = all allowed. |
| `expiry` | `u64` | Unix timestamp after which the key is invalid |

**Auth**: owner

#### `revoke_session_key(session_id) → Result<()>`

Immediately invalidates a session key.

| Parameter | Type | Description |
|---|---|---|
| `session_id` | `Bytes` | Identifier of the session to revoke |

**Auth**: owner

#### `get_session(session_id) → Result<SessionKey>`

Fetches a session key's metadata if it exists and has not expired.

| Parameter | Type | Description |
|---|---|---|
| `session_id` | `Bytes` | Session identifier |

**Returns**: `SessionKey { public_key, scope, expiry }`  
**Errors**: `SessionKeyRevoked`, `SessionKeyExpired`

---

### Allow-Lists

#### `set_allow_list(target, allowed) → Result<()>`

Permits or blocks a specific contract address from being called by this wallet.

| Parameter | Type | Description |
|---|---|---|
| `target` | `Address` | Contract address to allow or block |
| `allowed` | `bool` | `true` to permit, `false` to block |

**Auth**: owner

#### `set_allow_list_enabled(enabled) → Result<()>`

Enables or disables the allow-list feature entirely. When disabled, no contract filtering is applied.

| Parameter | Type | Description |
|---|---|---|
| `enabled` | `bool` | `true` to enforce allow-list, `false` to disable |

**Auth**: owner

#### `check_allow_list(target) → Result<()>`

Checks whether the target is permitted. A no-op when allow-lists are disabled.

| Parameter | Type | Description |
|---|---|---|
| `target` | `Address` | Contract address being checked |

**Errors**: `NotAllowListed`

---

### Guardian Recovery

#### `add_guardian(guardian) → Result<()>` / `remove_guardian(guardian) → Result<()>`

Adds or removes a guardian address from the recovery set.

| Parameter | Type | Description |
|---|---|---|
| `guardian` | `Address` | Guardian's Stellar address |

**Auth**: owner

#### `set_recovery_threshold(threshold) → Result<()>`

Sets the N in N-of-M — how many guardians must approve a recovery.

| Parameter | Type | Description |
|---|---|---|
| `threshold` | `u32` | Minimum approvals required (default: `2`) |

**Auth**: owner

#### `set_recovery_timelock(delay_secs) → Result<()>`

Sets the delay between reaching threshold and execution.

| Parameter | Type | Description |
|---|---|---|
| `delay_secs` | `u64` | Delay in seconds (recommended: `172800` = 48h testnet, `604800` = 7d mainnet) |

**Auth**: owner

#### `propose_recovery(caller, new_credential_id, new_public_key) → Result<u64>`

A guardian proposes replacing the wallet's signer with a new passkey.

| Parameter | Type | Description |
|---|---|---|
| `caller` | `Address` | The proposing guardian (must be registered) |
| `new_credential_id` | `Bytes` | Replacement passkey credential identifier |
| `new_public_key` | `Bytes` | Replacement uncompressed secp256r1 public key |

**Returns**: `proposal_id: u64`  
**Auth**: `caller.require_auth()` + must be a registered guardian  
**Errors**: `InvalidGuardian`

#### `approve_recovery(caller, proposal_id) → Result<()>`

A second (or subsequent) guardian co-signs an active proposal.

| Parameter | Type | Description |
|---|---|---|
| `caller` | `Address` | The approving guardian |
| `proposal_id` | `u64` | ID returned by `propose_recovery` |

**Auth**: `caller.require_auth()` + must be a registered guardian  
**Errors**: `InvalidGuardian`, `ProposalNotFound`, `ProposalNotActive`, `AlreadyApproved`

#### `execute_recovery(wallet, proposal_id) → Result<()>`

Finalises an approved, timelocked recovery. Callable by anyone once conditions are met.

| Parameter | Type | Description |
|---|---|---|
| `wallet` | `Address` | The wallet contract to rotate the signer on |
| `proposal_id` | `u64` | ID of the proposal to execute |

**Errors**: `ProposalNotFound`, `ProposalNotActive`, `NotEnoughApprovals`, `TimelockNotExpired`

**On success**: Calls `wallet.recover_signer(new_credential_id, new_public_key)`.

---

## 🔴 Error Code Reference

### Wallet Contract Errors

| Code | Name | When thrown |
|---|---|---|
| `1` | `NotInitialized` | Wallet function called before `init` |
| `2` | `AlreadyInitialized` | `init` called more than once |
| `3` | `InvalidSignature` | secp256r1 signature verification failed |
| `4` | `SignerNotFound` | Credential ID not registered |
| `5` | `SignerAlreadyExists` | Credential ID already registered |
| `6` | `Unauthorized` | Caller lacks required auth |
| `7` | `PolicyCallFailed` | Policy contract returned an error |
| `8` | `CannotRemoveLastSigner` | Attempt to remove the only signer |

### Policy Contract Errors

| Code | Name | When thrown |
|---|---|---|
| `1` | `NotInitialized` | Policy function called before `init` |
| `2` | `Unauthorized` | Caller lacks owner auth |
| `3` | `SpendLimitExceeded` | Spend would exceed rolling cap |
| `4` | `SessionKeyExpired` | Session key timestamp is past `expiry` |
| `5` | `SessionKeyWrongScope` | Target contract not in session key scope |
| `6` | `SessionKeyRevoked` | Session key has been removed |
| `7` | `SessionKeyInvalidSignature` | Session key secp256r1 verification failed |
| `8` | `NotAllowListed` | Target contract not in allow-list |
| `9` | `InvalidGuardian` | Caller is not a registered guardian |
| `10` | `ProposalNotFound` | Recovery proposal ID does not exist |
| `11` | `AlreadyApproved` | Guardian already approved this proposal |
| `12` | `TimelockNotExpired` | Timelock delay has not elapsed yet |
| `13` | `NotEnoughApprovals` | Fewer approvals than threshold |
| `14` | `ProposalNotActive` | Proposal already executed or cancelled |
