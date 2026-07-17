# Profit Metrics Table Enhancement - Implementation Plan

## 📊 **CURRENT STATE ANALYSIS**

### **Existing Columns in ProfitMetrics.tsx:**
```tsx
const headers = [
  { key: 'segment', label: 'Segment Partition' },
  { key: 'nodes', label: 'Nodes' },
  { key: 'profitPerHour', label: 'ETH/Hour' },
  { key: 'tradesPerHour', label: 'Trades/Node' },  // ❌ WRONG: Should be "Trades/Node/Hr"
  { key: 'winRate', label: 'Win EMA' },
  { key: 'latency', label: 'Latency' },
  { key: 'marketWatchShare', label: 'Mkt Share' }
];
```

### **Issues Identified:**
1. ❌ "Trades/Node" label is incorrect - should be "Trades/Node/Hr"
2. ❌ Missing critical columns: Regions, Chains, Pairs, DEXs, Gas Costs, Total Trades, Profit/Trade, NPM
3. ❌ No horizontal scrollbar implementation (table will be too wide)
4. ❌ Profit per node per hour not explicitly shown
5. ❌ No profit multiplier (NPM) column

---

## 🎯 **PROPOSED ENHANCEMENTS**

### **New Column Structure (14 columns total) - LOGICAL ORDER:**

#### **IDENTITY & SCALE (Columns 1-6):**

**Column 1:** `segment` - "Segment" - Diamond/Gold/Bronze partition  
**Column 2:** `nodes` - "Nodes" - Active runners in segment  
**Column 3:** `regions` - "Regions" - Geographic regions active  

| # | Column Key | Label | Unit | Description |
|---|-----------|-------|------|-------------|
| 1 | segment | Segment | - | Diamond/Gold/Bronze partition |
| 2 | nodes | Nodes | count | Active runners in segment |
| 3 | regions | Regions | count | Geographic regions active |

#### **PROFIT METRICS (Columns 7-10):**

**Column 7:** `profitPerHour` - **Header: "Profit/Hr (ETH)"** - Total segment profit per hour  
**Column 8:** `profitPerNode` - **Header: "Profit/Node/Hr (ETH)"** - Profit per runner per hour  

**⭐ Column 9:** `profitShare` - **Header: "Profit Share (%)"** - Segment % of total fleet profit  
- **Purpose:** Shows each segment's contribution to total fleet profit
- **Calculation:** `(segment.profitPerHour / totalFleetProfit) * 100`
- **Example:** Diamond 52.6, Gold 33.1, Bronze 14.3

**Column 10:** `profitPerTrade` - **Header: "Profit/Trade (ETH)"** - Average profit per trade  

**UNIT PLACEMENT RULE:** Units go in COLUMN HEADERS only, not in data cells

| # | Column Key | Header Label | Data Format | Example Value |
|---|-----------|-------------|-------------|---------------|
| 7 | profitPerHour | **Profit/Hr (ETH)** | `45.23` | `45.23` |
| 8 | profitPerNode | **Profit/Node/Hr (ETH)** | `0.129` | `0.129` |
| 9 | profitShare | **Profit Share (%)** | `52.6` | `52.6` |
| 10 | profitPerTrade | **Profit/Trade (ETH)** | `0.00015` | `0.00015` |

**BEFORE (Wrong):**
```tsx
// ❌ Units in data cells
<td className="p-3">45.23 ETH</td>
<td className="p-3">52.6%</td>
```

**AFTER (Correct):**
```tsx
// ✅ Units in headers only, clean data cells
<th>Profit/Hr (ETH)</th>
<td className="p-3">45.23</td>

<th>Profit Share (%)</th>
<td className="p-3">52.6</td>
```

#### **PROFIT EFFICIENCY (Columns 11-12):**
| # | Column Key | Label | Unit | Description |
|---|-----------|-------|------|-------------|
| 11 | npm | NPM | x | Net Profit Margin multiplier |
| 12 | totalTrades | Total Trades | count | Total trades per hour |

