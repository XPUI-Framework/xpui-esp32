//! The two firmware images, under `all` only: whether each links.
//!
//! The RISC-V one is a plain `cargo build`. The Xtensa one needs the `esp`
//! fork and its linker; `s3_links` says what it does about them.

use crate::cargo;

/// The `esp` fork's toolchain name.
const ESP_TOOLCHAIN: &str = "esp";
/// The Xtensa board's triple.
const ESP_TARGET: &str = "xtensa-esp32s3-none-elf";

/// The RISC-V image links.
pub fn c3_links() -> Result<String, String> {
    cargo::cargo(&[
        "build",
        "--release",
        "--bin",
        "x3",
        "--features",
        "x3",
        "--target",
        "riscv32imc-unknown-none-elf",
    ])?;
    Ok("x3".into())
}

/// The Xtensa image links, if this machine can build Xtensa at all.
///
/// The `esp` fork is a whole second toolchain and its linker is a GCC that
/// ships beside it, so both are looked for and a missing one skips rather than
/// fails: nobody can fix it from inside this repository.
pub fn s3_links() -> Result<String, String> {
    let out = std::process::Command::new("rustup")
        .args(["toolchain", "list"])
        .output()
        .map_err(|e| format!("rustup: {e}"))?;
    if !String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l.starts_with(ESP_TOOLCHAIN))
    {
        return Ok(format!(
            "skipped: the '{ESP_TOOLCHAIN}' toolchain is not installed.\n\
             cargo install espup && espup install"
        ));
    }
    let home = std::env::var("RUSTUP_HOME")
        .unwrap_or_else(|_| format!("{}/.rustup", std::env::var("HOME").unwrap_or_default()));
    let Some(linker) = find(
        std::path::Path::new(&home)
            .join("toolchains")
            .join(ESP_TOOLCHAIN),
        "xtensa-esp32s3-elf-gcc",
    ) else {
        return Ok(format!(
            "skipped: the '{ESP_TOOLCHAIN}' toolchain is installed but its linker is not.\n\
             Re-run 'espup install', or source ~/export-esp.sh."
        ));
    };
    let path = format!(
        "{}:{}",
        linker.parent().expect("a directory").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    // Through `rustup run`, not `cargo +esp`: `env!("CARGO")` is the cargo
    // binary itself, and a `+toolchain` directive is rustup's shim's to read.
    let status = std::process::Command::new("rustup")
        .env("PATH", path)
        .args([
            "run",
            ESP_TOOLCHAIN,
            "cargo",
            "build",
            "--release",
            "--bin",
            "sticky",
            "--features",
            "sticky",
            "--target",
            ESP_TARGET,
        ])
        .status()
        .map_err(|e| format!("cargo: {e}"))?;
    if status.success() {
        Ok("sticky".into())
    } else {
        Err("the ESP32-S3 image does not link".into())
    }
}

/// The first file named `name` anywhere under `root`.
fn find(root: std::path::PathBuf, name: &str) -> Option<std::path::PathBuf> {
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if entry.file_name() == name {
                return Some(path);
            }
        }
    }
    None
}
