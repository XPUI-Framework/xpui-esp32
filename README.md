# The gallery, as ESP32 firmware

> ⚠️ **Under heavy development.** Not production-ready. The API can break
> without notice. Use at your own risk.

Two boards on two architectures, running the same screens the desktop
simulator opens and the RP2040 binaries flash.

| Binary | Board | Chip | Target |
|---|---|---|---|
| `x3` | Xteink X3 | ESP32-C3 | `riscv32imc-unknown-none-elf` |
| `sticky` | Seeed Sticky | ESP32-S3 | `xtensa-esp32s3-none-elf` |

```bash
cargo build --release --bin x3 --features x3 --target riscv32imc-unknown-none-elf

# The Sticky needs Espressif's compiler fork; see docs/devices.md.
cargo +esp build --release --bin sticky --features sticky --target xtensa-esp32s3-none-elf
```

`--features` is not optional: the chip is a feature of `esp-hal` rather than a
target, so two binaries in one crate cannot each choose their own.

## There is no panel driver

[`src/panel.rs`](src/panel.rs) owns the framebuffer and stops at one function,
`Panel::present`. That is where a driver goes, and why there is none is
[`docs/devices.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/devices.md#the-panel-drivers) — including what
you would need if you want to write one.

Everything above that line is real and is exercised by the build: the board's
geometry, the chrome sized from it, the frame loop, the allocator, the panic
handler, and the whole gallery laid out for the right panel. The firmware
reports its ink count over the serial port, which without a driver is the only
evidence a frame reached pixels rather than stopping in layout.

What is missing is the last step, and it is marked rather than faked.

## What the build taught us

Three things that cost a build each, all recorded in
[`docs/devices.md`](https://github.com/XPUI-Framework/xpui-framework/blob/main/docs/devices.md):

- `esp-backtrace` fails its own build script unless exactly one of `defmt` or
  `println` is enabled, and says so.
- The link fails without `esp-hal`'s `linkall.x`, on undefined interrupt
  symbols the vector table names. It is asked for in `build.rs` rather than
  `.cargo/config.toml`, so a build from the workspace root gets it too.
- **`-C force-frame-pointers` breaks the Xtensa build**, in LLVM, while
  compiling `compiler_builtins`. The C3 keeps it.
