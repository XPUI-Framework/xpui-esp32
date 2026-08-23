#!/usr/bin/env bash

# Everything CI checks in this repository, in one command.
#
#   ./build-and-test.sh          format, lint, test, and the prose
#   ./build-and-test.sh check    the same thing; the name CI uses
#   ./build-and-test.sh all      the above, plus linking both firmware images
#   ./build-and-test.sh fix      format Rust in place first
#
# **Half of what runs is in `bin/gate-common.sh`**, of which every repository
# in the organisation carries a byte-identical copy. This file is what this
# repository configures, what only it checks, and the order they run in.
# `xpui-dev` compares the nine copies and runs all nine gates.
#
# Two boards, two architectures: the X3 is an ESP32-C3, RISC-V, and reachable
# from stable Rust; the Sticky is an ESP32-S3, **Xtensa**, which stable Rust
# cannot target at all. That is why `.cargo/config.toml` sets no default target
# and why the second board's build lives in `all`.
#
# **Neither board has ever been run**, and neither panel has a driver — there
# is no published Rust or C++ one for either. The seam where it goes is marked
# in both binaries. Linking is all any check here proves.

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${PROJECT_DIR}"

SOURCE_ROOTS=(.)

# No default target in `.cargo/config.toml`, so the crate builds for the host
# too — which is what lets the shared checks run at all here. The device code
# sits behind a `device` cfg that a host build never parses; the RISC-V lint
# below is the only gate that reaches it.
TEST_FEATURES=""
HOST_WORKSPACE=1

# The X3. The chip is a feature of `esp-hal`, not a target, so the board's
# feature and the triple are named together.
LINT_TARGETS=("riscv32imc-unknown-none-elf")
LINT_TARGET_CRATES=(--bin x3 --features x3)

# The firmware has no tests, and cannot on a host.
#
# `test = false` on every target, because libtest does not exist for
# `riscv32imc-unknown-none-elf`. What is unreachable from a laptop is the glue:
# panel init, the frame loop, the allocator, the panic handler. The screens are
# `xpui-gallery`'s and are snapshotted seventy ways there.
UNTESTED_CRATES=(
  ".:the firmware; test = false, and esp-hal cannot compile for a laptop"
)

. bin/gate-common.sh

# ---------------------------------------------------------------------------
# What only this repository checks.
# ---------------------------------------------------------------------------

ESP32_S3_TOOLCHAIN="esp"
ESP32_S3_TARGET="xtensa-esp32s3-none-elf"

# The X3, linked. `all` rather than `check` because it is a full codegen pass.
esp32_c3_links() {
  say "The ESP32-C3 image links"
  cargo build --release --bin x3 --features x3 --target riscv32imc-unknown-none-elf
}

# The Sticky, linked — Xtensa, and so the one check that needs a toolchain
# stable Rust does not ship.
#
# Two prerequisites, and each skips with the command that fixes it: the `esp`
# fork from `espup`, and its `xtensa-esp32s3-elf-gcc`, which is the linker.
# `-Z build-std` because no precompiled `core` exists for any Xtensa target;
# the crate's own `.cargo/config.toml` asks for it, so this runs from here.
esp32_s3_links() {
  say "The ESP32-S3 image links"
  if ! rustup toolchain list | grep -q "^${ESP32_S3_TOOLCHAIN}"; then
    echo "    skipped: the '${ESP32_S3_TOOLCHAIN}' toolchain is not installed."
    echo "      cargo install espup && espup install"
    return 0
  fi
  local rustup_home linker
  rustup_home="${RUSTUP_HOME:-${HOME}/.rustup}"
  linker="$(find "${rustup_home}/toolchains/${ESP32_S3_TOOLCHAIN}" \
    -name 'xtensa-esp32s3-elf-gcc' 2>/dev/null | head -n1 || true)"
  if [[ -z "${linker}" ]]; then
    echo "    skipped: the 'esp' toolchain is installed but its linker is not."
    echo "      Re-run 'espup install', or source ~/export-esp.sh."
    return 0
  fi
  PATH="$(dirname "${linker}"):${PATH}" \
    cargo "+${ESP32_S3_TOOLCHAIN}" build --release \
      --bin sticky --features sticky --target "${ESP32_S3_TARGET}"
}

# ---------------------------------------------------------------------------

gates() {
  file_sizes
  crates_are_tested
  every_check_runs
  readmes_warn
  prose_is_compiled
  doc_paths
  commands_resolve
  cpp_snippets_compile
  lint
  test_suite
  doc_tests
  doc_links
}

case "${1:-check}" in
  check)
    run_all "${FORMAT_CHECK[@]}"
    gates
    printf '\nChecks passed. "./build-and-test.sh all" also links both images.\n'
    ;;
  fix)
    run_all "${FORMAT_FIX[@]}"
    gates
    printf '\nFormatted and checked.\n'
    ;;
  all)
    run_all "${FORMAT_CHECK[@]}"
    gates
    esp32_c3_links
    esp32_s3_links
    printf '\nEverything passed.\n'
    ;;
  *)
    echo "usage: ./build-and-test.sh [check|fix|all]" >&2
    exit 2
    ;;
esac
