# Wallet Contracts Threat Model

## Assumptions
- **Host Environment**: The Stellar network and Soroban environment securely execute WebAssembly.
- **Cryptography**: `secp256r1` signature verification is cryptographically secure. The host-provided payload hashes cannot be forged.
- **Client**: The client securely generates and stores WebAuthn passkeys (credentials).

## Assets
- The wallet's registered signers (credentials and public keys)
- The wallet's assets (tokens, balances, NFTs) stored on the Stellar network
- The active Policy configuration (guardians, spend limits, session keys)

## Actors
- **Owner**: Holds the active WebAuthn passkeys. Can perform any action, rotate keys, or set policies.
- **Guardian**: A designated trusted party that can propose and approve key recovery.
- **Attacker**: An external actor attempting to drain funds or brick the wallet.
- **Malicious dApp**: A target contract that a session key interacts with.

## Threat Vectors and Mitigations

### 1. Unauthorized Spend
**Threat**: An attacker attempts to submit a transaction to drain the wallet's funds.
**Mitigation**: The wallet enforces `__check_auth`, which requires a valid secp256r1 signature matching a registered passkey. If using a session key, the signature is verified against the session's public key, and the `policy` contract verifies the target scope and spend limits.

### 2. Session Key Abuse
**Threat**: A session key is leaked or a dApp goes rogue and tries to drain funds.
**Mitigation**:
- **Scope**: Session keys are restricted to specific target contracts (Allow-lists).
- **Expiry**: Session keys automatically expire after a set time.
- **Spend Limits**: Rolling spend limits restrict the maximum value a session key can drain within a time window.
- **Revocation**: The owner can instantly revoke a session key via `revoke_session_key`.

### 3. Guardian Collusion / Account Takeover
**Threat**: Guardians collude to steal the wallet by replacing the owner's signer.
**Mitigation**:
- **N-of-M Threshold**: Requires multiple guardians to approve.
- **Timelock**: Recoveries are delayed by a configurable timelock (e.g., 48 hours for testnet, 7 days for mainnet). The real owner can intervene during this period to remove the malicious guardians.

### 4. Bricking the Wallet
**Threat**: The owner accidentally removes all signers, locking the wallet forever.
**Mitigation**: The `remove_signer` function explicitly prevents removing the last registered signer.

### 5. Replay Attacks
**Threat**: An attacker replays a valid signed transaction.
**Mitigation**: Soroban's native sequence numbers and transaction envelopes inherently protect against transaction replay attacks.
