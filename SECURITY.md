# Security Policy

## Reporting a Vulnerability

**Please do NOT open a public GitHub issue for security vulnerabilities.**

If you discover a security vulnerability in this project, please report it responsibly:

1. **Email**: `security@rayos.xyz` *(or open a private security advisory on GitHub)*
2. **GitHub Private Advisory**: Go to [Security → Advisories → New Draft](https://github.com/Rayos-Org/wallet-contracts/security/advisories/new)

We will acknowledge your report within **72 hours** and aim to provide a fix or mitigation timeline within **14 days** for critical issues.

---

## Scope

The following are **in scope** for responsible disclosure:

| Contract | File | Priority |
|---|---|---|
| Wallet auth bypass | `contracts/wallet/src/auth.rs` | 🔴 Critical |
| Guardian recovery exploit | `contracts/policy/src/guardians.rs` | 🔴 Critical |
| Session key forgery / scope escape | `contracts/policy/src/session_keys.rs` | 🔴 Critical |
| Spend limit bypass | `contracts/policy/src/spend_limits.rs` | 🟠 High |
| Allow-list bypass | `contracts/policy/src/allow_lists.rs` | 🟠 High |
| Factory determinism attack | `contracts/factory/src/lib.rs` | 🟡 Medium |
| Storage / TTL manipulation | Any `storage.rs` | 🟡 Medium |

**Out of scope**: Issues in upstream dependencies (report to `soroban-sdk` or the Stellar Foundation), general network-level attacks, or theoretical issues without a proof of concept.

---

## Supported Versions

| Version | Supported |
|---|---|
| `main` branch (Testnet) | ✅ Active |
| Mainnet | Not yet deployed |

---

## Disclosure Policy

- We follow **coordinated disclosure**: we aim to patch and release before public disclosure.
- Reporters will be **credited** in the release notes unless they prefer to remain anonymous.
- We do not currently offer a formal bug bounty, but exceptional contributions will be recognised publicly.

---

## Security Architecture

See [`docs/threat-model.md`](./docs/threat-model.md) for the full documented threat model, including threat actors, assets, mitigations, and known limitations.
