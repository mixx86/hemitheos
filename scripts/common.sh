#!/bin/bash

COMMON_SH_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$COMMON_SH_DIR/.." && pwd)"

PROJECT_NAME="${PROJECT_NAME:-hemitheos}"
TARGET_TRIPLE="${TARGET_TRIPLE:-aarch64-unknown-none}"
PROFILE="${PROFILE:-release}"

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*"
}

if [[ -z "${TARGET_ARCH:-}" ]]; then
  _raw_arch="$(printf '%s' "$TARGET_TRIPLE" | cut -d- -f1)"
  case "$_raw_arch" in
  aarch64) TARGET_ARCH="aarch64" ;;
  riscv64*) TARGET_ARCH="riscv64" ;;
  *) die "unsupported target architecture: $_raw_arch" ;;
  esac
  unset _raw_arch
fi

case "$TARGET_ARCH" in
aarch64)
  TARGET_CC="${TARGET_CC:-aarch64-linux-gnu-gcc}"
  OBJCOPY="${OBJCOPY:-aarch64-linux-gnu-objcopy}"
  GDB_BIN="${GDB_BIN:-gdb-multiarch}"
  ;;
riscv64)
  TARGET_CC="${TARGET_CC:-riscv64-linux-gnu-gcc}"
  OBJCOPY="${OBJCOPY:-riscv64-linux-gnu-objcopy}"
  GDB_BIN="${GDB_BIN:-gdb-multiarch}"
  ;;
*)
  die "unsupported target architecture: $TARGET_ARCH"
  ;;
esac

if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  TARGET_DIR_ROOT="$CARGO_TARGET_DIR"
else
  TARGET_DIR_ROOT="$REPO_ROOT/target"
fi

CARGO_DIR="${PROFILE}"
[[ "$PROFILE" == "dev" ]] && CARGO_DIR="debug"
IMAGE_DIR="$TARGET_DIR_ROOT/$TARGET_TRIPLE/$CARGO_DIR"

update_image_paths() {
  mkdir -p "$IMAGE_DIR"
  local found_elf
  found_elf=$(find "$IMAGE_DIR" -maxdepth 1 -type f ! -name "*.*" -printf "%s %p\n" 2>/dev/null | sort -nr | head -n1 | cut -d' ' -f2)

  if [[ -n "$found_elf" ]]; then
    IMAGE_ELF="$found_elf"
    IMAGE_BIN="${IMAGE_ELF}.bin"
  else
    IMAGE_ELF="$IMAGE_DIR/$PROJECT_NAME"
    IMAGE_BIN="$IMAGE_DIR/$PROJECT_NAME.bin"
  fi
}

update_image_paths

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

require_file() {
  if [[ ! -f "$1" ]]; then
    ls -la "$IMAGE_DIR" 2>/dev/null || true
    die "missing file: $1"
  fi
}

cargo_profile_args() {
  CARGO_PROFILE_ARGS=()
  if [[ "$PROFILE" == "release" ]]; then
    CARGO_PROFILE_ARGS=(--release)
  elif [[ "$PROFILE" != "dev" ]]; then
    CARGO_PROFILE_ARGS=(--profile "$PROFILE")
  fi
}

ensure_image() {
  require_file "$IMAGE_BIN"
}

timeout_bin() {
  if command -v timeout >/dev/null 2>&1; then
    printf 'timeout\n'
  elif command -v gtimeout >/dev/null 2>&1; then
    printf 'gtimeout\n'
  else
    die "missing timeout/gtimeout"
  fi
}
