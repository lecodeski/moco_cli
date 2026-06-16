# MOCO CLI (`mococli`)

A small command-line tool to track your working time with [MOCO](https://www.mocoapp.com/)
right from your terminal. It talks to the MOCO REST API to list, create, edit and
delete activities, control activity timers and show your overtime report.

## Overview

`mococli` is a Rust binary that wraps the MOCO API. After a one-time login it stores your
credentials locally and lets you manage time-tracking activities without leaving the shell.

Features:

- **Login** to a MOCO workspace and persist credentials locally.
- **List** activities for a day, week or month (with totals).
- **New** activity creation (interactive or via flags), optionally starting a timer.
- **Edit** an existing activity (date, hours, description).
- **Rm** (delete) an activity.
- **Timer** start/stop for activities.
- **Overtime** report (current overtime, or a monthly breakdown).

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) toolchain (stable) with Cargo. The crate
  uses Rust **edition 2021**.
  <!-- TODO: confirm and pin the minimum supported Rust version (MSRV). -->
- Network access to your MOCO instance (`https://<company>.mocoapp.com`).
- A MOCO account with:
    - a personal **API key**,
    - a **Bot API key** (required for the `overtime` report),
    - the company/subdomain name used to access MOCO.

A [dev container](.devcontainer/devcontainer.json) configuration is provided for a
ready-to-use Rust development environment.

## Setup & Run

Build and run from source with Cargo:

```sh
# Build a release binary
cargo build --release

# The binary is then available at:
./target/release/mococli --help
```

During development you can run directly via Cargo:

```sh
cargo run -- <command> [options]
```

Install it onto your `PATH`:

```sh
cargo install --path .
```

### First-time login

```sh
mococli login moco
```

You will be prompted for:

- Moco company name (your MOCO subdomain),
- your personal API key,
- the Moco Bot API key,
- your first and last name (used to resolve your user id).

Credentials are stored in a local config file (see [Configuration](#configuration)).

## Commands

| Command    | Description                             |
|------------|-----------------------------------------|
| `login`    | Login into Moco (`mococli login moco`). |
| `list`     | List activities.                        |
| `new`      | Create a new activity.                  |
| `edit`     | Edit an existing activity.              |
| `rm`       | Delete an activity.                     |
| `timer`    | Start/stop an activity timer.           |
| `overtime` | Show your overtime report.              |

A global `--debug` flag enables trace-level logging.

### Examples

```sh
# List today's activities
mococli list

# List the current week / month
mococli list --week
mococli list --month

# List activities a number of weeks/months backward
mococli list --week --backward 1

# List activities for a specific date
mococli list --date 2022-01-31

# Create a new activity (interactive prompts fill in the rest)
mococli new --project 123 --task 456 --hours 1.5 --date 2022-01-31 --description "Work"

# Create an activity and start a timer (omit --hours)
mococli new --project 123 --task 456

# Edit an activity
mococli edit --activity 789

# Delete an activity
mococli rm --activity 789

# Timer control
mococli timer start --activity 789
mococli timer stop

# Overtime
mococli overtime
mococli overtime --monthly
```

<!-- TODO: confirm exact accepted values/defaults for each flag against `--help`. -->

## Scripts

This project has no custom script runner; use standard Cargo commands:

| Task    | Command                 |
|---------|-------------------------|
| Build   | `cargo build`           |
| Release | `cargo build --release` |
| Run     | `cargo run -- <args>`   |
| Test    | `cargo test`            |
| Lint    | `cargo clippy`          |
| Format  | `cargo fmt`             |

<!-- TODO: add a CI configuration / Makefile if scripted workflows are desired. -->

## Configuration

On login, configuration is written as JSON to your OS config directory under
`mococli/mococp.json`:

- Linux: `~/.config/mococli/mococp.json`
- macOS: `~/Library/Application Support/mococli/mococp.json`
- Windows: `%APPDATA%\mococli\mococp.json`

(The exact base directory is resolved by the [`dirs`](https://crates.io/crates/dirs) crate.)

### Example config

```json
{
  "moco_company": "your-company",
  "moco_api_key": "your-personal-api-key",
  "moco_bot_api_key": "your-bot-api-key",
  "moco_user_id": 123456
}
```

These fields are normally populated by `mococli login moco`; manual editing is optional.

## Environment Variables

- `RUST_LOG` — controls log verbosity via [`env_logger`](https://crates.io/crates/env_logger)
  (e.g. `RUST_LOG=debug`). The `--debug` flag forces trace-level logging regardless of this
  variable.

<!-- TODO: document any additional environment variables if introduced. -->

<!-- TODO: Add unit/integration tests. -->

## Project Structure

```
.
├── Cargo.toml            # Package manifest & dependencies
├── Cargo.lock            # Locked dependency versions
├── LICENSE               # MIT license
├── .devcontainer/        # VS Code dev container setup
└── src/
    ├── main.rs           # Entry point: CLI dispatch & command handling
    ├── cli.rs            # CLI definition (clap: commands, flags)
    ├── config.rs         # App config: load/save credentials (mococp.json)
    ├── utils.rs          # Prompts, table rendering, date helpers
    └── moco/
        ├── mod.rs        # Module declarations
        ├── client.rs     # MOCO REST API client
        └── model.rs      # Request/response data models
```

## Tech Stack

- **Language:** Rust (edition 2021)
- **Package manager / build:** Cargo
- **CLI:** [`clap`](https://crates.io/crates/clap) (derive)
- **Async runtime:** [`tokio`](https://crates.io/crates/tokio)
- **HTTP client:** [`reqwest`](https://crates.io/crates/reqwest) (JSON, rustls)
- **Config:** [`config`](https://crates.io/crates/config), [`dirs`](https://crates.io/crates/dirs)
- **Serialization:** [`serde`](https://crates.io/crates/serde), `serde_json`
- **Dates:** [`chrono`](https://crates.io/crates/chrono), `chronoutil`, `now`
- **Logging:** [`log`](https://crates.io/crates/log), [`env_logger`](https://crates.io/crates/env_logger)
- **Misc:** `derive_more`, `num-traits`

## License

Licensed under the [MIT License](LICENSE). Copyright (c) 2022 Emanuel Vollmer.
