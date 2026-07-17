#!/bin/bash
# verify_kpi_evidence.sh - Evidence gate for 72-KPI verification
# Part of Phase 3: Verification Pipeline
# Validates that KPI claims have corresponding evidence

set -euo pipefail

EVIDENCE_DIR="./evidence"
KPI_TABLE="./KPIS_VERIFICATION_TABLE.md"
MODULE_REGISTRY="./MODULE_REGISTRY.toml"
REPORT_DIR="./verification-reports"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Evidence types by verification tier
declare -A EVIDENCE_TYPES=(
  ["SIMULATION"]="shadow-replay|benchmark|simulation-log"
  ["PILOT"]="pilot-test|gated-deployment|limited-capital"
  ["LIVE"]="production-log|transaction-hash|onchain-proof"
)

echo "=========================================="
echo "72-KPI Evidence Verification Pipeline"
echo "Timestamp: $TIMESTAMP"
echo "=========================================="

# Ensure directories exist
mkdir -p "$EVIDENCE_DIR" "$REPORT_DIR"

# Counters
TOTAL_KPIS=0
PASSED=0
FAILED=0
WARNINGS=0

validate_evidence() {
  local kpi_id="$1"
  local status="$2"
  local evidence_type="$3"
  
  TOTAL_KPIS=$((TOTAL_KPIS + 1))
  
  if [ "$status" = "EXTERNAL" ]; then
    echo -e "${YELLOW}⚠️  SKIP${NC} $kpi_id - External dependency"
    WARNINGS=$((WARNINGS + 1))
    return 0
  fi
  
  if [ "$status" != "IMPLEMENTED" ]; then
    echo -e "${RED}❌ FAIL${NC} $kpi_id - Status: $status (requires evidence)"
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  # Check for evidence directory for this KPI
  local evidence_path="$EVIDENCE_DIR/$kpi_id"
  
  if [ ! -d "$evidence_path" ]; then
    echo -e "${RED}❌ FAIL${NC} $kpi_id - Missing evidence directory: $evidence_path"
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  # Check for at least one evidence file
  local evidence_found=0
  for pattern in ${EVIDENCE_TYPES[$evidence_type]}; do
    if ls "$evidence_path"/*"$pattern"* 1> /dev/null 2>&1; then
      evidence_found=1
      break
    fi
  done
  
  if [ "$evidence_found" -eq 0 ]; then
    echo -e "${RED}❌ FAIL${NC} $kpi_id - No valid evidence files found in $evidence_path"
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  # Verify evidence file integrity (non-empty)
  local file_count=$(find "$evidence_path" -type f | wc -l)
  if [ "$file_count" -eq 0 ]; then
    echo -e "${RED}❌ FAIL${NC} $kpi_id - Evidence directory empty"
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  echo -e "${GREEN}✅ PASS${NC} $kpi_id - Evidence verified ($file_count files)"
  PASSED=$((PASSED + 1))
  return 0
}

# Parse KPIS_VERIFICATION_TABLE.md and validate evidence
# This is a simplified parser; in production, use a proper markdown parser
while IFS='|' read -r _ kpi_num kpi_name _ module status _; do
  # Skip header/separator lines
  if [[ "$kpi_num" =~ ^[[:space:]]*# ]] || [[ "$kpi_num" =~ ^[[:space:]]*- ]] || [[ -z "$kpi_num" ]]; then
    continue
  fi
  
  # Trim whitespace
  kpi_num=$(echo "$kpi_num" | xargs)
  kpi_name=$(echo "$kpi_name" | xargs)
  module=$(echo "$module" | xargs)
  status=$(echo "$status" | xargs)
  
  # Map status to evidence type
  if [[ "$status" == *"SIMULATION"* ]]; then
    evidence_type="SIMULATION"
  elif [[ "$status" == *"PILOT"* ]]; then
    evidence_type="PILOT"
  elif [[ "$status" == *"LIVE"* ]]; then
    evidence_type="LIVE"
  else
    evidence_type="SIMULATION" # Default
  fi
  
  validate_evidence "$kpi_num" "$status" "$evidence_type"
done < "$KPI_TABLE"

echo ""
echo "=========================================="
echo "Evidence Verification Summary"
echo "=========================================="
echo "Total KPIs: $TOTAL_KPIS"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Warnings: $WARNINGS${NC}"

# Calculate pass rate
if [ $TOTAL_KPIS -gt 0 ]; then
  pass_rate=$(awk "BEGIN {print ($PASSED / $TOTAL_KPIS) * 100}")
  echo "Pass Rate: ${pass_rate}%"
else
  pass_rate=0
  echo "Pass Rate: N/A"
fi

# Generate evidence report
cat > "$REPORT_DIR/evidence-report-$TIMESTAMP.md" << EOF
# KPI Evidence Verification Report
**Timestamp:** $TIMESTAMP
**Total KPIs:** $TOTAL_KPIS
**Passed:** $PASSED
**Failed:** $FAILED
**Warnings:** $WARNINGS
**Pass Rate:** ${pass_rate}%

## Status
$(if [ $FAILED -eq 0 ]; then echo "✅ ALL EVIDENCE VERIFIED"; else echo "❌ EVIDENCE GAPS DETECTED"; fi)

## Failed KPIs
$(if [ $FAILED -gt 0 ]; then echo "Review evidence directories for:"; else echo "None"; fi)
EOF

# Exit with appropriate code
if [ $FAILED -eq 0 ]; then
  echo ""
  echo -e "${GREEN}✅ Evidence gate passed${NC}"
  exit 0
else
  echo ""
  echo -e "${RED}❌ Evidence gate failed - $FAILED KPIs missing evidence${NC}"
  exit 1
fi