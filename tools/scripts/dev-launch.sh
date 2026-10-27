#!/usr/bin/env bash
set -euo pipefail

# macOS: panel must run in the foreground to get display access.
# The fake-agent opens in a new Terminal window and waits for the panel to bind port 9000.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "Building panel and fake-agent..."
cargo build --manifest-path "$REPO_ROOT/panel/Cargo.toml"
cargo build --manifest-path "$REPO_ROOT/tools/fake-agent/Cargo.toml"

echo "Opening fake-agent in a new Terminal window..."
osascript -e "tell application \"Terminal\" to do script \"until nc -z 127.0.0.1 9000 2>/dev/null; do sleep 0.1; done && cd '$REPO_ROOT' && cargo run --manifest-path tools/fake-agent/Cargo.toml\""

echo "Starting panel..."
cargo run --manifest-path "$REPO_ROOT/panel/Cargo.toml"
