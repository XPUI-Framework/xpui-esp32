# Documentation

[`../README.md`](../README.md) is the front page. Every document in this
repository, and what each is for:

| | |
|---|---|
| [tutorial.md](tutorial.md) | your first screen on an ESP32: the board as data, the palette trap, the loop and the driver seam, memory, flashing — compiled by `docs-test/` |
| [docs-test/README.md](../docs-test/README.md) | the crate with no code that compiles the tutorial, and why it is a workspace of its own |
| [reference.md](reference.md) | every public item: `run`, `Panel` and its four methods, `init_heap` and `runtime::park` — checked against the board's rustdoc, its `rust` fence compiled by `docs-test/` |
| [hardware.md](hardware.md) | what runs and what does not, where the panel driver goes, memory, the two chips, and what the build taught |
| [contributing.md](contributing.md) | the target, the fork, `espflash`, the gate in both modes and what CI builds, the five review steps, and how a commit is written |
