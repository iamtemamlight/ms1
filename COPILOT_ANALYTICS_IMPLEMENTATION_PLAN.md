# Copilot Analytics Report - Implementation Plan

## Overview
Add context-aware "Copilot Analytics Report" button to each dashboard page with time selection dropdown and mode selection on report page.

## Button Placement (Per Page)

| Page | Button Text | Tooltip |
|------|-------------|---------|
| ProfitMetrics | "Profit Analytics Report" | Generate copilot report analyzing profit metrics for selected time period |
| AutoOptimizationPage | "Optimization Analytics Report" | Generate copilot report analyzing optimization parameters for selected time period |
| SecurityMetricsSidebar | "Security Analytics Report" | Generate copilot report analyzing security layer metrics for selected time period |
| WalletSystem | "Wallet Analytics Report" | Generate copilot report analyzing wallet performance for selected time period |
| EngineControl | "Execution Analytics Report" | Generate copilot report analyzing execution modes for selected time period |
| ReportsCompliance | "Compliance Analytics Report" | Generate copilot report analyzing compliance metrics for selected time period |
| ModelManager | "Model Analytics Report" | Generate copilot report analyzing AI models for selected time period |

## Time Selection Dropdown Options

```
[ Last Hour ] [ Last 6 Hours ] [ Last 24 Hours ] [ Last 7 Days ] [ Last 30 Days ] [ Last 90 Days ] [ Last Year ]
```

## Report Page Additions

- Time selection dropdown (same options)
- Mode selection dropdown:
  ```
  [ AUTO ] [ MANUAL ] [ INACTIVE ] [ ALL MODES ]
  ```

## Implementation Steps

### Step 1: Create Reusable CopilotAnalyticsButton Component
```tsx
// apps/dashboard/src/components/CopilotAnalyticsButton.tsx
interface CopilotAnalyticsButtonProps {
  pageContext: string;  // 'profit', 'optimization', 'security', etc.
  selectedTime: string;
  onTimeChange: (time: string) => void;
  onGenerateReport: () => void;
}
```

### Step 2: Add API Endpoint for Copilot Reports
```rust
// backend/main.rs
// /api/copilot/report
async fn get_copilot_analytics_report(
    Path(context): Path<String>,
    Query(time_range): Query<TimeRange>
) -> Result<Json<serde_json::Value>, AppError>
```

### Step 3: Add Context-Aware Report Generation to ai_agents
```rust
// backend/ai_agents.rs
// Add generate_analytics_report(context, time_range) function
```

### Step 4: Update Each Page Component
- Add button to header area (right side)
- Add time selection dropdown
- Connect to analytics API

### Step 5: Update Report Page
- Add mode selection dropdown
- Add time selection dropdown  
- Make report generation context-aware

## File Structure

```
apps/dashboard/src/components/
├── CopilotAnalyticsButton.tsx    (NEW - reusable component)
├── ProfitMetrics.tsx             (UPDATE - add button)
├── AutoOptimizationPage.tsx      (UPDATE - add button)  
├── SecurityMetricsSidebar.tsx    (UPDATE - add button)
├── WalletSystem.tsx              (UPDATE - add button)
├── EngineControl.tsx             (UPDATE - add button)
├── ReportsCompliance.tsx         (UPDATE - add button)
└── ModelManager.tsx              (UPDATE - add button)

backend/
├── ai_agents.rs                  (UPDATE - add analytics report)
└── main.rs                       (UPDATE - add /api/copilot/report)
```

## Mock API Response Format

```json
{
  "page": "profit",
  "timeRange": "24h",
  "analytics": {
    "summary": "Profit analytics for last 24 hours",
    "keyInsights": ["insight 1", "insight 2"],
    "trend": "positive",
    "recommendations": ["rec 1", "rec 2"]
  }
}
```

---

**Requires your approval before implementation.**