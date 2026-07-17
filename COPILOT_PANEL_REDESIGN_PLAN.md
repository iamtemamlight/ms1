# Copilot Panel & Engine Control Redesign Plan

## ExecutivePanel Changes (Copilot Panel)

### Current Issues:
- Header has too many icons (Sparkles, multiple icons in status bar)
- Model selection is at top in Models tab (confusing)
- Pre-flight button exists but user wants it removed
- Connection status icons are scattered

### Proposed Changes:

| Area | Current | Proposed |
|------|---------|----------|
| Header | Sparkles icon + title | Clean title only |
| Status Bar | Multiple icons | Simplified status text |
| Control Bar | Resume, Kill Switch, Pre-Flight | Resume, Kill Switch only (remove Pre-Flight) |
| Model Selection | In Models tab | Moved to bottom bar when in Chat mode |

### New Bottom Bar (Chat mode):
```
[ Model: GPT-4 ] [ Auto Mode: ON ] [ Send ]
```

---

## Engine Control Changes

### .env File Import Integration:
1. When user imports .env file:
   - Auto-trigger security validation (all 10 layers)
   - Show tooltip: "Security validation initiated for imported configuration"
   - Validate: Stealth Network, HSM, Vault, Memory, Installer, Windows Policies, etc.

2. Security Auto-Trigger Flow:
   ```
   .env imported → validate_all() called → show layer status → user clicks "CONNECT SYSTEMS"
   ```

3. Mode Analytics Reports:
   - Each mode gets detailed completion report
   - Report includes: KPIs, performance metrics, recommendations, next actions

---

## Implementation Steps

### Step 1: ExecutivePanel.tsx
- Remove Sparkles icon from header
- Remove Pre-Flight button
- Add model selector to bottom bar
- Add auto-mode toggle to bottom bar
- Clean up status bar icons

### Step 2: EngineControl.tsx
- Add auto-security trigger on .env import
- Add security validation status display
- Remove PREFLIGHT from ENGINE_MODES
- Add detailed tooltips for each mode

### Step 3: Connect .env Import to Security
- Import triggers SecurityGate::validate_all()
- Show real-time layer status in UI

**Requires your approval before implementation.**