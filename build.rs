//! Two jobs, both of which only mean anything on the hardware.
//!
//! The first is the `device` cfg that every item in this crate sits behind. It
//! is on for bare metal and nowhere else, so the workspace's host gates —
//! which build every member for the machine they run on — see an empty crate
//! rather than an ESP32 HAL that cannot compile for a laptop.
//!
//! The second is the linker script. `esp-hal` ships `linkall.x` and the link
//! fails without it, on undefined interrupt symbols like `WIFI_MAC` that the
//! vector table names. Asked for **here** rather than in `.cargo/config.toml`,
//! because cargo reads that file only when invoked from this directory or
//! below — and the gates, and the spec's own build command, run from the
//! workspace root.

use std::env;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    // Declared unconditionally, including on the host, or every `cfg(device)`
    // in the crate becomes an `unexpected_cfgs` warning — and warnings fail.
    println!("cargo::rustc-check-cfg=cfg(device)");

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if os != "none" || (arch != "riscv32" && arch != "xtensa") {
        return;
    }

    println!("cargo::rustc-cfg=device");
    println!("cargo::rustc-link-arg-bins=-Tlinkall.x");
}
