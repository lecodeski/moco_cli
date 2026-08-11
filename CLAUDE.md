# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```sh
cargo build                    # debug build
cargo clippy -- -D warnings    # lint — CI fails on any warning
cargo run -- <command> [opts]  # run during development, e.g. `cargo run -- list --week 12`
cargo fmt                      # format
```

There are no tests. CI (`.github/workflows/ci.yaml`) runs clippy with `-D warnings` and builds release binaries on Linux/macOS/Windows; releases upload the binary as an asset.

Rust edition 2024. All items use `pub(crate)` visibility, never `pub`.

## Architecture

`mococli` is a single-binary interactive CLI for the MOCO time-tracking REST API. Flow: `cli.rs` (clap derive) parses into `Commands` → `main.rs` holds one large `match` that implements every subcommand → `moco/client.rs` performs the HTTP calls → `utils.rs` supplies the interactive layer.

- **`src/cli.rs`** — clap `Parser`/`Subcommand` definitions only (Login, List, New, Edit, Rm, Timer, Overtime) plus custom help styles. No logic.
- **`src/config.rs`** — `AppConfig` (company, API key, bot API key, user id) persisted as JSON at `<os-config-dir>/mococli/mococli.json`. Loaded once in `main`, shared as `Rc<RefCell<AppConfig>>` so `login` can write credentials that the client reads in the same run (single-threaded tokio use; client methods carry `#[allow(clippy::await_holding_refcell_ref)]`).
- **`src/moco/client.rs`** — `MocoClient`, one method per API endpoint. Every method matches on the required config keys and returns `MocoClientError::NotLoggedIn` when they are absent. Most endpoints use the personal API key; `performance_report` and `work_time_adjustments` require the separate **bot** API key.
- **`src/moco/model.rs`** — serde request/response structs, no logic.
- **`src/utils.rs`** — interactive prompts and table rendering:
  - `ask_question` / `ask_question_mandatory` re-prompt in place with ANSI escapes (`\x1b[F\x1b[2K`) on invalid input.
  - `render_table` (tabled): row 0 is the bold header; any row containing `ARROW` (`==>`) is a bold footer/total row separated by a green line; other rows alternate plain/green.
  - `render_list_select` / `render_list_select_all` render a numbered table and loop until a valid index (or `A` for all) is entered; commands accept entity ids as flags and fall back to these pickers when flags are absent.
  - `select_from_to_date` converts `--day/--week/--month` (+ `--backward`) into a `(from, to)` date range and prints a human label for it.

Error handling is uniform: everything returns `Result<_, BoxedError>` (`Box<dyn Error>`) and bubbles up to `main`.
