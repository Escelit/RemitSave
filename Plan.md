# 10-Day Intensive Development Plan: RemitSave Africa (65% Completion)

This plan simulates a 10-day "sprint" to build the core engine of RemitSave Africa. By Day 10, the system will have a functional on-chain split mechanism, a working backend for user/rule management, and a high-fidelity frontend prototype.

## Objectives
*   **On-Chain (Soroban):** Fully functional Remittance + Savings logic with path-payment simulation.
*   **Backend (Rust/Axum):** Integrated auth, rule management, and event monitoring.
*   **Frontend (React Native):** Onboarding, "Send" flow, and Savings Dashboard.
*   **Infrastructure:** Automated local dev environment (Docker + Soroban RPC).

---

## Daily Schedule & Prompts

### Day 1: Foundation & Soroban Workspace
*   **Goal:** Initialize the workspace, project structure, and core contract storage.
*   **Prompt:** "Initialize the project structure as defined in README.md. Set up a Soroban workspace in `/contracts` with three crates: `remit-save`, `vault-pool`, and `shared`. Implement the `UserProfile` and `SavingsPlan` data models in the `shared` crate using Soroban SDK's storage patterns. Ensure the project builds with a root-level Makefile."

### Day 2: Core Remittance Logic (The Split)
*   **Goal:** Implement the logic that calculates and routes funds.
*   **Prompt:** "In the `remit-save` contract, implement `set_remittance_rule` and the core `execute_remittance` function. For now, focus on the logic that splits an `i128` amount based on `RemittanceRule` basis points and transfers 'test' tokens to the beneficiary and the savings plan. Add unit tests for the split logic."

### Day 3: Backend Infrastructure (Auth & User Service)
*   **Goal:** Set up the Rust/Axum backend and database.
*   **Prompt:** "Create the `backend/auth-service` and `backend/shared` crates. Use Axum for the API and Diesel/SQLx for PostgreSQL. Implement the `POST /auth/register` and `GET /auth/me` endpoints. Configure a Docker Compose file that spins up PostgreSQL, Redis, and a Stellar Quickstart container."

### Day 4: Stellar Integration & Path Payments
*   **Goal:** Integrate the contract with Stellar's asset/DEX concepts.
*   **Prompt:** "Update the `execute_remittance` contract function to interact with the Stellar Asset Contract (SAC). Mock the DEX path-payment by ensuring the contract can accept USDC and emit events for the 'converted' local asset. Write a Python or TS script in `/scripts` to deploy the contract to a local Soroban network and initialize it."

### Day 5: Vault Pool & Yield Math
*   **Goal:** Build the savings engine.
*   **Prompt:** "Implement the `vault-pool` contract. Focus on the `vault_deposit` and `update_share_price` functions. Ensure the math correctly handles share minting and APY-based price updates. Add exhaustive tests for the `share_price` math to prevent rounding errors or value leakage."

### Day 6: Remit Service & Event Watcher
*   **Goal:** Connect the backend to the blockchain.
*   **Prompt:** "Implement the `backend/remit-service`. It should allow users to save rules to the DB and provide an endpoint to trigger `execute_remittance` on-chain. Build a small 'Event Watcher' service in Rust that listens for `RemittanceExecuted` events and logs them to the database."

### Day 7: Frontend: Onboarding & Wallet Setup
*   **Goal:** Start the User App.
*   **Prompt:** "Initialize the `frontend/diaspora-app` using Expo and TypeScript. Build the onboarding screens: Welcome, Phone Auth (mocked), and Wallet Creation. Integrate `stellar-sdk` to generate a keypair locally and save it securely (simulated)."

### Day 8: Frontend: The "Send Money" Flow
*   **Goal:** The core user action.
*   **Prompt:** "Build the 'Send Money' screen in the Diaspora App. Implement the UI for selecting a beneficiary, entering an amount, and seeing the 'Auto-Save' split preview (70/30). Connect this to the `remit-service` API created on Day 6."

### Day 9: Savings Dashboard & Yield Charts
*   **Goal:** Visualizing the "Save" in RemitSave.
*   **Prompt:** "Create the Savings Dashboard in the frontend. Display the user's active goals, progress rings for targets, and a 'Yield Earned' summary. Use mock data for the yield chart but pull actual 'SavingsPlan' balances from the backend/contract."

### Day 10: Integration & Contributor Foundation
*   **Goal:** Finalize the developer experience.
*   **Prompt:** "Create a `scripts/bootstrap.sh` that installs dependencies, builds contracts, runs migrations, and starts all services via Docker. Update `CONTRIBUTING.md` with setup instructions and a 'Good First Issues' list based on the remaining 35% (e.g., USSD gateway, real KYC integration, multi-sig admin)."

---

## 65% Completion Definition
*   **Contracts (90%):** All core logic finished and tested.
*   **Backend (60%):** Core API and Event Watcher functional; logic for real Anchor integration pending.
*   **Frontend (60%):** Main Diaspora app flows complete; Beneficiary PWA and USSD pending.
*   **Infra (100%):** Local development environment is fully automated.
