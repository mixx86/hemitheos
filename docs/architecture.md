# Architecture

AArch64 hypervisor running at Non-secure EL2. Boot uses the ARM64 Linux Image protocol; platform data comes mainly from a DTB today.

Implementation detail lives in the tree (`src/`, `scripts/`). This file is intentionally minimal so it does not drift from the code.
