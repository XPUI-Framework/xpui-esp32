//! The xpui gallery as ESP32 firmware.
//!
//! Two boards, two architectures, one set of screens:
//!
//! | Binary | Board | Chip | Target |
//! |---|---|---|---|
//! | `x3` | Xteink X3 | ESP32-C3 | `riscv32imc-unknown-none-elf` |
//! | `sticky` | Seeed Sticky | ESP32-S3 | `xtensa-esp32s3-none-elf` |
//!
//! The screens come from `examples/gallery`, unchanged — the same library the
//! desktop simulator opens and the RP2040 binaries flash. What differs between
//! all four is a [`Board`](xpui_boards::Board) and an entry point.
//!
//! **There is no panel driver.** See [`panel`] for why, and for the one
//! function a firmware fills in.
//!
//! Everything is behind a `device` cfg that `build.rs` turns on for bare metal
//! only, so this is an empty crate on a laptop and the workspace's host gates
//! build it as one.
//!
//! # `--all-features` cannot work here
//!
//! It turns on both `x3` and `sticky`, which are two chips and two
//! architectures. There is deliberately **no `compile_error!` guarding that**,
//! because one could not fire: cargo builds dependencies first, so
//! `esp-metadata-generated` fails with "the name `chip` is defined multiple
//! times" before this crate is reached. A guard that cannot run is decoration.
//! Name one board.

#![cfg_attr(device, no_std)]

#[cfg(device)]
extern crate alloc;

#[cfg(device)]
mod frame;
#[cfg(device)]
mod panel;
#[cfg(device)]
pub mod runtime;

#[cfg(device)]
pub use frame::run;
#[cfg(device)]
pub use panel::Panel;
#[cfg(device)]
pub use runtime::init_heap;