#### **PROFIT EFFICIENCY (Columns 11-12):**
| # | Column Key | Label | Unit | Description |
|---|-----------|-------|------|-------------|
| 11 | npm | NPM | x | Net Profit Margin multiplier |
| 12 | totalTrades | Total Trades | count | Total trades per hour |

#### **VOLUME & PERFORMANCE (Columns 13-14):**
| # | Column Key | Label | Unit | Description |
|---|-----------|-------|------|-------------|
| 13 | tradesPerNode | Trades/Node/Hr | count/hr | Trade frequency per node |
| 14 | winRate | Win Rate | % | EMA success rate |

---

## 📐 **IMPLEMENTATION PLAN**

### **Phase 1: Update Data Interface**
**File:** `apps/dashboard/src/lib/api.ts`

```typescript
export interface SegmentProfitData {
  mode: string;
  label: string;
  nodes: number;
  regions: number;           // NEW
  chains: number;            // NEW
  pairs: number;             // NEW
  dexs: number;              // NEW
  profitPerHour: number;     // RENAME from profitPerHour
  profitPerNode: number;     // NEW: profitPerHour / nodes
  profitPerTrade: number;    // NEW: average profit per trade
  npm: number;               // NEW: net profit margin (1.5x - 3.0x)
  totalTrades: number;       // NEW: total trades per hour
  tradesPerNode: number;     // RENAME from tradesPerHour
  winRate: number;           // EXISTS
  latency: number;           // EXISTS (keep for reference)
  gasCosts: number;          // NEW: hourly gas costs
  marketWatchShare: number;  // EXISTS
}
```

### **Phase 2: Update Headers Array (ULTRA-SHORT)**
**File:** `apps/dashboard/src/components/ProfitMetrics.tsx`

```tsx
const headers = [
  { key: 'segment', label: 'Segment' },
  { key: 'nodes', label: 'Nodes' },
  { key: 'regions', label: 'Regions' },
  { key: 'chains', label: 'Chains' },
  { key: 'pairs', label: 'Pairs' },
  { key: 'dexs', label: 'DEXs' },
  { key: 'profitPerHour', label: 'P/Hr' },           // Ultra-short
  { key: 'profitPerNode', label: 'P/Node' },         // Ultra-short
  { key: 'profitShare', label: 'Share' },            // ⭐ Ultra-short
  { key: 'profitPerTrade', label: 'P/Trade' },       // Ultra-short
  { key: 'npm', label: 'NPM' },
  { key: 'totalTrades', label: 'Trades' },
  { key: 'tradesPerNode', label: 'T/Node' },         // Ultra-short
  { key: 'winRate', label: 'Win' },                  // Ultra-short
  { key: 'gasCosts', label: 'Gas' }                  // Ultra-short
];
```
**CRITICAL RULES:**
1. ✅ Max 8 characters per header
2. ✅ Use abbreviations: "P/Hr", "P/Node", "T/Node", "Win", "Gas"
3. ✅ No long words like "Profit" or "Trades" - use "P" and "T"
4. ✅ Horizontal scroll enabled for 15 narrow columns

**Header Lengths:**
- Segment (7) ✓
- Nodes (5) ✓
- Regions (7) ✓
- Chains (6) ✓
- Pairs (5) ✓
- DEXs (4) ✓
- P/Hr (4) ✓
- P/Node (6) ✓
- Share (5) ✓ ⭐
- P/Trade (7) ✓
- NPM (3) ✓
- Trades (6) ✓
- T/Node (6) ✓
- Win (3) ✓
- Gas (3) ✓
```

### **Phase 3: Add Horizontal Scroll**
**File:** `apps/dashboard/src/components/ProfitMetrics.tsx`

```tsx
<div className="overflow-x-auto border border-[#374762]/40 rounded-xl scrollbar-none bg-[#1e2a42]/30" 
     style={{ maxWidth: '100%', overflowY: 'auto' }}>
  <table className="w-full text-left border-collapse min-w-[1200px]">
    {/* Table content */}
  </table>
</div>
```

**Add scrollbar styling:**
```css
/* In index.css or component styles */
.scrollbar-none::-webkit-scrollbar {
  height: 8px;
}

