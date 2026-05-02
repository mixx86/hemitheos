#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    printf 'Usage: %s [--target <arch|triple>]\n\n' "$(basename "$0")"
    printf 'Boot the hypervisor image in QEMU with serial output on stdout.\n\n'
    printf 'Options:\n'
    printf '  --target <arch|triple>  Target architecture or full Rust triple.\n'
    printf '                          Short names: aarch64 (default), riscv64.\n'
    printf '                          Overrides the TARGET_TRIPLE env var.\n'
    printf '  --help                  Show this message.\n\n'
    printf 'QEMU parameters (env overrides):\n'
    printf '  QEMU_BIN      QEMU binary  (default: qemu-system-aarch64)\n'
    printf '  QEMU_MACHINE  -machine arg (default: virt,virtualization=on,gic-version=3)\n'
    printf '  QEMU_CPU      -cpu arg     (default: cortex-a72)\n'
    printf '  QEMU_RAM      -m arg       (default: 512M)\n\n'
    printf 'Examples:\n'
    printf '  %s\n' "$(basename "$0")"
    printf '  %s --target aarch64\n' "$(basename "$0")"
    printf '  QEMU_CPU=cortex-a53 %s\n' "$(basename "$0")"
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

source "$SCRIPT_DIR/qemu-common.sh"

require_cmd "$QEMU_BIN"
ensure_image
qemu_base_args

exec "$QEMU_BIN" "${QEMU_BASE_ARGS[@]}" -monitor none -serial stdio
