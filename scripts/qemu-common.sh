#!/bin/bash

QEMU_COMMON_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$QEMU_COMMON_DIR/common.sh"

# Derive QEMU defaults from the target architecture.
# All variables can be overridden via environment.
case "$TARGET_ARCH" in
    aarch64)
        QEMU_BIN="${QEMU_BIN:-qemu-system-aarch64}"
        QEMU_MACHINE="${QEMU_MACHINE:-virt,virtualization=on,gic-version=3}"
        QEMU_CPU="${QEMU_CPU:-cortex-a72}"
        ;;
    riscv64)
        QEMU_BIN="${QEMU_BIN:-qemu-system-riscv64}"
        QEMU_MACHINE="${QEMU_MACHINE:-virt}"
        QEMU_CPU="${QEMU_CPU:-rv64}"
        ;;
    *)
        die "unsupported target architecture for QEMU: $TARGET_ARCH"
        ;;
esac

QEMU_RAM="${QEMU_RAM:-512M}"
QEMU_LOG_PATH="${QEMU_LOG_PATH:-$REPO_ROOT/qemu.log}"
GDB_PORT="${GDB_PORT:-1234}"

qemu_base_args() {
    QEMU_BASE_ARGS=(
        -machine "$QEMU_MACHINE"
        -cpu "$QEMU_CPU"
        -m "$QEMU_RAM"
        -nographic
        -kernel "$IMAGE_BIN"
    )
}
