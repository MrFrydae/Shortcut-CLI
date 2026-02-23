# shortcut — Shortcut CLI

[![CI](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/ci.yml/badge.svg)](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/ci.yml)
[![Release](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/release.yml/badge.svg)](https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/release.yml)
[![Renovate enabled](https://img.shields.io/badge/Renovate-enabled-brightgreen?logo=renovatebot)](https://github.com/renovatebot/renovate)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)
[![Homebrew](https://img.shields.io/badge/Homebrew-MrFrydae%2Ftap-orange.svg)](https://github.com/MrFrydae/homebrew-tap)

A fast, ergonomic command-line interface for the [Shortcut](https://shortcut.com) project management API.

## Highlights

- **Full API coverage** — stories, epics, iterations, labels, objectives, categories, groups, projects, documents, custom fields, and more
- **Interactive wizards** — `login` prompts for your token; story/epic creation walks you through required fields
- **Git integration** — generate branch names from stories and create commits prefixed with `[sc-ID]`
- **Shortcut Template Language (STL)** — declare stories, epics, and tasks in YAML and apply them in one command
- **Flexible output** — `--json`, `--quiet`, `--format "{id} {name}"`, `--dry-run`, and color control
- **Local caching** — workspace members, epic states, and workflow data are cached and refreshed on miss
- **Per-project config** — each project directory gets its own token and cache under `~/.shortcut/`
- **Secure token storage** — API tokens are written with `0600` permissions

## Dependency Updates

Dependency updates are managed by Renovate using `.github/renovate.json5`:

- Weekly update window (early Monday morning) to keep noise low
- Grouped non-major updates for Cargo crates and GitHub Actions
- Automatic merge for safe patch/minor updates
- Major updates require manual dashboard approval
- Weekly lockfile maintenance and separate security alert handling

## Installation

```sh
brew tap MrFrydae/tap
brew install shortcut
```

## Quick Start

```sh
shortcut init                  # Set up config directory for the current project
shortcut login                 # Authenticate with your Shortcut API token
shortcut story list            # List stories in your workspace
shortcut story create          # Create a story (interactive)
shortcut search stories "bug"  # Search across stories
```

## Commands

| Command | Subcommands | Description |
|---|---|---|
| `init` | — | Initialize `~/.shortcut/` directory for token and cache storage |
| `login` | — | Authenticate with your Shortcut API token |
| `story` | `list` `create` `get` `update` `delete` `task` `link` `comment` `history` `branch` `commit` | Full story management with tasks, links, comments, git integration |
| `epic` | `list` `create` `get` `update` `delete` `comment` `docs` | Manage epics with comments and linked docs |
| `iteration` | `list` `create` `get` `update` `delete` `stories` | Manage iterations and view their stories |
| `label` | `list` `create` `get` `update` `delete` `stories` `epics` | Manage labels and view associated entities |
| `objective` | `list` `create` `get` `update` `delete` `epics` | Manage objectives and their epics |
| `category` | `list` `create` `get` `update` `delete` `milestones` `objectives` | Manage categories |
| `project` | `list` `create` `get` `update` `delete` `stories` | Manage projects and view their stories |
| `group` | `list` `create` `get` `update` `stories` | Manage groups (teams) and view their stories |
| `doc` | `list` `create` `get` `update` `delete` `link` `unlink` `epics` | Manage documents with linking support |
| `custom-field` | `list` `get` | View custom field definitions |
| `template` | `list` `create` `get` `update` `delete` `use` `run` `validate` `init` | Entity templates and STL execution |
| `search` | `all` `stories` `epics` `iterations` `milestones` `objectives` `documents` | Search across Shortcut entities |
| `member` | — | List or look up workspace members by UUID or @mention |
| `workflow` | — | List workflows or view a workflow's states |

Run `shortcut <command> --help` for full flag documentation.

## Git Integration

`story branch` generates a branch name from a story's type and title. `story commit` prefixes your message with the Shortcut story ID.

```sh
# Generate a branch name from story 12345
shortcut story branch --id 12345
# feature/sc-12345/add-user-authentication

# Create and checkout the branch
shortcut story branch --id 12345 --checkout

# Override the type prefix
shortcut story branch --id 12345 --prefix hotfix

# Commit with auto-detected story ID from branch name
shortcut story commit -m "fix login redirect"
# [sc-12345] fix login redirect

# Explicit story ID
shortcut story commit --id 12345 -m "fix login redirect"

# Pass extra args to git commit
shortcut story commit -m "fix login redirect" -- --no-verify
```

## Output Options

Every command supports these global flags:

| Flag | Effect |
|---|---|
| `--json` | Output raw JSON instead of human-readable text |
| `--quiet` / `-q` | Suppress output; print only IDs |
| `--format <TPL>` | Format output using a template string (e.g. `"{id} {name}"`) |
| `--dry-run` | Preview the API request without sending it |
| `--color` | Force colored output |
| `--no-color` | Disable colored output |

```sh
shortcut story list --json
shortcut story list --quiet
shortcut story list --format "{id} - {name} ({type})"
shortcut story create --dry-run
```

## STL (Shortcut Template Language)

Declare stories, epics, and related entities in a `.shortcut.yml` file and apply them in one command.

```yaml
version: 1

meta:
  description: Sprint kickoff stories

vars:
  sprint_name: "Sprint 42"
  team_epic: "Backend Improvements"

operations:
  - action: create
    entity: epic
    alias: epic
    fields:
      name: "$var(team_epic)"

  - action: create
    entity: story
    alias: parent
    fields:
      epic_id: "$ref(epic)"
      name: "Design API schema for $var(sprint_name)"
      type: feature
      description: >
        Draft the OpenAPI spec for the new endpoints
        and get sign-off from the team before implementation.

  - action: create
    entity: story
    alias: tasks
    repeat:
      - name: "Implement endpoint scaffolding"
        type: feature
        tasks:
          - description: "Create route handlers"
          - description: "Add request validation"
      - name: "Write integration tests"
        type: chore
    fields:
      parent_story_id: "$ref(parent)"
```

```sh
shortcut template validate my-template.shortcut.yml   # Validate without executing
shortcut template run my-template.shortcut.yml         # Execute the template
shortcut template run my-template.shortcut.yml --dry-run  # Preview API calls
```

**Features:** variables (`$var()`), cross-operation references (`$ref()`), repeat blocks for batch creation, parent/child story relationships, inline tasks, block scalar descriptions, and configurable error handling (`on_error: continue`).

See [STL_SPEC.md](STL_SPEC.md) for the full specification.

<details>
<summary><strong>Configuration</strong></summary>

All config lives under `~/.shortcut/`, with per-project directories keyed by path hash:

```
~/.shortcut/
└── projects/
    └── <hash>/
        ├── token                       # API token (chmod 0600)
        └── cache/
            ├── epic_state_cache.json   # state name → ID mapping
            └── member_cache.json       # @mention → UUID mapping
```

- **Project discovery** walks up from the current directory to find a registered project, so `shortcut` works from any subdirectory.
- **Caches** are populated automatically on first use and refreshed on cache miss.

</details>

<details>
<summary><strong>Shell Completions</strong></summary>

Generate completion scripts with the hidden `completions` command:

```sh
# Bash
shortcut completions bash > ~/.local/share/bash-completion/completions/shortcut

# Zsh
shortcut completions zsh > ~/.zfunc/_shortcut

# Fish
shortcut completions fish > ~/.config/fish/completions/shortcut.fish
```

</details>

<details>
<summary><strong>Development</strong></summary>

```sh
cargo build          # Build
cargo test           # Run tests
cargo fmt --check    # Check formatting
cargo clippy         # Lint
```

The API client is auto-generated at compile time from `spec/shortcut.openapi.json` using [progenitor](https://github.com/oxidecomputer/progenitor).

</details>

## Releasing

Use the release helper script to cut a new semver release with one command:

```sh
scripts/release.sh 0.0.12
```

What it does:

- Updates the package `version` in `Cargo.toml`
- Regenerates `Cargo.lock` and verifies it includes the new `shortcut-cli` version
- Runs `cargo build --locked --release` locally
- Commits `Cargo.toml` + `Cargo.lock` and pushes `main`
- Triggers CI on `main`; release runs only after CI succeeds
- Release workflow extracts `v<version>` from `Cargo.toml`, creates the tag, and publishes artifacts from the CI run
- Publishes detailed release notes listing all non-merge commits since the previous release tag

## License

[AGPL-3.0](LICENSE)
