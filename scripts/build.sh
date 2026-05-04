#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  printf 'Usage: %s [--target <arch|triple>]\n\n' "$(basename "$0")"
  printf 'Compile the hypervisor and strip the ELF to a raw binary.\n\n'
  printf 'Options:\n'
  printf '  --target <arch|triple>  Target architecture or full Rust triple.\n'
  printf '                          Short names: aarch64 (default), riscv64.\n'
  printf '                          Overrides the TARGET_TRIPLE env var.\n'
  printf '  --help                  Show this message.\n\n'
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

source "$SCRIPT_DIR/common.sh"

case "$TARGET_ARCH" in
aarch64) LINKER_SCRIPT="asm/aarch64/linker.ld" ;;
riscv64) LINKER_SCRIPT="asm/riscv64/linker.ld" ;;
*) die "No linker script for $TARGET_ARCH" ;;
esac

require_cmd cargo
require_cmd "$TARGET_CC"
require_cmd "$OBJCOPY"

cargo_profile_args

export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-T$LINKER_SCRIPT"

TARGET_CC="$TARGET_CC" cargo build --target "$TARGET_TRIPLE" --features selftest "${CARGO_PROFILE_ARGS[@]}"

update_image_paths
require_file "$IMAGE_ELF"

"$OBJCOPY" -O binary "$IMAGE_ELF" "$IMAGE_BIN"

note "built $IMAGE_BIN"
