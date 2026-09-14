# Contributing to `xpui-esp32`

## Building it

What the firmware is and what it cannot do yet is [hardware.md](hardware.md);
putting a first screen on a board is [tutorial.md](tutorial.md).

`rust-toolchain.toml` pins the toolchain and `riscv32imc-unknown-none-elf`,
so the [X3](https://www.xteink.com/products/xteink-x3) image builds on a fresh clone with nothing else. The [Sticky](https://www.seeedstudio.com/reTerminal-Sticky-p-6861.html) is
Xtensa: `cargo install espup && espup install` puts the `esp` fork and its
linker in place, `. ~/export-esp.sh` puts that linker on this shell's `PATH`,
and every command for it goes through `cargo +esp`. Skip the source line and
the build stops at ``linker `xtensa-esp32s3-elf-gcc` not found``; the gate
hunts for it and a bare `cargo` line does not. Flashing either needs
[`espflash`](https://crates.io/crates/espflash).

```bash
cargo build --release --bin x3 --features x3 --target riscv32imc-unknown-none-elf
./build-and-test.sh                        # everything CI checks
./build-and-test.sh all                    # the above, plus linking both images
```

**Exactly one board feature at a time.** The chip is a feature of [`esp-hal`](https://crates.io/crates/esp-hal),
so `--all-features` is two chips and fails inside `esp-metadata-generated`
before this crate is reached; there is deliberately no `compile_error!` guard,
because one could not fire first.

## Two workspaces

The firmware crate never builds for a laptop, so the tutorial's doctests live
in `docs-test/`, a workspace of its own that pulls the framework from git and
runs on the host; the gate runs it as `the prose compiles`. Unlike
`xpui-rp2040`, no `--target` is needed there: `.cargo/config.toml` sets no
default target, because the two boards are two architectures.

## The gate

A change is not finished until `./build-and-test.sh` passes. It is the same
command CI runs, so a green run locally means what a green tick means there.
The checks are listed in [`AGENTS.md`](../AGENTS.md) and implemented in
[`xtask/`](../xtask/); `./build-and-test.sh fix` formats in place first.
`all` additionally links both images — the Sticky's only where the fork is
installed, and it says so otherwise. CI links neither; run `all` before
pushing anything the linker could reject. CI lints the X3 on every run and
never builds the Sticky: installing the fork for each pull request would cost
minutes for one board.

Three things bite here more than anywhere else:

- **Neither target has an atomic compare-and-swap.** Load and store only;
  never `swap`, `fetch_or` or `compare_exchange`. The RISC-V [clippy](https://github.com/rust-lang/rust-clippy) run is
  the only check that reaches the `no_std` paths before a build does.
- **The heap is mostly framebuffer** — four fifths of it on the X3, leaving
  about 13 kB. `Panel` is leaked with the backend; a screen that buffers
  anything needs `HEAP_SIZE` raised first.
- **`esp32s3` is pinned to 0.35.2**, under a renamed optional dependency, and
  `Cargo.toml` says when to drop the pin. Do not `cargo update` it away.

## The review

Five steps, in order, none skipped:

1. The gate passes, with the real exit code read.
2. The [code-reviewer](../.claude/agents/code-reviewer.md) agent reviews the
   change — every finding resolved, not noted.
3. The [docs-reviewer](../.claude/agents/docs-reviewer.md) agent reviews the
   prose, last: it runs every command a document gives and resolves every
   snippet against the API.
4. The author flashes a board. That is their step; hardware is not an
   agent's to sign off, and neither board has been run yet.
5. They say commit.

## Commits

The subject says what was done — imperative, under fifty characters, one
concern. The body says what changed and why, in under about ten lines,
carrying the fact that is not in the diff. Nothing about how the bug was
found. No self-attribution.

## Working across the repositories

This crate depends on four siblings through `git` dependencies on `main`, and
nothing depends on it. Before pushing a change that reaches into one of them,
run the umbrella:

```bash
for d in ../xpui*/; do git -C "$d" fetch --quiet --all; done
cd ../xpui-dev && ./build-and-test.sh cross
```

It builds every crate from the sibling checkouts on disk and says which one
broke. `cross` is that repository's gate, not this one's — run it from there,
not here. The fetch first, because its link check resolves every
`github.com/XPUI-Framework/…` URL against each sibling's `origin/main`, and a
stale remote is a stale answer. `xpui`'s [`docs/orientation.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/orientation.md)
describes the layout it expects.
