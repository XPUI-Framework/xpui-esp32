//! The gallery on a Seeed Sticky.
//!
//! ESP32-S3, **Xtensa**, and the X4's framebuffer in a smaller body: 480x800
//! at 234 ppi against the X4's 218, so every pixel here is 7% smaller. A touch
//! device, so it takes the larger chrome its firmware profile gives it — a
//! list row is 5.2 mm.
//!
//! ```bash
//! cargo +esp build --release --bin sticky --features sticky \
//!   --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
//! ```
//!
//! **Stable Rust cannot target Xtensa at all.** It needs the `esp` toolchain
//! fork, installed with `espup`, and `core` built from source because none is
//! shipped.
//!
//! **CI does not build it, and does not mention it**: installing the fork on
//! every pull request would cost minutes for one board. On a laptop,
//! `./build-and-test.sh all` looks for the fork and skips loudly when it is
//! missing.

#![cfg_attr(device, no_std)]
#![cfg_attr(device, no_main)]

#[cfg(device)]
use {
    esp_hal::main,
    gallery::Menu,
    xpui_boards_seeed as seeed,
    xpui_esp32::{Panel, init_heap, run},
};

#[cfg(device)]
use esp_backtrace as _;

#[cfg(device)]
esp_bootloader_esp_idf::esp_app_desc!();

/// One bit per pixel, sized **from the board** rather than restated here.
///
/// `seeed::STICKY` is a `const`, so this is computed at compile time — and a
/// panel size corrected in `xpui-boards` cannot leave a firmware with a
/// framebuffer one row short, which would draw fine and corrupt whatever
/// follows it.
#[cfg(device)]
const FRAMEBUFFER_BYTES: usize =
    (seeed::STICKY.width as usize).div_ceil(8) * seeed::STICKY.height as usize;

#[cfg(device)]
#[main]
fn start() -> ! {
    init_heap();

    let _peripherals = esp_hal::init(esp_hal::Config::default());

    let panel = Panel::<FRAMEBUFFER_BYTES>::new(seeed::STICKY.width, seeed::STICKY.height);
    run(panel, seeed::STICKY, Menu::new())
}

#[cfg(not(device))]
fn main() {}
