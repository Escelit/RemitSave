#!/usr/bin/env npx tsx
/**
 * RemitSave Africa — Day 4: TypeScript Deploy Script
 *
 * Deploys the remit-save contract to a local Soroban network
 * using the soroban CLI for contract operations and the
 * Stellar SDK for account management.
 *
 * Usage: npx tsx deploy.ts
 */

import * as StellarSdk from "@stellar/stellar-sdk";
import { execSync } from "child_process";
import * as fs from "fs";

const RPC_URL = "http://localhost:8000/soroban/rpc";
const FRIENDBOT_URL = "http://localhost:8000/friendbot";
const NETWORK_PASSPHRASE = "Standalone Network ; February 2017";

function run(cmd: string, cwd?: string): string {
  return execSync(cmd, { cwd, encoding: "utf8", stdio: ["inherit", "pipe", "pipe"] }).trim();
}

async function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function fundAccount(publicKey: string) {
  console.log(`  Funding ${publicKey}...`);
  const res = await fetch(`${FRIENDBOT_URL}?addr=${publicKey}`);
  if (!res.ok) console.warn(`  Friendbot warning: ${await res.text()}`);
}

async function main() {
  console.log("--- Day 4: Stellar Integration & Deployment (TypeScript) ---\n");

  // 1. Build contracts
  console.log("Building Soroban contracts...");
  run("cargo build --target wasm32-unknown-unknown --release", "../contracts");
  console.log("Contracts built.\n");

  // 2. Set up network in soroban CLI
  console.log("Configuring soroban CLI network...");
  try {
    run(`soroban network add local --rpc-url "${RPC_URL}" --network-passphrase "${NETWORK_PASSPHRASE}"`);
  } catch {
    console.log("  Network 'local' already exists.\n");
  }

  // 3. Generate identities
  for (const name of ["admin", "user1", "beneficiary1"]) {
    try {
      run(`soroban config identity generate ${name}`);
    } catch {
      console.log(`  Identity '${name}' already exists.`);
    }
  }

  const ADMIN_ADDR = run("soroban config identity address admin");
  const USER_ADDR = run("soroban config identity address user1");
  const BENE_ADDR = run("soroban config identity address beneficiary1");

  console.log("\nAccounts:");
  console.log(`  Admin:        ${ADMIN_ADDR}`);
  console.log(`  User:         ${USER_ADDR}`);
  console.log(`  Beneficiary:  ${BENE_ADDR}`);

  // 4. Fund accounts
  console.log("\nFunding accounts...");
  await fundAccount(ADMIN_ADDR);
  await fundAccount(USER_ADDR);
  await fundAccount(BENE_ADDR);
  await sleep(2000);
  console.log("All funded.\n");

  // 5. Deploy contract
  console.log("Deploying remit-save contract...");
  const wasmPath = "../contracts/remit-save/target/wasm32-unknown-unknown/release/remit_save.wasm";
  const CONTRACT_ID = run(
    `soroban contract deploy --wasm ${wasmPath} --source admin --network local`
  );
  console.log(`  Contract ID: ${CONTRACT_ID}\n`);

  // 6. Initialize
  console.log("Initializing contract...");
  run(
    `soroban contract invoke --id ${CONTRACT_ID} --source admin --network local -- initialize ` +
    `--admin ${ADMIN_ADDR} --fee_recipient ${ADMIN_ADDR} --protocol_fee_bps 50`
  );
  console.log("  Initialized.\n");

  // 7. Register user
  console.log("Registering user...");
  run(
    `soroban contract invoke --id ${CONTRACT_ID} --source user1 --network local -- ` +
    `register_user --user ${USER_ADDR} --country 'NG' --phone '2348012345678'`
  );
  console.log("  User registered.\n");

  // 8. Create savings plan
  console.log("Creating savings plan...");
  const PLAN_ID = run(
    `soroban contract invoke --id ${CONTRACT_ID} --source user1 --network local -- ` +
    `create_savings_plan --owner ${USER_ADDR} --goal_name 'School Fees' --target_amount 10000 ` +
    `--local_asset ${ADMIN_ADDR} --lock_until 'null'`
  ).replace(/"/g, "");
  console.log(`  Plan ID: ${PLAN_ID}\n`);

  // 9. Set remittance rule (70/30 split)
  console.log("Setting up remittance rule (70/30 split)...");
  run(
    `soroban contract invoke --id ${CONTRACT_ID} --source user1 --network local -- ` +
    `set_remittance_rule --sender ${USER_ADDR} --rule ` +
    `'{"sender":"${USER_ADDR}","beneficiary":"${BENE_ADDR}","incoming_asset":"${ADMIN_ADDR}",` +
    `"local_asset":"${ADMIN_ADDR}","split_type":"Percentage","split_value":3000,` +
    `"savings_plan_id":${PLAN_ID},"active":true}'`
  );
  console.log("  Rule set.\n");

  // 10. Summary
  console.log("--- Deployment Complete ---");
  console.log(`RemitSave Contract: ${CONTRACT_ID}`);
  console.log(`Admin:              ${ADMIN_ADDR}`);
  console.log(`User:               ${USER_ADDR}`);

  fs.writeFileSync(
    ".deployment.json",
    JSON.stringify(
      {
        contractId: CONTRACT_ID,
        admin: ADMIN_ADDR,
        user: USER_ADDR,
        beneficiary: BENE_ADDR,
        savingsPlanId: Number(PLAN_ID),
        timestamp: new Date().toISOString(),
      },
      null,
      2
    )
  );
  console.log("\nDeployment info saved to .deployment.json");
}

main().catch((err) => {
  console.error("Deployment failed:", err);
  process.exit(1);
});
