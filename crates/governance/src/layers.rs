//! Governance Layers (spec §7).
//!
//! The AllBright Governance Framework consists of five governance layers. Layer 5
//! (Zero Trust) is the cross-cutting policy that governs every other layer.
//!
//! This module is the canonical, typed model of those five layers and the
//! mapping from each Reflection Card to its governing layer. It is deliberately
//! lightweight so it can seed Zero-Trust scoping without coupling the governance
//! crate to the (heavier) backend.

use serde::{Deserialize, Serialize};

/// The five governance layers (spec §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceLayer {
    /// Layer 1: AllBright System Governance (architecture, modules, infra, ops, performance).
    System,
    /// Layer 2: Copilot Governance (engineering assistance, recommendations, AI reasoning).
    Copilot,
    /// Layer 3: Intelligence Governance (external intelligence, blockchain, market, context).
    Intelligence,
    /// Layer 4: Commander Governance (strategic decisions, approvals, risk acceptance).
    Commander,
    /// Layer 5: Zero Trust Governance — the cross-cutting policy over every other layer.
    ZeroTrust,
}

impl GovernanceLayer {
    /// All five layers in canonical order (§7).
    pub fn all() -> [GovernanceLayer; 5] {
        [
            GovernanceLayer::System,
            GovernanceLayer::Copilot,
            GovernanceLayer::Intelligence,
            GovernanceLayer::Commander,
            GovernanceLayer::ZeroTrust,
        ]
    }

    /// Human-readable name used in reflections and audit records.
    pub fn display_name(&self) -> &'static str {
        match self {
            GovernanceLayer::System => "AllBright System Governance",
            GovernanceLayer::Copilot => "Copilot Governance",
            GovernanceLayer::Intelligence => "Intelligence Governance",
            GovernanceLayer::Commander => "Commander Governance",
            GovernanceLayer::ZeroTrust => "Zero Trust Governance",
        }
    }

    /// Whether this layer is itself governed by the Zero-Trust policy (§7, §8).
    /// Only the Zero Trust layer is its own authority; every other layer is
    /// subject to Zero-Trust verification.
    pub fn subject_to_zero_trust(&self) -> bool {
        !matches!(self, GovernanceLayer::ZeroTrust)
    }
}

/// Map a Reflection Card id (spec §10) to its governing [`GovernanceLayer`].
pub fn layer_for_card(card_id: &str) -> Option<GovernanceLayer> {
    match card_id {
        "allbright" => Some(GovernanceLayer::System),
        "copilot" => Some(GovernanceLayer::Copilot),
        "intelligence" => Some(GovernanceLayer::Intelligence),
        "commander" => Some(GovernanceLayer::Commander),
        "zerotrust" => Some(GovernanceLayer::ZeroTrust),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_layers_defined() {
        assert_eq!(GovernanceLayer::all().len(), 5);
        // Non-Zero-Trust layers are subject to Zero-Trust verification.
        assert!(GovernanceLayer::System.subject_to_zero_trust());
        assert!(!GovernanceLayer::ZeroTrust.subject_to_zero_trust());
    }

    #[test]
    fn cards_map_to_layers() {
        assert_eq!(layer_for_card("allbright"), Some(GovernanceLayer::System));
        assert_eq!(layer_for_card("copilot"), Some(GovernanceLayer::Copilot));
        assert_eq!(layer_for_card("intelligence"), Some(GovernanceLayer::Intelligence));
        assert_eq!(layer_for_card("commander"), Some(GovernanceLayer::Commander));
        assert_eq!(layer_for_card("zerotrust"), Some(GovernanceLayer::ZeroTrust));
        assert_eq!(layer_for_card("unknown"), None);
    }
}
