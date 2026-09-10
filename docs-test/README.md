# `xpui-esp32-docs`

> ⚠️ **Under heavy development.** Not production-ready. The API can break
> without notice. Use at your own risk.

A crate with no code, whose only job is to compile this repository's tutorial.

## Using it

[`docs/tutorial.md`](../docs/tutorial.md) teaches somebody with an Xteink X3
in their hand, so it lives beside the firmware. But **rustdoc runs a snippet
on the host**, and nothing in the firmware crate builds for a laptop — every
item sits behind the `device` cfg an ESP32 HAL needs. So the crate that
compiles the prose cannot be the crate the prose is about. `src/lib.rs` is one
`#[doc = include_str!]` and nothing else, and every Rust fence in the tutorial
is a doctest here.

From the repository root, one directory up:

```bash
cargo test --manifest-path docs-test/Cargo.toml --doc
```

Its own workspace rather than a member of the firmware's, so its host-only
dependencies and its lock file stay out of the firmware's; and unlike
`xpui-rp2040`'s twin it needs no `--target`, because this repository's
`.cargo/config.toml` sets no default target — the two boards are two
architectures, and there is none to pick.

## Checking it

The gate is the repository's; `./build-and-test.sh` from the root runs this
crate's doctests as its `the prose compiles` stage.

## License

MIT — see [LICENSE](../LICENSE). Copyright (c) 2026 Thiago Holanda.
