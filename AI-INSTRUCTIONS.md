# shortcut CLI — AI Agent Reference

Binary: `shortcut` (aliased `sc`). Rust CLI for the [Shortcut](https://shortcut.com) project management API.

---

## Setup

```sh
shortcut init          # create ~/.shortcut/ config dir for current project
shortcut login         # authenticate (prompts for token; or --token <TOKEN>)
shortcut completions <shell>   # hidden; generate shell completions (bash|zsh|fish)
```

---

## Global Flags

Apply to every command.

| Flag | Short | Effect |
|------|-------|--------|
| `--json` | | Raw JSON output |
| `--quiet` | `-q` | Suppress output; print only IDs |
| `--format <TPL>` | | Template string output (e.g. `"{id} {name}"`) — supports dot notation `{stats.num_stories}` |
| `--dry-run` | | Preview API request without sending |
| `--color` | | Force colored output |
| `--no-color` | | Disable colored output |

Output mode precedence: `--json` > `--quiet` > `--format` > Human (default).

---

## Entity Commands

### story

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--owner <@mention\|UUID>` `--state <name\|ID>` `--epic-id <i64>` `--type <feature\|bug\|chore>` `--label <name>` `--project-id <i64>` `--limit <N>` (default 25) `--desc` |
| `create` | `--name <STR>` (unless `-i`) | `-i` (interactive) `--description` `--type` `--owner <csv>` `--state` `--epic-id` `--estimate` `--labels <csv>` `--group-id` `--iteration-id` `--custom-field <Key=Val>` (repeatable) `--parent-story-id` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--description` `--type` `--owner <csv>` (replaces all) `--add-owner <csv>` (appends; conflicts with --owner) `--state` `--epic-id` `--estimate` `--labels <csv>` `--iteration-id` `--custom-field <Key=Val>` (repeatable) `--parent-story-id` `--unless-state <csv>` (skip if in these states) |
| `delete` | `--id <i64>` `--confirm` | |
| `history` | `--id <i64>` | `--limit <N>` |
| `branch` | `--id <i64>` | `--prefix <STR>` (override type prefix) `-c`/`--checkout` |
| `commit` | `-m <MSG>` | `--id <i64>` (overrides branch detection) `-- <extra git args>` |

#### story task

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `add` | `--story-id <i64>` `--description <STR>` (repeatable for multiple) | |
| `list` | `--story-id <i64>` | |
| `get` | `--story-id <i64>` `--id <i64>` | |
| `check` | `--story-id <i64>` `--id <i64>` | |
| `uncheck` | `--story-id <i64>` `--id <i64>` | |
| `update` | `--story-id <i64>` `--id <i64>` | `--description` `--complete <bool>` |
| `delete` | `--story-id <i64>` `--id <i64>` | |

#### story link

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `create` | `--subject <i64>` `--object <i64>` `--verb <blocks\|blocked-by\|duplicates\|relates-to>` | |
| `list` | `--story-id <i64>` | |
| `delete` | `--id <i64>` | `--confirm` |

#### story comment

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | `--story-id <i64>` | |
| `add` | `--story-id <i64>` | `--text <STR>` or `--text-file <PATH>` (one required) |
| `get` | `--story-id <i64>` `--id <i64>` | |
| `update` | `--story-id <i64>` `--id <i64>` `--text <STR>` | |
| `delete` | `--story-id <i64>` `--id <i64>` | `--confirm` |
| `react` | `--story-id <i64>` `--comment-id <i64>` `--emoji <name>` | |
| `unreact` | `--story-id <i64>` `--comment-id <i64>` `--emoji <name>` | |

---

### epic

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--desc` |
| `create` | `--name <STR>` (unless `-i`) | `-i` `--description` `--state <name\|ID>` `--deadline <RFC3339>` `--owners <csv>` `--group-id <csv>` `--labels <csv>` `--objective-ids <csv i64>` `--followers <csv>` `--requested-by <@mention\|UUID>` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--description` `--deadline` `--archived <bool>` `--epic-state-id <name\|ID>` `--labels <csv>` `--objective-ids <csv>` `--owner <csv>` (replaces) `--add-owner <csv>` (appends; conflicts with --owner) `--follower <csv>` `--requested-by` `--unless-state <csv>` |
| `delete` | `--id <i64>` `--confirm` | |
| `docs` | `--id <i64>` | |

#### epic comment

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | `--epic-id <i64>` | |
| `add` | `--epic-id <i64>` | `--text <STR>` or `--text-file <PATH>` |
| `get` | `--epic-id <i64>` `--id <i64>` | |
| `update` | `--epic-id <i64>` `--id <i64>` `--text <STR>` | |
| `delete` | `--epic-id <i64>` `--id <i64>` | `--confirm` |

---

### iteration

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--state <started\|unstarted\|done>` |
| `create` | `--name` `--start-date` `--end-date` (unless `-i`) | `-i` `--description` `--followers <csv>` `--labels <csv>` `--group-ids <csv UUID>` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--start-date` `--end-date` `--description` `--followers <csv>` `--labels <csv>` `--group-ids <csv UUID>` |
| `delete` | `--id <i64>` `--confirm` | |
| `stories` | `--id <i64>` | `--desc` |

---

### label

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--desc` |
| `create` | `--name <STR>` | `--color <hex>` `--description` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--color` `--description` `--archived <bool>` |
| `delete` | `--id <i64>` `--confirm` | |
| `stories` | `--id <i64>` | `--desc` |
| `epics` | `--id <i64>` | `--desc` |

---

### objective

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--archived` |
| `create` | `--name <STR>` | `--description` `--state <"to do"\|"in progress"\|done>` `--categories <csv names>` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--description` `--state` `--archived <bool>` `--categories <csv>` |
| `delete` | `--id <i64>` `--confirm` | |
| `epics` | `--id <i64>` | `--desc` |

---

### category

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--archived` |
| `create` | `--name <STR>` | `--color <hex>` `--type` `--external-id` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--color` `--archived <bool>` |
| `delete` | `--id <i64>` `--confirm` | |
| `milestones` | `--id <i64>` | |
| `objectives` | `--id <i64>` | `--desc` |

---

### project

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--archived` |
| `create` | `--name <STR>` `--team-id <i64>` | `--description` `--color <hex>` `--abbreviation <3 chars>` `--iteration-length <N>` `--external-id` |
| `get` | `--id <i64>` | |
| `update` | `--id <i64>` | `--name` `--description` `--color` `--abbreviation` `--archived <bool>` `--team-id` `--days-to-thermometer` `--show-thermometer <bool>` |
| `delete` | `--id <i64>` `--confirm` | |
| `stories` | `--id <i64>` | `--desc` |

---

### group

ID arg accepts `@mention_name` or UUID.

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | `--archived` |
| `create` | `--name <STR>` `--mention-name <STR>` | `--description` `--color <hex>` `--member-ids <csv @mention\|UUID>` `--workflow-ids <csv i64>` |
| `get` | `--id <@mention\|UUID>` | |
| `update` | `--id <@mention\|UUID>` | `--name` `--mention-name` `--description` `--archived <bool>` `--color` `--member-ids <csv>` `--workflow-ids <csv>` |
| `stories` | `--id <@mention\|UUID>` | `--limit <N>` `--offset <N>` `--desc` |

---

### doc

Document IDs are UUIDs (strings), not integers.

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | |
| `create` | `--title <STR>` | `--content <STR>` or `--content-file <PATH>` (one required) `--content-format <markdown\|html>` |
| `get` | `--id <UUID>` | |
| `update` | `--id <UUID>` | `--title` `--content` or `--content-file` `--content-format` |
| `delete` | `--id <UUID>` `--confirm` | |
| `link` | `--doc-id <UUID>` `--epic-id <i64>` | |
| `unlink` | `--doc-id <UUID>` `--epic-id <i64>` | |
| `epics` | `--id <UUID>` | |

---

### custom-field

Read-only.

| Subcommand | Required Args |
|------------|---------------|
| `list` | |
| `get` | `--id <UUID>` |

---

### template

| Subcommand | Required Args | Optional Args |
|------------|---------------|---------------|
| `list` | | |
| `get` | `--id <UUID>` | |
| `create` | `--name <STR>` | `--story-name` `--description` or `--description-file` `--type` `--owner <csv>` `--state` `--epic-id` `--estimate` `--labels <csv>` `--iteration-id` `--custom-field <Key=Val>` (repeatable) |
| `update` | `--id <UUID>` | same as create |
| `delete` | `--id <UUID>` `--confirm` | |
| `use` | `--id <UUID>` `--name <STR>` | `--description` or `--description-file` `--type` `--owner <csv>` `--state` `--epic-id` `--estimate` `--labels <csv>` `--iteration-id` `--custom-field <Key=Val>` |
| `run` | `<FILE>` (positional; `-` for stdin) | `--confirm` (skip prompt) `--var <key=value>` (repeatable) `--dry-run` |
| `validate` | `<FILE>` (positional) | |
| `init` | | `--path <DIR>` `--stdout` |

---

### search

All subcommands share: `<QUERY>` (positional), `--page-size <N>` (default 25), `--next <TOKEN>` (pagination cursor), `--desc`.

| Subcommand | Scope |
|------------|-------|
| `all` | stories + epics + iterations + objectives |
| `stories` | stories |
| `epics` | epics |
| `iterations` | iterations |
| `milestones` | milestones (objectives) |
| `objectives` | objectives |
| `documents` | documents (title search) |

---

### member

Flat command (no subcommands).

| Flag | Effect |
|------|--------|
| `--list` | list all workspace members |
| `--id <UUID\|@mention>` | get specific member |
| `--role <admin\|member\|owner\|observer>` | filter by role |
| `--active` | show only non-disabled members |

---

### workflow

Flat command (no subcommands).

| Flag | Effect |
|------|--------|
| `--list` | list all workflows |
| `--id <i64>` | get specific workflow and its states |

---

## Field Reference Conventions

| Convention | Details |
|------------|---------|
| `@mention` resolution | `--owner`, `--group-id`, `--member-ids`, `--followers`, `--requested-by` accept `@mention_name` — resolved via member cache |
| State resolution | `--state` accepts name (e.g. "In Progress") or numeric ID — resolved via workflow/epic-state cache |
| Comma-separated lists | Flags like `--owner`, `--labels`, `--group-ids` accept `val1,val2` or repeated `--flag val1 --flag val2` |
| Repeatable flags | `--custom-field`, `--description` (on task add) — must repeat: `--custom-field A=1 --custom-field B=2` |
| `--unless-state` | story/epic update only; silently skips if entity already in named state |
| `--owner` vs `--add-owner` | `--owner` replaces all; `--add-owner` appends; mutually exclusive |
| Deletions | require `--confirm` (unless `--dry-run`) |
| Interactive mode | `story create`, `epic create`, `iteration create` support `-i` for TUI wizard (requires TTY) |

---

## Git Integration

### story branch

Generates branch name: `<type>/sc-<id>/<slugified-title>`.

```sh
shortcut story branch --id 12345              # feature/sc-12345/add-user-auth
shortcut story branch --id 12345 --checkout   # create + checkout
shortcut story branch --id 12345 --prefix hotfix  # hotfix/sc-12345/...
```

### story commit

Prefixes message with `[sc-ID]`. Auto-detects story ID from branch name if `--id` omitted.

```sh
shortcut story commit -m "fix login redirect"              # [sc-12345] fix login redirect
shortcut story commit --id 99 -m "fix bug"                 # [sc-99] fix bug
shortcut story commit -m "fix" -- --no-verify              # pass extra git args
```

---

## STL (Shortcut Template Language) Quick Reference

YAML-based DSL for batch API mutations. Full spec: [STL_SPEC.md](STL_SPEC.md).

### Commands

```sh
shortcut template validate <file>             # validate without executing
shortcut template run <file>                  # execute (prompts for confirm)
shortcut template run <file> --confirm        # skip confirmation prompt
shortcut template run <file> --dry-run        # preview API calls
shortcut template run <file> --var key=value  # pass/override variables
shortcut template run - --confirm < file.yml  # read from stdin
```

### Top-Level Keys

`version` (=1, required), `meta?` {description, author}, `vars?`, `on_error?` (continue), `operations` (required)

### Actions

`create` `update` `delete` `comment` `link` `unlink` `check` `uncheck`

### Entities

`story` `epic` `iteration` `label` `objective` `milestone` `category` `group` `document` `project` `task` `comment` `story_link`

### Action x Entity Matrix

```
          story epic iter label obj  mile cat  grp  doc  proj task cmnt slink
create      Y    Y    Y     Y    Y    Y    Y    Y    Y    Y    Y    -    -
update      Y    Y    Y     Y    Y    Y    Y    Y    Y    Y    -    -    -
delete      Y    Y    Y     Y    Y    Y    Y    -    Y    Y    Y    Y    Y
comment     Y    Y    -     -    -    -    -    -    -    -    -    -    -
link        -    -    -     -    -    -    -    -    -    -    -    -    Y
unlink      -    -    -     -    -    -    -    -    -    -    -    -    Y
check       -    -    -     -    -    -    -    -    -    -    Y    -    -
uncheck     -    -    -     -    -    -    -    -    -    -    Y    -    -
```

### Fields Per Entity (* = required on create)

| Entity | Fields |
|--------|--------|
| story | `name*` description type owner owners state epic_id iteration_id project_id group_id estimate labels followers requested_by deadline custom_fields tasks comments story_links parent_story_id |
| epic | `name*` description state deadline owners followers requested_by labels objective_ids milestone_id group_ids planned_start_date |
| iteration | `name*` `start_date*` `end_date*` description followers labels group_ids |
| label | `name*` color description |
| objective | `name*` description categories |
| milestone | `name*` description categories state |
| category | `name*` color type description |
| group | `name*` description member_ids mention_name workflow_ids |
| document | `name*` content content_file |
| project | `name*` description team_id abbreviation color |
| task | `description*` story_id complete owners |
| comment | story_id epic_id text text_file |
| story_link | subject_id object_id verb |

`id` required for: update, delete, comment, unlink, check, uncheck.

### Variables and References

- `$var(name)` — name must match `[a-zA-Z][a-zA-Z0-9_]*`, declared in `vars`
- `$ref(alias)` — resolved to `id` of aliased operation result
- `$ref(alias.field)` — specific field from result
- `$ref(alias.N)` — Nth result (0-indexed) from repeat operation
- `$ref(alias.N.field)` — specific field from Nth result
- Full-value preserves type; inline interpolation stringifies
- No forward references; duplicate aliases rejected

### Repeat Blocks

```yaml
repeat:
  - name: "Story A"
    estimate: 3
  - name: "Story B"
    estimate: 5
fields:
  project_id: 123    # merged into each repeat entry (repeat overrides)
```

### Error Handling

`on_error: continue` at document or operation level. Operation overrides document. Default: stop on first failure.

### Example

```yaml
# Shortcut Template Language (STL) v1
# Validate: shortcut template validate <this-file>
# Run:      shortcut template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

version: 1
vars:
  sprint: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"

operations:
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint)"
      start_date: "$var(start)"
      end_date: "$var(end)"

  - action: create
    entity: epic
    alias: my-epic
    fields:
      name: "Auth Hardening"

  - action: create
    entity: story
    alias: stories
    fields:
      epic_id: $ref(my-epic)
      iteration_id: $ref(sprint)
    repeat:
      - name: "Implement JWT rotation"
        estimate: 5
        tasks:
          - description: "Add token refresh endpoint"
          - description: "Update auth middleware"
      - name: "Add rate limiting"
        estimate: 3

  - action: comment
    entity: story
    id: $ref(stories.0)
    fields:
      text: "Linked to epic $ref(my-epic)"
```

---

## Config & Cache Layout

```
~/.shortcut/
└── projects/
    └── <path-hash>/
        ├── token                       # API token (chmod 0600)
        └── cache/
            ├── epic_state_cache.json   # state name -> ID
            ├── member_cache.json       # @mention -> UUID
            └── workflow_cache.json     # workflow state data
```

- Project discovery walks up from cwd to find registered project
- Caches auto-populated on first use, refreshed on miss
- `shortcut init` creates the directory; `shortcut login` stores the token
