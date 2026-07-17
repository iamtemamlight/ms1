// ==============================================================================
// Sub-Microsecond Latency Optimization Modules
// Phase 1: Core Infrastructure
// ==============================================================================

pub mod memory_pool;
pub mod lock_free_queue;
pub mod state_prediction;
pub mod simd_fixed_point;
pub mod branchless_execution;
pub mod predictive_batching;
pub mod zero_copy_network;
pub mod unified_engine;

#[cfg(test)]
mod benches;

pub use memory_pool::MemoryPool;
pub use lock_free_queue::{LockFreeQueue, SpscQueue};
pub use state_prediction::{StatePredictionTable, PredictedState};
pub use simd_fixed_point::{SimdFixedPoint, SimdBatchProcessor, Fixed64};
pub use branchless_execution::{BranchlessMask, BranchlessValidator, branchless_select, branchless_select_i64, branchless_validation_mask, BitwiseReciprocalTable, DirectStepArray};
pub use predictive_batching::{BatchOptimizer, BatchingWindow, Opportunity, BatchResult, PreComputedMatrix};
pub use zero_copy_network::{ZeroCopyMessage, ZeroCopyRingBuffer, BinaryProtocolEncoder, BinaryProtocolDecoder};
pub use unified_engine::{UnifiedEngine, UnifiedEngineConfig, PipelineResult};

#[cfg(test)]
pub use benches::run_all_benchmarks;
