#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  printf 'Usage: %s [--target <arch|triple>]\n\n' "$(basename "$0")"
  printf 'Connect GDB to a QEMU instance started with qemu-gdb.sh.\n\n'
  printf 'Options:\n'
  printf '  --target <arch|triple>  Target architecture or full Rust triple.\n'
  printf '                          Short names: aarch64 (default), riscv64.\n'
  printf '                          Overrides the TARGET_TRIPLE env var.\n'
  printf '  --help                  Show this message.\n\n'
  printf 'Env overrides:\n'
  printf '  GDB_BIN   GDB binary (default: gdb-multiarch)\n'
  printf '  GDB_PORT  Port to connect to (default: 1234)\n\n'
  printf 'Examples:\n'
  printf '  %s\n' "$(basename "$0")"
  printf '  %s --target aarch64\n' "$(basename "$0")"
  printf '  GDB_PORT=4321 %s\n' "$(basename "$0")"
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
  --target)
    shift
    [[ $# -gt 0 ]] || {
      printf 'error: --target requires an argument\n' >&2
      exit 1
    }
    case "$1" in
    aarch64) TARGET_TRIPLE="aarch64-unknown-none" ;;
    riscv64) TARGET_TRIPLE="riscv64gc-unknown-none-elf" ;;
    *) TARGET_TRIPLE="$1" ;;
    esac
    ;;
  --help | -h) usage ;;
  *)
    printf 'error: unknown argument: %s\n' "$1" >&2
    exit 1
    ;;
  esac
  shift
done

# GDB needs the unstripped debug build; default to the dev profile
# (target/<triple>/debug/) unless the caller overrides PROFILE explicitly.
PROFILE="${PROFILE:-dev}"

source "$SCRIPT_DIR/qemu-common.sh"

require_cmd "$GDB_BIN"
require_file "$IMAGE_ELF"

case "$TARGET_ARCH" in
aarch64) _gdb_arch="aarch64" ;;
riscv64) _gdb_arch="riscv:rv64" ;;
*) _gdb_arch="$TARGET_ARCH" ;;
esac

exec "$GDB_BIN" \
  "$IMAGE_ELF" \
  -ex "set architecture $_gdb_arch" \
  -ex "target remote :$GDB_PORT"
