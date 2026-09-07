# Migration Guide (CAP-0071 / CAP-0072)

This document tracks protocol evolution and the migration path for this wallet implementation.

## Current State: Protocol 26
Currently, this wallet implementation uses an app-level policy contract to enforce spend limits, session keys, and guardian recovery. 

The `Wallet` contract implements `CustomAccountInterface` with a custom `__check_auth` to verify secp256r1 WebAuthn passkeys natively. The `Policy` contract acts as a decoupled rules engine.

## Future: Protocol 27+ (CAP-0071)
Protocol 27 introduces native delegated-auth host functions. This allows for native delegation rather than relying entirely on custom `__check_auth` plumbing.

**Migration Plan (v2 Wallet)**:
1. When Protocol 27 lands on mainnet, we will introduce a `v2` wallet contract.
2. This `v2` wallet will optionally leverage native delegation host functions to reduce transaction sizes and simplify simulation.
3. We will provide an upgrade path so existing wallets can seamlessly swap their implementation.

## Future: Protocol 28+ (CAP-0072)
CAP-0072 brings delegatable G-accounts.

**Evaluation**:
We will evaluate whether a contract-based wallet remains the right primitive for all use cases. It is highly likely that for some use cases (e.g., standard users), a delegated G-account will be significantly cheaper and more efficient than a full contract-based wallet. However, contract-based wallets will still be necessary for complex, custom multisig logic and highly granular spend policies that exceed the native host features.
