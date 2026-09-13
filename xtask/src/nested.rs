//! The firmware's own documentation, and the workspace beside it.
//!
//! Two gaps a plain `--workspace` run leaves open here. Every item under
//! `src/` sits behind the `device` cfg, so a host rustdoc parses an empty
//! crate and reads none of its doc comments; and `docs-test/` is a workspace
//! of its own, which `--workspace` from the root never reaches.

use crate::cargo;

/// Rustdoc, for the board and for the workspace beside the root.
///
/// The board run is the one this repository could not do any other way: a
/// doc comment behind `cfg(device)` is not parsed by a host rustdoc, and
/// `src/` is nothing but such code. It reaches the library and the `x3`
/// binary; `sticky` needs the Xtensa fork, so no run here reads it.
pub fn rustdoc_links() -> Result<String, String> {
    let mut notes = Vec::new();
    cargo::rustdoc(&["--workspace"])?;
    notes.push("the workspace on the host".to_string());

    let board = "riscv32imc-unknown-none-elf";
    if cargo::target_installed(board) {
        cargo::rustdoc(&["--release", "--features", "x3", "--target", board])?;
        notes.push(board.to_string());
    } else {
        notes.push(format!("{board} SKIPPED — rustup target add {board}"));
    }

    cargo::rustdoc(&["--manifest-path", "docs-test/Cargo.toml"])?;
    notes.push("docs-test on the host".into());
    Ok(notes.join(", "))
}

/// Clippy for `docs-test/`, which `--workspace` from the root cannot reach.
pub fn docs_test_lints() -> Result<String, String> {
    cargo::cargo(&[
        "clippy",
        "--manifest-path",
        "docs-test/Cargo.toml",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    Ok("docs-test on the host".into())
}

/// `docs-test/` lints under the same rules as the firmware.
///
/// Clippy reads the `clippy.toml` nearest the workspace root, and `docs-test/`
/// is its own workspace — so without a copy it would lint under cargo's
/// defaults rather than this organisation's `msrv`. `xpui-dev` compares the
/// root file across every repository and cannot see this one: its list is one
/// path, `clippy.toml`, and this is a directory down.
pub fn nested_clippy_agrees() -> Result<String, String> {
    let root = std::fs::read_to_string("clippy.toml").map_err(|e| format!("  clippy.toml: {e}"))?;
    let wanted: Vec<&str> = settings(&root);
    if wanted.is_empty() {
        return Err("clippy.toml sets nothing, so this would compare nothing".into());
    }

    let nested = "docs-test/clippy.toml";
    let text = std::fs::read_to_string(nested).map_err(|_| {
        format!("  {nested} is missing, so that workspace lints under cargo's defaults")
    })?;
    if settings(&text) == wanted {
        Ok(format!("{} setting(s), in {nested}", wanted.len()))
    } else {
        Err(format!(
            "  {nested} sets different values from clippy.toml\n\n\
             It needs the root file's values, or it lints under different rules\n\
             from the firmware one directory up."
        ))
    }
}

/// The lines of a `clippy.toml` that set something.
fn settings(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}
