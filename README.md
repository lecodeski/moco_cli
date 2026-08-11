# ⏱️ MOCO CLI (`mococli`)

A small but mighty command-line tool to track your working time with
[MOCO](https://www.mocoapp.com/) right from your terminal — because alt-tabbing to a browser just to log "yep, I worked
today" is so last sprint. 🦀 It talks to the MOCO REST API to list, create, edit and delete activities, control activity
timers and show your overtime report.

## 🗺️ Overview

`mococli` is a Rust binary that wraps the MOCO API. After a one-time login, it tucks your credentials away locally and
lets you manage time-tracking activities without ever leaving the comfort of your shell. Your terminal, your rules. 😎

Features:

- 🔐 **Login** to a MOCO workspace and persist credentials locally.
- 📋 **List** activities for a date, day, week or month (with totals, so you don't have to do mental math).
- ➕ **New** activity creation (interactive or via flags), optionally starting a timer.
- ✏️ **Edit** an existing activity (date, hours, description).
- 🗑️ **Rm** (delete) an activity — single, or in a loop until the day is empty.
- ⏯️ **Timer** start/stop for activities.
- 📊 **Overtime** report (current overtime, or a monthly breakdown incl. work-time adjustments).

## 📦 Requirements

- [Rust](https://www.rust-lang.org/tools/install) toolchain (stable, 1.85 or newer) with Cargo. The crate uses Rust
  **edition 2024**, which requires Rust 1.85+.
- Network access to your MOCO instance (`https://<company>.mocoapp.com`).
- A MOCO account with:
    - a personal **API key**,
    - a **Bot API key** (required for the `overtime` report),
    - the company/subdomain name used to access MOCO.

A [dev container](.devcontainer/devcontainer.json) configuration (plus its [Dockerfile](.devcontainer/Dockerfile)) is
provided for a ready-to-use Rust development environment — zero "works on my machine" excuses. 🐳

## 🚀 Setup & Run

Build and run from source with Cargo:

```sh
# Build a release binary
cargo build --release

# The binary is then available at:
./target/release/mococli --help
```

During development, you can run directly via Cargo:

```sh
cargo run -- <command> [options]
```

Install it onto your `PATH`:

```sh
cargo install --path .
```

### 👋 First-time login

```sh
mococli login
```

You will be prompted for:

- MOCO company name (your MOCO subdomain),
- your personal API key,
- the MOCO Bot API key,
- your first and last name (used to resolve your user id).

Credentials are stored in a local config file (see [Configuration](#-configuration)).

## 🛠️ Commands

| Command    | Flags                                                       | Description                                              |
|------------|-------------------------------------------------------------|----------------------------------------------------------|
| `login`    | –                                                           | Log into MOCO and store the credentials.                 |
| `list`     | `--date`, `--day`, `--week`, `--month`, `--backward`        | List activities with a total. Defaults to today.         |
| `new`      | `--project`, `--task`, `--hours`, `--date`, `--description` | Create a new activity; missing values are prompted for.  |
| `edit`     | `--date`, `--activity`                                      | Edit date, hours and description of an activity.         |
| `rm`       | `--activity`, `--date`, `--loop`                            | Delete an activity, or several in a row with `--loop`.   |
| `timer`    | `start` \| `stop` (positional), `--activity`                | Start the timer on an activity, or stop the running one. |
| `overtime` | `--monthly`                                                 | Show your overtime report.                               |

Every id flag is optional: when it is omitted, `mococli` renders a numbered table and lets you pick the entry
interactively (in `rm --loop` you can also enter `A` to delete all listed entries).

For `list`, the selectors have a fixed precedence: `--date` > `--day` > `--week` > `--month`. Without `--backward` they
select within the current year (day of year, ISO calendar week, month number); with `--backward` they count back from
today (`--week 1` = last week, `--month 2` = two months ago, `--day 0` = today). `--backward` is ignored together with
`--date`.

A global `--debug` flag enables trace-level logging — for when things go sideways and you need the gory details. 🐛

### 💡 Examples

```sh
# List today's activities
mococli list

# List a specific day / week / month of the current year
mococli list --day 42
mococli list --week 12
mococli list --month 3

# List activities a number of days/weeks/months backward from now
mococli list --day 1 --backward
mococli list --week 1 --backward
mococli list --month 2 --backward

# List activities for a specific date
mococli list --date 2022-01-31

# Create a new activity (interactive prompts fill in the rest)
mococli new --project 123 --task 456 --hours 1.5 --date 2022-01-31 --description "Work"

# Create an activity and start a timer
# (leave the duration prompt empty — an activity with 0 hours starts the timer)
mococli new --project 123 --task 456

# Edit an activity (pick it from the activities of that date)
mococli edit --date 2022-01-31
mococli edit --activity 789

# Delete an activity
mococli rm --activity 789
mococli rm --date 2022-01-31

# Delete several activities of a day in one go ('A' deletes all)
mococli rm --date 2022-01-31 --loop

# Timer control
mococli timer start --activity 789
mococli timer stop

# Overtime
mococli overtime
mococli overtime --monthly
```

Run `mococli <command> --help` for the authoritative flag list.

## 📜 Scripts

No fancy custom script runner here — just good old reliable Cargo commands:

| Task    | Command                       |
|---------|-------------------------------|
| Build   | `cargo build`                 |
| Release | `cargo build --release`       |
| Run     | `cargo run -- <args>`         |
| Lint    | `cargo clippy -- -D warnings` |
| Format  | `cargo fmt`                   |

### 🤖 CI

[`.github/workflows/ci.yaml`](.github/workflows/ci.yaml) runs `cargo clippy -- -D warnings` and builds release binaries
for Linux, macOS and Windows on every push and pull request. On a published release the binaries are uploaded as release
assets.
[`.github/workflows/rust-clippy.yml`](.github/workflows/rust-clippy.yml) additionally reports clippy findings to GitHub
code scanning.

<!-- TODO: add unit/integration tests (there are none yet, so `cargo test` is a no-op). -->

## ⚙️ Configuration

On login, configuration is written as JSON to your OS config directory under
`mococli/mococli.json`:

- Linux: `~/.config/mococli/mococli.json`
- macOS: `~/Library/Application Support/mococli/mococli.json`
- Windows: `%APPDATA%\mococli\mococli.json`

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

These fields are normally populated by `mococli login`; manual editing is optional (but we won't stop you 🤷).

## 🌱 Environment Variables

- `RUST_LOG` — controls log verbosity via [`env_logger`](https://crates.io/crates/env_logger)
  (e.g. `RUST_LOG=debug`). The `--debug` flag forces trace-level logging regardless of this variable.

No other environment variables are read by `mococli`.

## 🗂️ Project Structure

```
.
├── Cargo.toml            # Package manifest & dependencies
├── Cargo.lock            # Locked dependency versions
├── LICENSE               # MIT license
├── README.md             # This file
├── CLAUDE.md             # Architecture notes for AI coding agents
├── .devcontainer/        # Dev container setup (devcontainer.json + Dockerfile)
├── .github/              # CI workflows (ci.yaml, rust-clippy.yml) & pull.yml
└── src/
    ├── main.rs           # Entry point: CLI dispatch & command handling
    ├── cli.rs            # CLI definition (clap: commands, flags)
    ├── config.rs         # App config: load/save credentials (mococli.json)
    ├── utils.rs          # Prompts, table rendering, date helpers
    └── moco/
        ├── mod.rs        # Module declarations
        ├── client.rs     # MOCO REST API client
        └── model.rs      # Request/response data models
```

## 🧰 Tech Stack

- **Language:** Rust (edition 2024)
- **Package manager / build:** Cargo
- **CLI:** [`clap`](https://crates.io/crates/clap) (derive)
- **Async runtime:** [`tokio`](https://crates.io/crates/tokio)
- **HTTP client:** [`reqwest`](https://crates.io/crates/reqwest) (JSON, rustls)
- **Config:** [`config`](https://crates.io/crates/config), [`dirs`](https://crates.io/crates/dirs)
- **Serialization:** [`serde`](https://crates.io/crates/serde), `serde_json`
- **Dates:** [`chrono`](https://crates.io/crates/chrono), `now`
- **Tables:** [`tabled`](https://crates.io/crates/tabled) (ansi)
- **Truncation:** [`unicode-ellipsis`](https://crates.io/crates/unicode-ellipsis)
- **Colors & Styling:** [`owo-colors`](https://crates.io/crates/owo-colors)
- **Logging:** [`log`](https://crates.io/crates/log), [`env_logger`](https://crates.io/crates/env_logger)
- **Misc:** `derive_more`, `num-traits`, `constcat`

## 📄 License

Licensed under the [MIT License](LICENSE) — short, sweet and permissive. 🎉 Copyright © 2022–2026 Emanuel Vollmer.
