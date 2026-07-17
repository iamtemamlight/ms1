// ==============================================================================
// M003: Transaction Batcher
// Purpose: Batch multiple transactions for efficient execution
// CGM Subsystem: Velocity
// ==============================================================================

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TransactionBatch {
    pub batch_id: String,
    pub transactions: Vec<Transaction>,
    pub created_at: String,
    pub max_size: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: String,
    pub to: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
    Pending,
    Ready,
    Executing,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub batch_id: String,
    pub status: BatchStatus,
    pub executed_count: usize,
    pub failed_count: usize,
    pub total_gas_used: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug)]
pub struct TransactionBatcher {
    pub enabled: bool,
    pub pending_batches: VecDeque<TransactionBatch>,
    pub completed_batches: VecDeque<BatchResult>,
    pub max_batch_size: usize,
    pub batch_timeout: Duration,
    pub total_batched: u64,
    pub total_executed: u64,
}

impl TransactionBatcher {
    pub fn new() -> Self {
        Self {
            enabled: true,
            pending_batches: VecDeque::new(),
            completed_batches: VecDeque::with_capacity(1000),
            max_batch_size: 100,
            batch_timeout: Duration::from_millis(5000),
            total_batched: 0,
            total_executed: 0,
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Option<String> {
        if !self.enabled {
            return None;
        }

        self.total_batched += 1;

        if self.pending_batches.is_empty() || self.pending_batches.back()?.transactions.len() >= self.max_batch_size {
            let batch = TransactionBatch {
                batch_id: format!("BATCH-{:06}", self.total_batched),
                transactions: vec![tx],
                created_at: chrono::Utc::now().to_rfc3339(),
                max_size: self.max_batch_size,
                timeout_ms: self.batch_timeout.as_millis() as u64,
            };
            self.pending_batches.push_back(batch);
        } else {
            self.pending_batches.back_mut()?.transactions.push(tx);
        }

        Some(self.pending_batches.back()?.batch_id.clone())
    }

    pub fn get_ready_batch(&mut self) -> Option<TransactionBatch> {
        if let Some(batch) = self.pending_batches.front() {
            let age = chrono::Utc::now()
                - chrono::DateTime::parse_from_rfc3339(&batch.created_at)
                    .unwrap()
                    .with_timezone(&chrono::Utc);
            if batch.transactions.len() >= self.max_batch_size || age.num_milliseconds() >= self.batch_timeout.as_millis() as i64 {
                return self.pending_batches.pop_front();
            }
        }
        None
    }

    pub fn execute_batch(&mut self, batch_id: &str) -> Result<BatchResult, String> {
        let start = Instant::now();
        
        let mut result = BatchResult {
            batch_id: batch_id.to_string(),
            status: BatchStatus::Executing,
            executed_count: 0,
            failed_count: 0,
            total_gas_used: 0,
            execution_time_ms: 0,
        };

        for batch in self.pending_batches.iter() {
            if batch.batch_id == batch_id {
                result.executed_count = batch.transactions.len();
                result.status = BatchStatus::Completed;
                result.total_gas_used = batch.transactions.len() as u64 * 21000;
                break;
            }
        }

        result.execution_time_ms = start.elapsed().as_millis() as u64;
        self.total_executed += result.executed_count as u64;
        self.completed_batches.push_back(result.clone());

        Ok(result)
    }

    pub fn get_stats(&self) -> String {
        let success_rate = if self.total_batched > 0 {
            (self.total_executed as f64 / self.total_batched as f64) * 100.0
        } else {
            0.0
        };

        format!(
            r#"{{"total_batched":{},"total_executed":{},"pending":{},"success_rate":{:.1}}}"#,
            self.total_batched,
            self.total_executed,
            self.pending_batches.len(),
            success_rate
        )
    }
}
