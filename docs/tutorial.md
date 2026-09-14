# Your first screen on an ESP32

You have an [Xteink X3](https://www.xteink.com/products/xteink-x3) — 528×792 of e-ink at 257 ppi, four keys along the
bottom and a page key on each edge, an [ESP32-C3](https://www.espressif.com/en/products/socs/esp32-c3), no touchscreen. This puts a
screen on it, as far as a screen can go without a panel driver.

It assumes you have written one for the simulator already;
[the framework's tutorial](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/tutorial.md) is that, and
nothing here repeats it. It follows [`xpui-rp2040`'s tutorial](https://github.com/XPUI-Framework/xpui-rp2040/blob/main/docs/tutorial.md)
step for step, and says where an ESP32 differs from an [RP2040](https://www.raspberrypi.com/products/rp2040/). **What is
different on this board is the subject.**

Every [Rust](https://rust-lang.org/) block below is compiled by `cargo test` in `docs-test/`. The two
device-only ones — the frame loop, which needs a HAL's clock, and the flash
commands — are fenced `text`, and each says why it cannot be compiled here. The C3 is the board a reader can
follow on stable Rust; the [Sticky](https://www.seeedstudio.com/reTerminal-Sticky-p-6861.html), an [ESP32-S3](https://www.espressif.com/en/products/socs/esp32-s3), needs the `esp` fork and is
noted where it differs.

## Nothing about the screen changes

That is the whole claim, so it goes first. This is a complete screen, and the
same source runs in the simulator window, on a [Badger](https://shop.pimoroni.com/products/badger-2040), inside a C++ firmware,
and — once one of these panels has a driver — on an X3:

```rust
use xpui::screen::Screen;
use xpui::{List, ListRow, NavigationScreen, View};

pub struct Battery {
    percent: i32,
}

impl Battery {
    pub fn new() -> Self {
        Battery { percent: 72 }
    }
}

impl Screen for Battery {
    type Message = ();

    fn body(&self) -> impl View<()> {
        NavigationScreen::new(
            List::new().push(ListRow::new("Charge").value(if self.percent > 20 {
                "Good"
            } else {
                "Low"
            })),
        )
        .title("Battery")
    }

    fn update(&mut self, _message: ()) {}
}
```

No pixel offsets, no panel size, no board. If yours has any of those in it, the
rest of this will not help — take them out first.

## 1. The board is data

A `Board` is what the panel and the case *are*: size, orientation, whether it
has a touchscreen, how long a refresh takes, what the keys along the bottom
mean, and how big the glass is in millimetres. Both the simulator and the
firmware read the same value, which is what makes "develop in a window, then
flash it" true rather than aspirational.

**It is not the chrome.** Nothing below your firmware knows what a board is —
not the backend, not the components that paint. Joining the two is your job,
and it is one derivation and five builder calls.

```rust
use xpui_boards_xteink as xteink;

let x3 = xteink::X3;

assert_eq!((x3.width, x3.height), (528, 792));
assert!(!x3.touch);
// The panel scans landscape and is held portrait: the framebuffer a driver
// pushes is the canvas turned a quarter, and a driver needs this number.
assert_eq!(x3.framebuffer, (792, 528));
```

Here is the wiring. Each line answers one question: three of them with a value
the board holds, and two with the measurements the first line derived:

```rust
use xpui::host::Canvas;
use xpui_boards_xteink as xteink;
use xpui_eg::{Backend, Fonts, Labels, Metrics, Palette};
use xpui_screenshot::Framebuffer;

let x3 = xteink::X3;

// How big everything is. Derived from the panel's size and the board's UI
// scale; the last argument is whether to reserve the band along the bottom
// that names the keys. `!x3.touch` is `true` here — the X3 has four keys
// under the panel, and the band is where their words go. `gallery::wire`
// derives it the same way.
let metrics = Metrics::for_device(x3.width, x3.height, x3.ui_scale_percent, !x3.touch);

let backend = Backend::new(
    Framebuffer::new(x3.width, x3.height),
    // This board's polarity: a set bit in the firmware's own framebuffer is
    // ink. Not every panel agrees — step 2.
    Palette::INK_IS_ON,
)
.with_metrics(metrics)
// What the hint band says. A reader-sized panel gets the long words.
.with_labels(Labels::for_panel(x3.width, x3.height))
// What the keys along the bottom mean, in order: Back, Confirm, and the pair
// that walks a list. A hardware fact, so it comes off the board rather than
// out of a preset.
.with_keys(x3.keys)
// Whether Left and Right exist as keys, which decides how a value can be
// edited. On the X3 they are the third and fourth footer keys, labelled Up
// and Down; the edge keys send PageBack and PageForward, which this does not
// count.
.with_left_right_keys(x3.has_left_right_keys())
// Faces chosen to fit the measurements.
.with_fonts(Fonts::for_metrics(&metrics));

// The size is the display's, whatever the board says — `Backend::new` measures
// what it is handed.
assert_eq!(backend.screen_size().width, 528);
```

The gallery packages exactly that as `gallery::wire`, and both binaries here
call it rather than repeating the block. Yours can too; write it out once if
you would rather see it.

> `Framebuffer` is the host-side target these snippets draw into so they can be
> tested. On the board it is [`src/panel.rs`](../src/panel.rs)'s `Panel` — a
> one-bit framebuffer of the board's exact size, with the driver seam at the
> end of it. Step 3.

## 2. The trap that inverts your panel

**`Palette` decides which colour is ink, and getting it wrong is invisible
until the board is in your hand.**

```rust
use xpui_eg::Palette;

// This firmware's framebuffer: a set bit is ink, so ink is "on".
let x3 = Palette::INK_IS_ON;
// A driver that maps `BinaryColor::Off` to black wants the other one.
let inverted = Palette::INK_IS_OFF;

assert_ne!(x3.ink, inverted.ink);
```

Both compile. Both run. The wrong one gives you a fully inverted panel — white
text on black, every screen — with no error anywhere, because nothing in the
framework knows which way your driver wired it.

`Panel` here initialises to zero and treats a set bit as ink, so the firmware
uses `INK_IS_ON`, which is why step 1 did. **When you write the driver, this is
the first thing to check against the glass**: the panel's controller has its
own idea of which bit is black, and if the first frame comes up inverted the
palette is the one-word fix, not the driver.

## 3. The frame loop, and where the driver goes

On a device you own the loop. It is short, and every line of it is there for a
reason that costs you if you drop it:

```text
app.render();                            // paint once, before the loop
backend.clear_dirty();
present(backend, board);

while app.is_running() {
    backend.begin_frame(now_millis());   // advance the clock, clear input edges

    app.tick();                          // one frame of input
    if app.render_if_dirty() {           // paint ONLY if something changed
        backend.clear_dirty();
        present(backend, board);         // with_display: present, then count
    }

    delay.delay_millis(FRAME_INTERVAL_MS);   // 10 ms
}
```

Fenced `text` because it needs [`esp-hal`](https://crates.io/crates/esp-hal)'s clock and delay —
[`src/frame.rs`](../src/frame.rs) is the compiled version, and it is this with
the types filled in. Four things about it:

**It paints once before looping.** An e-ink panel holds whatever survived
reset, which is the last firmware's frame rather than a blank screen, so the
first frame cannot wait for something to change.

**There is no input yet.** Unlike the RP2040 loop, nothing polls a GPIO: the
X3's keys are not wired in this firmware, so the loop ticks with no presses
and paints the root screen. Wiring them means polling each pin and pushing an
edge into the backend, which the RP2040 firmware's `buttons.rs` is the worked
example of.

**`Panel::present` is empty, on purpose.** No published Rust driver exists for
these panels, and vendoring a hand-written one into a repository that cannot
power a panel to test it would add something nobody here could verify. Every
byte of the frame is in `Panel::bytes()`, waiting.
[hardware.md](hardware.md) says what is known about the two controllers and
why the gap is meant to stay open.

**The wait is not optional.** A loop with nothing in it spins the core at full
clock for the life of the battery. Ten milliseconds is what both firmwares in
the organisation use.

**`App::keep_root()` is why Back will not park the board.** It declines the
pop at the root screen; without it, Back on the first screen empties the
stack, ends the loop, and looks exactly like a crash. Wire the keys and you
meet this immediately.

## 4. What you have to fit in

The ESP32-C3 has 400 KB of SRAM, the S3 512 KB, and the framework does not
hide either.

The heap is 64 kB and the X3's framebuffer is 52,272 bytes of it, leaked with
the backend — so about 13 kB is left for the screen stack and everything
`body()` builds. [hardware.md](hardware.md#memory) has both boards' figures.
A screen that buffers an image has nowhere to put one until the heap is
raised. The rule that keeps you inside it: **`body()` runs
on every paint and on every frame carrying input.** Build `String`s when the
screen is built, not while describing it — a `format!` in `body()` allocates
several times a second and drags `core::fmt` onto a path that has to stay
cheap.

```rust
use alloc::string::String;
extern crate alloc;

pub struct Reading {
    // Formatted once, when the value changes.
    label: String,
}

impl Reading {
    pub fn set(&mut self, percent: i32) {
        self.label = alloc::format!("{percent}%");
    }

    // `body()` borrows it. No allocation, no formatting.
    pub fn label(&self) -> &str {
        &self.label
    }
}
```

## 5. Flash it

```text
# The X3, on stable Rust. `.cargo/config.toml` names `espflash` as the runner,
# so `cargo run` flashes and opens the serial monitor.
cargo install espflash
cargo run --release --bin x3 --features x3 --target riscv32imc-unknown-none-elf

# The Sticky is Xtensa, which stable Rust cannot target: install the `esp`
# fork with espup, source its environment so the linker is on PATH, then
# build through it. `core` is built from source.
cargo install espup && espup install
. ~/export-esp.sh
cargo +esp run --release --bin sticky --features sticky --target xtensa-esp32s3-none-elf
```

Fenced `text`: these move a binary onto hardware, which is not something a test
can do.

`--features` is not optional: the chip is a feature of `esp-hal` rather than a
target, and [the front page](../README.md) says what that costs. Name the
board and its target together, and never `--all-features`.

## What has been proven, and where

Both images build and link, under `./build-and-test.sh all` — which is yours
to run, because CI runs `check` and nothing there reaches a linker. **Neither
board has been run.** What is proven is everything above the last inch: the
board's geometry, the chrome sized for it, the frame loop, the allocator, the
panic handler, and the whole gallery laid out for the right panel — in the
simulator, against the same `Board` this firmware reads. What is not is the
inch itself: a frame reaching glass. The firmware reports its ink count over
the serial port on every repaint, which is the evidence a first run should
look for before a driver exists, and the first thing to check once one does.
