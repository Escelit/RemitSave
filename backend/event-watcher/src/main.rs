use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use serde::{Deserialize, Serialize};
use stellar_xdr::curr::{ScVal, Limits, ReadXdr, ScAddress};
use base64::{engine::general_purpose, Engine as _};
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u32,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    result: Option<T>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GetEventsResponse {
    events: Vec<EventResult>,
    #[serde(rename = "latestLedger")]
    latest_ledger: String,
}

#[derive(Debug, Deserialize)]
struct EventResult {
    #[serde(rename = "ledger")]
    ledger: String,
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "pagingToken")]
    paging_token: String,
    #[serde(rename = "topic")]
    topic: Vec<String>,
    #[serde(rename = "value")]
    value: EventValue,
}

#[derive(Debug, Deserialize)]
struct EventValue {
    #[serde(rename = "xdr")]
    xdr: String,
}

#[derive(Debug)]
struct RemittanceExecuted {
    remittance_id: u32,
    sender: String,
    beneficiary: String,
    total_amount: i128,
    payout_amount: i128,
    savings_amount: i128,
    fee_amount: i128,
    incoming_asset: String,
    local_asset: String,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "event_watcher=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://remitsave:remitsave_dev@localhost:5432/remitsave".into());
    let poll_interval_secs = std::env::var("POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);

    let pool = backend_shared::init_pool(&database_url).await;

    let soroban_rpc = std::env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8000/soroban/rpc".into());
    let contract_id = std::env::var("REMIT_CONTRACT_ID")
        .unwrap_or_else(|_| "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into());

    tracing::info!(
        "event-watcher starting, polling every {}s from {} for contract {}",
        poll_interval_secs,
        soroban_rpc,
        contract_id
    );

    let mut interval = tokio::time::interval(Duration::from_secs(poll_interval_secs));
    let mut last_ledger: u32 = 0;

    loop {
        interval.tick().await;
        match poll_events(&pool, &soroban_rpc, &contract_id, last_ledger).await {
            Ok(new_last_ledger) => {
                if new_last_ledger > 0 {
                    last_ledger = new_last_ledger;
                }
            }
            Err(e) => {
                tracing::error!("poll cycle failed: {e}");
            }
        }
    }
}

async fn poll_events(
    pool: &PgPool,
    rpc_url: &str,
    contract_id: &str,
    start_ledger: u32,
) -> Result<u32, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // 1. Get current ledger to know where we are if start_ledger is 0
    let current_ledger = if start_ledger == 0 {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "getLatestLedger".into(),
            params: serde_json::json!({}),
        };
        let resp: JsonRpcResponse<serde_json::Value> = client.post(rpc_url).json(&req).send().await?.json().await?;
        let ledger = resp.result.and_then(|v| v.get("sequence").and_then(|s| s.as_u64())).unwrap_or(1) as u32;
        ledger.saturating_sub(100) // Start from 100 ledgers ago
    } else {
        start_ledger
    };

    // 2. Poll for events
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: 2,
        method: "getEvents".into(),
        params: serde_json::json!({
            "startLedger": current_ledger,
            "filters": [
                {
                    "type": "contract",
                    "contractIds": [contract_id]
                }
            ]
        }),
    };

    let resp: JsonRpcResponse<GetEventsResponse> = client.post(rpc_url).json(&req).send().await?.json().await?;
    
    if let Some(err) = resp.error {
        return Err(format!("RPC Error: {:?}", err).into());
    }

    let result = resp.result.ok_or("No result in getEvents response")?;
    let latest_ledger = result.latest_ledger.parse::<u32>().unwrap_or(current_ledger);

    for event in result.events {
        // Topic 0 should be "remittance_executed"
        if event.topic.is_empty() { continue; }
        
        let topic0_xdr = general_purpose::STANDARD.decode(&event.topic[0])?;
        let topic0 = ScVal::from_xdr(&topic0_xdr, Limits::none())?;
        
        match topic0 {
            ScVal::Symbol(s) if s.to_string() == "remittance_executed" => {
                let value_xdr = general_purpose::STANDARD.decode(&event.value.xdr)?;
                let value = ScVal::from_xdr(&value_xdr, Limits::none())?;
                
                if let Some(remittance) = parse_remittance_event(value) {
                    tracing::info!("Found RemittanceExecuted event: {:?}", remittance);
                    save_event(pool, remittance, &event.id).await?;
                }
            }
            _ => {}
        }
    }

    Ok(latest_ledger + 1)
}

