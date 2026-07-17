# Allbright Engine Mode: Tactical Briefing Summary

This document summarizes the briefing panels and confirmation protocols integrated into the Allbright C2 Cockpit for safe state transitions.

| Engine Mode | Briefing Title | Articulation (What) | Objective (Why) | Mechanism (How) | Confirmation Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **DEBUG** | DEBUG: SANDBOX MODE | Low-risk development environment utilizing mocked data and local loops. | Required for testing UI logic and gRPC structures without risking collateral. | Bypasses HW locks; uses `tauri-mock.ts` to simulate 850-node telemetry. | `ENABLE_DEBUG` |
| **PREFLIGHT**| PREFLIGHT: HARDWARE AUDIT | Diagnostic layer performing a silicon-level audit of the host machine. | Ensures the NR-solver has access to AVX-512/VNNI registers for 19.8µs execution. | Runs register integrity checks, measures P99 jitter, and verifies gRPC handshake. | `RUN_AUDIT` |
| **SIMULATION**| SIMULATION: M58 SHADOW REPLAY| High-fidelity shadow-fork test replaying real-time market data against local strategies. | Required to reconcile "Dark Alpha" yields against historical data before capital deployment. | Forks mainnet state (Anvil/Hardhat), executes DNA, and calculates Latency-Decay ROI. | `START_SIMULATION` |
| **PILOT** | PILOT: GATED PRODUCTION | Live market execution mode capped at 10% of total collateral. | Safely transitions the engine to live pools while monitoring Alpha-Copilot scores. | Bridges to OpenRouter AI, enforces 10-block MPC shard rotation, and monitors deflection. | `DEPLOY_PILOT` |
| **LIVE** | LIVE: SOVEREIGN APEX | Full-scale autonomous arbitrage with uncapped capital utilization. | Final production state for maximum profit extraction and global grid saturation. | Requires constant YubiKey heartbeat and full AVX-512 hardware-acceleration. | `AUTHORIZE_APEX` |

## Post-Confirmation Workflow
1. **HUD Engagement**: The system displays a progress bar indicating the specific `runningMode`.
2. **Real-time Metrics**: Visualizes "Register Handshake" and "Nanosecond Offsets" during execution.
3. **Auto-Archival**: On 100% completion, a "Tactical Execution Audit Log" is generated.
4. **Ledger Sync**: The report is automatically injected into the `reports` state and archived to the sidebar ledger with a unique Audit ID.

*Document Version: 60.0.0 (Tactical Alignment)*