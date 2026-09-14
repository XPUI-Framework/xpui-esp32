# `xpui-esp32`

## What this is, and what it may not become

The gallery as ESP32 firmware, for two boards on two architectures: `x3`, an
Xteink X3 on an ESP32-C3, and `sticky`, a Seeed Sticky on an ESP32-S3. The
crate supplies a heap, a panic handler, a frame loop that paints only when
the framework asks, and a one-bit framebuffer with the panel driver seam at
its end; the screens are `xpui-gallery`'s and the boards are `xpui-boards`'.
Both images build and link; neither board has been run.

**There is no panel driver, and this repository does not grow one.** No
published driver exists for these panels, and a hand-written one cannot be
verified from a repository that cannot power a panel; a screen reaches these
devices through their own firmware hosting `xpui` over the C ABI. `Panel::present`
is marked, not faked. Nothing here is a screen, a component or a board
description, and nothing compiles for a host: every item sits behind the
`device` cfg, so the tutorial's doctests are a workspace of their own; the
gate is a member, so one `cargo test` reaches its tests.

## The gate

```bash
./build-and-test.sh          # everything below
./build-and-test.sh all      # the above, plus the two lines marked `+`
./build-and-test.sh fix      # the same as check, formatting in place first
```

```text
format · file sizes · crates are tested · READMEs warn · prose is compiled · documented paths resolve · rustdoc links resolve · the reference mirrors rustdoc · documented commands resolve · lint · tests · doctests · the prose compiles · the nested clippy config agrees · README sections · AGENTS.md · published crates deny missing_docs · comment blocks · comment narration
+ the ESP32-C3 image links
+ the ESP32-S3 image links
```

A last stage, `the gate is documented`, compares this list to what ran. Run
it before saying a change is done, and read the real exit code; run `all`
before pushing anything the linker could reject. The S3 stage skips with a
note where the `esp` fork is not installed.

## What only this repository checks

- **`the prose compiles`** — `docs-test/`'s doctests, on the host, because
  the firmware crate cannot run a doctest on a laptop.
- **`rustdoc links resolve` runs for the board as well as the host.** Every
  item under `src/` sits behind the `device` cfg, so a host rustdoc parses an
  empty crate and reads none of its doc comments. The `riscv32imc` run is the
  only one that reads them, and it documents the library and the `x3`
  binary; `src/bin/sticky.rs` needs the Xtensa fork, so no rustdoc run reads
  its doc comments.
- **`the reference mirrors rustdoc`** reads that board run's output, under
  `target/riscv32imc-unknown-none-elf/doc/xpui_esp32`, never the empty host
  one. With no board output it fails rather than skips.
- **`the nested clippy config agrees`** — `docs-test/` is a workspace of its
  own, so `--workspace` reaches neither its lints nor its `clippy.toml`.
  `lint` names it explicitly and this stage keeps its config equal to the
  root's; `xpui-dev` compares only the root file and cannot see it.
- **`the ESP32-C3 image links`** and **`the ESP32-S3 image links`**, under
  `all` only: the linker script, `build.rs`'s `linkall.x`, and both binaries.
- **`lint` runs for one board**: the host workspace, then
  `cargo clippy --release --bin x3 --features x3` on `riscv32imc`, then
  `docs-test/`. The Xtensa half is never linted here; it needs the fork.
- `published crates deny missing_docs` prints `no publishable crates`, this
  repository's permanent truth, so it checks nothing.
  `#![deny(missing_docs)]` in `src/lib.rs` is what actually holds the rule,
  and only the board build reads it.

## Style that bites here

- **Two architectures, one crate, exactly one board feature at a time.**
  `--features x3` or `--features sticky`, never `--all-features`; the chip
  is a feature of `esp-hal`, and two at once fail inside a dependency.
- **Neither target has an atomic compare-and-swap.** `riscv32imc` has no A
  extension; load and store only, never `swap`, `fetch_or` or
  `compare_exchange`.
- **The Xtensa fork.** The Sticky builds only through `cargo +esp`, with
  `core` from source and **no `-C force-frame-pointers`** — the fork's LLVM
  fails to build `compiler_builtins` with it. `.cargo/config.toml` sets no
  default target and says why; do not give it one.
- **The PAC pin.** `esp32s3` is pinned to exactly 0.35.2 under a renamed
  optional dependency; 0.35.3 renames two fields `esp-hal` calls.
- **`esp-backtrace` needs exactly one of `defmt` or `println`**, and
  `linkall.x` comes from `build.rs`, not the cargo config.
- **The heap is four fifths framebuffer**: 52,272 of 65,536 bytes on the X3.
  Raise `HEAP_SIZE` before a screen buffers anything.
- **`no_std`, no `format!` on the frame path.**
- **Every `pub` item is documented.**
- **A file under `src/` is at most 400 lines.**

## Where the documentation lives, and what proves each piece

| Document | Proven by |
|---|---|
| [`README.md`](README.md), [`docs-test/README.md`](docs-test/README.md) | their paths and commands resolve; neither carries a `rust` fence |
| [`docs/README.md`](docs/README.md) | its paths resolve; the README-heading check exempts it, because it is the index of `docs/`, not a front page |
| [`docs/tutorial.md`](docs/tutorial.md) | every `rust` fence is a doctest in `docs-test/`, mounted by `docs-test/src/lib.rs` |
| [`docs/reference.md`](docs/reference.md) | `the reference mirrors rustdoc`, against the board's rustdoc; every `rust` fence is a doctest in `docs-test/`, mounted by `docs-test/src/lib.rs` |
| [`docs/hardware.md`](docs/hardware.md) | its paths resolve, and nothing else: **no stage recomputes a number in it.** Every figure here is checked by hand: the framebuffer sizes against `xteink::X3` and `seeed::STICKY` in `xpui-boards`, the heap against `src/runtime.rs`, the PAC versions against `Cargo.toml`, and the SRAM against Espressif's datasheets |
| [`docs/contributing.md`](docs/contributing.md) | every path and command it gives resolves; the umbrella command is `xpui-dev`'s |
| `AGENTS.md` | the stage list above is compared to what the gate runs, in both modes |
| every `///` and `//!` | the two comment checks; and `rustdoc links resolve` — the host workspace, `docs-test/`, and on the board the library and `src/bin/x3.rs`, never `src/bin/sticky.rs` |

## Git

Never stage, never commit, never push without being asked, each time. The
index is the reviewer's queue; leave new work unstaged. No self-attribution
in a commit message. Never rewrite a commit that exists; a correction is a new
commit. The rules that apply to all ten repositories, and the five review
steps, are in [`xpui`'s `docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md);
how a change is built and reviewed here is in
[`docs/contributing.md`](docs/contributing.md).
