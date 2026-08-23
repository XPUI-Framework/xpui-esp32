//! The gate for `xpui-esp32`.
//!
//! Everything CI checks, in one command, and **only what this repository has
//! to check**. It is firmware for two different
//! architectures, so the lint names a board rather than a workspace, and only
//! the RISC-V half is checked here — the Xtensa half needs the `esp` fork.
//!
//! ```text
//! ./build-and-test.sh          check everything
//! ./build-and-test.sh fix      format in place first
//! ./build-and-test.sh all      the above, plus linking both firmware images
//! ```
//!
//! Each repository in the organisation has its own copy of this shape, holding
//! its own list. **This file is the part that is meant to differ**; the modules
//! under it are byte-identical, and `shared_files_agree` in `xpui-dev` hashes
//! all seven across the nine, so a fix to the fence scanner cannot land in one
//! repository and not the rest.
//!
//! A check written and never listed below is a dead function, which clippy
//! fails the build over. That is what a hand-written "is every check
//! dispatched?" check used to do, and it does it better.

mod cargo;
mod commands;
mod docs;
mod faults;
mod fences;
mod paths;
mod prose;
mod tree;

use std::process::ExitCode;

/// Files under a `src/` may not exceed this. A ratchet, not a law of nature:
/// raising it is a decision to argue for in a commit message, never a way to
/// land a file.
const LINE_LIMIT: usize = 400;

/// Crates with no tests, and why. The reason prints on every run so it is
/// re-read rather than accumulated — and an exemption for a crate that has
/// since grown tests fails, rather than sitting there as a comment nobody
/// removes.
const UNTESTED: [(&str, &str); 1] = [(
    ".",
    "the firmware; test = false, and esp-hal cannot compile for a laptop",
)];

/// Fence languages this repository's prose is written in.
///
/// The list exists so that ` ```rustt ` is an error rather than a shrug: an
/// unknown language silently compiles nothing, and a typo is the likeliest
/// way for a Rust block to stop being checked.
const KNOWN_LANGUAGES: [&str; 19] = [
    "text", "bash", "sh", "shell", "console", "cpp", "c", "toml", "yaml", "yml", "json", "ini",
    "diff", "ascii", "mermaid", "markdown", "md", "python", "cmake",
];

/// Documents whose ```rust is illustrative rather than compilable.
const NOT_COMPILED: [&str; 0] = [];

/// Pages that are not a repository's front door and carry no banner.
const NOT_A_FRONT_PAGE: [&str; 0] = [];

/// The RISC-V board, and it is required. The Xtensa one needs the `esp` fork
/// of the toolchain, which no laptop has by default and which CI installs
/// separately, so it is not a gate here.
const BARE_METAL: [(&str, bool); 1] = [("riscv32imc-unknown-none-elf", true)];

/// The `esp` fork's toolchain name and the Xtensa board's triple. Named here
/// because `all` is the only mode that uses them, and only if they exist.
const ESP_TOOLCHAIN: &str = "esp";
const ESP_TARGET: &str = "xtensa-esp32s3-none-elf";

/// One board's binary and its feature, because the two boards are two
/// architectures and there is no single default to pick.
const LINT_CRATES: [&str; 4] = ["--bin", "x3", "--features", "x3"];

