# STL (Shortcut Template Language) — AI Agent Specification

> Machine-readable spec for AI coding agents generating `.shortcut.yml` files.
> Source of truth: `src/stl/validator.rs`

---

## File Format

- **YAML** (recommended): file extension `.shortcut.yml`
- **JSON**: file extension `.shortcut.json`
- Both formats are accepted by all `sc template` subcommands (`run`, `sync`, `validate`)
- A **JSON Schema** is available at [`stl-schema.json`](stl-schema.json) for IDE validation and autocompletion
- Every generated YAML file SHOULD begin with this header comment:

```yaml
# Shortcut Template Language (STL) v1
# Validate: shortcut template validate <this-file>
# Run:      shortcut template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md
```

### IDE Schema Association

**VS Code** — add to `.vscode/settings.json`:

```json
{
  "yaml.schemas": {
    "./stl-schema.json": "*.shortcut.yml"
  },
  "json.schemas": [
    { "fileMatch": ["*.shortcut.json"], "url": "./stl-schema.json" }
  ]
}
```

---

## Top-Level Keys

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `version` | integer | YES | Must be `1` |
| `meta` | mapping | no | `description?`, `author?` — informational only, not sent to API |
| `vars` | mapping | no | Key-value pairs for variable substitution |
| `on_error` | string | no | `continue` — continue executing on failure (default: stop) |
| `operations` | sequence | YES | List of operations to execute |

---

## Operations

Each operation is a mapping with these keys:

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `action` | string | YES | One of the action enum values |
| `entity` | string | YES | One of the entity enum values |
| `alias` | string | no | Name for referencing this operation's result via `$ref()` |
| `id` | any | conditional | Target entity ID — required for `update`, `delete`, `comment`, `unlink`, `check`, `uncheck` |
| `on_error` | string | no | Operation-level override: `continue` |
| `fields` | mapping | no | Field values to send to the API |
| `repeat` | sequence | no | List of mappings; each entry merged with `fields` and executed as separate operation |

### Alias naming rule

Pattern: `[a-zA-Z][a-zA-Z0-9_-]*`

---

## Action Enum

```
create | update | delete | comment | link | unlink | check | uncheck
```

## Entity Enum

```
story | epic | iteration | label | objective | milestone | category | group | document | project | task | comment | story_link
```

---

## Action × Entity Compatibility Matrix

| | story | epic | iteration | label | objective | milestone | category | group | document | project | task | comment | story_link |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **create** | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | - | - |
| **update** | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | - | - | - |
| **delete** | Y | Y | Y | Y | Y | Y | Y | - | Y | Y | Y | Y | Y |
| **comment** | Y | Y | - | - | - | - | - | - | - | - | - | - | - |
| **link** | - | - | - | - | - | - | - | - | - | - | - | - | Y |
| **unlink** | - | - | - | - | - | - | - | - | - | - | - | - | Y |
| **check** | - | - | - | - | - | - | - | - | - | - | Y | - | - |
| **uncheck** | - | - | - | - | - | - | - | - | - | - | Y | - | - |

`Y` = valid, `-` = rejected by validator.

---

## Required Fields on `create`

| Entity | Required Fields |
|--------|----------------|
| story | `name` |
| epic | `name` |
| iteration | `name`, `start_date`, `end_date` |
| label | `name` |
| objective | `name` |
| milestone | `name` |
| category | `name` |
| group | `name` |
| document | `name` |
| project | `name` |
| task | `description` |
| comment | _(none)_ |
| story_link | _(none)_ |

Note: When `repeat` is present, required fields may come from repeat entries instead of `fields`.

---

## Fields Per Entity

Required fields marked with `*`.

### story
`name*` `description` `type` `owner` `owners` `state` `epic_id` `iteration_id` `project_id` `group_id` `estimate` `labels` `followers` `requested_by` `deadline` `custom_fields` `tasks` `comments` `story_links` `parent_story_id`

### epic
`name*` `description` `state` `deadline` `owners` `followers` `requested_by` `labels` `objective_ids` `milestone_id` `group_ids` `planned_start_date`

