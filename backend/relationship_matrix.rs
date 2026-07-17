// AllBright Enterprise Relationship Intelligence System
// 6x6 Subsystem Relationship Matrix - AIGUIDE Part IV

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Subsystem identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Subsystem {
    Profit,
    Growth,
    Velocity,
    Efficiency,
    Security,
    Quality,
}

impl Subsystem {
    pub fn all() -> [Self; 6] {
        [Self::Profit, Self::Growth, Self::Velocity, Self::Efficiency, Self::Security, Self::Quality]
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Self::Profit => "Profit",
            Self::Growth => "Growth",
            Self::Velocity => "Velocity",
            Self::Efficiency => "Efficiency",
            Self::Security => "Security",
            Self::Quality => "Quality",
        }
    }
}

/// Relationship type per AIGUIDE
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    Reinforcing,   // Positive feedback loop
    Balancing,     // Stabilizing effect
    Constraining,  // Limiting effect
}

/// Causal relationship between subsystems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipNode {
    pub influencer: Subsystem,
    pub influenced: Subsystem,
    pub strength: f64,      // 0.0 - 1.0
    pub relationship_type: RelationshipType,
    pub time_lag_seconds: u64,
    pub confidence: f64,    // 0.0 - 1.0
    pub stability: f64,     // 0.0 - 1.0
}

/// 6x6 Relationship Matrix
pub struct RelationshipMatrix {
    pub matrix: HashMap<(Subsystem, Subsystem), RelationshipNode>,
}

impl Default for RelationshipMatrix {
    fn default() -> Self {
        let mut matrix = HashMap::new();
        
        // Initialize with neutral relationships
        for influencer in Subsystem::all() {
            for influenced in Subsystem::all() {
                if influencer != influenced {
                    matrix.insert((influencer, influenced), RelationshipNode {
                        influencer,
                        influenced,
                        strength: 0.5,
                        relationship_type: RelationshipType::Balancing,
                        time_lag_seconds: 300,
                        confidence: 0.5,
                        stability: 0.7,
                    });
                }
            }
        }
        
        Self { matrix }
    }
}

impl RelationshipMatrix {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get relationship strength between two subsystems
    pub fn get_strength(&self, influencer: Subsystem, influenced: Subsystem) -> f64 {
        self.matrix.get(&(influencer, influenced))
            .map(|r| r.strength * r.confidence)
            .unwrap_or(0.0)
    }
    
    /// Update relationship based on observed data
    pub fn update_relationship(&mut self, influencer: Subsystem, influenced: Subsystem, 
                              observed_impact: f64, time_lag: u64) {
        if let Some(node) = self.matrix.get_mut(&(influencer, influenced)) {
            // Adjust strength based on observed impact
            let delta = observed_impact * 0.1;
            node.strength = (node.strength + delta).max(0.0).min(1.0);
            
            // Update time lag
            node.time_lag_seconds = time_lag;
            
            // Increase confidence as we gather more data
            node.confidence = (node.confidence + 0.05).min(1.0);
            
            // Update stability based on consistency
            node.stability = if node.strength > 0.7 { 0.9 } else { node.stability };
            
            // Determine relationship type
            if node.strength > 0.6 && node.confidence > 0.6 {
                node.relationship_type = RelationshipType::Reinforcing;
            } else if node.strength < 0.3 {
                node.relationship_type = RelationshipType::Constraining;
            } else {
                node.relationship_type = RelationshipType::Balancing;
            }
        }
    }
    
    /// Calculate enterprise-wide impact of a change
    pub fn evaluate_impact(&self, changes: &[(Subsystem, f64)]) -> HashMap<Subsystem, f64> {
        let mut impacts = HashMap::new();
        
        // Initialize with direct changes
        for (subsystem, delta) in changes {
            impacts.insert(*subsystem, *delta);
        }
        
        // Propagate effects through relationship matrix
        for (influencer, influenced) in self.matrix.keys() {
            if let Some(&delta) = impacts.get(&influencer) {
                let strength = self.get_strength(*influencer, *influenced);
                let current = impacts.get(influenced).copied().unwrap_or(0.0);
                impacts.insert(*influenced, current + (delta * strength));
            }
        }
        
        impacts
    }
}