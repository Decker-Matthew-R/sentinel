#!/usr/bin/env bash

BRANCH="$1"

if [[ "$BRANCH" =~ ^([0-9]+)- ]]; then
    NUM="${BASH_REMATCH[1]}"
fi

NUM=$((10#${NUM:-0}))

if [[ "$NUM" -gt 0 ]]; then
    echo "$NUM"
fi