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
//! The tutorial's doctests live in `docs-test/`, a workspace of its own,
//! because nothing in the firmware builds for a laptop; `images.rs` holds
//! the two link stages `all` adds.
//!
//! Each repository in the organisation has its own copy of this shape, holding
//! its own list. **This file is the part that is meant to differ**; the modules
//! under it are byte-identical, and `shared_files_agree` in `xpui-dev` hashes
//! all ten across the nine, so a fix to the fence scanner cannot land in one
//! repository and not the rest.
//!
//! A check written and never listed below is a dead function, which clippy
//! fails the build over.

mod agents;
mod cargo;
mod commands;
mod comments;
mod docs;
mod faults;
mod fences;
mod images;
mod nested;
mod paths;
mod prose;
mod readme;
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
const UNTESTED: [(&str, &str); 2] = [
    (
        ".",
        "the firmware; test = false, and esp-hal cannot compile for a laptop",
    ),
    (
        "docs-test",
        "a doctest mount; its only content is this repository's tutorial",
    ),
];

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

/// Pages that are not a repository's front door.
const NOT_A_FRONT_PAGE: [&str; 1] = ["docs/README.md"];

/// The root README's headings, in order.
const README_ORDER: &[&str] = &[
    "Which crate you want",
    "Using it",
    "Requirements",
    "Checking it",
    "Where it sits",
    "License",
];
const README_OPTIONAL: &[&str] = &["Which crate you want", "Requirements"];
const NESTED_ORDER: &[&str] = &[
    "Using it",
    "Requirements",
    "Checking it",
    "Where next",
    "License",
];
const NESTED_OPTIONAL: &[&str] = &["Requirements", "Where next"];

/// `AGENTS.md` exists and `CLAUDE.md` is a symlink to it.
const AGENTS_FILE: bool = true;

/// Every publishable crate denies `missing_docs`. `true` here says so for
/// none: nothing in this repository is published.
const DOCUMENTED: bool = true;

/// How long a comment may be. `None` is not adopted.
const COMMENT_CAPS: Option<comments::Caps> = Some(comments::Caps {
    doc: 15,
    header: 15,
    run: 10,
});
/// No comment is about the past.
const NARRATION_CHECKED: bool = true;
/// Which files the two comment checks read. `None` is every tracked source,
/// manifest and C++ file outside `tests/`.
const COMMENT_SCOPE: Option<&str> = None;

/// The RISC-V board, and it is required. The Xtensa one needs the `esp` fork
/// of the toolchain, which no laptop has by default and which CI installs
/// separately, so it is not a gate here.
const BARE_METAL: [(&str, bool); 1] = [("riscv32imc-unknown-none-elf", true)];

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

    // A typo is not a check: a gate that silently treats `fx` as `check`
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
                // Two workspaces: `docs-test/` is its own, so `--all` from
                // the root never reaches it.
                for manifest in ["Cargo.toml", "docs-test/Cargo.toml"] {
                    let mut arguments = vec!["fmt", "--manifest-path", manifest, "--all"];
                    if !fix {
                        arguments.push("--check");
                    }
                    cargo::cargo(&arguments)?;
                }
                Ok("the firmware and docs-test".into())
            }),
        ),
        ("file sizes", Box::new(|| tree::file_sizes(LINE_LIMIT))),
        (
            "crates are tested",
            Box::new(|| tree::crates_are_tested(&UNTESTED)),
        ),
        ("READMEs warn", Box::new(tree::readmes_warn)),
        (
            "prose is compiled",
            Box::new(|| prose::is_compiled(&NOT_COMPILED, &KNOWN_LANGUAGES)),
        ),
        ("documented paths resolve", Box::new(docs::doc_paths)),
        ("rustdoc links resolve", Box::new(nested::rustdoc_links)),
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
        ("the prose compiles", Box::new(docs_test)),
        (
            "the nested clippy config agrees",
            Box::new(nested::nested_clippy_agrees),
        ),
        (
            "README sections",
            Box::new(|| {
                readme::readme_sections(
                    README_ORDER,
                    README_OPTIONAL,
                    NESTED_ORDER,
                    NESTED_OPTIONAL,
                    &NOT_A_FRONT_PAGE,
                )
            }),
        ),
        (
            "AGENTS.md",
            Box::new(|| agents::agents_file_exists(AGENTS_FILE)),
        ),
        (
            "published crates deny missing_docs",
            Box::new(|| tree::published_crates_deny_missing_docs(DOCUMENTED)),
        ),
        (
            "comment blocks",
            Box::new(|| comments::comment_blocks(COMMENT_CAPS, COMMENT_SCOPE)),
        ),
        (
            "comment narration",
            Box::new(|| comments::comment_narration(NARRATION_CHECKED, COMMENT_SCOPE)),
        ),
    ];

    // `all` is what a laptop runs before a board is flashed, and what CI runs
    // in a job provisioned for it. It is separate from `check` because these
    // stages need a toolchain or a build system that a quick run should not
    // demand.
    if everything {
        gate.extend::<Vec<(&str, Box<dyn Fn() -> Result<String, String>>)>>(vec![
            ("the ESP32-C3 image links", Box::new(images::c3_links)),
            ("the ESP32-S3 image links", Box::new(images::s3_links)),
        ]);
    }

    // Last, after every insert and extend, owning the names: a closure in
    // the vector cannot borrow the vector.
    let names: Vec<String> = gate.iter().map(|(n, _)| n.to_string()).collect();
    gate.push((
        "the gate is documented",
        Box::new(move || agents::agents_documents_the_gate(&names)),
    ));

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
    notes.push(nested::docs_test_lints()?);
    Ok(notes.join(", "))
}

/// Clippy on the one bare-metal target, with warnings as errors.
///
/// Nothing here compiles for a host, so this is the only gate that reads
/// the code before a board does. The target has no atomic compare-and-swap:
/// load and store only, never `swap`, `fetch_or` or `compare_exchange`.
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

/// The tutorial's doctests, from the host workspace beside the firmware.
///
/// No `--target`, unlike `xpui-rp2040`'s twin: `.cargo/config.toml` here
/// sets no default target, so the host is what cargo builds for.
fn docs_test() -> Result<String, String> {
    cargo::cargo(&["test", "--manifest-path", "docs-test/Cargo.toml", "--doc"])?;
    Ok("docs-test/Cargo.toml on the host".into())
}
