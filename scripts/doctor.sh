#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    printf 'Usage: %s [--target <arch|triple>]\n\n' "$(basename "$0")"
    printf 'Check that all required build tools are present for the given target.\n\n'
    printf 'Options:\n'
    printf '  --target <arch|triple>  Target architecture or full Rust triple.\n'
    printf '                          Short names: aarch64 (default), riscv64.\n'
    printf '                          Overrides the TARGET_TRIPLE env var.\n'
    printf '  --help                  Show this message.\n\n'
    printf 'Examples:\n'
    printf '  %s\n' "$(basename "$0")"
    printf '  %s --target aarch64\n' "$(basename "$0")"
    printf '  %s --target riscv64\n' "$(basename "$0")"
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

failures=0

check_required_cmd() {
    local cmd="$1"
    if command -v "$cmd" >/dev/null 2>&1; then
        printf '[ok] %s -> %s\n' "$cmd" "$(command -v "$cmd")"
    else
        printf '[missing] %s\n' "$cmd" >&2
        failures=1
    fi
}

check_optional_cmd() {
    local cmd="$1"
    if command -v "$cmd" >/dev/null 2>&1; then
        printf '[ok] %s -> %s\n' "$cmd" "$(command -v "$cmd")"
    else
        printf '[optional] %s\n' "$cmd"
    fi
}

check_required_rust_target() {
    if ! command -v rustup >/dev/null 2>&1; then
        printf '[missing] rustup (required for target %s)\n' "$TARGET_TRIPLE" >&2
        failures=1
        return
    fi
    if rustup target list --installed | grep -Fxq "$TARGET_TRIPLE"; then
        printf '[ok] rust target %s\n' "$TARGET_TRIPLE"
    else
        printf '[missing] rust target %s\n' "$TARGET_TRIPLE" >&2
        failures=1
    fi
}

check_required_cargo_subcommand() {
    local label="$1"; shift
    if "$@" >/dev/null 2>&1; then
        printf '[ok] %s\n' "$label"
    else
        printf '[missing] %s\n' "$label" >&2
        failures=1
    fi
}

check_optional_cargo_subcommand() {
    local label="$1"; shift
    if "$@" >/dev/null 2>&1; then
        printf '[ok] %s\n' "$label"
    else
        printf '[optional] %s\n' "$label"
    fi
}

printf 'Checking build toolchain for %s (arch: %s)\n' "$TARGET_TRIPLE" "$TARGET_ARCH"

check_required_cmd cargo
check_required_cmd rustup
check_required_cmd "$TARGET_CC"
check_required_cmd "$OBJCOPY"
check_required_rust_target
check_required_cargo_subcommand "cargo fmt" cargo fmt --version
check_optional_cargo_subcommand "cargo clippy" cargo clippy --version

[[ "$failures" -ne 0 ]] && exit 1

printf '\nOptional tooling for emulation/debug flows\n'
source "$SCRIPT_DIR/qemu-common.sh"
check_optional_cmd "$QEMU_BIN"
check_optional_cmd "$GDB_BIN"

if command -v timeout >/dev/null 2>&1 || command -v gtimeout >/dev/null 2>&1; then
    printf '[ok] timeout helper\n'
else
    printf '[optional] timeout/gtimeout\n'
fi

note "build environment ready"
