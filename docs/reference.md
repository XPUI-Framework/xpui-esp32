# Reference

Every public item in `xpui-esp32`: the frame loop, the one-bit framebuffer it
paints into, and the two calls a bare-metal board needs around them.
[The tutorial](tutorial.md) puts a first screen on an [X3](https://www.xteink.com/products/xteink-x3) with them;
[hardware.md](hardware.md) is what is known about the panels and the memory.

Every item sits behind the `device` cfg, which `build.rs` sets for the board
only, so on a laptop the crate is empty. An example that calls one of these is
fenced `text` and says why; the one example a laptop can check is a doctest in
`docs-test/`.

## Topics

### Running a screen

| | |
|---|---|
| [`run`](#run) | Installs `panel` as the host, runs `root` on it, and reports what it drew. |
| [`Panel`](#panel) | The panel's memory, one bit per pixel. |

### The runtime

| | |
|---|---|
| [`init_heap`](#init_heap) | Hands the allocator its memory. |
| [`runtime::park`](#runtimepark) | Stops, keeping whatever the panel last showed. |

## `run`

Installs `panel` as the host, runs `root` on it, and reports what it drew.

```text
pub fn run<const BYTES: usize, S>(panel: Panel<BYTES>, board: Board, root: S) -> !
where
    S: Screen + 'static,
```

| Parameter | Meaning |
|---|---|
| `panel` | The framebuffer, sized for `board` by the binary. It is wired into a backend with `gallery::wire` and leaked, so on the X3 four fifths of the heap is this. |
| `board` | What the case is: `xteink::X3`. The chrome is sized from it and the boot line names it. Nothing checks that it agrees with `panel`. |
| `root` | The first screen, kept for the life of the loop: Back never finishes it. |

The palette is fixed at `Palette::INK_IS_ON`, because [`Panel`](#panel) treats a
set bit as ink. The loop paints once before it starts, since an e-ink panel
holds the previous firmware's last frame through a reset. Then, every 10 ms, it
starts a frame on the chip's clock, ticks the app, and paints only if the
framework asked. After every paint it calls [`Panel::present`](#panelpresent),
and prints one line over the serial port:

```text
xpui: <board name> 528x792, <n> ink pixels
```

With no panel driver, that count is the only evidence a frame reached pixels
rather than stopping in layout. A count that never changes is a frozen loop.

> [!NOTE]
> **Nothing polls a key.** The X3's keys are not wired in this firmware, so the
> root screen is painted and nothing moves it. Wiring them means sampling each
> pin and pushing its edges into the backend, which `xpui-rp2040`'s
> `ButtonPins` is the worked example of.

It never returns. If the root ever stopped being kept, the loop would end in
[`runtime::park`](#runtimepark).

**Example — the X3's entry point**

```text
const FRAMEBUFFER_BYTES: usize =
    (xteink::X3.width as usize).div_ceil(8) * xteink::X3.height as usize;

#[main]
fn start() -> ! {
    init_heap();
    let _peripherals = esp_hal::init(esp_hal::Config::default());

    let panel = Panel::<FRAMEBUFFER_BYTES>::new(xteink::X3.width, xteink::X3.height);
    run(panel, xteink::X3, Menu::new())
}
```

Fenced `text`: an [`esp-hal`](https://crates.io/crates/esp-hal) entry point, which compiles for the [ESP32-C3](https://www.espressif.com/en/products/socs/esp32-c3) and
nowhere else. [`src/bin/x3.rs`](../src/bin/x3.rs) is all of it.

**See also:** [`Panel`](#panel), [`init_heap`](#init_heap)

## `Panel`

The panel's memory, one bit per pixel.

```text
pub struct Panel<const BYTES: usize>
```

Rows run top to bottom, `(width + 7) / 8` bytes each, most significant bit
first. **A set bit is ink** and a new panel is all paper, which is why
[`run`](#run) paints with `Palette::INK_IS_ON`. It implements `DrawTarget` with
`BinaryColor`; a pixel outside the panel is dropped rather than clamped, so an
out-of-bounds bug stays invisible instead of drawing a line along the edge.

`BYTES` is a const generic so that the binary works the size out from its board
at compile time, and [`Panel::new`](#panelnew) checks the two agree at boot.

| Board | Canvas | `BYTES` |
|---|---|---|
| [Xteink](https://www.xteink.com/) X3 | 528 × 792 | 52,272 |
| [Seeed Sticky](https://www.seeedstudio.com/reTerminal-Sticky-p-6861.html) | 800 × 480 | 48,000 |

> [!WARNING]
> **There is no panel driver.** [`Panel::present`](#panelpresent) does nothing,
> so both images build, link and run, and nothing reaches glass.
> [Where the panel driver goes](hardware.md#where-the-panel-driver-goes) says
> why, and what a driver has to do.

**Example — sizing the framebuffer from the board**

```rust
use xpui_boards_xteink as xteink;

// Computed from the board at compile time, so a panel size corrected in
// `xpui-boards` cannot leave a firmware with a framebuffer one row short.
const FRAMEBUFFER_BYTES: usize =
    (xteink::X3.width as usize).div_ceil(8) * xteink::X3.height as usize;

assert_eq!(FRAMEBUFFER_BYTES, 52_272);
```

### Creating a panel

#### `Panel::new`

A blank panel of `width` by `height`.

```text
pub fn new(width: i32, height: i32) -> Self
```

| Parameter | Meaning |
|---|---|
| `width`, `height` | The canvas in pixels, as the board describes it: portrait on the X3. |

Panics if `BYTES` is not exactly `(width + 7) / 8 * height`. A framebuffer one
row short draws fine and corrupts whatever follows it in memory, so it is
refused at boot.

### Handing a frame to a driver

#### `Panel::bytes`

The bytes a driver would push, for whoever writes one.

```text
pub fn bytes(&self) -> &[u8]
```

They are in the canvas's orientation. The X3's panel scans landscape and is
held portrait, so its driver pushes the canvas turned a quarter:
`xteink::X3.framebuffer` is `(792, 528)`, and a driver needs that number.

#### `Panel::present`

**Where the panel driver goes.**

```text
pub fn present(&mut self)
```

[`run`](#run) calls it after every paint, before counting the ink, and it does
nothing. Both boards drive their glass over SPI: command and data writes, a
waveform table, and a wait on a BUSY line. The Sticky's controller is
[SSD1677](https://www.solomon-systech.com/product/ssd1677)-class; the X3's is not established.

A firmware fills this in, or replaces `Panel` with a driver crate that is
already a `DrawTarget`. Then the backend holds the driver, and `Panel` goes
away.

### Reading it back

#### `Panel::ink_count`

How much ink is on the panel, reported over the serial port on every repaint.

```text
pub fn ink_count(&self) -> usize
```

The number of set bits, a row at a time. When the width is not a multiple of
eight, the padding bits at the end of each row are masked off, so they are
never counted as ink.

## `init_heap`

Hands the allocator its memory.

```text
pub fn init_heap()
```

64 kB of the chip's SRAM, through [`esp-alloc`](https://crates.io/crates/esp-alloc). The framebuffer is inside it,
because [`Panel`](#panel) is leaked with the backend: 52,272 bytes on the X3,
leaving about 13 kB for the screen stack and the view tree `body()` rebuilds.
Raise `HEAP_SIZE` in [`src/runtime.rs`](../src/runtime.rs) before a screen
buffers anything; [the memory table](hardware.md#memory) has both boards.

Call it once, as the first line of the entry point: `App::new` allocates on
its first line.

The panic handler is not this crate's. It comes from [`esp-backtrace`](https://crates.io/crates/esp-backtrace), which
each binary links with `use esp_backtrace as _;`.

## `runtime::park`

Stops, keeping whatever the panel last showed.

```text
pub fn park() -> !
```

It spins, forever, without sleeping the core. E-ink holds its image with the
power off, so leaving the last frame up is the most useful thing a firmware
with nowhere to go can do; a firmware that cares about the battery sleeps the
core here instead.

It is the one item not re-exported at the root: `xpui_esp32::runtime::park`.
[`run`](#run) ends in it if its root screen ever finishes, which cannot happen
while the root is kept.