fn parse_remittance_event(val: ScVal) -> Option<RemittanceExecuted> {
    if let ScVal::Map(Some(map)) = val {
        let mut remittance_id = 0u32;
        let mut sender = String::new();
        let mut beneficiary = String::new();
        let mut total_amount = 0i128;
        let mut payout_amount = 0i128;
        let mut savings_amount = 0i128;
        let mut fee_amount = 0i128;
        let mut incoming_asset = String::new();
        let mut local_asset = String::new();
        let mut timestamp = 0u64;

        for entry in map.iter() {
            if let ScVal::Symbol(key) = &entry.key {
                let key_str = key.to_string();
                match key_str.as_str() {
                    "remittance_id" => remittance_id = parse_u32(&entry.val)?,
                    "sender" => sender = parse_address(&entry.val)?,
                    "beneficiary" => beneficiary = parse_address(&entry.val)?,
                    "total_amount" => total_amount = parse_i128(&entry.val)?,
                    "payout_amount" => payout_amount = parse_i128(&entry.val)?,
                    "savings_amount" => savings_amount = parse_i128(&entry.val)?,
                    "fee_amount" => fee_amount = parse_i128(&entry.val)?,
                    "incoming_asset" => incoming_asset = parse_address(&entry.val)?,
                    "local_asset" => local_asset = parse_address(&entry.val)?,
                    "timestamp" => timestamp = parse_u64(&entry.val)?,
                    _ => {}
                }
            }
        }
        
        Some(RemittanceExecuted {
            remittance_id, sender, beneficiary, total_amount,
            payout_amount, savings_amount, fee_amount,
            incoming_asset, local_asset, timestamp
        })
    } else {
        None
    }
}

fn parse_u32(val: &ScVal) -> Option<u32> {
    if let ScVal::U32(v) = val { Some(*v) } else { None }
}

fn parse_u64(val: &ScVal) -> Option<u64> {
    if let ScVal::U64(v) = val { Some(*v) } else { None }
}

fn parse_i128(val: &ScVal) -> Option<i128> {
    if let ScVal::I128(v) = val {
        let hi = v.hi as i128;
        let lo = v.lo as i128;
        Some((hi << 64) | lo)
    } else {
        None
    }
}

fn parse_address(val: &ScVal) -> Option<String> {
    if let ScVal::Address(addr) = val {
        match addr {
            ScAddress::Account(id) => Some(id.to_string()),
            ScAddress::Contract(id) => {
                let mut buf = [0u8; 32];
                buf.copy_from_slice(&id.0);
                // For simplicity in this mock, we'll just return the hex or a placeholder
                // In real life, we'd use stellar_strkey to encode it
                Some(hex::encode(buf))
            }
        }
    } else {
        None
    }
}

async fn save_event(pool: &PgPool, ev: RemittanceExecuted, tx_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    // We need to find the user_id from the sender address
    // The sender address in the event is a Stellar address (e.g. G...)
    // Our users table has a stellar_address column.
    
    let user_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM users WHERE stellar_address = $1"
    )
    .bind(&ev.sender)
    .fetch_optional(pool)
    .await?;
    
    let user_id = match user_id {
        Some(id) => id,
        None => {
            // If user not found, we might want to skip or use a default
            tracing::warn!("User with stellar address {} not found in DB", ev.sender);
            return Ok(());
        }
    };

    // Upsert into remittance_events
    // We'll use the remittance_id from the contract to avoid duplicates
    sqlx::query(
        r#"
        INSERT INTO remittance_events (id, remittance_id, user_id, beneficiary,
                                        total_amount, payout_amount, savings_amount,
                                        fee_amount, incoming_asset, local_asset,
                                        status, tx_hash, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'completed', $11, $12)
        ON CONFLICT (id) DO NOTHING
        "#
    )
    .bind(uuid::Uuid::new_v4())
    .bind(ev.remittance_id as i32)
    .bind(user_id)
    .bind(&ev.beneficiary)
    .bind(BigDecimal::from(ev.total_amount))
    .bind(BigDecimal::from(ev.payout_amount))
    .bind(BigDecimal::from(ev.savings_amount))
    .bind(BigDecimal::from(ev.fee_amount))
    .bind(&ev.incoming_asset)
    .bind(&ev.local_asset)
    .bind(tx_hash)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}
