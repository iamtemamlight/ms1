use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::Json;
use chrono::Utc;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::{BroadcastStream, IntervalStream};
use tokio_stream::{Stream, StreamExt};
use tracing::info;

pub const DEFAULT_THRESHOLD_ETH: f64 = 0.05;
pub const DEFAULT_CHECK_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_COOLDOWN_SECS: u64 = 300;
pub const DEFAULT_MAX_TRANSFER_ETH: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTransferConfig {
    pub threshold_eth: f64,
    pub check_interval_secs: u64,
    pub cooldown_secs: u64,
    pub max_transfer_eth: f64,
    pub enabled: bool,
}

impl Default for AutoTransferConfig {
    fn default() -> Self {
        Self {
            threshold_eth: DEFAULT_THRESHOLD_ETH,
            check_interval_secs: DEFAULT_CHECK_INTERVAL_SECS,
            cooldown_secs: DEFAULT_COOLDOWN_SECS,
            max_transfer_eth: DEFAULT_MAX_TRANSFER_ETH,
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TransferStatus {
    #[default]
    Idle,
    InFlight,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTransferEvent {
    pub at: String,
    pub status: TransferStatus,
    pub profit_eth: f64,
    pub amount_eth: f64,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

impl AutoTransferEvent {
    fn idle(profit_eth: f64, reason: &str) -> Self {
        Self {
            at: now_iso(),
            status: TransferStatus::Idle,
            profit_eth,
            amount_eth: 0.0,
            tx_hash: None,
            error: Some(reason.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct AutoTransferState {
    pub config: AutoTransferConfig,
    pub last_transfer_at: Option<Instant>,
    pub last_checked_at: Option<Instant>,
    pub total_transferred_eth: f64,
    pub swept_profit_eth: f64,
    pub cached_accumulated_eth: f64,
    pub transfers_executed: u64,
    pub last_event: Option<AutoTransferEvent>,
}

pub struct AutoTransferScheduler {
    pub state: Arc<Mutex<AutoTransferState>>,
    pub event_tx: broadcast::Sender<AutoTransferEvent>,
}

static SCHEDULER: OnceCell<AutoTransferScheduler> = OnceCell::new();

pub fn init_global(config: AutoTransferConfig) -> &'static AutoTransferScheduler {
    SCHEDULER.get_or_init(|| {
        let (event_tx, _rx) = broadcast::channel(256);
        AutoTransferScheduler {
            state: Arc::new(Mutex::new(AutoTransferState {
                config,
                ..Default::default()
            })),
            event_tx,
        }
    })
}

pub fn global() -> Option<&'static AutoTransferScheduler> {
    SCHEDULER.get()
}

pub fn accumulated_profit_eth() -> f64 {
    match crate::TRADE_RECORDS.get() {
        Some(records) => {
            let mut total = 0.0_f64;
            for entry in records.iter() {
                total += entry.value().net_profit_eth;
            }
            total
        }
        None => 0.0,
    }
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub async fn simulate_smart_contract_call(amount_eth: f64) -> Result<String, String> {
    // PRODUCTION INTEGRATION POINT: replace with a real on-chain sweep, e.g.
    //   let provider = ethers::prelude::Provider::<Http>::try_from(rpc_url)?;
    //   let contract = AutoTransfer::new(contract_addr, provider);
    //   let tx = contract.sweep_profit(parse_eth(amount_eth)).send().await?;
    //   Ok(format!("{:#x}", tx.tx_hash()))
    info!(
        "SIMULATED on-chain transfer of {:.6} ETH (production contract call not yet wired)",
        amount_eth
    );
    tokio::time::sleep(Duration::from_millis(25)).await;
    let mut bytes = [0u8; 32];
    for b in bytes.iter_mut() {
        *b = rand::random::<u8>();
    }
    Ok(format!("0x{}", hex::encode(bytes)))
}

impl AutoTransferScheduler {
    pub async fn refresh_accumulated(&self) {
        let mut state = self.state.lock().await;
        state.cached_accumulated_eth = accumulated_profit_eth();
    }

    async fn execute_transfer(&self, amount_eth: f64, profit_eth: f64) -> AutoTransferEvent {
        match simulate_smart_contract_call(amount_eth).await {
            Ok(tx_hash) => {
                let mut state = self.state.lock().await;
                state.total_transferred_eth += amount_eth;
                state.swept_profit_eth += amount_eth;
                state.transfers_executed += 1;
                state.last_transfer_at = Some(Instant::now());
                let ev = AutoTransferEvent {
                    at: now_iso(),
                    status: TransferStatus::Completed,
                    profit_eth,
                    amount_eth,
                    tx_hash: Some(tx_hash),
                    error: None,
                };
                state.last_event = Some(ev.clone());
                ev
            }
            Err(e) => {
                let mut state = self.state.lock().await;
                let ev = AutoTransferEvent {
                    at: now_iso(),
                    status: TransferStatus::Failed,
                    profit_eth,
                    amount_eth,
                    tx_hash: None,
                    error: Some(e),
                };
                state.last_event = Some(ev.clone());
                ev
            }
        }
    }

    pub async fn check_and_trigger_auto_transfer(&self) -> AutoTransferEvent {
        let decision = {
            let mut state = self.state.lock().await;
            state.last_checked_at = Some(Instant::now());
            state.cached_accumulated_eth = accumulated_profit_eth();
            if !state.config.enabled {
                let ev = AutoTransferEvent::idle(0.0, "scheduler disabled");
                state.last_event = Some(ev.clone());
                return ev;
            }
            let unswept = (state.cached_accumulated_eth - state.swept_profit_eth).max(0.0);
            let cooldown_ok = state
                .last_transfer_at
                .map_or(true, |t| t.elapsed() >= Duration::from_secs(state.config.cooldown_secs));
            if unswept >= state.config.threshold_eth && cooldown_ok {
                state.config.max_transfer_eth.min(unswept)
            } else {
                let reason = if unswept < state.config.threshold_eth {
                    format!(
                        "unswept profit {:.6} ETH below threshold {:.6}",
                        unswept, state.config.threshold_eth
                    )
                } else {
                    "cooldown active".to_string()
                };
                let ev = AutoTransferEvent::idle(unswept, &reason);
                state.last_event = Some(ev.clone());
                return ev;
            }
        };

        let ev = self.execute_transfer(decision, decision).await;
        let _ = self.event_tx.send(ev.clone());
        ev
    }

    pub async fn manual_trigger(&self) -> AutoTransferEvent {
        let decision = {
            let state = self.state.lock().await;
            if !state.config.enabled {
                return AutoTransferEvent::idle(0.0, "scheduler disabled");
            }
            let unswept = (state.cached_accumulated_eth - state.swept_profit_eth).max(0.0);
            let cooldown_ok = state
                .last_transfer_at
                .map_or(true, |t| t.elapsed() >= Duration::from_secs(state.config.cooldown_secs));
            if !cooldown_ok {
                return AutoTransferEvent::idle(unswept, "cooldown active");
            }
            if unswept <= 0.0 {
                return AutoTransferEvent::idle(unswept, "no unswept profit to transfer");
            }
            state.config.max_transfer_eth.min(unswept)
        };

        let ev = self.execute_transfer(decision, decision).await;
        let _ = self.event_tx.send(ev.clone());
        ev
    }

    pub async fn run_periodic_check(&self) {
        let (interval_secs, threshold) = {
            let s = self.state.lock().await;
            (s.config.check_interval_secs.max(1), s.config.threshold_eth)
        };
        let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        info!(
            "Auto-transfer scheduler started: interval={}s threshold={:.6} ETH",
            interval_secs, threshold
        );
        loop {
            ticker.tick().await;
            let ev = self.check_and_trigger_auto_transfer().await;
            if ev.status != TransferStatus::Idle {
                info!(
                    "Auto-transfer event: {:?} amount={:.6} tx={:?}",
                    ev.status, ev.amount_eth, ev.tx_hash
                );
            }
        }
    }
}

async fn build_status() -> Value {
    let empty = serde_json::json!({ "error": "auto-transfer scheduler not initialized" });
    let sched = match global() {
        Some(s) => s,
        None => return empty,
    };
    let state = sched.state.lock().await;
    let accumulated = state.cached_accumulated_eth;
    let unswept = (accumulated - state.swept_profit_eth).max(0.0);
    let next_check_in = match state.last_checked_at {
        Some(t) => {
            let elapsed = t.elapsed().as_secs();
            let iv = state.config.check_interval_secs;
            if elapsed >= iv { 0 } else { iv - elapsed }
        }
        None => state.config.check_interval_secs,
    };
    serde_json::json!({
        "enabled": state.config.enabled,
        "threshold_eth": crate::round8(state.config.threshold_eth),
        "max_transfer_eth": crate::round8(state.config.max_transfer_eth),
        "check_interval_secs": state.config.check_interval_secs,
        "cooldown_secs": state.config.cooldown_secs,
        "accumulated_profit_eth": crate::round8(accumulated),
        "swept_profit_eth": crate::round8(state.swept_profit_eth),
        "unswept_profit_eth": crate::round8(unswept),
        "total_transferred_eth": crate::round8(state.total_transferred_eth),
        "transfers_executed": state.transfers_executed,
        "next_check_in_secs": next_check_in,
        "last_event": state.last_event,
    })
}

pub async fn get_status() -> Result<Json<Value>, crate::AppError> {
    let sched = global()
        .ok_or_else(|| crate::AppError::Internal("auto-transfer scheduler not initialized".into()))?;
    sched.refresh_accumulated().await;
    Ok(Json(build_status().await))
}

pub async fn post_trigger(headers: HeaderMap) -> Result<Json<Value>, crate::AppError> {
    match std::env::var("AUTO_TRANSFER_ADMIN_KEY") {
        Ok(expected) => {
            let provided = headers
                .get("x-auto-transfer-key")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if provided != expected {
                return Err(crate::AppError::Forbidden("invalid auto-transfer admin key".into()));
            }
        }
        Err(_) => {
            return Err(crate::AppError::Forbidden(
                "auto-transfer admin key not configured (set AUTO_TRANSFER_ADMIN_KEY)".into(),
            ));
        }
    }
    let sched = global()
        .ok_or_else(|| crate::AppError::Internal("auto-transfer scheduler not initialized".into()))?;
    let ev = sched.manual_trigger().await;
    Ok(Json(serde_json::json!({
        "triggered": ev.status == TransferStatus::Completed,
        "event": ev,
    })))
}

pub async fn stream_events() -> Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>> {
    let scheduler = match global() {
        Some(s) => s,
        None => {
            let fallback: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
                Box::pin(futures::stream::once(async {
                    Ok(Event::default().event("error").data("auto-transfer scheduler not initialized"))
                }));
            return Sse::new(fallback).keep_alive(KeepAlive::default());
        }
    };

    let event_rx = scheduler.event_tx.subscribe();
    let event_stream = BroadcastStream::new(event_rx).map(|res| match res {
        Ok(ev) => Ok(Event::default()
            .event("transfer")
            .json_data(&ev)
            .unwrap_or_else(|_| Event::default().data("serialize-error"))),
        Err(_) => Ok(Event::default().event("transfer").data("lagged")),
    });

    let ticker = IntervalStream::new(tokio::time::interval(Duration::from_secs(5)));
    let heartbeat = ticker.then(move |_| async move {
        let snapshot = build_status().await;
        Ok(Event::default()
            .event("profit")
            .json_data(&snapshot)
            .unwrap_or_else(|_| Event::default().data("serialize-error")))
    });

    let merged: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(event_stream.merge(heartbeat));
    Sse::new(merged).keep_alive(KeepAlive::default())
}
