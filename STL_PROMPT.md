# STL Quick Reference (Prompt-Ready)

File: `.sc.yml` | YAML 1.0 | `version: 1` required
Validate: `sc template validate <file>` | Run: `sc template run <file> --confirm`
Full spec: https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md

---

HEADER — prepend to every generated `.sc.yml`:
```yaml
# Shortcut Template Language (STL) v1
# Validate: sc template validate <this-file>
# Run:      sc template run <this-file> --confirm
# Docs:     https://github.com/MrFrydae/Shortcut-CLI/blob/main/STL_SPEC.md
```

---

TOP-LEVEL: `version`(=1) `meta?`{description,author} `vars?` `on_error?`(continue) `operations`

ACTIONS: `create` `update` `delete` `comment` `link` `unlink` `check` `uncheck`
ENTITIES: `story` `epic` `iteration` `label` `objective` `milestone` `category` `group` `document` `project` `task` `comment` `story_link`

ACTION×ENTITY MATRIX (Y=valid):
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

FIELDS (* = required on create):
- story: `name*` description type owner owners state epic_id iteration_id project_id group_id estimate labels followers requested_by deadline custom_fields tasks comments story_links
- epic: `name*` description state deadline owners followers requested_by labels objective_ids milestone_id group_ids planned_start_date
- iteration: `name*` `start_date*` `end_date*` description followers labels group_ids
- label: `name*` color description
- objective: `name*` description categories
- milestone: `name*` description categories state
- category: `name*` color type description
- group: `name*` description member_ids mention_name workflow_ids
- document: `name*` content content_file
- project: `name*` description team_id abbreviation color
- task: `description*` story_id complete owners
- comment: story_id epic_id text text_file
- story_link: subject_id object_id verb

`id` REQUIRED for: update, delete, comment, unlink, check, uncheck

---

`$var(name)` — name matches `[a-zA-Z][a-zA-Z0-9_]*`, must be declared in `vars`. Full-value preserves type; inline interpolation stringifies.
`$ref(alias)` — forms: `$ref(a)` `$ref(a.field)` `$ref(a.N)` `$ref(a.N.field)`. No forward refs. Alias pattern: `[a-zA-Z][a-zA-Z0-9_-]*`

REPEAT — `repeat:` list of mappings merged with `fields` (repeat overrides). Results stored as array under alias; access via `$ref(alias.N)`.

ERROR — `on_error: continue` at document or operation level. Operation overrides document. Default: stop on first failure.

---

EXAMPLE:
```yaml
# Shortcut Template Language (STL) v1
# Validate: sc template validate <this-file>
# Run:      sc template run <this-file> --confirm
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
      - name: "Add rate limiting"
        estimate: 3

  - action: comment
    entity: story
    id: $ref(stories.0)
    fields:
      text: "Linked to epic $ref(my-epic)"
```