### iteration
`name*` `start_date*` `end_date*` `description` `followers` `labels` `group_ids`

### label
`name*` `color` `description`

### objective
`name*` `description` `categories`

### milestone
`name*` `description` `categories` `state`

### category
`name*` `color` `type` `description`

### group
`name*` `description` `member_ids` `mention_name` `workflow_ids`

### document
`name*` `content` `content_file`

### project
`name*` `description` `team_id` `abbreviation` `color`

### task
`description*` `story_id` `complete` `owners`

### comment
`story_id` `epic_id` `text` `text_file`

### story_link
`subject_id` `object_id` `verb`

---

## Variable Syntax — `$var(name)`

- **Naming rule:** `[a-zA-Z][a-zA-Z0-9_]*`
- **Declaration:** Must be declared in top-level `vars` mapping before use
- **Type preservation:** When the entire value is `$var(name)`, the raw YAML type is preserved (integer stays integer, boolean stays boolean). When embedded in a larger string (`"Story: $var(title)"`), the value is stringified and interpolated.
- **Undeclared variable → validation error**

---

## Reference Syntax — `$ref(alias)`

References resolve to results of previously executed operations.

| Form | Resolves To |
|------|-------------|
| `$ref(alias)` | `id` field from the result of the operation with that alias |
| `$ref(alias.field)` | Named field from the result object |
| `$ref(alias.N)` | `id` field from the Nth result (zero-indexed) of a repeat operation |
| `$ref(alias.N.field)` | Named field from the Nth result of a repeat operation |

**Rules:**
- No forward references — alias must be defined in an earlier operation
- Duplicate aliases are rejected
- Type preservation: same rules as `$var()` (full-value → raw type, inline → stringified)
- Undefined alias → validation error

---

## Repeat Blocks

```yaml
- action: create
  entity: story
  alias: stories
  fields:
    project_id: 123
  repeat:
    - name: "Story A"
      estimate: 3
    - name: "Story B"
      estimate: 5
```

- Each repeat entry is **merged** with shared `fields` (repeat values override shared fields)
- Each entry executes as a separate API call
- Results stored as a JSON array under the alias
- Access individual results: `$ref(stories.0)`, `$ref(stories.1.name)`
- Each iteration counts separately in progress reporting
- `$var()` and `$ref()` are valid inside repeat entries

---

## Multiline Text (Block Scalars)

YAML block scalars work in any text field (`description`, `text`, `content`). `serde_yaml` handles them natively — no special STL syntax is needed.

### Literal block (`|`) — preserves newlines

```yaml
description: |
  Line one.
  Line two.

  Paragraph two.
```

Produces: `"Line one.\nLine two.\n\nParagraph two.\n"`

### Folded block (`>`) — joins lines with spaces

```yaml
description: >
  This is a long sentence
  that spans multiple lines
  in the YAML source.
```

Produces: `"This is a long sentence that spans multiple lines in the YAML source.\n"`

### Chomping indicators

| Indicator | Trailing newline |
|-----------|-----------------|
| `\|` or `>` (clip) | Single trailing `\n` |
| `\|-` or `>-` (strip) | No trailing `\n` |
| `\|+` or `>+` (keep) | All trailing `\n` preserved |

---

## Error Handling

| Scope | Key | Values | Default |
|-------|-----|--------|---------|
| Document-level | `on_error` | `continue` | stop on first error |
| Operation-level | `on_error` | `continue` | inherit from document-level |

Operation-level overrides document-level. When no `on_error` is specified at either level, execution stops at the first failure.

---

## Validation Error Conditions

| Condition | Trigger |
|-----------|---------|
| `unsupported version N` | `version` is not `1` |
| `invalid variable name 'X'` | Variable name doesn't match `[a-zA-Z][a-zA-Z0-9_]*` |
| `'action' action is not valid for 'entity' entity` | Action-entity pair not in compatibility matrix |
| `create entity requires field 'X'` | Missing required field on create (when no repeat block) |
| `action requires an 'id' field` | `update`/`delete`/`comment`/`unlink`/`check`/`uncheck` without `id` |
| `invalid alias name 'X'` | Alias doesn't match `[a-zA-Z][a-zA-Z0-9_-]*` |
| `duplicate alias 'X'` | Same alias used in multiple operations |
| `$ref(X) references undefined alias 'Y'` | Reference to alias not defined in a prior operation |
| `$var(X) references undeclared variable 'X'` | Variable not declared in `vars` section |
| `unknown field 'X' for entity entity` | Field name not in entity's known field list |

