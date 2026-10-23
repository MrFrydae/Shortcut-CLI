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
- **Flexible output** — `--json`, `--toon`, `--quiet`, `--format "{id} {name}"`, `--dry-run`, and color control
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
| `config-dir` | — | Print the current project's hashed config directory, without authentication |
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
| `template` | `list` `create` `get` `update` `delete` `use` `run` `sync` `validate` `init` | Entity templates and STL execution |
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
| `--toon` | Output TOON instead of human-readable text |
| `--quiet` / `-q` | Suppress output; print only IDs |
| `--format <TPL>` | Format output using a template string (e.g. `"{id} {name}"`) |
| `--dry-run` | Preview the API request without sending it |
| `--color` | Force colored output |
| `--no-color` | Disable colored output |

Output mode precedence: `--json` > `--toon` > `--quiet` > `--format` > Human (default).

```sh
shortcut story list --json
shortcut story list --toon
shortcut story list --quiet
shortcut story list --format "{id} - {name} ({type})"
shortcut story create --dry-run
```

### Story JSON / TOON output

`story get`, `story create`, `story update`, and `story list` emit the full API
story object in `--json` and `--toon` modes (including `group_id`,
`iteration_id`, `app_url`, `story_links`, `label_ids`, `custom_fields`, and
timestamps), so a script can verify a write without a second lookup. On top of
the raw object:

- `labels` is a list of label names; the full label objects are in `label_details`.
- `pull_requests[].status` is `merged`, `closed`, `draft`, or `open`.
- `custom_fields[]` entries carry `field_id`, `value_id`, `value`, and a resolved `field_name`.
- Optional fields are always present (`null` or `[]`), so the key set is stable.

## Authentication via Environment Variable

For CI/non-interactive environments, you can authenticate with:

```sh
export SHORTCUT_API_TOKEN="<your-token>"
shortcut story list
```

`SHORTCUT_API_TOKEN` takes precedence over a stored project token file.

## Find the Config Directory and Recover a Saved Token

Run `shortcut config-dir` from the project where you previously ran `shortcut init`
or `shortcut login`, or from any subdirectory of that project:

```sh
shortcut config-dir
# /Users/you/.shortcut/projects/0123456789abcdef
```

The output is a single absolute directory path followed by a newline, with no
label or color codes. The example hash is illustrative; the command returns your
project's actual directory. `--quiet` produces the same path.

To inspect the directory or read your saved token in a local terminal (Bash/Zsh):

```sh
# List the token and cache files
config_dir=$(shortcut config-dir) && ls -la "$config_dir"

# Display your saved API token; keep the output private
config_dir=$(shortcut config-dir) && cat "$config_dir/token"

# Get just the token file's path, without displaying its contents
shortcut config-dir --format '{token_path}'
```

The `token` file contains the API token in plain text, with `0600` permissions on
Unix. `config-dir` only reports paths: it does not read or print the token, check
its validity, contact the API, or create or modify configuration. It works even
when the token is missing, expired, or unreadable. `--dry-run` returns the same
paths because this command makes no changes.

For scripts, use `--json` or `--toon` to return an object with `config_dir`,
`token_path`, and `cache_dir`. The same fields are available to `--format`:

```sh
shortcut config-dir --json
# {
#   "cache_dir": "/Users/you/.shortcut/projects/0123456789abcdef/cache",
#   "config_dir": "/Users/you/.shortcut/projects/0123456789abcdef",
#   "token_path": "/Users/you/.shortcut/projects/0123456789abcdef/token"
# }
shortcut config-dir --format '{config_dir}'
```

Directory selection follows the same rules as API commands:

- From a subdirectory, it uses the nearest registered ancestor.
- Linked Git worktrees use their own registration when present, otherwise the
  main checkout's registration (including registered ancestors of that checkout),
  then a registered ancestor above the worktree.
- The 16-character directory name is a 64-bit FNV-1a hash of the canonical
  project path, not of your token. Symbolic links resolve to the same canonical
  path; moving a project can change which hash is found.

If no registration is found, the command exits with status 1 and prints an error
to stderr, leaving stdout empty. For recovery, run it from the original project
location or a checkout that shares its registration. Running `init` at a new
location creates a new directory; it does not recover the old token. Existing
directories can also be inspected under `~/.shortcut/projects/`.

If the reported directory has no `token` file, no token is saved there. A token
provided only through `SHORTCUT_API_TOKEN` is not saved automatically; that
variable overrides file-based authentication but does not affect `config-dir`.
Use `shortcut login` to save a token when needed.

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
shortcut template sync my-template.shortcut.yml --confirm  # Reconcile template with prior state
shortcut template sync my-template.shortcut.yml --prune --confirm  # Also delete orphans from state
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

- **Project discovery** walks up from the current directory to find a registered project, so `shortcut` works from any subdirectory. In a linked Git worktree, it uses the main checkout's token and cache when the worktree has no registration of its own. Running `shortcut init` or `shortcut login` from an unregistered worktree registers the main checkout.
- **Find storage paths** with `shortcut config-dir`; see [token recovery examples](#find-the-config-directory-and-recover-a-saved-token).
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
