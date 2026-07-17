//! Governance API surface (spec §21 / future HTTP bindings).
//!
//! Placeholder for the future HTTP/gRPC layer. The engine types are re-exported here
//! so consumers can build an API without touching the internal modules.

pub use crate::audit::{AuditEntry, AuditStore};
pub use crate::gatekeeper::{Decision, Gatekeeper, GatekeeperVerdict, Reflection};
pub use crate::reflection::{CardStatus, Observation, ReflectionCard, ReflectionEngine, CARD_IDS};
pub use crate::agents::{
    AgentId, AgentRole, EngineeringAgent, IndependenceViolation, IndependentAuditorAgent,
    ensure_independent_verifier,
};
pub use crate::orchestrator::{GovernanceOrchestrator, OrchestrationResult};
pub use crate::layers::{GovernanceLayer, layer_for_card};
pub use crate::deployment::{
    DeploymentCondition, DeploymentGate, DeploymentReadiness, DeploymentState,
};
pub use crate::benchmark::{BenchmarkDomain, BenchmarkEngine, DomainScore};
pub use crate::acceptance::{AcceptanceItem, acceptance_report, acceptance_passed};