.scrollbar-none::-webkit-scrollbar-track {
  background: #1e2a42;
  border-radius: 4px;
}

.scrollbar-none::-webkit-scrollbar-thumb {
  background: #374762;
  border-radius: 4px;
}

.scrollbar-none::-webkit-scrollbar-thumb:hover {
  background: #38bdf8;
}
```

### **Phase 4: Update Table Body**
**File:** `apps/dashboard/src/components/ProfitMetrics.tsx`

```tsx
<tbody>
  {isSegmentTableExpanded ? (
    sortedProfitSegments.map((row) => {
      const isSelected = viewMode === row.mode;
      return (
        <tr key={row.mode} className={...}>
          <td className="p-3 font-bold text-[#7dd3fc]">{row.label}</td>
          <td className="p-3">{row.nodes}</td>
          <td className="p-3">{row.regions}</td>
          <td className="p-3">{row.chains}</td>
          <td className="p-3">{row.pairs}</td>
          <td className="p-3">{row.dexs}</td>
          <td className="p-3 font-bold text-emerald-400">+{row.profitPerHour.toFixed(4)} ETH</td>
          <td className="p-3 text-emerald-400">+{row.profitPerNode.toFixed(4)} ETH</td>
          <td className="p-3 text-emerald-400">+{row.profitPerTrade.toFixed(6)} ETH</td>
          <td className="p-3 text-amber-400 font-bold">{row.npm.toFixed(2)}x</td>
          <td className="p-3">{row.totalTrades.toLocaleString()}</td>
          <td className="p-3">{(row.tradesPerNode || 0).toFixed(1)}</td>
          <td className="p-3 text-emerald-400">{row.winRate.toFixed(2)}%</td>
          <td className="p-3 text-rose-400">-{row.gasCosts.toFixed(2)} ETH</td>
        </tr>
      );
    })
  ) : (
    <tr className="...">
      <td colSpan={14} className="p-3 text-center">
        {sortedProfitSegments.length} SEGMENT PARTITIONS COLLAPSED
      </td>
    </tr>
  )}
</tbody>
```

### **Phase 5: Update Footer**
```tsx
<tfoot>
  <tr className="border-t-2 border-[#374762] bg-[#1e2a42]/60">
    <td className="p-3 font-black text-[#38bdf8] uppercase">TOTAL</td>
    <td className="p-3 text-white font-bold">{sortedProfitSegments.reduce((sum, r) => sum + (r.nodes || 0), 0)}</td>
    <td className="p-3 text-zinc-300">{sortedProfitSegments.reduce((sum, r) => sum + (r.regions || 0), 0)}</td>
    <td className="p-3 text-zinc-300">{sortedProfitSegments.reduce((sum, r) => sum + (r.chains || 0), 0)}</td>
    <td className="p-3 text-zinc-300">{sortedProfitSegments.reduce((sum, r) => sum + (r.pairs || 0), 0)}</td>
    <td className="p-3 text-zinc-300">{sortedProfitSegments.reduce((sum, r) => sum + (r.dexs || 0), 0)}</td>
    <td className="p-3 text-emerald-400 font-extrabold">+{sortedProfitSegments.reduce((sum, r) => sum + (r.profitPerHour || 0), 0).toFixed(4)} ETH</td>
    <td className="p-3 text-emerald-400">+{(sortedProfitSegments.reduce((sum, r) => sum + (r.profitPerNode || 0), 0) / (sortedProfitSegments.length || 1)).toFixed(4)} ETH</td>
    <td className="p-3 text-emerald-400">+{(sortedProfitSegments.reduce((sum, r) => sum + (r.profitPerTrade || 0), 0) / (sortedProfitSegments.length || 1)).toFixed(6)} ETH</td>
    <td className="p-3 text-amber-400 font-bold">{((sortedProfitSegments.reduce((sum, r) => sum + (r.npm || 0), 0) / (sortedProfitSegments.length || 1))).toFixed(2)}x</td>
    <td className="p-3 text-white font-bold">{sortedProfitSegments.reduce((sum, r) => sum + (r.totalTrades || 0), 0).toLocaleString()}</td>
    <td className="p-3 text-zinc-300">{(sortedProfitSegments.reduce((sum, r) => sum + (r.tradesPerNode || 0), 0) / (sortedProfitSegments.length || 1)).toFixed(1)}</td>
    <td className="p-3 text-emerald-400">{(sortedProfitSegments.reduce((sum, r) => sum + (r.winRate || 0), 0) / (sortedProfitSegments.length || 1)).toFixed(2)}%</td>
    <td className="p-3 text-rose-400">-{sortedProfitSegments.reduce((sum, r) => sum + (r.gasCosts || 0), 0).toFixed(2)} ETH</td>
  </tr>
