# Hardware notes

Two boards on two architectures, and what is known about running the gallery
on them. The short version: both images build and link, neither board has
been run, and there is no panel driver — on purpose.

[tutorial.md](tutorial.md) is how to put a screen on one of these boards;
[contributing.md](contributing.md) is how a change to this firmware is made
and reviewed; [reference.md](reference.md) is every public item.

## What runs, and what does not

Real and exercised by the build: the board's geometry, the chrome sized from
it, the frame loop, the allocator, the panic handler, and the whole gallery
laid out for the right panel. The firmware reports its ink count over the
serial port on every repaint, which is how you can tell a frame reached pixels
rather than stopping in layout.

What is missing is the last step — moving those bytes onto glass — and it is
marked rather than faked.

## Where the panel driver goes

[`src/panel.rs`](../src/panel.rs) owns a one-bit framebuffer of the board's
exact size and hands off at one function, [`Panel::present`](reference.md#panelpresent).
**That is where a panel driver goes.** Every byte of the frame is in
[`Panel::bytes`](reference.md#panelbytes).

No published [Rust](https://rust-lang.org/) or C++ driver exists for these panels. The [RP2040](https://www.raspberrypi.com/products/rp2040/) boards
could name [`uc8151`](https://crates.io/crates/uc8151) and [`mipidsi`](https://crates.io/crates/mipidsi) because those are published crates; the [X3](https://www.xteink.com/products/xteink-x3)
and the [Sticky](https://www.seeedstudio.com/reTerminal-Sticky-p-6861.html) have no equivalent. The nearest working code is a hand-written
[SSD1677](https://www.solomon-systech.com/product/ssd1677) driver in an unrelated project — some seven hundred lines,
strip-streamed, tested against one specific 800×480 panel, which is exactly
the Sticky's framebuffer geometry. The X3's is 792×528. Vendoring it into a repository
that cannot power a panel to test it would add something nobody here could
verify.

**The gap is not meant to be closed here.** These panels already have firmware
that drives them, so a screen reaches an X3 or a Sticky by that firmware
hosting `xpui` over the C ABI — see
[`xpui-cpp`](https://github.com/XPUI-Framework/xpui-cpp) — rather than by this
repository growing a second driver for the same glass. If you are building
something else on this framework and do want one: both drive their glass over
SPI, strip-streamed rather than double-buffered, with a command sequence, a
waveform table and a wait on a BUSY line. The Sticky's is an SSD1677-class
controller; **the X3's this repository has not established**, and a guess in
place of a blank is worse than nothing to a driver author. `Panel::present`
is where it goes. Or replace `Panel` with a driver crate that
is already a `DrawTarget`, the better answer when one exists: then `Backend`
holds the driver and `panel.rs` goes away.

## Memory

| | |
|---|---|
| Heap | 64 kB, in [`src/runtime.rs`](../src/runtime.rs), from [`esp-alloc`](https://crates.io/crates/esp-alloc), handed over by [`init_heap`](reference.md#init_heap) |
| The X3's framebuffer | 52,272 bytes, **inside the heap**: `Panel` is leaked with the backend |
| The Sticky's | 48,000 bytes, likewise |
| SRAM | 400 KB on the [ESP32-C3](https://www.espressif.com/en/products/socs/esp32-c3), 512 KB on the [ESP32-S3](https://www.espressif.com/en/products/socs/esp32-s3) |

On the X3 the framebuffer is four fifths of the heap, leaving about 13 kB for
the screen stack and the view tree `body()` rebuilds on every paint and every
frame carrying input; on the Sticky it is under three quarters, leaving about
17 kB. Raise `HEAP_SIZE` before adding a screen that buffers anything; the chip
has room.

## The two chips

| | ESP32-C3 (`x3`) | ESP32-S3 (`sticky`) |
|---|---|---|
| Architecture | RISC-V, `riscv32imc-unknown-none-elf` | Xtensa, `xtensa-esp32s3-none-elf` |
| Toolchain | stable Rust, the pinned channel | the `esp` fork, from [`espup`](https://crates.io/crates/espup); `core` built from source |
| `-C force-frame-pointers` | on | **off** — with it the fork's [LLVM](https://llvm.org/) fails to build `compiler_builtins`; `.cargo/config.toml` says why |

Neither has an atomic compare-and-swap in the instruction set the framework is
built for: load and store only.

**The Xtensa peripheral access crate is pinned to an exact version.** [`esp-hal`](https://crates.io/crates/esp-hal)
1.1.2 asks for `esp32s3 ^0.35`, and 0.35.3 renamed two `RTC_CNTL` fields
`esp-hal` calls, so the build stops compiling inside somebody else's crate.
`Cargo.toml` pins 0.35.2 under a renamed optional dependency — a bare
`Cargo.lock` entry is undone by the next `cargo update` — and says when to
drop the pin.

## What the build taught us

Three things that cost a build each:

- [`esp-backtrace`](https://crates.io/crates/esp-backtrace) fails its own build script unless exactly one of [`defmt`](https://defmt.ferrous-systems.com/) or
  `println` is enabled, and says so.
- The link fails without `esp-hal`'s `linkall.x`, on undefined interrupt
  symbols the vector table names. It is asked for in `build.rs` rather than
  `.cargo/config.toml`, so a build from the workspace root gets it too.
- **`-C force-frame-pointers` breaks the Xtensa build**, in LLVM, while
  compiling `compiler_builtins`. The C3 keeps it.