---

## Examples

### Minimal — Create a single story

```yaml
# Shortcut Template Language (STL) v1
# Validate: shortcut template validate <this-file>
# Run:      shortcut template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

version: 1
operations:
  - action: create
    entity: story
    fields:
      name: "Fix login redirect bug"
      type: bug
      state: "In Progress"
```

### With References — Epic + stories linked to it

```yaml
# Shortcut Template Language (STL) v1
# Validate: shortcut template validate <this-file>
# Run:      shortcut template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

version: 1
vars:
  sprint_name: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"

operations:
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint_name)"
      start_date: "$var(start)"
      end_date: "$var(end)"

  - action: create
    entity: epic
    alias: auth-epic
    fields:
      name: "Auth Hardening"
      description: |
        Harden authentication across all services.
        This includes JWT rotation, session management,
        and rate limiting.

  - action: create
    entity: story
    fields:
      name: "Implement JWT rotation"
      epic_id: $ref(auth-epic)
      iteration_id: $ref(sprint)
      estimate: 5
      labels:
        - security
```

### Complex — Repeat blocks, references, error handling

```yaml
# Shortcut Template Language (STL) v1
# Validate: shortcut template validate <this-file>
# Run:      shortcut template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

version: 1
meta:
  description: "Onboarding checklist for new microservice"
  author: "platform-team"

vars:
  service_name: "payments-api"
  team_group_id: "abc-123"

on_error: continue

operations:
  - action: create
    entity: epic
    alias: onboard-epic
    fields:
      name: "Onboard $var(service_name)"
      group_ids:
        - "$var(team_group_id)"

  - action: create
    entity: label
    alias: service-label
    fields:
      name: "$var(service_name)"
      color: "#0066cc"

  - action: create
    entity: story
    alias: stories
    fields:
      epic_id: $ref(onboard-epic)
      labels:
        - $ref(service-label.name)
    repeat:
      - name: "Set up CI/CD pipeline for $var(service_name)"
        type: chore
        estimate: 3
      - name: "Create monitoring dashboards for $var(service_name)"
        type: chore
        estimate: 2
      - name: "Write API documentation for $var(service_name)"
        type: chore
        estimate: 5

  - action: create
    entity: story_link
    fields:
      subject_id: $ref(stories.1)
      object_id: $ref(stories.0)
      verb: "blocks"

  - action: comment
    entity: story
    id: $ref(stories.0)
    fields:
      text: "Tracking epic: $ref(onboard-epic)"
```

### JSON Format — Minimal example

The same template structure works in JSON. Note: `$var()` and `$ref()` expressions are used as regular string values.

```json
{
  "$schema": "./stl-schema.json",
  "version": 1,
  "operations": [
    {
      "action": "create",
      "entity": "story",
      "fields": {
        "name": "Fix login redirect bug",
        "type": "bug",
        "state": "In Progress"
      }
    }
  ]
}
```

---

## Declarative Sync — `sc template sync`

The `sync` command enables Terraform-style declarative resource management. The same `.shortcut.yml` files work with both `run` (imperative, fire-and-forget) and `sync` (declarative, state-tracked).

### Command

```
sc template sync <file> [--state <path>] [--var key=value]... [--confirm] [--prune]
```

| Flag | Purpose |
|------|---------|
| `--state <path>` | Override state file location (default: `<file>.state.json`) |
| `--prune` | Delete resources that exist in state but were removed from the template |
| `--confirm` | Skip interactive confirmation |
| `--var` | Same as `run` — override template variables |

### State File

Stored alongside the template by default: `sprint.shortcut.yml` → `sprint.shortcut.yml.state.json`.

