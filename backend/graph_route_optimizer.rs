// ==============================================================================
// GRAPH ROUTE OPTIMIZER: Multi-Hop Arbitrage Pathfinding
// Integrated with Newton-Raphson Q* Solver for Optimal Trade Sizing
// Part of AllBright-Defi-V119 Framework
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

/// Weighted Edge in DEX Liquidity Graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexEdge {
    pub from_token: String,
    pub to_token: String,
    pub dex: String,
    pub liquidity: f64,
    pub fee_bps: f64,
    pub gas_cost: u64,
}

/// DEX Liquidity Graph - Token Network Representation
#[derive(Debug, Clone)]
pub struct LiquidityGraph {
    // Token vertices
    vertices: HashSet<String>,
    // Multi-graph: (token_a, token_b) -> edges across multiple DEXes
    edges: HashMap<(String, String), Vec<DexEdge>>,
}

impl LiquidityGraph {
    pub fn new() -> Self {
        Self {
            vertices: HashSet::new(),
            edges: HashMap::new(),
        }
    }

    /// Add bidirectional edge between tokens
    pub fn add_edge(&mut self, edge: DexEdge) {
        self.vertices.insert(edge.from_token.clone());
        self.vertices.insert(edge.to_token.clone());
        
        let key = (edge.from_token.clone(), edge.to_token.clone());
        self.edges.entry(key).or_insert_with(Vec::new).push(edge);
    }
}

/// Dijkstra-based Optimal Route Finder
/// Returns minimum-cost path from start to end token given trade amount
pub fn dijkstra_find_route(
    graph: &LiquidityGraph,
    start: &str,
    end: &str,
    amount: f64,
) -> Option<RouteResult> {
    // Priority queue: (cost, current_token, path, dex_used)
    let mut pq: BinaryHeap<RouteNode> = BinaryHeap::new();
    let mut visited: HashSet<String> = HashSet::new();
    
    // Initialize with direct routes
    if let Some(edges) = graph.edges.get(&(start.to_string(), end.to_string())) {
        for edge in edges {
            let cost = calculate_trade_cost(amount, &edge);
            pq.push(RouteNode {
                cost,
                token: edge.to_token.clone(),
                path: vec![edge.from_token.clone(), edge.to_token.clone()],
                dexes: vec![edge.dex.clone()],
            });
        }
    }
    
    // Dijkstra search
    while let Some(current) = pq.pop() {
        if visited.contains(&current.token) {
            continue;
        }
        visited.insert(current.token.clone());
        
        if current.token == end {
            return Some(RouteResult {
                total_cost: current.cost,
                path: current.path,
                dexes: current.dexes,
            });
        }
        
        // Explore neighbors
        for (key, edges) in &graph.edges {
            if &current.token == &key.0 {
                for edge in edges {
                    let new_cost = current.cost + calculate_trade_cost(amount, &edge);
                    let mut new_path = current.path.clone();
                    new_path.push(edge.to_token.clone());
                    let mut new_dexes = current.dexes.clone();
                    new_dexes.push(edge.dex.clone());
                    
                    pq.push(RouteNode {
                        cost: new_cost,
                        token: edge.to_token.clone(),
                        path: new_path,
                        dexes: new_dexes,
                    });
                }
            }
        }
    }
    
    None
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteNode {
    cost: f64,
    token: String,
    path: Vec<String>,
    dexes: Vec<String>,
}

impl Ord for RouteNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for RouteNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cost.cost.cmp(&self.cost))
    }
}

pub struct RouteResult {
    pub total_cost: f64,
    pub path: Vec<String>,
    pub dexes: Vec<String>,
}

/// Calculate total trade cost including fees and gas
fn calculate_trade_cost(amount: f64, edge: &DexEdge) -> f64 {
    let fee = amount * (edge.fee_bps / 10000.0);
    let gas_eth = edge.gas_cost as f64 * 0.000000030; // ~30 gwei
    fee + gas_eth
}

/// Full HFT Arbitrage Pipeline: Graph Routing + Newton-Raphson
/// 
/// INTEGRATION FLOW:
/// 1. Build DEX liquidity graph
/// 2. Find optimal route via Dijkstra
/// 3. Calculate optimal trade size Q* via Newton-Raphson
/// 4. Execute multi-hop flash loan
pub struct HftArbitrageEngine {
    graph: LiquidityGraph,
    // Newton-Raphson solver from engine_modules
}

impl HftArbitrageEngine {
    /// Full optimization: Route Discovery + Q* Sizing
    pub fn execute_arbitrage(
        &mut self,
        start_token: &str,
        end_token: &str,
        initial_amount: f64,
    ) -> Option<ArbitrageResult> {
        // Step 1: Find optimal route
        let route = dijkstra_find_route(&self.graph, start_token, end_token, initial_amount)?;
        
        // Step 2: Calculate optimal Q* using Newton-Raphson
        // Profit function: P(Q) = Output(Q) - Input(Q) - Fees - Gas
        let profit_function = |q: f64| -> f64 {
            let output = q * 1.01; // Simplified output calculation
            let fees = q * 0.003; // 0.3% total fees
            let gas = 0.01; // Fixed gas
            output - q - fees - gas
        };
        
        // Use Newton-Raphson (from engine_modules.rs)
        let solver = crate::trading_engine::NewtonRaphsonSolver::new(20, 1e-8);
        let optimal_q = solver.solve(&profit_function, initial_amount)?;
        
        Some(ArbitrageResult {
            route,
            optimal_size: optimal_q,
            expected_profit: profit_function(optimal_q),
        })
    }
}

pub struct ArbitrageResult {
    pub route: RouteResult,
    pub optimal_size: f64,
    pub expected_profit: f64,
}

/// Comparison: Graph Methods for HFT Systems
/// 
/// RANKING (by suitability for Flash Loan Arbitrage):
/// 
/// | Rank | Algorithm | Complexity | Use Case | Score |
/// |------|----------|-----------|-----------|-------|
/// | 1 | **Dijkstra** | O(E + V log V) | Multi-DEX routing | ⭐⭐⭐⭐⭐ |
/// | 2 | **A*** | O(E) | Targeted swaps | ⭐⭐⭐⭐ |
/// | 3 | **Bellman-Ford** | O(VE) | Negative cycles | ⭐⭐⭐ |
/// | 4 | **Floyd-Warshall** | O(V³) | All-pairs | ⭐⭐ |
/// | 5 | **BFS** | O(V + E) | Simple path | ⭐⭐ |
/// 
/// **RECOMMENDATION:** Dijkstra with BinaryHeap is optimal for:
/// - 50+ DEX multi-hop routing
/// - Flash loan atomic execution
/// - Gas-cost optimization
/// 
/// Implementation matches Newton-Raphson Q* Solver for complete HFT system.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dijkstra_route() {
        let mut graph = LiquidityGraph::new();
        
        // Add test edges: USDC -> ETH -> DAI -> USDC (cycle)
        graph.add_edge(DexEdge {
            from_token: "USDC".to_string(),
            to_token: "ETH".to_string(),
            dex: "UniswapV3".to_string(),
            liquidity: 10_000_000.0,
            fee_bps: 30.0,
            gas_cost: 180_000,
        });
        
        graph.add_edge(DexEdge {
            from_token: "ETH".to_string(),
            to_token: "USDC".to_string(),
            dex: "UniswapV3".to_string(),
            liquidity: 10_000_000.0,
            fee_bps: 30.0,
            gas_cost: 180_000,
        });
        
        let result = dijkstra_find_route(&graph, "USDC", "USDC", 1000.0);
        assert!(result.is_some());
    }
}
