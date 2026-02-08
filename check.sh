#!/usr/bin/env bash
# This script runs CI-like checks in a convenient way.
set -euo pipefail

find_tool() {
    local name="$1"
    if command -v "$name" >/dev/null 2>&1; then
        command -v "$name"
        return 0
    fi
    if command -v "${name}.exe" >/dev/null 2>&1; then
        command -v "${name}.exe"
        return 0
    fi

    for candidate in "$HOME/.cargo/bin/$name" "$HOME/.cargo/bin/${name}.exe"; do
        if [ -x "$candidate" ]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    echo "error: required tool '$name' not found" >&2
    return 1
}

CARGO_BIN="$(find_tool cargo)"
TRUNK_BIN="$(find_tool trunk)"

"$CARGO_BIN" check --quiet --workspace --all-targets
"$CARGO_BIN" check --quiet --workspace --all-features --lib --target wasm32-unknown-unknown
"$CARGO_BIN" fmt --all -- --check
"$CARGO_BIN" clippy --quiet --workspace --all-targets --all-features -- -D warnings -W clippy::all
"$CARGO_BIN" test --quiet --workspace --all-targets --all-features
"$CARGO_BIN" test --quiet --workspace --doc
"$TRUNK_BIN" build
