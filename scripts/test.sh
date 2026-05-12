#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

require_cmd cargo

# The binary target requires --features bin-target and an embedded toolchain;
# it is excluded here so tests run natively on the dev machine.
cargo test --lib --tests "$@"
