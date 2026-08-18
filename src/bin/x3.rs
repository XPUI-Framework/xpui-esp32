//! The gallery on an Xteink X3.
//!
//! ESP32-C3, RISC-V, and the densest panel this framework describes: 528x792
//! at 257 ppi, held portrait. A button board with no touchscreen, so it keeps
//! the baseline chrome — its 40px list row is 3.9 mm, the smallest of the five
//! Xteink profiles, and deliberately so.
//!
//! ```bash
//! cargo build --release --bin x3 --features x3 --target riscv32imc-unknown-none-elf
//! ```
//!
//! Stable Rust targets this chip, so it needs no toolchain of its own — which
//! is why CI builds this one and only announces the Sticky.

#![cfg_attr(device, no_std)]
#![cfg_attr(device, no_main)]

#[cfg(device)]
use {
    esp_hal::main,
    gallery::Menu,
    xpui_boards::Board,
    xpui_esp32::{Panel, init_heap, run},
};

#[cfg(device)]
use esp_backtrace as _;

// The application descriptor the second-stage bootloader looks for. Without
// it the image is rejected before a line of this runs.
#[cfg(device)]
esp_bootloader_esp_idf::esp_app_desc!();

/// One bit per pixel, sized **from the board** rather than restated here.
///
/// `Board::X3` is a `const`, so this is computed at compile time — and a
/// panel size corrected in `crates/boards` cannot leave a firmware with a
/// framebuffer one row short, which would draw fine and corrupt whatever
/// follows it.
#[cfg(device)]
const FRAMEBUFFER_BYTES: usize = (Board::X3.width as usize).div_ceil(8) * Board::X3.height as usize;

#[cfg(device)]
#[main]
fn start() -> ! {
    // First, because `App::new` allocates on its first line.
    init_heap();

    let _peripherals = esp_hal::init(esp_hal::Config::default());

    let panel = Panel::<FRAMEBUFFER_BYTES>::new(Board::X3.width, Board::X3.height);
    run(panel, Board::X3, Menu::new())
}

// Off the device this file is empty, and `main` still has to exist for the
// host build the workspace gates run.
#[cfg(not(device))]
fn main() {}
