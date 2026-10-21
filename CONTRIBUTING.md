# Contributing

Early-stage project: **AArch64** is the only target that is built and tested in CI. Scripts accept `--target aarch64` (default) and `--help`.

## Branches

- **`main`** — last known-good bring-up (what most people should clone).
- **`dev`** — active work in progress; may not build or boot.

Use PRs into `main` when something is self-contained and QEMU-clean. Large WIP can stay on `dev`.

## Before you send a PR

```bash
bash scripts/doctor.sh
bash scripts/check.sh      # rustfmt + clippy
bash scripts/test.sh       # host tests (library + integration)
bash scripts/build.sh      # bare-metal image (needs bin-target toolchain)
```

Optional: `bash scripts/qemu-run.sh` for a quick boot check.

Open an issue before a **new architecture port** or a large refactor; the tree is still moving.

## Layout (high level)

Single crate: Rust under `src/`, boot/linker/asm under `src/arch/arm/`, assembly build in `build.rs`, helpers in `scripts/`. Exact module boundaries change — follow what is already there rather than this list.

## Code style

- `no_std`; no heap until the project adds an allocator
- Every `unsafe` block needs a `// SAFETY:` line explaining the invariant
- Commit subjects: imperative, present tense, ~72 characters or less (`mmu: enable EL2 stage-1 boot tables`)
