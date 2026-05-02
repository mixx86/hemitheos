#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    printf 'Usage: %s [--target <arch|triple>]\n\n' "$(basename "$0")"
    printf 'Run cargo fmt check and cargo clippy for the given target.\n\n'
    printf 'Options:\n'
    printf '  --target <arch|triple>  Target architecture or full Rust triple.\n'
    printf '                          Short names: aarch64 (default), riscv64.\n'
    printf '                          Overrides the TARGET_TRIPLE env var.\n'
    printf '  --help                  Show this message.\n\n'
    printf 'Examples:\n'
    printf '  %s\n' "$(basename "$0")"
    printf '  %s --target aarch64\n' "$(basename "$0")"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)
            shift
            [[ $# -gt 0 ]] || { printf 'error: --target requires an argument\n' >&2; exit 1; }
            case "$1" in
                aarch64) TARGET_TRIPLE="aarch64-unknown-none" ;;
                riscv64) TARGET_TRIPLE="riscv64gc-unknown-none-elf" ;;
                *)       TARGET_TRIPLE="$1" ;;
            esac
            ;;
        --help|-h) usage ;;
        *) printf 'error: unknown argument: %s\n' "$1" >&2; exit 1 ;;
    esac
    shift
done

source "$SCRIPT_DIR/common.sh"

require_cmd cargo
require_cmd "$TARGET_CC"
cargo_profile_args

note "running cargo fmt --all -- --check"
cargo fmt --all -- --check

note "running cargo check for $TARGET_TRIPLE"
TARGET_CC="$TARGET_CC" cargo check --target "$TARGET_TRIPLE" "${CARGO_PROFILE_ARGS[@]}"

if cargo clippy --version >/dev/null 2>&1; then
    note "running cargo clippy for $TARGET_TRIPLE"
    TARGET_CC="$TARGET_CC" cargo clippy --target "$TARGET_TRIPLE" "${CARGO_PROFILE_ARGS[@]}"
else
    note "skipping cargo clippy (component not installed)"
fi
