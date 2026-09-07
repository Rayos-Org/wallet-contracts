# Contributing to Rayos Wallet Contracts

Thank you for your interest in contributing! This is an open-source project in active development on the Stellar testnet. Community participation is the most important driver of quality and security before we consider any mainnet deployment.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Project Conventions](#project-conventions)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)
- [Full Documentation](./docs/index.md)

---

## Code of Conduct

By participating, you agree to our [Code of Conduct](./CODE_OF_CONDUCT.md). We are committed to a welcoming and inclusive community.

---

## How to Contribute

We welcome contributions in several forms:

| Type | Description |
|---|---|
| 🐛 **Bug reports** | Found unexpected behaviour? Open an issue with reproduction steps |
| 💡 **Feature requests** | Have an idea? Open a GitHub Discussion first |
| 📖 **Documentation** | Fix typos, improve clarity, add examples |
| ✅ **Tests** | Add missing unit tests or property-based fuzz cases |
| 🔐 **Security** | See [SECURITY.md](./SECURITY.md) for responsible disclosure |
| 🔧 **Code** | Bug fixes and enhancements via PR |

---

## Development Setup

### Prerequisites

```bash
# Rust stable (pinned via rust-toolchain.toml)
rustup show

# Soroban WASM target
rustup target add wasm32v1-none

# Stellar CLI (for deployment)
cargo install stellar-cli --locked
```

### Clone & Build

```bash
git clone https://github.com/Rayos-Org/wallet-contracts.git
cd wallet-contracts

# Build contracts for tests
cargo build

# Build release WASM
cargo wasm-build

# Run all tests
cargo test
```

### Run Lints Locally

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo audit
```

---

## Project Conventions

### Code Style

- **`cargo fmt`** is enforced — format before every commit.
- **`cargo clippy -- -D warnings`** must pass — no warnings allowed.
- Use typed errors (`contracterror`) — never return raw strings.
- All storage keys must be defined in the contract's `storage.rs`.

### Testing

- Every new contract function needs at least one unit test.
- Numeric/accumulator logic should have a `proptest` property test.
- Use `env.mock_all_auths()` in tests — never hardcode key material.

### Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add batch signer registration
fix: prevent spend underflow on window reset
test: add proptest for session key expiry edge cases
docs: update threat model with guardian collusion mitigation
```

---

## Pull Request Process

1. **Fork** the repository and create a branch from `main`.
2. **Write tests** for any new logic.
3. **Run the full CI suite locally** before opening a PR.
4. **Open a PR** with a clear description of what and why.
5. **Respond to review feedback** — we aim to review within 48 hours.
6. PRs require at least **one maintainer approval** before merging.

> **Note**: PRs touching cryptographic code (`auth.rs`, `session_keys.rs`, `guardians.rs`) require a more thorough review and may take longer.

---

## Reporting Issues

Use [GitHub Issues](https://github.com/Rayos-Org/wallet-contracts/issues) with the appropriate label:

- `bug` — reproducible unexpected behaviour
- `enhancement` — feature request
- `question` — clarification needed
- `security` — use [SECURITY.md](./SECURITY.md) instead for vulnerabilities

---

Thank you for helping make Rayos better! 🙏
