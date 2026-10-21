# hemitheos

Type-1 bare-metal hypervisor in Rust. **Target:** AArch64 only, early development.

```bash
bash scripts/doctor.sh    # prerequisites
bash scripts/build.sh
bash scripts/test.sh      # host integration tests
bash scripts/qemu-run.sh  # serial on stdout
```

Scripts take `--target aarch64` (default) and `--help`. Other helpers: `check.sh`, `qemu-gdb.sh`, `gdb.sh`, `qemu-trace.sh`.

More context (kept short on purpose): [`docs/roadmap.md`](docs/roadmap.md), [`CONTRIBUTING.md`](CONTRIBUTING.md).

MIT
