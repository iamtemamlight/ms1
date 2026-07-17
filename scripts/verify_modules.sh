#!/bin/bash
# verify_modules.sh - Module registry enforcement
# Part of Phase 3: Verification Pipeline
# Ensures all registered modules match expected implementations

set -euo pipefail

MODULE_REGISTRY="./MODULE_REGISTRY.toml"
REPORT_DIR="./verification-reports"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=========================================="
echo "Module Registry Verification"
echo "Timestamp: $TIMESTAMP"
echo "=========================================="

mkdir -p "$REPORT_DIR"

# Counters
TOTAL_MODULES=0
PASSED=0
FAILED=0
WARNINGS=0

# Verify a single module
verify_module() {
  local mod_id="$1"
  local mod_name="$2"
  local file_path="$3"
  local status="$4"
  local kpis="$5"
  
  TOTAL_MODULES=$((TOTAL_MODULES + 1))
  
  # Check status validity
  case "$status" in
    IMPLEMENTED|PARTIAL|STUB|MISSING|EXTERNAL)
      ;;
    *)
      echo -e "${RED}❌ FAIL${NC} $mod_id - Invalid status: $status"
      FAILED=$((FAILED + 1))
      return 1
      ;;
  esac
  
  if [ "$status" = "EXTERNAL" ]; then
    echo -e "${YELLOW}⚠️  SKIP${NC} $mod_id - External dependency ($mod_name)"
    WARNINGS=$((WARNINGS + 1))
    return 0
  fi
  
  if [ "$status" = "MISSING" ]; then
    echo -e "${RED}❌ FAIL${NC} $mod_id - Module marked as MISSING ($mod_name)"
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  # Verify file exists for implemented/partial/stub modules
  if [ "$status" != "EXTERNAL" ]; then
    if [ ! -f "$file_path" ]; then
      echo -e "${RED}❌ FAIL${NC} $mod_id - File not found: $file_path"
      FAILED=$((FAILED + 1))
      return 1
    fi
    
    # Verify file is not empty
    if [ ! -s "$file_path" ]; then
      echo -e "${RED}❌ FAIL${NC} $mod_id - File empty: $file_path"
      FAILED=$((FAILED + 1))
      return 1
    fi
  fi
  
  # Verify KPI assignments are valid (1-72)
  local kpi_valid=1
  for kpi in $(echo "$kpis" | tr ',' ' '); do
    kpi=$(echo "$kpi" | tr -d '[]" ')
    if [[ -n "$kpi" && ! "$kpi" =~ ^KPI-[0-9]+$ ]]; then
      echo -e "${RED}❌ FAIL${NC} $mod_id - Invalid KPI format: $kpi"
      kpi_valid=0
    fi
  done
  
  if [ $kpi_valid -eq 0 ]; then
    FAILED=$((FAILED + 1))
    return 1
  fi
  
  echo -e "${GREEN}✅ PASS${NC} $mod_id - $mod_name ($status)"
  PASSED=$((PASSED + 1))
  return 0
}

# Parse MODULE_REGISTRY.toml - simplified parser
# In production, use a proper TOML parser
current_id=""
current_name=""
current_file=""
current_status=""
current_kpis=""

while IFS= read -r line; do
  # Trim leading/trailing whitespace
  line=$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  
  # Skip comments and empty lines
  [[ "$line" =~ ^# ]] && continue
  [[ -z "$line" ]] && continue
  
  # Parse module entries
  if [[ "$line" =~ ^\[\[module\]\]$ ]]; then
    # If we have a previous module, verify it
    if [ -n "$current_id" ]; then
      verify_module "$current_id" "$current_name" "$current_file" "$current_status" "$current_kpis"
    fi
    # Reset for new module
    current_id=""
    current_name=""
    current_file=""
    current_status=""
    current_kpis=""
    continue
  fi
  
  # Parse key = value pairs
  if [[ "$line" =~ ^id\s*=\s*\"(.+)\"$ ]]; then
    current_id="${BASH_REMATCH[1]}"
  elif [[ "$line" =~ ^name\s*=\s*\"(.+)\"$ ]]; then
    current_name="${BASH_REMATCH[1]}"
  elif [[ "$line" =~ ^file\s*=\s*\"(.+)\"$ ]]; then
    current_file="${BASH_REMATCH[1]}"
  elif [[ "$line" =~ ^status\s*=\s*\"(.+)\"$ ]]; then
    current_status="${BASH_REMATCH[1]}"
  elif [[ "$line" =~ ^kpis\s*=\s*\[(.+)\]$ ]]; then
    current_kpis="${BASH_REMATCH[1]}"
  fi
done < "$MODULE_REGISTRY"

# Verify last module in file
if [ -n "$current_id" ]; then
  verify_module "$current_id" "$current_name" "$current_file" "$current_status" "$current_kpis"
fi

echo ""
echo "=========================================="
echo "Module Verification Summary"
echo "=========================================="
echo "Total Modules: $TOTAL_MODULES"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Warnings: $WARNINGS${NC}"

# Calculate pass rate
if [ $TOTAL_MODULES -gt 0 ]; then
  pass_rate=$(awk "BEGIN {print ($PASSED / $TOTAL_MODULES) * 100}")
  echo "Pass Rate: ${pass_rate}%"
else
  pass_rate=0
  echo "Pass Rate: N/A"
fi

# Verify summary counts match
echo ""
echo "Verifying module summary counts..."
expected_total=$(grep -A5 '\[summary\]' "$MODULE_REGISTRY" | grep 'total' | head -1 | sed 's/.*=\s*\([0-9]*\).*/\1/')
if [ -n "$expected_total" ]; then
  echo "Expected total modules: $expected_total"
  if [ "$expected_total" -eq "$TOTAL_MODULES" ]; then
    echo -e "${GREEN}✅ Count matches${NC}"
  else
    echo -e "${YELLOW}⚠️  Count mismatch: expected $expected_total, found $TOTAL_MODULES${NC}"
  fi
fi

# Generate report
cat > "$REPORT_DIR/module-report-$TIMESTAMP.md" << EOF
# Module Registry Verification Report
**Timestamp:** $TIMESTAMP
**Total Modules:** $TOTAL_MODULES
**Passed:** $PASSED
**Failed:** $FAILED
**Warnings:** $WARNINGS
**Pass Rate:** ${pass_rate}%

## Status
$(if [ $FAILED -eq 0 ]; then echo "✅ ALL MODULES VERIFIED"; else echo "❌ MODULE ISSUES DETECTED"; fi)
EOF

# Exit code
if [ $FAILED -eq 0 ]; then
  echo ""
  echo -e "${GREEN}✅ Module verification passed${NC}"
  exit 0
else
  echo ""
  echo -e "${RED}❌ Module verification failed - $FAILED modules have issues${NC}"
  exit 1
fi