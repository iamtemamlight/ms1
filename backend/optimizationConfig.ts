/**
 * @license
 * Sovereign Configuration Registry for Allbright DeFi
 * Module 55: Optimization Dimensions
 */

export interface DimensionToggle {
  id: number;
  name: string;
  label: string;
  enabled: boolean;
  current_value: number;
  min: number;
  automation_mode: 'MANUAL' | 'AUTO_TUNE' | 'OPTIMIZE';
  max: number;
  step: number;
  unit: string;
  kpi_override_active: boolean;
  kpi_override_value: number;
  category: "GAS" | "STRATEGY" | "PERFORMANCE" | "MASTER";
  description: string;
}

export const INITIAL_DIMENSIONS: DimensionToggle[] = [
  { 
    id: 1, 
    name: "corridor_width_bps", 
    label: "Corridor Width (bps)", 
    category: "GAS",
    enabled: true,
    current_value: 150,
    automation_mode: 'AUTO_TUNE',
    min: 10,
    max: 1000,
    step: 10,
    unit: "bps",
    kpi_override_active: true,
    kpi_override_value: 150,
    description: "Max variance buffer tolerated before automatic balancing triggers on-chain asset transfers."
  },
  { 
    id: 2, 
    name: "bribe_opt_multiplier", 
    label: "Bribe Multiplier", 
    category: "GAS",
    enabled: true,
    current_value: 1.15,
    automation_mode: 'OPTIMIZE',
    min: 1.0,
    max: 5.0,
    step: 0.05,
    unit: "x",
    kpi_override_active: false,
    kpi_override_value: 1.25,
    description: "Fee multiplier paid to block builders relative to base gas to guarantee block prioritization."
  },
  { 
    id: 6, 
    name: "market_regime", 
    label: "Market Regime", 
    category: "STRATEGY",
    enabled: true,
    current_value: 1.0,
    automation_mode: 'MANUAL',
    min: 1.0,
    max: 3.0,
    step: 1.0,
    unit: "mode", 
    kpi_override_active: true,
    kpi_override_value: 1.0,
    description: "Algorithmic profile tuning (1: Conservative-Stable, 2: Volatile-Expansion, 3: Flash-Crash Protection)."
  },
  { 
    id: 24, 
    name: "vacuum_profit_floor", 
    label: "Vacuum Profit Floor ($)", 
    category: "STRATEGY",
    enabled: true,
    current_value: 0.50,
    automation_mode: 'OPTIMIZE',
    min: 0.01,
    max: 10.0,
    step: 0.05,
    unit: "$",
    kpi_override_active: false,
    kpi_override_value: 0.50,
    description: "Dynamic threshold for micro-arbitrage 'Vacuuming'. Active only when simulation confidence is near-perfect (Dark Alpha strategy)."
  },
  { 
    id: 21, 
    name: "kpi_driven_auto_tune", 
    label: "KPI Auto-Tune (Dim 23)", 
    category: "MASTER",
    enabled: true,
    current_value: 1,
    automation_mode: 'AUTO_TUNE',
    min: 0,
    max: 1,
    step: 1,
    unit: "bool",
    kpi_override_active: false,
    kpi_override_value: 1,
    description: "Enable automated dynamic scaling based on real-time P99 latency & rebalancing queue status."
  },
  // Additional Dimensions omitted for brevity but preserved in the registry
];