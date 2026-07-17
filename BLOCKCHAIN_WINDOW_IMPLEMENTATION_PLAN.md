# Blockchain Window + AISE System - Implementation Plan

## Overview
Create a "Blockchain Window" that shows blockchain events filtered by mode, and integrate real-time AISE agent activity display.

## Components to Create

### 1. BlockchainWindow.tsx (NEW)
- Mode selection dropdown above the window
- Real-time blockchain event feed
- Events grouped by six pillars (modes):
  - ALPHA (KPIs 1-12): Corridor, Bribe, Bundle, FlashLoan
  - VELOCITY (KPIs 13-24): Block Phase, Solver, JIT
  - SHIELD (KPIs 25-36): Shield Routing, Pool Tier, Regional
  - EFFICIENCY (KPIs 37-48): Capital, Multi-Hop, Gas
  - CONTINUITY (KPIs 49-60): Runner Capacity, Chain Selection
  - MARKET (KPIs 61-72): Pair, Region, Mode

### 2. CopilotActivityWindow.tsx (NEW)
- Real-time copilot decision stream
- Shows: AI suggestions, overrides, optimizations triggered
- Timestamped activity log

### 3. AISEActivityWindow.tsx (NEW)
- Real-time AISE agent activity across all 91 agents
- Shows: Agent ID, action taken, timestamp, result status
- Filter by agent category

## Features

### Blockchain Window Header
```
[ Select Mode ▼ ] [ Time Filter: 24h ▼ ] [ Refresh ]
```

### Event Display Format
| Time | Event Type | Mode | Description |
|------|------------|------|-------------|
| 14:23 | KPI Trigger | ALPHA | Corridor Width adjusted |
| 14:22 | Transaction | VELOCITY | Block 19,845,234 mined |
| 14:21 | Security | SHIELD | Shield layer activated |

### Copilot Window Format
| Time | Action | Severity | Message |
|------|--------|----------|---------|
| 14:23 | OPTIMIZE | INFO | Adjusting parameter X |
| 14:22 | ALERT | WARN | Profit deficit detected |

### AISE Window Format
| Agent | Last Action | Status | Result |
|-------|-------------|--------|--------|
| AI001 | analyze_market | ACTIVE | 12 signals processed |
| AI054 | optimize_params | DONE | 5 parameters tuned |

## Implementation Steps

1. Create `BlockchainWindow.tsx` component
2. Create `CopilotActivityWindow.tsx` component
3. Create `AISEActivityWindow.tsx` component
4. Add API endpoints:
   - `/api/blockchain/events?mode={mode}&time={time}`
   - `/api/copilot/activity?time={time}`
   - `/api/aise/activity?time={time}`
5. Integrate into App.tsx sidebar

**Requires your approval before implementation.**