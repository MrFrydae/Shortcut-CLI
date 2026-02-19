# `sc` — Shortcut CLI

[![CI](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/ci.yml/badge.svg)](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)

A fast, ergonomic command-line interface for the [Shortcut](https://shortcut.com) project management API.

## Features

- **Authenticate** with the Shortcut API via token (interactive or CLI flag)
- **Manage epics** — list, update with smart state resolution (e.g. `in_progress`, `In-Progress`, and `in progress` all match)
- **View workspace members** — list all, or look up by UUID or `@mention_name`
- **Browse workflows** and their states in a formatted table
- **Local caching** for fast repeated lookups (epic states, member mentions)
- **Per-project config** — multiple Shortcut workspaces in different directories
- **Secure token storage** with `0600` file permissions

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) (stable).

```sh
git clone https://github.com/MrFrydae/Shortcut-CLI.git
cd Shortcut-CLI
cargo install --path .
```

## Quick Start

```sh
sc init              # Set up config directory for the current project
sc login             # Authenticate with your Shortcut API token
sc epic list         # List all epics in your workspace
```

## Usage

### `sc init`

Initialize the `~/.sc/` directory structure for the current project.

```sh
sc init
# Initialized sc for /path/to/your/project
```

### `sc login`

Authenticate with your Shortcut API token. Prompts interactively if `--token` is omitted.

```sh
sc login
# Shortcut API token: ****

sc login --token <TOKEN>
# Logged in as Jane Doe (@jane)
```

### `sc epic list`

List all epics in your workspace.

```sh
sc epic list
# 42 - My Epic
# 43 - Another Epic

sc epic list --desc
# 42 - My Epic
#   Build the thing
# 43 - Another Epic
#   Ship the feature
```

### `sc epic update`

Update an epic by ID. All options except `--id` are optional.

```sh
sc epic update --id 42 --name "Renamed Epic"
sc epic update --id 42 --epic-state-id in_progress
sc epic update --id 42 --deadline 2026-06-01T00:00:00Z --labels "backend,priority"
# Updated epic 42 - Renamed Epic
```

| Flag | Description |
|---|---|
| `--id <ID>` | **(required)** Epic ID to update |
| `--name <NAME>` | New name |
| `--description <DESC>` | New description |
| `--deadline <RFC3339>` | Deadline (e.g. `2026-12-31T00:00:00Z`) |
| `--archived <BOOL>` | Archive or unarchive |
| `--epic-state-id <ID\|NAME>` | State ID or name (e.g. `500000042` or `in_progress`) |
| `--labels <L1,L2,...>` | Label names (comma-separated) |
| `--objective-ids <ID,...>` | Objective IDs (comma-separated) |
| `--owner-ids <UUID,...>` | Owner member UUIDs (comma-separated) |
| `--follower-ids <UUID,...>` | Follower member UUIDs (comma-separated) |
| `--requested-by-id <UUID>` | Requester member UUID |

### `sc member`

List or look up workspace members.

```sh
sc member --list
# 12345678-abcd-... - @jane - Jane Doe (admin)
# 87654321-dcba-... - @bob  - Bob Smith (member)

sc member --id @jane
sc member --id 12345678-abcd-1234-abcd-1234567890ab
```

### `sc workflow`

List workflows or view a specific workflow's states.

```sh
sc workflow --list
# 500000001 - Development
# 500000002 - Bug Tracking

sc workflow --id 500000001
# Development (500000001)
# ID          Type        Name
# 500000010   unstarted   Backlog
# 500000011   started     In Progress
# 500000012   done        Completed
```

## Configuration

All config lives under `~/.sc/`, with per-project directories keyed by path hash:

```
~/.sc/
└── projects/
    └── <hash>/
        ├── token                       # API token (chmod 0600)
        └── cache/
            ├── epic_state_cache.json   # state name → ID mapping
            └── member_cache.json       # @mention → UUID mapping
```

- **Project discovery** walks up from the current directory to find a registered project, so `sc` works from any subdirectory.
- **Caches** are populated automatically on first use and refreshed on cache miss.

## Development

```sh
cargo build          # Build
cargo test           # Run tests
cargo fmt --check    # Check formatting
cargo clippy         # Lint
```

The API client is auto-generated at compile time from `spec/shortcut.openapi.json` using [progenitor](https://github.com/oxidecomputer/progenitor).

## License

[MIT](LICENSE)
