use std::sync::atomic::{AtomicU64, Ordering};
use crate::GlobalFleetState;

pub struct TelemetryService {
    pub events_emitted: AtomicU64,
}

impl TelemetryService {
    pub fn new() -> Self {
        Self {
            events_emitted: AtomicU64::new(0),
        }
    }

    pub fn record_fleet_telemetry(&self, state: &GlobalFleetState) {
        let _ = state;
        self.events_emitted.fetch_add(1, Ordering::SeqCst);
    }
}
