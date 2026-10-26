#!/usr/bin/env bash
set -euo pipefail

# Validates your working tree against every CI gate before you push for tighter feedback loops.
# Fixes formatting automatically, then checks linting and tests match what the pipeline enforces.

# ── Colours ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ─────────────────────────────────────────────────────────────────────
pass() { echo -e "  ${GREEN}✔${RESET}  $1"; }
fail() { echo -e "  ${RED}✘${RESET}  $1"; }
info() { echo -e "  ${CYAN}→${RESET}  $1"; }
header() { echo -e "\n${BOLD}${YELLOW}$1${RESET}"; }

FAILED=()

run_step() {
  local label="$1"
  shift
  if "$@" > /tmp/ci-gates-output 2>&1; then
    pass "$label"
  else
    fail "$label"
    echo -e "\n${RED}--- Output ---${RESET}"
    cat /tmp/ci-gates-output
    echo -e "${RED}--------------${RESET}\n"
    FAILED+=("$label")
  fi
}

echo "Validates your working tree against every CI gate before you push for tighter feedback loops.
Fixes formatting automatically, then checks linting and tests match what the pipeline enforces."

# ── Format (auto-fix) ───────────────────────────────────────────────────────────
header "Formatting"
info "Auto-formatting agent and panel — any changes will be applied"
dotnet format agent/ > /dev/null 2>&1
pass "Agent formatted (dotnet format)"
cargo fmt --manifest-path panel/Cargo.toml
pass "Panel formatted (cargo fmt)"

# ── Lint ────────────────────────────────────────────────────────────────────────
header "Linting"
run_step "Agent — dotnet format --verify-no-changes" \
  dotnet format --verify-no-changes agent/
run_step "Panel — cargo fmt --check" \
  cargo fmt --manifest-path panel/Cargo.toml -- --check
run_step "Panel — cargo clippy" \
  cargo clippy --manifest-path panel/Cargo.toml -- -D warnings

# ── Tests ───────────────────────────────────────────────────────────────────────
header "Tests"
run_step "Agent — dotnet test" \
  dotnet test agent/
run_step "Panel — cargo test" \
  cargo test --manifest-path panel/Cargo.toml

# ── Summary ─────────────────────────────────────────────────────────────────────
echo -e "\n${BOLD}────────────────────────────────${RESET}"
if [ ${#FAILED[@]} -eq 0 ]; then
  echo -e "${GREEN}${BOLD}  All gates passed. Safe to push.${RESET}"
else
  echo -e "${RED}${BOLD}  ${#FAILED[@]} gate(s) failed:${RESET}"
  for step in "${FAILED[@]}"; do
    echo -e "  ${RED}•${RESET} $step"
  done
  echo -e "\n${YELLOW}  Fix the issues above and re-run this script before pushing.${RESET}"
  exit 1
fi
echo -e "${BOLD}────────────────────────────────${RESET}\n"
