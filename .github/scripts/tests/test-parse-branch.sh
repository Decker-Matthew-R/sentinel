#!/usr/bin/env bash

SCRIPT="$(dirname "$0")/../parse-branch.sh"
PASS=0
FAIL=0
GREEN='\033[0;32m'
RED='\033[0;31m'
RESET='\033[0m'

check() {
    local branch="$1"
    local expected="$2"
    local actual
    actual="$("$SCRIPT" "$branch")"
    if [[ "$actual" == "$expected" ]]; then
        echo -e "${GREEN}PASS${RESET}: '$branch' → '$actual'"
        ((PASS++))
    else
        echo -e "${RED}FAIL${RESET}: '$branch' → expected '$expected', got '$actual'"
        ((FAIL++))
    fi
}

check "42-some-issue-title"  "42"
check "6-chore-document-branch-and-commit-conventions"  "6"
check "main"                ""
check "experiment/foo"      ""

echo ""
if [[ "$FAIL" -eq 0 ]]; then
    echo -e "${GREEN}$PASS passed, $FAIL failed${RESET}"
else
    echo -e "${RED}$PASS passed, $FAIL failed${RESET}"
fi
[[ "$FAIL" -eq 0 ]]