</tfoot>
```

### **Phase 6: Update Mock Data**
**File:** `apps/dashboard/src/lib/tauri-mock.ts`

```typescript
case 'stream_kpis':
  return {
    segments: [
      {
        mode: 'DIAMOND',
        label: 'Diamond',
        nodes: 350,
        regions: 8,
        chains: 5,
        pairs: 450,
        dexs: 25,
        profitPerHour: 45.23,
        profitPerNode: 0.129,      // 45.23 / 350
        profitPerTrade: 0.00015,   // 0.15 ETH * 0.001
        npm: 2.8,                  // Net Profit Margin
        totalTrades: 2847,
        tradesPerNode: 8.1,        // 2847 / 350
        winRate: 99.4,
        latency: 62000,
        gasCosts: 2.34
      },
      {
        mode: 'GOLD',
        label: 'Gold',
        nodes: 420,
        regions: 6,
        chains: 4,
        pairs: 320,
        dexs: 18,
        profitPerHour: 28.45,
        profitPerNode: 0.068,
        profitPerTrade: 0.00008,
        npm: 2.1,
        totalTrades: 1923,
        tradesPerNode: 4.6,
        winRate: 97.8,
        latency: 68000,
        gasCosts: 1.89
      },
      {
        mode: 'BRONZE',
        label: 'Bronze',
        nodes: 180,
        regions: 4,
        chains: 3,
        pairs: 150,
        dexs: 12,
        profitPerHour: 12.34,
        profitPerNode: 0.069,
        profitPerTrade: 0.00005,
        npm: 1.6,
        totalTrades: 1234,
        tradesPerNode: 6.9,
        winRate: 95.2,
        latency: 75000,
        gasCosts: 0.95
      }
    ]
  };
```

---

## 🎨 **VISUAL LAYOUT**

### **Before (7 columns):**
```
┌──────────────────────────────────────────────────────┐
│ Segment │ Nodes │ ETH/Hr │ Trades/Node │ Win │ Lat │ Mkt │
│ Diamond │   350 │ 45.23  │    8.1      │99.4 │ 62µs │ 88%│
│ Gold    │   420 │ 28.45  │    4.6      │97.8 │ 68µs │ 72%│
└──────────────────────────────────────────────────────┘
```

### **After (14 columns with horizontal scroll):**
```
┌──────────────────────────────────────────────────────────────────────────────────┐
│ Seg │ Nodes │ Regions │ Chains │ Pairs │ DEXs │ Profit/Hr │ Profit/Node │ Profit/  │
│     │       │         │        │       │      │           │             │ Trade    │
├──────────────────────────────────────────────────────────────────────────────────┤
│ Dmnd│  350  │    8    │   5    │  450  │  25  │  45.23 ETH│  0.129 ETH  │ 0.00015  │
│     │       │         │        │       │      │           │             │          │
│ Gold│  420  │    6    │   4    │  320  │  18  │  28.45 ETH│  0.068 ETH  │ 0.00008  │
├──────────────────────────────────────────────────────────────────────────────────┤
│     │       │         │        │       │      │           │             │   NPM    │
│ Dmnd│  350  │    8    │   5    │  450  │  25  │  45.23 ETH│  0.129 ETH  │  2.8x ⚡  │
├──────────────────────────────────────────────────────────────────────────────────┤
│     │       │         │        │       │      │           │             │ Tot Trades│
│ Dmnd│  350  │    8    │   5    │  450  │  25  │  45.23 ETH│  0.129 ETH  │  2,847    │
└──────────────────────────────────────────────────────────────────────────────────┘

