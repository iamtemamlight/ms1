#!/bin/bash
# Allbright Apex Cockpit: Unified Build Pipeline
# Purpose: Compile the Sovereign Engine + Apex Dashboard into a signed desktop binary.

echo "🚀 INITIATING APEX COCKPIT BUILD..."

# 0. Orchestration Sync
echo "🔄 Synchronizing V60 Orchestrator for clean environment..."
bash scripts/v60_master_orchestrator.sh

echo "✅ ALLBRIGHT APEX COCKPIT: SYSTEM SYNCHRONIZED."
echo "===================================================="