fn main() -> ExitCode {
    // Every path in every check is relative to the repository root, so the
    // gate answers the same from anywhere it is invoked.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/..");
    std::env::set_current_dir(root).expect("the repository root");

    // A typo is not a check. The shell this replaced rejected an unknown
    // argument, and a gate that silently treats `fx` as `check` is a gate that
    // reports a pass for a run nobody asked for.
    let (fix, everything) = match std::env::args().nth(1).as_deref() {
        None | Some("check") => (false, false),
        Some("fix") => (true, false),
        Some("all") => (false, true),
        Some(other) => {
            eprintln!("unknown argument `{other}`\nusage: ./build-and-test.sh [check|fix|all]");
            return ExitCode::from(2);
        }
    };
    let mut failed = 0;

    let mut gate: Vec<(&str, Box<dyn Fn() -> Result<String, String>>)> = vec![
        (
            "format",
            Box::new(move || {
                if fix {
                    cargo::cargo(&["fmt", "--all"])
                } else {
                    cargo::cargo(&["fmt", "--all", "--check"])
                }
            }),
        ),
        ("file sizes", Box::new(|| tree::file_sizes(LINE_LIMIT))),
        (
            "crates are tested",
            Box::new(|| tree::crates_are_tested(&UNTESTED)),
        ),
        (
            "READMEs warn",
            Box::new(|| tree::readmes_warn(&NOT_A_FRONT_PAGE)),
        ),
        (
            "prose is compiled",
            Box::new(|| prose::is_compiled(&NOT_COMPILED, &KNOWN_LANGUAGES)),
        ),
        ("documented paths resolve", Box::new(docs::doc_paths)),
        (
            "rustdoc links resolve",
            Box::new(|| cargo::rustdoc(&["--workspace"])),
        ),
        (
            "documented commands resolve",
            Box::new(|| commands::resolve(&cargo::packages(), &[])),
        ),
        ("lint", Box::new(lint)),
        ("tests", Box::new(|| cargo::cargo(&["test", "--workspace"]))),
        (
            "doctests",
            Box::new(|| cargo::cargo(&["test", "--workspace", "--doc"])),
        ),
    ];

    // `all` is what a laptop runs before a board is flashed, and what CI runs
    // in a job provisioned for it. It is separate from `check` because these
    // stages need a toolchain or a build system that a quick run should not
    // demand.
    if everything {
        gate.extend::<Vec<(&str, Box<dyn Fn() -> Result<String, String>>)>>(vec![
            ("the ESP32-C3 image links", Box::new(c3_links)),
            ("the ESP32-S3 image links", Box::new(s3_links)),
        ]);
    }

    for (name, check) in gate.drain(..) {
        println!("\n==> {name}");
        match check() {
            Ok(note) if note.is_empty() => println!("    ok"),
            Ok(note) => println!("    {}", note.replace('\n', "\n    ")),
            Err(why) => {
                println!("{why}");
                eprintln!("FAILED: {name}");
                failed += 1;
            }
        }
    }

    if failed == 0 {
        if everything {
            println!("\nEverything passed.");
        } else {
            println!("\nChecks passed. `./build-and-test.sh all` also links both images.");
        }
        ExitCode::SUCCESS
    } else {
        eprintln!("\n{failed} check(s) failed.");
        ExitCode::FAILURE
    }
}

/// Clippy on the host, and for the RISC-V board.
///
/// The host run reaches the crate's own host-compilable half; the board run is
/// the only gate that reads what sits behind `cfg(target_os = "none")`.
fn lint() -> Result<String, String> {
    cargo::cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ])?;
    let mut notes = vec!["host".to_string()];
    bare_metal(&mut notes)?;
    Ok(notes.join(", "))
}

/// Clippy on each bare-metal target, with warnings as errors.
///
/// The host build never parses code behind `cfg(target_os = "none")` — no
/// allocator, no panic handler — so these are the only gates that reach it
/// before a firmware build does. Neither target has atomic compare-and-swap:
/// load and store only, never `swap`, `fetch_or` or `compare_exchange`. The
/// second is a second architecture rather than a stricter one.
fn bare_metal(notes: &mut Vec<String>) -> Result<(), String> {
    {
        for (triple, required) in BARE_METAL {
            {
                if !cargo::target_installed(triple) {
                    {
                        if required {
                            {
                                return Err(format!(
                                    "{triple} is not installed, and it is the only gate that reaches\n\
                     this repository's no_std paths. `rustup target add {triple}`"
                                ));
                            }
                        }
                        notes.push(format!("{triple} SKIPPED — rustup target add {triple}"));
                        continue;
                    }
                }
                let mut arguments = vec!["clippy", "--release"];
                arguments.extend_from_slice(&LINT_CRATES);
                arguments.extend_from_slice(&["--target", triple, "--", "-D", "warnings"]);
                cargo::cargo(&arguments)?;
                notes.push(triple.to_string());
            }
        }
        Ok(())
    }
}

/// The RISC-V image links.
fn c3_links() -> Result<String, String> {
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
fn s3_links() -> Result<String, String> {
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