[Horizontal Scrollbar →]
```

---

## ✅ **CRITICAL CORRECTIONS**

### **1. Column Metrics Validation:**

| Metric | Calculation | Source | Correct? |
|--------|-------------|--------|----------|
| **Profit/Hr** | Sum of all trades in segment per hour | Backend aggregation | ✅ EXISTS (rename from ETH/Hour) |
| **Profit/Node/Hr** | profitPerHour / nodes | Frontend calculation | ❌ MISSING - ADD |
| **Profit/Trade** | grossProfit - gasCosts - fees | Backend per-trade calc | ❌ MISSING - ADD |
| **Win Rate** | successfulTrades / totalTrades * 100 | Backend EMA | ✅ EXISTS (rename from Win EMA) |
| **Trades/Node/Hr** | totalTrades / nodes | Frontend calculation | ❌ EXISTS but label wrong (rename from Trades/Node) |
| **NPM** | (profit - gasCosts) / gasCosts | Backend calculation | ❌ MISSING - ADD (CRITICAL) |
| **Gas Costs** | Sum of gas spent per hour | Backend telemetry | ❌ MISSING - ADD |
| **Total Trades** | Total trades per hour | Backend counter | ❌ MISSING - ADD |
| **Regions** | Count of active regions | Backend config | ❌ MISSING - ADD |
| **Chains** | Count of active chains | Backend config | ❌ MISSING - ADD |
| **Pairs** | Count of trading pairs | Backend config | ❌ MISSING - ADD |
| **DEXs** | Count of DEXes | Backend config | ❌ MISSING - ADD |

### **2. NPM (Net Profit Margin) Formula:**
```rust
npm = (total_profit - total_gas_costs) / total_gas_costs

// Example:
// Profit: 45.23 ETH
// Gas: 2.34 ETH
// NPM: (45.23 - 2.34) / 2.34 = 18.3x

// Target ranges:
// Diamond: 2.5x - 3.5x (high margin, low risk)
// Gold: 1.5x - 2.5x (balanced)
// Bronze: 1.2x - 1.5x (volume-based)
```

### **3. Horizontal Scroll Implementation:**
```tsx
{/* Wrapper with horizontal scroll */}
<div className="overflow-x-auto border border-[#374762]/40 rounded-xl 
                scrollbar-none bg-[#1e2a42]/30"
     style={{ 
       maxWidth: '100%',
       overflowY: 'auto',
       overflowX: 'auto'
     }}>
  
  {/* Table with minimum width to force scroll */}
  <table className="w-full text-left border-collapse min-w-[1200px]">
    {/* ... content ... */}
  </table>
  
</div>
```

---

## 🚀 **IMPLEMENTATION STEPS**

### **Step 1:** Update ProfitMetrics.tsx headers (10 min)
### **Step 2:** Add horizontal scroll CSS (5 min)
### **Step 3:** Update table body with new columns (20 min)
### **Step 4:** Update footer calculations (15 min)
### **Step 5:** Update mock data (10 min)
### **Step 6:** Update backend data structures (20 min)
### **Step 7:** Test with real data (15 min)

**Total Estimated Time:** 1.5 hours

---

## 📋 **READY FOR APPROVAL**

This plan:
1. ✅ Fixes incorrect "Trades/Node" label → "Trades/Node/Hr"
2. ✅ Adds 7 new critical columns (Regions, Chains, Pairs, DEXs, Gas Costs, Total Trades, NPM)
3. ✅ Adds Profit/Node/Hr and Profit/Trade columns
4. ✅ Implements horizontal scrollbar for wide table
5. ✅ Maintains professional formatting with color coding
6. ✅ Updates footer with correct aggregations
7. ✅ Updates mock data for testing

**Toggle to Act mode to implement this enhancement plan.**