# Contributing

## Architecture Support

AArch64 is the primary and only implemented target. The build infrastructure is structured to support additional architectures in the future: toolchain, linker script, QEMU binary, and GDB architecture are all derived automatically from `--target`.

To add support for a new architecture the following pieces are needed:

| What | Where |
|---|---|
| Boot entry + linker script | `asm/<arch>/` |
| Arch module (exception handlers, etc.) | `src/arch/<arch>/mod.rs`, gated with `#[cfg(target_arch = "...")]` |
| ASM compilation | `build.rs` — add a new `match` arm |
| Linker flags | `.cargo/config.toml` — add a `[target.<triple>]` section |
| Toolchain defaults | `scripts/common.sh` — extend the `case "$TARGET_ARCH"` block |
| QEMU defaults | `scripts/qemu-common.sh` — extend the `case "$TARGET_ARCH"` block |
| Short name alias | All scripts — extend the `--target` `case` block |
| Arch-conditional DTB fields | `src/dtb.rs` — `PlatformInfo` and `parse_dtb` |

Open an issue before starting a port so scope and readiness can be discussed.

---

## Development Environment

The build system derives the cross-compilation toolchain, linker script, and QEMU binary automatically from the target. All scripts accept `--target <arch>` (short name or full triple) and `--help`.

### Required Tools — AArch64 (current target)

| Tool | Notes |
|---|---|
| Rust target `aarch64-unknown-none` | `rustup target add aarch64-unknown-none` |
| `aarch64-linux-gnu-gcc` | AArch64 cross-compiler; available in most distro package managers |
| `aarch64-linux-gnu-objcopy` | Usually part of the same `binutils` cross package |
| `qemu-system-aarch64` *(optional)* | For emulation bring-up |
| `gdb-multiarch` *(optional)* | For source-level debugging |

Run `bash scripts/doctor.sh` to verify what is and isn't present.

All scripts accept `--target <arch|triple>` to select the target and `--help` to show full usage. The `TARGET_TRIPLE` env var is also supported; `--target` takes precedence.

```bash
bash scripts/doctor.sh --target aarch64   # verify prerequisites
bash scripts/check.sh  --target aarch64   # formatter + clippy
bash scripts/build.sh  --target aarch64   # compile and strip to raw binary
```

The build compiles assembly via the `cc` crate, links with the target-specific linker script under `asm/<arch>/`, and strips the ELF to a raw binary. The following variables can be overridden via env or derived automatically from the target:

| Variable | Default (AArch64) | Purpose |
|---|---|---|
| `TARGET_TRIPLE` | `aarch64-unknown-none` | Rust target triple |
| `TARGET_CC` | `aarch64-linux-gnu-gcc` | C cross-compiler (assembles `.s` files) |
| `OBJCOPY` | `aarch64-linux-gnu-objcopy` | Strips ELF to raw binary |
| `PROFILE` | `release` | Cargo profile |

### QEMU Bring-Up

```bash
bash scripts/qemu-run.sh   [--target aarch64]  # serial output on stdout
bash scripts/qemu-trace.sh [--target aarch64]  # trace flags; output → qemu.log
bash scripts/qemu-gdb.sh   [--target aarch64]  # start paused, GDB stub on :1234
bash scripts/gdb.sh        [--target aarch64]  # connect GDB (second terminal)
```

The QEMU binary, machine, and CPU are derived from the target automatically. They can be overridden via `QEMU_BIN`, `QEMU_MACHINE`, and `QEMU_CPU`. The GDB architecture is also set automatically. The `qemu-*` scripts are scoped to the `virt` machine; they are bring-up aids, not the boundary of supported deployment targets.

---

## Project Layout

```
src/uart.rs              UartDriver trait, global registry, DTB UART discovery
src/drivers/uart/        One file per UART hardware variant
src/arch/<arch>/         Architecture-specific Rust code, gated with #[cfg(target_arch)]
src/dtb.rs               DTB parsing → PlatformInfo (interrupt controller fields are arch-gated)
src/log.rs               Hypervisor logging (stub — contributions welcome)
asm/<arch>/              Assembly entry point, vector table, and linker script per architecture
```

---

## Code Style

- `no_std` everywhere; never pull in `std`
- No heap allocations until an allocator is explicitly introduced
- `unsafe` blocks must have a `// SAFETY:` comment explaining the invariant being upheld
- No comments that describe *what* the code does — only *why* when it is non-obvious
- Architecture-specific code lives under `src/arch/<arch>/` and `asm/<arch>/`, gated with `#[cfg(target_arch = "...")]`
- New hardware drivers go under `src/drivers/<subsystem>/<variant>.rs` and expose an `init(base: u64) -> &'static dyn <Trait>` function

---

## Commit Messages

- Imperative mood, present tense: `add PL011 baud rate calculation`, not `added` or `adds`
- Reference the subsystem in the subject when relevant: `uart: wire init in main`
- Keep the subject under 72 characters
