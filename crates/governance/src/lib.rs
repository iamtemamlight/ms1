//! AllBright AgentOS — Governance crate (Phase A).
//!
//! Implements the constitutional governance primitives from `AllBright.Agent.md`:
//! - `Gatekeeper`: enforces the 7 §11.2 validation criteria + Zero Trust gate.
//! - `ReflectionEngine`: produces the 5 Reflection Cards from verified signals.
//! - `audit`: append-only, persistence-backed audit trail (survives restart).

pub mod gatekeeper;
pub mod reflection;
pub mod audit;
pub mod agents;
pub mod orchestrator;
pub mod layers;
pub mod deployment;
pub mod benchmark;
pub mod acceptance;
pub mod api;