```json
{
  "version": 1,
  "created_at": "2026-02-24T12:00:00Z",
  "updated_at": "2026-02-24T12:30:00Z",
  "resources": {
    "my-epic": {
      "type": "single",
      "entity": "epic",
      "id": 55
    },
    "child-stories": {
      "type": "repeat",
      "entity": "story",
      "entries": {
        "story-a": { "id": 1001 },
        "story-b": { "id": 1002 }
      }
    }
  },
  "applied": ["op-3-comment", "op-4-link"]
}
```

- **`resources`**: Keyed by alias. Single ops store one ID. Repeat ops store a map of `key → id`.
- **`applied`**: Tracks imperative side-effect operations (comment/link/check) that have already been executed, keyed by `"op-{index}-{action}"`.
- State is saved incrementally after each successful operation, so partial failures don't lose progress.

### Sync-Specific Validation

In addition to standard validation, `sync` enforces:

| Condition | Error |
|-----------|-------|
| `create` operation without `alias` | `sync requires an 'alias' on every create operation` |
| Repeat entry without `key` field | `repeat entry N is missing required 'key' field for sync` |
| Duplicate `key` within a repeat block | `duplicate key 'X' in repeat entry N` |
| Inline task without `key` field | `task N is missing required 'key' field for sync` |
| Duplicate task `key` within a story | `duplicate task key 'X' in task N` |

### Key-Based Matching for Repeat Operations

Repeat entries use an explicit `key` field for stable identity across sync runs. The `key` is stripped before sending to the API.

```yaml
operations:
  - action: create
    entity: story
    alias: child-stories
    repeat:
      - key: auth-design        # required in sync mode
        name: "Auth Design Doc"
      - key: auth-impl
        name: "Auth Implementation"
    fields:
      epic_id: $ref(my-epic)
```

This means reordering or inserting entries is safe — keys are matched by name, not position.

### Reconciliation Logic

| Template operation | State entry? | Sync action |
|---|---|---|
| `create` (no repeat), alias not in state | No | **Create** — POST, save ID to state |
| `create` (no repeat), alias in state | Yes | **Update** — PATCH using stored ID |
| `create` repeat, entry key not in state | No | **Create** that entry |
| `create` repeat, entry key in state | Yes | **Update** that entry using stored ID |
| `create` repeat, key in state but removed from template | Orphan entry | **Warn** (delete with `--prune`) |
| `comment` / `link` / `check` etc. | Already in `applied` | **Skip** |
| `comment` / `link` / `check` etc. | Not in `applied` | **Execute** and record in `applied` |
| `update` / `delete` (explicit `id:`) | N/A | **Execute** as-is (not state-managed) |
| Alias in state but not in template | Orphan | **Warn** (delete with `--prune`) |

### Inline Task Sync

Stories can embed tasks in their `tasks` field. During sync, tasks require `key` fields:

```yaml
- action: create
  entity: story
  alias: my-story
  fields:
    name: "Feature X"
    tasks:
      - key: design
        description: "Design"
      - key: implement
        description: "Implement"
```

On first sync, task IDs are extracted from the story creation response and stored in state. On subsequent syncs, tasks are updated individually (not via the story update), and the `complete` status is never sent — preserving whatever the user set in Shortcut.

### Sync Example

```yaml
# Shortcut Template Language (STL) v1
# Sync:     shortcut template sync <this-file>
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

version: 1
vars:
  sprint: "Sprint 25"

operations:
  - action: create
    entity: epic
    alias: auth-epic
    fields:
      name: "Auth Hardening"

  - action: create
    entity: story
    alias: auth-stories
    repeat:
      - key: jwt-rotation
        name: "Implement JWT rotation"
        estimate: 5
      - key: session-mgmt
        name: "Session management overhaul"
        estimate: 8
    fields:
      epic_id: $ref(auth-epic)
      labels:
        - security

  - action: comment
    entity: story
    id: $ref(auth-stories.0)
    fields:
      text: "Tracking epic: $ref(auth-epic)"
```

First run: creates the epic, 2 stories, and a comment. Writes state file.
Edit template (e.g., change an estimate), run again: patches the story, skips the comment.
Remove a story entry, run with `--prune`: deletes the orphaned story from Shortcut.
