#!/usr/bin/env bash
# Local CI for the xtop kernel repo (single-crate layout).
#
# Intentionally NOT wired into git: no GitHub Actions, no git hooks. Run it
# yourself from the repo root:
#
#   ./scripts/ci.sh            # run every stage
#   ./scripts/ci.sh fmt        # run one stage
#
# Stages: fmt | clippy | check | test | no-default
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f Cargo.toml ]]; then
    echo "[ci] no Cargo.toml at repo root; nothing to run."
    exit 0
fi

stages=(fmt clippy check test no-default)
requested=("$@")
if [[ ${#requested[@]} -eq 0 ]]; then
    requested=("${stages[@]}")
fi

fmt() {
    echo "==> fmt (format check)"
    cargo fmt --all -- --check
}

clippy() {
    echo "==> clippy (all targets, warnings denied)"
    cargo clippy --all-targets -- -D warnings
}

check() {
    echo "==> check"
    cargo check
}

test() {
    echo "==> test"
    cargo test
}

no_default() {
    echo "==> check core only (built without the samurai plugin and mcp)"
    cargo check --no-default-features
}

for stage in "${requested[@]}"; do
    case "$stage" in
        fmt) fmt ;;
        clippy) clippy ;;
        check) check ;;
        test) test ;;
        no-default) no_default ;;
        *)
            echo "[ci] unknown stage: $stage (expected one of: ${stages[*]})" >&2
            exit 1
            ;;
    esac
done

echo "[ci] all stages passed."
