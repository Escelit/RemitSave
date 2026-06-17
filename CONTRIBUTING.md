# Contributing to RemitSave Africa

Thank you for your interest in contributing! This guide covers everything you need to get started.

## Table of Contents

- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Code Standards](#code-standards)
- [Running Tests](#running-tests)
- [Submitting Changes](#submitting-changes)
- [Good First Issues](#good-first-issues)

---

## Development Setup

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | stable (1.78+) | https://rustup.rs |
| Node.js | 20+ | https://nodejs.org / `nvm install 20` |
| Docker & Docker Compose | 24+ | https://docs.docker.com/get-docker/ |
| soroban CLI | latest | `cargo install soroban-cli` |

### One-command bootstrap

```bash
bash scripts/bootstrap.sh
```

This will:
1. Check prerequisites and install the `wasm32-unknown-unknown` Rust target
2. Build all Soroban contracts
3. Run contract tests
4. Build all backend services
5. Install frontend dependencies
6. Start Postgres, Redis, and Stellar via Docker Compose
7. Apply database migrations

### Optional: deploy contracts to the local Stellar network

```bash
DEPLOY_CONTRACTS=true bash scripts/bootstrap.sh
# or after the first bootstrap:
NETWORK=local bash scripts/deploy.sh
```

Contract addresses are saved to `.env.deployment`.

### Running services individually

```bash
# Backend services (each in its own terminal)
cd backend && cargo run -p auth-service
cd backend && cargo run -p remit-service

# Frontend (Expo dev server)
cd frontend/diaspora-app && npx expo start

# Stop infrastructure
docker compose down
```

---

## Project Structure

```
contracts/          Soroban smart contracts (Rust)
  remit-save/       Core remittance + savings logic
  vault-pool/       Yield-bearing vault pools
  shared/           Shared types and helpers

backend/            Off-chain services (Rust / Axum)
  auth-service/     JWT auth, KYC, user registration
  remit-service/    Remittance rules + on-chain execution
  event-watcher/    Stellar event listener → DB writer
  shared/           Shared models, DB pool, error types
  migrations/       SQL migration files

frontend/
  diaspora-app/     React Native (Expo) sender app
  beneficiary-web/  React PWA for beneficiaries
  ui-lib/           Shared component library

scripts/
  bootstrap.sh      One-command local environment setup
  deploy.sh         Deploy contracts to a Stellar network
ussd-gateway/       USSD interface (Go)
infra/              Terraform + Helm charts
```

---

## Code Standards

| Area | Standard |
|---|---|
| Rust formatting | `cargo fmt` before committing |
| Rust linting | `cargo clippy -- -D warnings` (zero warnings) |
| TypeScript | ESLint + Prettier (`npm run lint` in each frontend package) |
| Commit messages | [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `test:`, `chore:` |
| Branch naming | `feat/`, `fix:`, `chore/`, `docs/` prefix (e.g. `feat/ussd-gateway`) |
| PR size | Keep PRs under 400 lines where possible; split larger changes |

---

## Running Tests

```bash
# All tests
make test

# Contracts only
cd contracts && cargo test

# Backend only
cd backend && cargo test

# Frontend only
cd frontend/diaspora-app && npm test
```

---

## Submitting Changes

1. Fork the repo and create a branch: `git checkout -b feat/my-feature`
2. Make your changes and add tests where applicable
3. Ensure `make test` passes and `cargo clippy` is clean
4. Open a pull request against `main` with a clear description of what changed and why

---

## Good First Issues

The following areas represent the remaining ~35% of the platform. They are well-scoped starting points:

### Smart Contracts

- **[ ] `vault_withdraw` lockup enforcement** — add a check that `env.ledger().timestamp() >= lock_until` before allowing vault withdrawal, and write a test that verifies early withdrawal is rejected.
- **[ ] Emergency pause mechanism** — add an `is_paused: bool` flag to contract storage; gate `execute_remittance`, `vault_deposit`, and `vault_withdraw` behind it; expose `pause` / `unpause` admin functions.
- **[ ] Two-phase admin transfer** — replace the single `admin` setter with `propose_admin` + `accept_admin` to prevent accidental ownership loss.

### Backend

- **[ ] Savings service** — implement `POST /savings/plan`, `GET /savings/plans`, `POST /savings/plan/:id/deposit`, `POST /savings/plan/:id/withdraw` in `backend/savings-service` (skeleton exists).
- **[ ] Real anchor integration (Cowrie / NGN)** — replace the stub in `anchor-relayer` with a real SEP-24 deposit/withdrawal flow against the Cowrie sandbox API.
- **[ ] KYC provider integration** — wire up SmileID or IdentityPass in `auth-service/src/providers/` behind the existing `POST /auth/kyc` endpoint.
- **[ ] Notification service** — implement the NATS consumer in `notification-service` that listens for `RemittanceExecuted` events and sends an SMS via Africa's Talking.
- **[ ] Schedule keeper jobs** — implement `update_fx_rates` and `expire_lockups` cron jobs in `backend/schedule-keeper`.

### Frontend

- **[ ] Beneficiary PWA** — build the `frontend/beneficiary-web` React app: balance view, transaction history, withdrawal request.
- **[ ] Vault pools screen** — add a "Vault Pools" tab in the diaspora app: list pools, show APY, deposit/withdraw UI.
- **[ ] Offline mode** — implement a service worker + IndexedDB sync queue in the diaspora app so the last-known balance is visible without a network connection.
- **[ ] Localization** — add `react-i18next` to the diaspora app and provide translations for Hausa, Yoruba, and French.

### Infrastructure & DX

- **[ ] USSD gateway** — implement the menu-driven USSD handler in `ussd-gateway/` (Go) using Africa's Talking; integrate with the savings service API.
- **[ ] Helm charts** — fill in `infra/helm/remit-save/templates/` with Deployment, Service, and HorizontalPodAutoscaler for each backend service.
- **[ ] GitHub Actions CI** — complete `.github/workflows/test.yml` to run `cargo test`, `cargo clippy`, and `npm test` on every pull request.
- **[ ] Load test script** — write a `k6` script in `scripts/` that simulates 1 000 concurrent `execute_remittance` calls against the local stack.

If you pick one of these up, please comment on the issue (or open one) so others know it's in progress.
