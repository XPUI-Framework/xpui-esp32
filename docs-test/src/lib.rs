//! Nothing but a home for the tutorial's doctests.
//!
//! `../docs/tutorial.md`'s and `../docs/reference.md`'s snippets are compiled
//! and run from here, because
//! the firmware they teach cannot compile anything on a laptop: every item in
//! it is bare metal, behind the `device` cfg, and rustdoc runs snippets on the
//! host.
//!
//! A snippet that stops matching the API fails this crate's `cargo test`. That
//! is the whole of its job — it exports nothing and is never flashed.
//!
//! Its own workspace, so its host-only dependencies stay out of the
//! firmware's. No `--target`: the repository's `.cargo/config.toml` sets no
//! default one. Run it from the repository root:
//!
//! ```bash
//! cargo test --manifest-path docs-test/Cargo.toml --doc
//! ```

#[cfg(doctest)]
#[doc = include_str!("../../docs/tutorial.md")]
mod tutorial {}

#[cfg(doctest)]
#[doc = include_str!("../../docs/reference.md")]
mod reference {}
