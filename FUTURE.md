# Future Functionality

> Detailed ideas for expanding `sc` beyond its current command set (`init`, `login`, `epic`, `member`, `story`, `task`, `workflow`). Each section includes motivation, proposed CLI syntax, API endpoints, implementation notes, example output, and a complexity estimate.

---

## Table of Contents

1. [Complete Existing Commands](#1-complete-existing-commands)
2. [Iteration / Sprint Management](#2-iteration--sprint-management)
3. [Label Management](#3-label-management)
4. [Story Links & Relationships](#4-story-links--relationships)
5. [Comments](#5-comments)
6. [Search](#6-search)
7. [Objectives & Key Results](#7-objectives--key-results)
8. [Milestones & Categories](#8-milestones--categories)
9. [Groups / Teams](#9-groups--teams)
10. [Documents](#10-documents)
11. [Projects](#11-projects)
12. [Custom Fields](#12-custom-fields)
13. [Story History & Audit](#13-story-history--audit)
14. [Output & Formatting](#14-output--formatting)
15. [UX Improvements](#15-ux-improvements)
16. [Bulk & Batch Operations](#16-bulk--batch-operations)
17. [Git Integration](#17-git-integration)
18. [Entity Templates](#18-entity-templates)
19. [Shortcut Template Language (STL)](#19-shortcut-template-language-stl)

## Completion Tracker

| Item  | Description | Status |
|:-----:|-------------|:------:|
|  1.1  | `epic create` |   ✅    |
|  1.2  | `epic get` |   ✅    |
|  1.3  | `epic delete` |   ✅    |
|  1.4  | `story list` |   ✅    |
|  1.5  | `story delete` |   ✅    |
|  1.6  | `member` enhancements |   ✅    |
|  2.1  | `iteration list` |   ✅    |
|  2.2  | `iteration create` |   ✅    |
|  2.3  | `iteration get` |   ✅    |
|  2.4  | `iteration update` |   ✅    |
|  2.5  | `iteration delete` |   ✅    |
|  2.6  | `iteration stories` |   ✅    |
|  2.7  | Sprint planning workflow |   ✅    |
|  3.1  | `label list` |        |
|  3.2  | `label create` |        |
|  3.3  | `label update` / `label delete` |        |
|  3.4  | `label stories` / `label epics` |        |
|  3.5  | Label name resolution & cache |        |
|  4.1  | `story link create` |        |
|  4.2  | `story link list` |        |
|  4.3  | `story link delete` |        |
|  4.4  | Relationship display in `story get` |        |
|  5.1  | Story comments |        |
|  5.2  | Epic comments |        |
|  5.3  | Reactions |        |
|  6.1  | `search` |        |dsa
|  7.1  | `objective list` |        |
|  7.2  | `objective create` |        |
|  7.3  | `objective get` |        |
|  7.4  | `objective update` / `objective delete` |        |
|  7.5  | `objective epics` |        |
|  8.1  | Milestone CRUD |        |
|  8.2  | Category CRUD |        |
|  9.1  | `group list` |        |
|  9.2  | `group get` |        |
|  9.3  | `group stories` |        |
|  9.4  | `group create` / `group update` |        |
| 10.1  | Document CRUD |        |
| 10.2  | Epic-document linking |        |
| 11.1  | Project CRUD |        |
| 12.1  | `custom-field list` |        |
| 12.2  | `custom-field get` |        |
| 12.3  | Display custom fields in `story get` |        |
| 12.4  | Set custom fields on `story create` / `story update` |        |
| 13.1  | `story history` |        |
| 14.1  | `--json` flag |        |
| 14.2  | `--format` flag (template-based) |        |
| 14.3  | `--color` / `--no-color` |        |
| 14.4  | `--quiet` mode |        |
| 14.5  | Table formatting |        |
| 15.1  | Shell completions |        |
| 15.2  | Interactive / wizard mode for create commands |        |
| 15.3  | Progress spinners for network calls |        |
| 15.4  | `--dry-run` flag |        |
| 15.5  | Cache management |        |
| 15.6  | Configurable defaults |        |
| 16.1  | `story bulk-update` |        |
| 16.2  | Batch create from file |        |
| 16.3  | Piping support |        |
| 17.1  | `story branch` |        |
| 17.2  | Auto-link commits to stories |        |
| 17.3  | Repository info & PR status |        |
| 18.1  | `template list` |        |
| 18.2  | `template get` |        |
| 18.3  | `template use` (create story from template) |        |
| 18.4  | `template create` / `template update` / `template delete` |        |
| 19.1  | STL CLI commands |        |
| 19.2  | STL document structure |        |
| 19.3  | STL actions |        |
| 19.4  | STL entities |        |
| 19.5  | STL cross-reference system |        |
| 19.6  | STL variables |        |
| 19.7  | STL null / unset handling |        |
| 19.8  | STL inline nested entities |        |
| 19.9  | STL bulk shorthand |        |
| 19.10 | STL execution semantics |        |
| 19.11 | STL output |        |
| 19.12 | STL complete workflow example |        |
| 19.13 | STL implementation notes |        |
| 19.14 | STL grammar summary |        |
| 19.15 | STL real-world example templates |        |

<!--
AI INSTRUCTION: When a feature listed in the Completion Tracker table above is fully
implemented, update its Status cell from " " to "✅". Only mark an item complete
when all functionality described in its corresponding section has been implemented,
tested, and committed. Do not mark partial implementations as complete.
-->

---

## 1. Complete Existing Commands

The current `epic`, `story`, `member`, and `workflow` commands cover only a subset of their possible operations. Filling these out rounds off the CLI before adding entirely new resource types.

### 1.1 `epic create`

**Motivation.** Users can update and list epics, but cannot create them from the CLI. This forces a context switch to the Shortcut web UI for one of the most common planning operations. A `create` subcommand completes the epic lifecycle.

**Proposed syntax:**

```
sc epic create --name "Q3 Platform Rewrite" \
  --description "Migrate the monolith to microservices" \
  --state "to do" \
  --deadline 2026-09-01T00:00:00Z \
  --owners @alice,@bob \
  --labels backend,infrastructure \
  --objective-ids 123,456 \
  --requested-by @carol
```

| Flag | Required | Type | Notes |
|---|---|---|---|
| `--name` | Yes | String | Epic title; maps to `CreateEpic.name` |
| `--description` | No | String | Markdown body |
| `--state` | No | String/i64 | Epic state name or ID; resolved via `getEpicWorkflow` + cache |
| `--deadline` | No | String | RFC 3339 timestamp |
| `--owners` | No | String | Comma-separated `@mention` names or UUIDs |
| `--labels` | No | String | Comma-separated label names; resolved to `CreateLabelParams` |
| `--objective-ids` | No | String | Comma-separated objective IDs |
| `--follower-ids` | No | String | Comma-separated member UUIDs or `@mention` names |
| `--requested-by` | No | String | Single `@mention` or UUID |

**API endpoint:** `POST /api/v3/epics` — operation `createEpic`

**Implementation notes:**

- Reuse the existing epic state resolution logic from `epic update` (normalize name → check `epic_state_cache.json` → fallback to `getEpicWorkflow`).
- Owner and follower flags should pass through the same `resolve_member_id` path used by `story create`, which checks the `member_cache.json` and falls back to `listMembers`.
- Labels present a new challenge: the `CreateEpic` body accepts `CreateLabelParams` (with `name`, optional `color`, optional `description`), not label IDs. If a label name is provided that already exists, the API will link the existing label. If the name is new, the API creates it. This means no local label resolution is needed — pass names through directly.
- The `--requested-by` flag is singular (one member). If omitted, the API defaults to the authenticated user.

**Example output:**

```
Created epic 42 - Q3 Platform Rewrite
```

**Complexity:** Small — mirrors `epic update` with nearly identical flag parsing and resolution logic.

---

### 1.2 `epic get`

**Motivation.** `sc story get` exists but there is no equivalent for epics. Users have no way to inspect a single epic's details — description, owner list, labels, state, deadline, linked stories count, and completion stats — from the terminal.

**Proposed syntax:**

```
sc epic get --id 42
```

**API endpoints:**

- `GET /api/v3/epics/{epic-public-id}` — operation `getEpic` — returns full `Epic` object
- `GET /api/v3/epics/{epic-public-id}/stories` — operation `listEpicStories` — needed if we want to show a story count or summary stats

**Implementation notes:**

- The `Epic` response includes `stats` (an `EpicStats` object) with fields like `num_stories_total`, `num_stories_started`, `num_stories_done`, `num_stories_unstarted`, `num_points_done`, etc. Display these in a stats block.
- Resolve the `epic_state_id` back to a human-readable name by reverse-looking up the epic workflow. Cache the reverse mapping (`i64 → String`) alongside the existing forward cache, or build it on the fly from the same `getEpicWorkflow` response.
- Owner UUIDs should be resolved to `@mention_name` for display. Use the member cache; batch-resolve any misses with a single `listMembers` call rather than N individual `getMember` calls.
- Labels are embedded in the response as `LabelSlim` objects with `name` — no resolution needed.

**Example output:**

```
42 - Q3 Platform Rewrite
  State:       In Progress
  Deadline:    2026-09-01
  Requested:   @carol
  Owners:      @alice, @bob
  Labels:      backend, infrastructure
  Objectives:  123, 456
  Description: Migrate the monolith to microservices

  Stats:
    Stories:   12 total (3 unstarted, 7 started, 2 done)
    Points:    45 total (8 unstarted, 30 started, 7 done)
```

**Complexity:** Small — single API call plus optional stats formatting.

---

### 1.3 `epic delete`

**Motivation.** Completing the CRUD lifecycle. Epics sometimes get created by mistake or become obsolete. Deleting from the CLI avoids a trip to the web UI.

**Proposed syntax:**

```
sc epic delete --id 42
sc epic delete --id 42 --confirm    # skip interactive prompt
```

**API endpoint:** `DELETE /api/v3/epics/{epic-public-id}` — operation `deleteEpic`

**Implementation notes:**

- Destructive operation — require a confirmation prompt by default. Print the epic name and ID, then ask `Delete epic 42 - "Q3 Platform Rewrite"? [y/N]`. Read from stdin; abort on anything other than `y` or `Y`.
- A `--confirm` (or `--yes` / `-y`) flag skips the prompt for scripting use.
- On success the API returns 204 No Content. Print a confirmation message.
- Consider fetching the epic name first (via `getEpic`) so the confirmation prompt is meaningful. If the epic doesn't exist, the GET will 404 and we can report the error before prompting.

**Example output:**

```
Delete epic 42 - "Q3 Platform Rewrite"? [y/N] y
Deleted epic 42
```

**Complexity:** Small — one GET + one DELETE, plus a stdin prompt.

---

### 1.4 `story list`

**Motivation.** There is currently no way to list stories. Users must know a story's ID to interact with it. A list command with filtering makes `sc` useful for daily standups, triage, and sprint reviews.

**Proposed syntax:**

```
sc story list
sc story list --owner @alice
sc story list --state "In Progress"
sc story list --epic-id 42
sc story list --type bug
sc story list --label backend
sc story list --project-id 10
sc story list --limit 25 --offset 0
sc story list --owner @alice --state "In Progress" --type feature
```

| Flag | Type | Notes |
|---|---|---|
| `--owner` | String | `@mention` or UUID; filter to stories owned by this member |
| `--state` | String/i64 | Workflow state name or ID |
| `--epic-id` | i64 | Filter to stories in this epic |
| `--type` | String | `feature`, `bug`, or `chore` |
| `--label` | String | Label name |
| `--project-id` | i64 | Filter to stories in this project |
| `--limit` | i64 | Page size (default 25) |
| `--offset` | i64 | Pagination offset |
| `--desc` | bool | Include descriptions in output |

**API endpoints:**

- `POST /api/v3/stories/search` — operation `queryStories` — accepts a rich query body with `owner_id`, `workflow_state_id`, `epic_id`, `story_type`, `label_name`, `project_id`, `page_size`, etc.
- `GET /api/v3/epics/{epic-public-id}/stories` — operation `listEpicStories` — alternative when filtering by epic only
- `GET /api/v3/projects/{project-public-id}/stories` — operation `listStories` — alternative when filtering by project only

**Implementation notes:**

- Prefer `queryStories` as the general-purpose backend. It accepts all filter combinations in a single request. The `ListStoriesParams` body supports `owner_id` (UUID), `workflow_state_id` (i64), `epic_id` (i64), `story_type` (string), `label_name` (string), `project_id` (i64 array), and `page_size` / `next` for cursor-based pagination.
- Resolve `--owner` via member cache (same path as `story create`).
- Resolve `--state` via workflow state cache (same path as `story create`/`story update`).
- The response returns `StorySlim` objects. Display ID, name, type, and state. Optionally include description with `--desc`.
- Pagination: `queryStories` uses cursor-based pagination with a `next` token. The `--offset` flag is a simplification — internally track the cursor. If the user wants page 2 with limit 25, issue the first query with `page_size=25`, discard the results, then use the `next` token for the second query. Alternatively, expose `--next <cursor>` for power users and keep `--offset` for convenience.

**Example output:**

```
123 - Fix login bug (bug, In Progress)
124 - Add dark mode (feature, Backlog)
125 - Update dependencies (chore, Done)

Showing 3 of 47 stories. Use --limit and --offset to paginate.
```

With `--desc`:

```
123 - Fix login bug (bug, In Progress)
  The login form crashes on Safari 17 when 2FA is enabled.
124 - Add dark mode (feature, Backlog)
  Support system-level dark mode preference.
```

**Complexity:** Medium — new subcommand, query body construction, cursor pagination, multiple filter resolutions.

---

### 1.5 `story delete`

**Motivation.** Complete CRUD for stories. Duplicate or test stories need cleanup.

**Proposed syntax:**

```
sc story delete --id 123
sc story delete --id 123 --confirm
```

**API endpoint:** `DELETE /api/v3/stories/{story-public-id}` — operation `deleteStory`

**Implementation notes:**

- Same confirmation prompt pattern as `epic delete`: fetch the story name first, prompt, then delete.
- The API returns 204 No Content on success.

**Example output:**

```
Delete story 123 - "Fix login bug"? [y/N] y
Deleted story 123
```

**Complexity:** Small.

---

### 1.6 `member` Enhancements

**Motivation.** The current `member` command only lists and looks up members. Adding `--list` with filtering (by role, disabled status) and displaying richer detail (profile picture URL, created date) makes team management more useful.

**Proposed enhancements:**

```
sc member --list --role admin
sc member --list --active          # exclude disabled members
```

| Flag | Type | Notes |
|---|---|---|
| `--role` | String | Filter: `admin`, `member`, `owner`, `observer` |
| `--active` | bool | Exclude disabled members |

**API endpoint:** `GET /api/v3/members` — operation `listMembers` (already used; just add client-side filtering)

**Implementation notes:**

- The `listMembers` response includes `role` and `disabled` fields. Apply filters client-side after fetching.
- No new API calls needed — this is purely display logic.

**Example output:**

```
sc member --list --role admin
12345678-... - @alice - Alice Smith (admin)
87654321-... - @carol - Carol Jones (admin)
```

**Complexity:** Small — client-side filtering on existing data.

---

## 2. Iteration / Sprint Management

**Motivation.** Iterations (sprints) are central to agile workflows. Teams using Shortcut's iteration feature currently have no CLI access to sprint planning, assignment, or review. This is one of the highest-value additions because it connects daily story work to sprint cadences.

### 2.1 `sc iteration list`

**Proposed syntax:**

```
sc iteration list
sc iteration list --state started     # started, done, unstarted
sc iteration list --desc
```

**API endpoint:** `GET /api/v3/iterations` — operation `listIterations`

**Implementation notes:**

- The response returns `IterationSlim` objects with `id`, `name`, `status` (started/done/unstarted), `start_date`, `end_date`, `stats`.
- Filter by `--state` client-side (the API doesn't support server-side filtering for list).
- Display: ID, name, status, date range.

**Example output:**

```
101 - Sprint 23 (started, 2026-02-16 → 2026-03-01)
100 - Sprint 22 (done, 2026-02-02 → 2026-02-15)
 99 - Sprint 21 (done, 2026-01-19 → 2026-02-01)
```

---

### 2.2 `sc iteration create`

**Proposed syntax:**

```
sc iteration create --name "Sprint 24" \
  --start-date 2026-03-02 \
  --end-date 2026-03-15 \
  --description "Focus on performance improvements"
```

| Flag | Required | Type | Notes |
|---|---|---|---|
| `--name` | Yes | String | Sprint name |
| `--start-date` | Yes | String | ISO 8601 date (YYYY-MM-DD) |
| `--end-date` | Yes | String | ISO 8601 date (YYYY-MM-DD) |
| `--description` | No | String | Sprint goal / description |
| `--follower-ids` | No | String | Comma-separated member `@mention` names or UUIDs |
| `--labels` | No | String | Comma-separated label names |
| `--group-ids` | No | String | Comma-separated group UUIDs |

**API endpoint:** `POST /api/v3/iterations` — operation `createIteration`

**Implementation notes:**

- The `CreateIteration` body requires `name`, `start_date`, `end_date`. Optional: `description`, `follower_ids`, `labels`, `group_ids`.
- Dates are date-only strings (not full RFC 3339). Validate format before sending.
- Follower IDs go through member resolution. Labels use `CreateLabelParams` (same as epic create — pass names through).

**Example output:**

```
Created iteration 102 - Sprint 24 (2026-03-02 → 2026-03-15)
```

---

### 2.3 `sc iteration get`

**Proposed syntax:**

```
sc iteration get --id 101
```

**API endpoint:** `GET /api/v3/iterations/{iteration-public-id}` — operation `getIteration`

**Implementation notes:**

- The full `Iteration` object includes `stats` with `num_stories_done`, `num_stories_started`, `num_stories_unstarted`, `num_points_done`, `num_points_started`, `num_points_unstarted`, `average_cycle_time`, `average_lead_time`.
- Display the iteration details plus a stats summary. This gives teams a quick sprint health check.

**Example output:**

```
101 - Sprint 23 (started)
  Start:       2026-02-16
  End:         2026-03-01
  Description: Focus on auth hardening

  Stats:
    Stories:      18 total (4 unstarted, 10 started, 4 done)
    Points:       55 total (12 unstarted, 30 started, 13 done)
    Avg Cycle:    2.3 days
    Avg Lead:     4.1 days
```

---

### 2.4 `sc iteration update`

**Proposed syntax:**

```
sc iteration update --id 101 --name "Sprint 23 (Extended)" --end-date 2026-03-08
```

**API endpoint:** `PUT /api/v3/iterations/{iteration-public-id}` — operation `updateIteration`

**Implementation notes:**

- Same optional-field pattern as `epic update` and `story update`: only send fields that were explicitly provided.
- Supports updating `name`, `description`, `start_date`, `end_date`, `follower_ids`, `labels`, `group_ids`.

**Example output:**

```
Updated iteration 101 - Sprint 23 (Extended)
```

---

### 2.5 `sc iteration delete`

**Proposed syntax:**

```
sc iteration delete --id 101
sc iteration delete --id 101 --confirm
```

**API endpoint:** `DELETE /api/v3/iterations/{iteration-public-id}` — operation `deleteIteration`

**Implementation notes:**

- Same confirmation prompt pattern as `epic delete`. Fetch iteration name first, prompt, then delete.
- Stories in the iteration are NOT deleted — they become unassigned to any iteration.

**Example output:**

```
Delete iteration 101 - "Sprint 23"? [y/N] y
Deleted iteration 101
```

---

### 2.6 `sc iteration stories`

**Proposed syntax:**

```
sc iteration stories --id 101
sc iteration stories --id 101 --desc
```

**API endpoint:** `GET /api/v3/iterations/{iteration-public-id}/stories` — operation `listIterationStories`

**Implementation notes:**

- Returns `StorySlim` objects. Display in the same format as `story list`.
- Optionally add `--owner`, `--state`, `--type` flags for client-side sub-filtering within the iteration.
- This is the primary "what's in this sprint?" view.

**Example output:**

```
Stories in Sprint 23:

123 - Fix login bug (bug, In Progress, @alice)
124 - Add dark mode (feature, Backlog, @bob)
125 - Update dependencies (chore, Done, @alice)

3 stories, 13 points
```

---

### 2.7 Sprint Planning Workflow

**Proposed syntax:**

```
sc story update --id 123 --iteration-id 102    # move story into sprint
sc story update --id 123 --iteration-id 0       # remove from sprint (unset)
```

**Implementation notes:**

- This doesn't require a new command — extend `story update` to accept `--iteration-id`.
- The `UpdateStory` body already supports `iteration_id` (nullable i64). Passing `null` or a sentinel value (0) removes the story from its current iteration.
- Consider adding `--iteration` as an alias that accepts an iteration name, with resolution logic (normalize name → cache lookup → `listIterations` fallback), following the same pattern as state resolution.

**Complexity for all iteration features:** Medium — new `commands/iteration.rs` module with 6 subcommands, new `IterationSubcommand` enum in `main.rs`, iteration name cache.

---

## 3. Label Management

**Motivation.** Labels are used for cross-cutting categorization (e.g., `backend`, `frontend`, `urgent`, `tech-debt`). The current CLI accepts label names as flags on `epic update` and `story create` but has no way to browse, create, or manage labels directly. Label resolution (name → ID) would benefit story and epic commands as well.

### 3.1 `sc label list`

**Proposed syntax:**

```
sc label list
sc label list --desc
```

**API endpoint:** `GET /api/v3/labels` — operation `listLabels`

**Example output:**

```
100 - backend (#3498db)
101 - frontend (#e74c3c)
102 - urgent (#e67e22)
103 - tech-debt (#95a5a6)
```

With `--desc`:

```
100 - backend (#3498db)
  Server-side and API work
101 - frontend (#e74c3c)
  UI and browser-facing changes
```

---

### 3.2 `sc label create`

**Proposed syntax:**

```
sc label create --name "p0-critical" --color "#e74c3c" --description "Production-blocking issues"
```

| Flag | Required | Type |
|---|---|---|
| `--name` | Yes | String |
| `--color` | No | String (hex color, e.g. `#ff0000`) |
| `--description` | No | String |

**API endpoint:** `POST /api/v3/labels` — operation `createLabel`

**Example output:**

```
Created label 104 - p0-critical (#e74c3c)
```

---

### 3.3 `sc label update` / `sc label delete`

**Proposed syntax:**

```
sc label update --id 104 --name "p0" --color "#c0392b"
sc label delete --id 104
sc label delete --id 104 --confirm
```

**API endpoints:**

- `PUT /api/v3/labels/{label-public-id}` — operation `updateLabel`
- `DELETE /api/v3/labels/{label-public-id}` — operation `deleteLabel`

**Implementation notes:**

- Deletion requires confirmation. Deleting a label removes it from all stories and epics that use it.

---

### 3.4 `sc label stories` / `sc label epics`

**Proposed syntax:**

```
sc label stories --id 100
sc label epics --id 100
```

**API endpoints:**

- `GET /api/v3/labels/{label-public-id}/stories` — operation `listLabelStories`
- `GET /api/v3/labels/{label-public-id}/epics` — operation `listLabelEpics`

**Implementation notes:**

- These return `StorySlim` and `EpicSlim` arrays. Display in the same format as `story list` and `epic list`.
- Useful for answering "what's tagged as tech-debt?" without constructing a search query.

**Example output:**

```
Stories with label "backend":

123 - Fix login bug (bug, In Progress)
130 - Optimize DB queries (chore, Backlog)
145 - Add rate limiting (feature, Unstarted)
```

---

### 3.5 Label Name Resolution & Cache

**Implementation notes:**

- Add a `label_cache.json` file: `HashMap<String, i64>` mapping normalized label names to IDs.
- Follow the same normalization pattern as epic states: lowercase, replace `_-` with spaces, trim.
- Populate on first `listLabels` call; refresh on cache miss.
- This enables `--label backend` on `story list`, `epic list`, etc. to resolve to an ID for API queries.

**Complexity for all label features:** Small — straightforward CRUD with one new cache file.

---

## 4. Story Links & Relationships

**Motivation.** Stories rarely exist in isolation. Blocking relationships, duplicates, and related-story links are critical for triage and dependency tracking. The Shortcut API has full support for story links, but they're completely inaccessible from the CLI today.

### 4.1 `sc story link create`

**Proposed syntax:**

```
sc story link create --subject 123 --object 456 --verb blocks
sc story link create --subject 123 --object 456 --verb "is blocked by"
sc story link create --subject 123 --object 456 --verb duplicates
sc story link create --subject 123 --object 456 --verb "relates to"
```

| Flag | Required | Type | Notes |
|---|---|---|---|
| `--subject` | Yes | i64 | The "source" story ID |
| `--object` | Yes | i64 | The "target" story ID |
| `--verb` | Yes | String | One of: `blocks`, `is blocked by`, `duplicates`, `relates to` |

**API endpoint:** `POST /api/v3/story-links` — operation `createStoryLink`

**Implementation notes:**

- The `CreateStoryLink` body takes `subject_id` (i64), `object_id` (i64), and `verb` (string enum).
- Normalize the verb: accept shorthand (`blocks`, `blocked-by`, `duplicates`, `relates`) and map to the API's expected values (`blocks`, `is blocked by`, `duplicates`, `relates to`).
- Display both story names in the confirmation by fetching both stories (can be done in parallel).

**Example output:**

```
Linked: 123 "Fix login bug" blocks 456 "Deploy auth service"
```

---

### 4.2 `sc story link list`

**Proposed syntax:**

```
sc story link list --story-id 123
```

**Implementation notes:**

- There is no dedicated "list links for story" endpoint. Instead, the `story_links` field is embedded in the `Story` response from `getStory`.
- Parse the `story_links` array from the story response. Each `TypedStoryLink` has `subject_id`, `object_id`, `verb`, and `type`.
- To show meaningful output, resolve the linked story IDs to names. This may require N additional `getStory` calls. Consider batching or caching story name lookups.

**Example output:**

```
Links for story 123 - "Fix login bug":

  blocks      456 - Deploy auth service
  relates to  789 - Update SSL certs
  blocked by  100 - Provision staging DB
```

---

### 4.3 `sc story link delete`

**Proposed syntax:**

```
sc story link delete --id 98765
```

**API endpoint:** `DELETE /api/v3/story-links/{story-link-public-id}` — operation `deleteStoryLink`

**Implementation notes:**

- The link ID comes from the `story_links` array on a story. Users would first run `sc story link list` to find the link ID, then delete.
- Add confirmation prompt.

---

### 4.4 Relationship Display in `story get`

**Implementation notes:**

- Extend the existing `story get` output to include a "Links" section if the story has any `story_links`.
- Display each link's verb and the linked story's ID + name.
- Requires resolving linked story names (batch fetch or cache).

**Enhanced `story get` output:**

```
123 - Fix login bug
  Type:        bug
  State:       In Progress
  Epic:        42 - Q3 Platform Rewrite
  Owners:      @alice
  Labels:      backend, urgent
  Description: The login form crashes on Safari 17

  Links:
    blocks      456 - Deploy auth service
    relates to  789 - Update SSL certs

  Tasks:
    [x] 1 - Reproduce in Safari
    [ ] 2 - Write regression test
    [ ] 3 - Fix the form handler
```

**Complexity:** Medium — new `story link` subcommand group, verb normalization, linked story name resolution.

---

## 5. Comments

**Motivation.** Comments are the primary collaboration mechanism on stories and epics. Being able to post a quick update, read discussion history, or react to a comment without leaving the terminal keeps developers in flow. This is especially valuable for remote teams who use comments for async standups and code review notes.

### 5.1 Story Comments

**Proposed syntax:**

```
sc story comment list --story-id 123
sc story comment add --story-id 123 --text "Deployed the fix to staging"
sc story comment add --story-id 123 --text-file ./notes.md
sc story comment get --story-id 123 --id 456
sc story comment update --story-id 123 --id 456 --text "Updated: fix verified on staging"
sc story comment delete --story-id 123 --id 456
```

| Flag | Type | Notes |
|---|---|---|
| `--story-id` | i64 | Required for all comment operations |
| `--id` | i64 | Comment ID (for get/update/delete) |
| `--text` | String | Comment body (Markdown) |
| `--text-file` | Path | Read comment body from file (useful for long comments) |

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/stories/{id}/comments` | `listStoryComment` |
| Create | POST | `/api/v3/stories/{id}/comments` | `createStoryComment` |
| Get | GET | `/api/v3/stories/{id}/comments/{cid}` | `getStoryComment` |
| Update | PUT | `/api/v3/stories/{id}/comments/{cid}` | `updateStoryComment` |
| Delete | DELETE | `/api/v3/stories/{id}/comments/{cid}` | `deleteStoryComment` |

**Implementation notes:**

- Comment listing should show author (`@mention_name`), timestamp, and a truncated preview of the text. Full text on `get`.
- Author UUIDs need member resolution. Cache all resolved members to avoid repeated lookups.
- The `--text-file` flag reads the file contents at the given path and uses it as the comment body. Validate that the file exists and is readable.
- The `CreateStoryComment` body takes `text` (string) and optionally `author_id` (UUID) — omit `author_id` to default to the authenticated user.
- Timestamps from the API are RFC 3339. Display as relative time ("2 hours ago") or local time, based on a future `--time-format` preference.

**Example output (`list`):**

```
Comments on story 123 - "Fix login bug":

  #456 @alice (2 hours ago)
    Deployed the fix to staging. Running smoke tests now.

  #457 @bob (1 hour ago)
    Smoke tests passed. LGTM to promote to prod.

  #458 @alice (30 minutes ago)
    Promoted to production. Monitoring for 30 min.
```

**Example output (`get`):**

```
Comment #456 on story 123
  Author:  @alice
  Created: 2026-02-19T14:30:00Z
  Updated: 2026-02-19T14:32:00Z

  Deployed the fix to staging. Running smoke tests now.

  Verified on:
  - Chrome 120
  - Safari 17
  - Firefox 121
```

---

### 5.2 Epic Comments

**Proposed syntax:**

```
sc epic comment list --epic-id 42
sc epic comment add --epic-id 42 --text "Sprint review notes: on track for Q3"
sc epic comment get --epic-id 42 --id 789
sc epic comment update --epic-id 42 --id 789 --text "Updated review notes"
sc epic comment delete --epic-id 42 --id 789
```

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/epics/{id}/comments` | `listEpicComments` |
| Create | POST | `/api/v3/epics/{id}/comments` | `createEpicComment` |
| Get | GET | `/api/v3/epics/{id}/comments/{cid}` | `getEpicComment` |
| Update | PUT | `/api/v3/epics/{id}/comments/{cid}` | `updateEpicComment` |
| Delete | DELETE | `/api/v3/epics/{id}/comments/{cid}` | `deleteEpicComment` |

**Implementation notes:**

- Epic comments also support threaded replies via `POST /api/v3/epics/{id}/comments/{cid}` (operation `createEpicCommentComment`). The thread parent is specified by the comment ID in the URL path.
- Display threaded comments with indentation.

**Example output (threaded):**

```
Comments on epic 42 - "Q3 Platform Rewrite":

  #789 @carol (3 days ago)
    Sprint review: 7/12 stories done. Auth service migration blocking.

    #790 @alice (3 days ago)
      Auth migration will be done by Wednesday. I'll update the story.

    #791 @bob (2 days ago)
      Confirmed — auth service deployed to staging.
```

---

### 5.3 Reactions

**Proposed syntax:**

```
sc story comment react --story-id 123 --comment-id 456 --emoji thumbsup
sc story comment unreact --story-id 123 --comment-id 456 --emoji thumbsup
```

**API endpoints:**

- `POST /api/v3/stories/{id}/comments/{cid}/reactions` — operation `createStoryReaction`
- `DELETE /api/v3/stories/{id}/comments/{cid}/reactions` — operation `deleteStoryReaction`

**Implementation notes:**

- The `CreateReaction` body takes `emoji` (string). The API uses standard emoji short codes (e.g., `thumbsup`, `heart`, `rocket`).
- Display reaction counts on comment listings: `[👍 3] [🚀 1]`.
- Lower priority than text comments — could be deferred.

**Complexity for all comment features:** Medium — two new subcommand groups (story comment, epic comment) with 5 operations each, member resolution for authors, timestamp formatting, optional threading/reactions.

---

## 6. Search

**Motivation.** Search is the fastest way to find anything in a large Shortcut workspace. The API provides both a unified search endpoint and type-specific search endpoints. A CLI search command replaces the need to open a browser, and pairs powerfully with pipes and scripts.

### 6.1 `sc search`

**Proposed syntax:**

```
sc search "login bug"
sc search stories "login bug"
sc search epics "Q3"
sc search iterations "Sprint 2"
sc search documents "architecture"
sc search milestones "v2"
sc search objectives "revenue"
```

| Positional | Required | Notes |
|---|---|---|
| Resource type | No | `stories`, `epics`, `iterations`, `documents`, `milestones`, `objectives`. Omit for unified search. |
| Query | Yes | Free-text search string |

| Flag | Type | Notes |
|---|---|---|
| `--limit` | i64 | Max results (default 25) |
| `--page-size` | i64 | Results per page |
| `--next` | String | Cursor token for next page |
| `--quiet` | bool | Output IDs only (for piping) |

**API endpoints:**

| Type | Method | Path | Operation ID |
|---|---|---|---|
| Unified | GET | `/api/v3/search` | `search` |
| Stories | GET | `/api/v3/search/stories` | `searchStories` |
| Epics | GET | `/api/v3/search/epics` | `searchEpics` |
| Iterations | GET | `/api/v3/search/iterations` | `searchIterations` |
| Documents | GET | `/api/v3/search/documents` | `searchDocuments` |
| Milestones | GET | `/api/v3/search/milestones` | `searchMilestones` |
| Objectives | GET | `/api/v3/search/objectives` | `searchObjectives` |

**Implementation notes:**

- All search endpoints accept a `query` parameter and return paginated results with a `next` cursor.
- The unified `search` endpoint returns results grouped by type. Display each group with a header.
- Type-specific endpoints return arrays of the corresponding `Slim` types.
- The `--quiet` flag outputs only IDs (one per line), enabling piping: `sc search stories "bug" --quiet | xargs -I{} sc story get --id {}`.
- Shortcut's search supports advanced syntax: `owner:@alice state:"In Progress" type:bug`. Document this in the help text.

**Example output (unified):**

```
sc search "login"

Stories (3 results):
  123 - Fix login bug (bug, In Progress)
  150 - Login page redesign (feature, Backlog)
  167 - Login rate limiting (feature, Unstarted)

Epics (1 result):
  42 - Authentication Overhaul

Documents (1 result):
  88 - Login Flow Architecture
```

**Example output (type-specific, quiet):**

```
sc search stories "login" --quiet
123
150
167
```

**Complexity:** Medium — one new `commands/search.rs` module, unified and type-specific modes, pagination, quiet mode.

---

## 7. Objectives & Key Results

**Motivation.** Objectives (formerly "Milestones" in some Shortcut contexts) represent high-level strategic goals. They're used to align epics with business outcomes. Teams that use OKR frameworks need CLI access to track progress against objectives without context-switching.

### 7.1 `sc objective list`

**Proposed syntax:**

```
sc objective list
sc objective list --desc
sc objective list --archived    # include archived objectives
```

**API endpoint:** `GET /api/v3/objectives` — operation `listObjectives`

**Example output:**

```
200 - Improve platform reliability
201 - Grow enterprise revenue 30%
202 - Reduce onboarding time to < 5 min
```

---

### 7.2 `sc objective create`

**Proposed syntax:**

```
sc objective create --name "Reduce P1 incidents by 50%" \
  --description "Focus on monitoring, alerting, and runbook automation" \
  --categories 10,11
```

| Flag | Required | Type |
|---|---|---|
| `--name` | Yes | String |
| `--description` | No | String |
| `--categories` | No | String (comma-separated category IDs) |

**API endpoint:** `POST /api/v3/objectives` — operation `createObjective`

**Example output:**

```
Created objective 203 - Reduce P1 incidents by 50%
```

---

### 7.3 `sc objective get`

**Proposed syntax:**

```
sc objective get --id 200
```

**API endpoint:** `GET /api/v3/objectives/{objective-public-id}` — operation `getObjective`

**Implementation notes:**

- The `Objective` response includes `stats` with completion data.
- Also fetch associated epics via `GET /api/v3/objectives/{id}/epics` (operation `listObjectiveEpics`) to show which epics are driving this objective.

**Example output:**

```
200 - Improve platform reliability
  Description: Reduce downtime and improve MTTR
  Categories:  10 - Engineering, 11 - SRE
  Archived:    false

  Associated Epics:
    42 - Q3 Platform Rewrite (In Progress)
    55 - Monitoring Overhaul (To Do)
    60 - Incident Response Automation (Done)

  Stats:
    Epics: 3 total (1 to do, 1 in progress, 1 done)
```

---

### 7.4 `sc objective update` / `sc objective delete`

**Proposed syntax:**

```
sc objective update --id 200 --name "Reduce P1 incidents by 75%" --archived true
sc objective delete --id 200
sc objective delete --id 200 --confirm
```

**API endpoints:**

- `PUT /api/v3/objectives/{id}` — operation `updateObjective`
- `DELETE /api/v3/objectives/{id}` — operation `deleteObjective`

---

### 7.5 `sc objective epics`

**Proposed syntax:**

```
sc objective epics --id 200
```

**API endpoint:** `GET /api/v3/objectives/{id}/epics` — operation `listObjectiveEpics`

**Example output:**

```
Epics for objective 200 - "Improve platform reliability":

42 - Q3 Platform Rewrite (In Progress)
55 - Monitoring Overhaul (To Do)
60 - Incident Response Automation (Done)
```

**Complexity for all objective features:** Small-Medium — straightforward CRUD, one associated-epics view.

---

## 8. Milestones & Categories

**Motivation.** Milestones group epics into high-level deliverables with target dates. Categories organize milestones and objectives into themes. Together they form the top of Shortcut's planning hierarchy.

### 8.1 Milestone CRUD

**Proposed syntax:**

```
sc milestone list
sc milestone create --name "V2 Launch" --description "Public launch of V2" --categories 10
sc milestone get --id 300
sc milestone update --id 300 --name "V2 Launch (Delayed)"
sc milestone delete --id 300
sc milestone delete --id 300 --confirm
```

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/milestones` | `listMilestones` |
| Create | POST | `/api/v3/milestones` | `createMilestone` |
| Get | GET | `/api/v3/milestones/{id}` | `getMilestone` |
| Update | PUT | `/api/v3/milestones/{id}` | `updateMilestone` |
| Delete | DELETE | `/api/v3/milestones/{id}` | `deleteMilestone` |
| Epics | GET | `/api/v3/milestones/{id}/epics` | `listMilestoneEpics` |

**Implementation notes:**

- `milestone get` should include associated epics (from `listMilestoneEpics`).
- Categories are referenced by ID. Add `--categories` flag accepting comma-separated category IDs.
- `CreateMilestone` requires `name`. Optional: `description`, `categories`, `state` (one of `in progress`, `to do`, `done`).

**Example output (`get`):**

```
300 - V2 Launch
  State:       In Progress
  Categories:  10 - Product
  Description: Public launch of V2 platform

  Epics:
    42 - Q3 Platform Rewrite (In Progress)
    43 - V2 Migration Guide (To Do)
```

---

### 8.2 Category CRUD

**Proposed syntax:**

```
sc category list
sc category create --name "Engineering" --description "Technical initiatives" --color "#3498db"
sc category get --id 10
sc category update --id 10 --name "Platform Engineering"
sc category delete --id 10
```

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/categories` | `listCategories` |
| Create | POST | `/api/v3/categories` | `createCategory` |
| Get | GET | `/api/v3/categories/{id}` | `getCategory` |
| Update | PUT | `/api/v3/categories/{id}` | `updateCategory` |
| Delete | DELETE | `/api/v3/categories/{id}` | `deleteCategory` |
| Milestones | GET | `/api/v3/categories/{id}/milestones` | `listCategoryMilestones` |
| Objectives | GET | `/api/v3/categories/{id}/objectives` | `listCategoryObjectives` |

**Implementation notes:**

- `category get` should list associated milestones and objectives.
- `CreateCategory` takes `name`, `color` (hex string), and `type` (string; usually "milestone").

**Example output (`get`):**

```
10 - Engineering (#3498db)
  Type: milestone
  Description: Technical initiatives

  Milestones:
    300 - V2 Launch (In Progress)
    301 - Infrastructure Upgrade (To Do)

  Objectives:
    200 - Improve platform reliability
    203 - Reduce P1 incidents by 50%
```

**Complexity:** Small — standard CRUD for both resource types.

---

## 9. Groups / Teams

**Motivation.** Groups represent teams within a Shortcut workspace (e.g., "Backend Team", "Mobile Squad"). They're used for story assignment, iteration scoping, and workload visibility. CLI access enables quick team-level views without navigating the web UI.

### 9.1 `sc group list`

**Proposed syntax:**

```
sc group list
sc group list --desc
```

**API endpoint:** `GET /api/v3/groups` — operation `listGroups`

**Example output:**

```
aaaaaaaa-... - Backend Team (3 members)
bbbbbbbb-... - Mobile Squad (5 members)
cccccccc-... - Platform SRE (2 members)
```

---

### 9.2 `sc group get`

**Proposed syntax:**

```
sc group get --id aaaaaaaa-...
```

**API endpoint:** `GET /api/v3/groups/{group-public-id}` — operation `getGroup`

**Implementation notes:**

- The `Group` response includes `member_ids` (array of UUIDs). Resolve to `@mention_name` via member cache.
- Also includes `num_stories_started`, `num_epics_started`, etc.

**Example output:**

```
Backend Team (aaaaaaaa-...)
  Description: Server-side development
  Members:     @alice, @bob, @carol
  Workflows:   500000001 - Development

  Stats:
    Stories started: 12
    Epics started:   3
```

---

### 9.3 `sc group stories`

**Proposed syntax:**

```
sc group stories --id aaaaaaaa-...
sc group stories --id aaaaaaaa-... --state "In Progress"
```

**API endpoint:** `GET /api/v3/groups/{group-public-id}/stories` — operation `listGroupStories`

**Implementation notes:**

- Returns `StorySlim` objects. Display in standard story list format.
- Client-side filtering by `--state`, `--owner`, `--type` for convenience.
- Useful for "what is my team working on right now?" views.

**Example output:**

```
Stories for Backend Team:

123 - Fix login bug (bug, In Progress, @alice)
130 - Optimize DB queries (chore, Started, @bob)
145 - Add rate limiting (feature, In Progress, @carol)

3 stories
```

---

### 9.4 `sc group create` / `sc group update`

**Proposed syntax:**

```
sc group create --name "DevOps" \
  --description "Infrastructure and deployment" \
  --member-ids @alice,@bob
sc group update --id aaaaaaaa-... --name "Platform Team" --member-ids @alice,@bob,@dave
```

**API endpoints:**

- `POST /api/v3/groups` — operation `createGroup`
- `PUT /api/v3/groups/{id}` — operation `updateGroup`

**Implementation notes:**

- Member IDs go through member resolution (accept `@mention` names).
- `CreateGroup` requires `name`, optional `description`, `member_ids`, `workflow_ids`, `mention_name`.
- The `mention_name` field sets the team's `@mention` handle (e.g., `@backend-team`). If omitted, Shortcut auto-generates one.

**Example output:**

```
Created group cccccccc-... - DevOps
```

**Complexity:** Small-Medium — CRUD plus member resolution for group membership.

---

## 10. Documents

**Motivation.** Shortcut Documents (Docs) are rich-text pages that live alongside stories and epics. They're used for technical specs, RFCs, architecture decisions, and meeting notes. CLI access enables quick browsing and creation, plus linking docs to epics for traceability.

### 10.1 Document CRUD

**Proposed syntax:**

```
sc doc list
sc doc create --name "Auth Service RFC" --content-file ./rfc.md
sc doc get --id 88
sc doc update --id 88 --name "Auth Service RFC (v2)" --content-file ./rfc-v2.md
sc doc delete --id 88
sc doc delete --id 88 --confirm
```

| Flag | Type | Notes |
|---|---|---|
| `--name` | String | Document title |
| `--content` | String | Inline content |
| `--content-file` | Path | Read content from file |

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/documents` | `listDocs` |
| Create | POST | `/api/v3/documents` | `createDoc` |
| Get | GET | `/api/v3/documents/{id}` | `getDoc` |
| Update | PUT | `/api/v3/documents/{id}` | `updateDoc` |
| Delete | DELETE | `/api/v3/documents/{id}` | `deleteDoc` |

**Implementation notes:**

- Documents have a `content` field (Markdown/rich text). For `create` and `update`, accept either `--content` (inline) or `--content-file` (read from disk).
- `doc get` should display the full content, rendered as plain text (strip any Shortcut-specific formatting).
- List output shows ID, name, and creator.

**Example output (`get`):**

```
88 - Auth Service RFC
  Created:  2026-01-15T10:00:00Z by @alice
  Updated:  2026-02-10T14:30:00Z

  # Auth Service RFC

  ## Overview
  This document describes the migration from session-based auth
  to JWT tokens for the platform API...
```

---

### 10.2 Epic-Document Linking

**Proposed syntax:**

```
sc doc link --doc-id 88 --epic-id 42
sc doc unlink --doc-id 88 --epic-id 42
sc doc epics --id 88              # list epics linked to this doc
sc epic docs --id 42              # list docs linked to this epic
```

**API endpoints:**

- `PUT /api/v3/documents/{doc-id}/epics/{epic-id}` — operation `linkDocumentToEpic`
- `DELETE /api/v3/documents/{doc-id}/epics/{epic-id}` — operation `unlinkDocumentFromEpic`
- `GET /api/v3/documents/{doc-id}/epics` — operation `listDocumentEpics`
- `GET /api/v3/epics/{epic-id}/documents` — operation `listEpicDocuments`

**Example output:**

```
Linked document 88 "Auth Service RFC" to epic 42 "Q3 Platform Rewrite"
```

```
Epics linked to document 88 - "Auth Service RFC":

42 - Q3 Platform Rewrite (In Progress)
55 - Security Hardening (To Do)
```

**Complexity:** Medium — CRUD with file I/O for content, plus linking subcommands.

---

## 11. Projects

**Motivation.** Projects in Shortcut are organizational containers that group stories (separate from epics, which represent initiatives). Many workspaces use projects to represent codebases, services, or team boundaries. CLI access enables project-scoped story views and creation.

### 11.1 Project CRUD

**Proposed syntax:**

```
sc project list
sc project create --name "API Service" --description "Backend REST API" --team-id aaaaaaaa-...
sc project get --id 10
sc project update --id 10 --name "API Service v2"
sc project delete --id 10
sc project delete --id 10 --confirm
```

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| List | GET | `/api/v3/projects` | `listProjects` |
| Create | POST | `/api/v3/projects` | `createProject` |
| Get | GET | `/api/v3/projects/{id}` | `getProject` |
| Update | PUT | `/api/v3/projects/{id}` | `updateProject` |
| Delete | DELETE | `/api/v3/projects/{id}` | `deleteProject` |
| Stories | GET | `/api/v3/projects/{id}/stories` | `listStories` |

**Implementation notes:**

- `CreateProject` requires `name`. Optional: `description`, `team_id` (group UUID), `abbreviation`, `color`.
- `project get` should show project details plus a story count.
- `project stories --id 10` (or `sc project stories --id 10`) lists stories scoped to the project, using the same display format as `story list`.

**Example output (`list`):**

```
10 - API Service (15 stories)
11 - Web Frontend (32 stories)
12 - Mobile App (8 stories)
```

**Example output (`get`):**

```
10 - API Service
  Description:  Backend REST API
  Team:         aaaaaaaa-... - Backend Team
  Abbreviation: API
  Archived:     false
  Stories:      15 (3 done, 8 in progress, 4 unstarted)
```

**Complexity:** Small — standard CRUD with one stories subview.

---

## 12. Custom Fields

**Motivation.** Many Shortcut workspaces define custom fields for domain-specific metadata — e.g., "Customer Impact", "Sprint Points Override", "Risk Level". These fields are invisible in the current CLI, making story views incomplete for teams that rely on them.

### 12.1 `sc custom-field list`

**Proposed syntax:**

```
sc custom-field list
```

**API endpoint:** `GET /api/v3/custom-fields` — operation `listCustomFields`

**Implementation notes:**

- Each `CustomField` has `id`, `name`, `field_type` (text, number, enum), `description`, `enabled`, and `values` (for enum types).
- Display the field name, type, and possible values.

**Example output:**

```
500 - Customer Impact (enum)
  Values: none, low, medium, high, critical
501 - Risk Level (enum)
  Values: low, medium, high
502 - External Ticket (text)
```

---

### 12.2 `sc custom-field get`

**Proposed syntax:**

```
sc custom-field get --id 500
```

**API endpoint:** `GET /api/v3/custom-fields/{id}` — operation `getCustomField`

---

### 12.3 Display Custom Fields in `story get`

**Implementation notes:**

- The `Story` response includes `custom_fields` — an array of `StoryCustomField` objects, each with `field_id`, `value_id`, and `value`.
- Resolve field names from the custom fields list (cache in `custom_field_cache.json`).
- For enum fields, resolve `value_id` to the value label.
- Display custom fields as additional rows in `story get` output.

**Enhanced `story get` output:**

```
123 - Fix login bug
  Type:            bug
  State:           In Progress
  Epic:            42 - Q3 Platform Rewrite
  Owners:          @alice
  Labels:          backend, urgent
  Customer Impact: high
  Risk Level:      medium
  External Ticket: JIRA-1234
  Description:     The login form crashes on Safari 17
```

---

### 12.4 Set Custom Fields on `story create` / `story update`

**Proposed syntax:**

```
sc story create --name "New bug" --custom-field "Customer Impact=high" --custom-field "Risk Level=low"
sc story update --id 123 --custom-field "Customer Impact=critical"
```

**Implementation notes:**

- The `--custom-field` flag is repeatable. Each value is `"Field Name=Value"`.
- Resolve field name → field ID via cache. For enum fields, resolve value name → value ID.
- Build the `custom_fields` array in the create/update request body.
- Error if the field name doesn't exist or the value is invalid for the field type.

**Complexity:** Medium — cache for custom field definitions, value resolution for enums, repeatable flag parsing.

---

## 13. Story History & Audit

**Motivation.** Understanding what changed, when, and by whom is critical for debugging issues, audit trails, and accountability. The Shortcut API exposes a rich history endpoint for stories that is currently inaccessible from the CLI.

### 13.1 `sc story history`

**Proposed syntax:**

```
sc story history --id 123
sc story history --id 123 --limit 10
```

**API endpoint:** `GET /api/v3/stories/{story-public-id}/history` — operation `storyHistory`

**Implementation notes:**

- The response is an array of `History` objects, each with `changed_at` (timestamp), `member_id` (who made the change), and `actions` (array of `HistoryAction` objects describing what changed).
- Each `HistoryAction` has `action` (string, e.g., "update"), `entity_type`, `name`, `changes` (object with `old`/`new` values for each changed field).
- Resolve `member_id` to `@mention_name` via member cache.
- Display as a chronological timeline.
- For field changes, show old → new values. For state changes, resolve state IDs to names.

**Example output:**

```
History for story 123 - "Fix login bug":

  2026-02-19 14:30 @alice
    Created story
    Type: bug
    State: Backlog
    Owner: @alice

  2026-02-19 15:00 @bob
    State: Backlog → In Progress

  2026-02-19 16:45 @alice
    Labels: added "urgent"
    Estimate: (none) → 5

  2026-02-20 09:00 @alice
    State: In Progress → Done
    Description: updated
```

**Implementation notes (continued):**

- The history response can be large for stories with many changes. The `--limit` flag truncates to the N most recent entries.
- Some changes are noisy (e.g., automatic position updates). Consider filtering out `position` changes by default, with a `--verbose` flag to include everything.
- State ID → name resolution should use the workflow state cache.

**Complexity:** Medium — new subcommand, history response parsing, field change formatting, state resolution.

---

## 14. Output & Formatting

**Motivation.** The current CLI outputs human-readable text only. For scripting, automation, and integration with other tools, machine-readable output is essential. Fine-grained formatting control also improves the experience for different terminal configurations and accessibility needs.

### 14.1 `--json` Flag

**Proposed syntax:**

```
sc story get --id 123 --json
sc epic list --json
sc search stories "login" --json
```

**Implementation notes:**

- Add a global `--json` flag (or per-subcommand) that outputs the raw API response as pretty-printed JSON.
- This is the simplest machine-readable format to implement: just `serde_json::to_string_pretty` the response object.
- Useful for piping into `jq`: `sc story get --id 123 --json | jq '.workflow_state_id'`.
- Consider making this a global flag in the top-level `Cli` struct so it's available everywhere.

**Example output:**

```json
{
  "id": 123,
  "name": "Fix login bug",
  "story_type": "bug",
  "workflow_state_id": 500000011,
  "epic_id": 42,
  "estimate": 5,
  "owner_ids": ["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"],
  "labels": [{"id": 100, "name": "backend"}, {"id": 102, "name": "urgent"}],
  "description": "The login form crashes on Safari 17"
}
```

---

### 14.2 `--format` Flag (Template-Based)

**Proposed syntax:**

```
sc story list --format "{id}\t{name}\t{state}"
sc epic list --format "{id} [{state}] {name}"
sc member --list --format "{mention_name}"
```

**Implementation notes:**

- Accept a format string with `{field_name}` placeholders.
- Parse the format string at runtime, replace placeholders with values from the response object.
- Field names correspond to the API response fields (e.g., `id`, `name`, `story_type`, `workflow_state_id`).
- This is more complex than `--json` but gives users precise control over output layout.
- Consider supporting a few built-in presets: `--format table` (default), `--format csv`, `--format tsv`.

---

### 14.3 `--color` / `--no-color`

**Proposed syntax:**

```
sc epic list --no-color           # disable ANSI color codes
sc story get --id 123 --color     # force color even when piped
```

**Implementation notes:**

- Currently the CLI does not use colors. Adding color would enhance readability:
  - Story types: feature (green), bug (red), chore (yellow)
  - Workflow states: unstarted (dim), started (cyan), done (green)
  - Labels: use the label's `color` hex value
  - Member mentions: bold
- Use a crate like `colored` or `owo-colors` for ANSI color support.
- Auto-detect TTY: color on by default for terminals, off when piped. `--color` forces on, `--no-color` forces off.
- Respect the `NO_COLOR` environment variable (https://no-color.org/).

---

### 14.4 `--quiet` Mode

**Proposed syntax:**

```
sc story list --quiet             # output IDs only, one per line
sc search stories "bug" --quiet
sc epic list --quiet
```

**Implementation notes:**

- Output only the primary ID of each result, one per line.
- Designed for piping: `sc story list --owner @alice --quiet | xargs -I{} sc story update --id {} --state done`.
- Simple to implement: just change the print format to `println!("{}", item.id)`.

---

### 14.5 Table Formatting

**Implementation notes:**

- The current `workflow --id` command already does basic column alignment with dynamic padding.
- Extend this to all list commands with a consistent table formatter.
- Calculate column widths based on the longest value in each column.
- Consider using the `comfy-table` or `tabled` crate for automatic table formatting with borders, alignment, and wrapping.
- Truncate long fields (descriptions, names) to terminal width with `...`.

**Example (polished table output):**

```
sc story list --owner @alice

  ID   Type     State        Name
  123  bug      In Progress  Fix login bug
  130  chore    Started      Optimize DB queries
  145  feature  In Progress  Add rate limiting
```

**Complexity:** Medium — `--json` is small, `--format` templates are medium, color support is medium, table formatting is small. Total: medium-large if done all at once, but each piece is independently useful and can be shipped incrementally.

---

## 15. UX Improvements

**Motivation.** Quality-of-life features that make the CLI faster and more pleasant to use. These don't add new Shortcut API coverage but significantly improve the daily experience.

### 15.1 Shell Completions

**Proposed syntax:**

```
sc completions bash > ~/.bash_completion.d/sc
sc completions zsh > ~/.zfunc/_sc
sc completions fish > ~/.config/fish/completions/sc.fish
```

**Implementation notes:**

- Clap 4 has built-in completion generation via `clap_complete`. Add a hidden `completions` subcommand that writes the completion script to stdout.
- Completions cover subcommands, flags, and flag values.
- For dynamic completions (e.g., `--state` completing to actual workflow state names), use `clap_complete`'s custom completer feature or generate a static list from cached data.
- Dynamic completion for `--owner` could suggest `@mention_name` values from the member cache.

**Complexity:** Small — Clap provides most of the infrastructure.

---

### 15.2 Interactive / Wizard Mode for Create Commands

**Proposed syntax:**

```
sc story create --interactive
sc epic create -i
```

**Implementation notes:**

- When `--interactive` (or `-i`) is passed, prompt the user for each field instead of requiring flags.
- Use a crate like `dialoguer` or `inquire` for rich interactive prompts:
  - Text input for name, description
  - Select/autocomplete for state, type, owner, labels
  - Confirm before submitting
- Pre-populate with default values where sensible.
- This lowers the barrier for new users who don't know all the flag names.

**Example session:**

```
$ sc story create --interactive
Name: Fix the login bug
Type: (feature/bug/chore) bug
Owner: @alice
State: In Progress
Epic ID (optional): 42
Estimate (optional): 5
Labels (comma-separated, optional): backend, urgent
Description (optional): The login form crashes on Safari 17

Create story "Fix the login bug"? [Y/n] y
Created story 123 - Fix the login bug
```

**Complexity:** Medium — requires an interactive prompt library, conditional logic for each field.

---

### 15.3 Progress Spinners for Network Calls

**Implementation notes:**

- API calls can take 200ms–2s. A spinner provides feedback that the CLI is working.
- Use a crate like `indicatif` for terminal spinners.
- Show the spinner on stderr so it doesn't interfere with piped stdout.
- Automatically hide the spinner when the call completes.
- Consider a global `--no-progress` flag to disable spinners in scripts.

**Example:**

```
$ sc story list --owner @alice
⠋ Fetching stories...
123 - Fix login bug (bug, In Progress)
...
```

**Complexity:** Small — wrap API calls in a spinner context.

---

### 15.4 `--dry-run` Flag

**Proposed syntax:**

```
sc story create --name "Test" --type bug --dry-run
sc epic update --id 42 --state done --dry-run
```

**Implementation notes:**

- Print the API request that *would* be sent without actually sending it.
- Show the HTTP method, URL, and request body (as JSON).
- Useful for debugging, learning the API, and scripting validation.

**Example output:**

```
[dry-run] POST /api/v3/stories
{
  "name": "Test",
  "story_type": "bug"
}
```

**Complexity:** Small — intercept before the API call and print instead.

---

### 15.5 Cache Management

**Proposed syntax:**

```
sc cache clear                # delete all cache files
sc cache clear --type members # delete only member cache
sc cache clear --type states  # delete only workflow state cache
sc cache refresh              # re-fetch and rebuild all caches
sc cache show                 # display cache file locations and sizes
```

**Implementation notes:**

- Cache files live in `~/.sc/projects/<hash>/cache/`. Enumerate files in this directory.
- `clear` deletes files. `refresh` calls the relevant list endpoints and writes new caches.
- `show` displays the cache directory path, file names, sizes, and last-modified timestamps.
- Useful when cached data becomes stale (e.g., new team member added, workflow states changed).

**Example output (`show`):**

```
Cache directory: /Users/alice/.sc/projects/a1b2c3d4/cache/

  member_cache.json          1.2 KB  (2026-02-18 10:30)
  workflow_state_cache.json  0.4 KB  (2026-02-17 14:00)
  epic_state_cache.json      0.2 KB  (2026-02-19 09:00)
```

**Complexity:** Small.

---

### 15.6 Configurable Defaults

**Proposed syntax:**

```
sc config set default-workflow 500000001
sc config set default-project 10
sc config set default-owner @alice
sc config set page-size 50
sc config get default-workflow
sc config list
sc config unset default-workflow
```

**Implementation notes:**

- Store configuration in `~/.sc/projects/<hash>/config.json` (per-project) or `~/.sc/config.json` (global).
- When a flag like `--state` is omitted, check if a default workflow is configured and use it to resolve ambiguous states.
- When `--owner` is omitted on `story create`, use the default owner.
- `page-size` sets the default for all list/search commands.
- Configuration precedence: flag > project config > global config > built-in default.

**Complexity:** Medium — new config module, precedence logic, integration with all commands.

---

## 16. Bulk & Batch Operations

**Motivation.** Triage sessions, sprint planning, and cleanup tasks often involve updating many stories at once. Doing this one at a time with `sc story update` is tedious. Bulk operations save significant time.

### 16.1 `sc story bulk-update`

**Proposed syntax:**

```
# Move all "In Progress" bugs to "Done"
sc story bulk-update --state "In Progress" --type bug --set-state "Done"

# Assign all backlog features in epic 42 to @alice
sc story bulk-update --epic-id 42 --state Backlog --type feature --set-owner @alice

# Add a label to all stories owned by @bob
sc story bulk-update --owner @bob --add-label tech-debt
```

| Filter Flag | Type | Notes |
|---|---|---|
| `--state` | String/i64 | Filter by current state |
| `--type` | String | Filter by story type |
| `--owner` | String | Filter by owner |
| `--epic-id` | i64 | Filter by epic |
| `--label` | String | Filter by label |

| Update Flag | Type | Notes |
|---|---|---|
| `--set-state` | String/i64 | New state |
| `--set-owner` | String | New owner (replaces all) |
| `--set-type` | String | New type |
| `--set-epic-id` | i64 | New epic |
| `--set-estimate` | i64 | New estimate |
| `--add-label` | String | Add a label |
| `--remove-label` | String | Remove a label |
| `--set-iteration-id` | i64 | Move to iteration |

**Implementation notes:**

- First, query stories matching the filter criteria using `queryStories`.
- Display the count and ask for confirmation: `Update 12 stories? [y/N]`.
- Then issue `updateStory` for each matched story. Consider parallelizing with `tokio::join!` or `futures::stream::FuturesUnordered` (with a concurrency limit to avoid rate limiting).
- Show progress: `Updated 8/12 stories...`.
- Respect Shortcut's rate limits (200 requests/minute for most endpoints). Add a configurable delay between requests if needed.
- The `--confirm` flag skips the prompt.

**Example output:**

```
Found 12 stories matching filters:
  State: In Progress
  Type: bug

Update all to State: Done? [y/N] y

  Updated 123 - Fix login bug
  Updated 130 - Fix signup crash
  Updated 145 - Fix timeout error
  ...
Updated 12/12 stories.
```

---

### 16.2 Batch Create from File

**Proposed syntax:**

```
sc story batch-create --file stories.json
sc story batch-create --file stories.yaml
```

**Example input file (JSON):**

```json
[
  {
    "name": "Implement login endpoint",
    "type": "feature",
    "owner": "@alice",
    "state": "Backlog",
    "epic_id": 42,
    "estimate": 3,
    "labels": ["backend"]
  },
  {
    "name": "Add login form UI",
    "type": "feature",
    "owner": "@bob",
    "state": "Backlog",
    "epic_id": 42,
    "estimate": 5,
    "labels": ["frontend"]
  }
]
```

**Implementation notes:**

- Parse the input file (JSON or YAML — add `serde_yaml` dependency for YAML support).
- Resolve all member mentions and state names upfront (batch-resolve, not per-story).
- Show a summary and prompt for confirmation.
- Create stories sequentially or with limited parallelism.
- Report each created story's ID and name.

**Example output:**

```
Loaded 2 stories from stories.json

Create 2 stories? [y/N] y

  Created 200 - Implement login endpoint
  Created 201 - Add login form UI

Created 2/2 stories.
```

---

### 16.3 Piping Support

**Implementation notes:**

- The `--quiet` flag (from Section 14.4) is the primary enabler for piping.
- Ensure all commands can read IDs from stdin when `--id -` or `--ids -` is passed, or when a story ID list is piped via `xargs`.
- Common patterns:

```bash
# Close all bugs in the current sprint
sc iteration stories --id 101 --quiet | xargs -I{} sc story update --id {} --state Done

# Add a label to all search results
sc search stories "legacy" --quiet | xargs -I{} sc story update --id {} --labels legacy,tech-debt

# Get full details for all stories in an epic
sc epic stories --id 42 --quiet | xargs -I{} sc story get --id {}
```

- No special implementation needed beyond `--quiet` mode — `xargs` handles the piping.

**Complexity:** Medium for bulk-update (query + batch update + progress), Small for batch-create (file parsing + sequential creates), Small for piping (just `--quiet` mode).

---

## 17. Git Integration

**Motivation.** Developers work in git branches that correspond to Shortcut stories. Automating the branch creation, naming, and story linking reduces context-switching and ensures consistent branch naming conventions.

### 17.1 `sc story branch`

**Proposed syntax:**

```
sc story branch --id 123
sc story branch --id 123 --prefix feature
sc story branch --id 123 --checkout
```

**Implementation notes:**

- Fetch the story via `getStory`.
- Generate a branch name from the story type, ID, and slugified name: `feature/sc-123-fix-login-bug` or `bug/sc-123-fix-login-bug`.
- Naming convention: `{type}/sc-{id}-{slugified-name}`.
  - Slugify: lowercase, replace non-alphanumeric with hyphens, collapse consecutive hyphens, truncate to 50 chars.
- The `--prefix` flag overrides the type-based prefix.
- The `--checkout` flag (or `-c`) creates the branch and checks it out (`git checkout -b <branch>`).
- Without `--checkout`, just print the suggested branch name.
- Use `std::process::Command` to run git commands.

**Example output:**

```
$ sc story branch --id 123
feature/sc-123-fix-login-bug

$ sc story branch --id 123 --checkout
Created and checked out branch: feature/sc-123-fix-login-bug
```

---

### 17.2 Auto-Link Commits to Stories

**Proposed syntax:**

```
sc story commit --id 123 -m "Fix the null pointer in auth handler"
```

**Implementation notes:**

- Wrapper around `git commit` that prepends `[sc-123]` to the commit message.
- Shortcut automatically links commits with `sc-XXX` or `ch-XXX` patterns to stories when the repository is connected.
- Alternatively, detect the current branch name and extract the story ID automatically:
  - If on branch `feature/sc-123-fix-login-bug`, the story ID is `123`.
  - Prepend `[sc-123]` to the commit message automatically.
- This could also be implemented as a git hook (`prepare-commit-msg`) that `sc` installs.

**Example:**

```
$ git checkout feature/sc-123-fix-login-bug
$ sc story commit -m "Fix the null pointer in auth handler"
[feature/sc-123-fix-login-bug abc1234] [sc-123] Fix the null pointer in auth handler
```

---

### 17.3 Repository Info & PR Status

**Proposed syntax:**

```
sc story get --id 123 --branches    # show linked branches and PRs
```

**API endpoints:**

- `GET /api/v3/repositories` — operation `listRepositories`
- `GET /api/v3/repositories/{id}` — operation `getRepository`

**Implementation notes:**

- The `Story` response includes `branches` and `pull_requests` arrays (from connected VCS integrations).
- Display branch names and PR status (open/merged/closed) in the `story get` output.
- This depends on having a VCS integration configured in Shortcut (GitHub, GitLab, Bitbucket).

**Enhanced `story get` output:**

```
123 - Fix login bug
  ...
  Branches:
    feature/sc-123-fix-login-bug (github/acme/api)

  Pull Requests:
    #456 "Fix login bug" (merged) — github/acme/api
```

**Complexity:** Small for branch creation, Small for commit wrapper, Medium for PR status display (depends on VCS integration data).

---

## 18. Entity Templates

**Motivation.** Teams often create stories with the same structure: bug reports need reproduction steps, features need acceptance criteria, spikes need timeboxes. Entity templates (called "Story Templates" in the Shortcut UI) pre-fill these fields. CLI access enables template-driven story creation in scripts and automation.

### 18.1 `sc template list`

**Proposed syntax:**

```
sc template list
```

**API endpoint:** `GET /api/v3/entity-templates` — operation `listEntityTemplates`

**Example output:**

```
900 - Bug Report Template
901 - Feature Request Template
902 - Spike Template
903 - Tech Debt Template
```

---

### 18.2 `sc template get`

**Proposed syntax:**

```
sc template get --id 900
```

**API endpoint:** `GET /api/v3/entity-templates/{id}` — operation `getEntityTemplate`

**Implementation notes:**

- The `EntityTemplate` includes a `story_contents` object with pre-filled `name`, `description`, `story_type`, `labels`, `estimate`, `owner_ids`, `workflow_state_id`, etc.
- Display the template's pre-filled values so users can see what they'll get.

**Example output:**

```
900 - Bug Report Template
  Story Type: bug
  Labels:     bug
  Estimate:   (none)

  Description Template:
    ## Steps to Reproduce
    1. ...

    ## Expected Behavior
    ...

    ## Actual Behavior
    ...

    ## Environment
    - OS:
    - Browser:
    - Version:
```

---

### 18.3 `sc template use` (Create Story from Template)

**Proposed syntax:**

```
sc template use --id 900 --name "Safari login crash" --owner @alice
sc template use --id 901 --name "Add dark mode" --owner @bob --epic-id 42
```

**Implementation notes:**

- Fetch the template via `getEntityTemplate`.
- Extract the `story_contents` and use it as the base for a `createStory` call.
- Override any fields specified by flags (`--name`, `--owner`, `--state`, etc.).
- The `--name` flag is required (templates usually have placeholder names).
- All other flags from `story create` should be accepted and override template values.

**Example output:**

```
Created story 250 - Safari login crash (from template "Bug Report Template")
```

---

### 18.4 `sc template create` / `sc template update` / `sc template delete`

**Proposed syntax:**

```
sc template create --name "Ops Incident Template" \
  --story-type bug \
  --labels ops,incident \
  --description-file ./incident-template.md

sc template update --id 900 --name "Bug Report Template (v2)"

sc template delete --id 903
sc template delete --id 903 --confirm
```

**API endpoints:**

| Operation | Method | Path | Operation ID |
|---|---|---|---|
| Create | POST | `/api/v3/entity-templates` | `createEntityTemplate` |
| Update | PUT | `/api/v3/entity-templates/{id}` | `updateEntityTemplate` |
| Delete | DELETE | `/api/v3/entity-templates/{id}` | `deleteEntityTemplate` |

**Implementation notes:**

- `CreateEntityTemplate` requires `name` and a `story_contents` object. Build the `story_contents` from the provided flags.
- The `--description-file` flag reads the template description from a file (useful for multi-line templates with Markdown structure).

**Complexity:** Small-Medium — template CRUD is standard, but `template use` requires merging template defaults with user overrides.

---

## 19. Shortcut Template Language (STL)

**Motivation.** As AI assistants become central to developer workflows, there is an opportunity for `sc` to act as an execution engine for AI-generated plans. A user describes what they want ("set up Sprint 24 with auth hardening stories"), an AI like Claude writes a structured `.sc.yml` template, and `sc template run` executes it against the Shortcut API. This also benefits non-AI use cases: teams can check reusable templates into version control for repeatable sprint setups, onboarding checklists, and release processes.

STL is a YAML-based DSL (domain-specific language) designed with three constraints:

1. **Claude can write it reliably.** YAML is within every major LLM's training data. The vocabulary is small and regular.
2. **Rust can parse it efficiently.** `serde_yaml` provides zero-copy deserialization into typed structs.
3. **Humans can read and audit it.** Before executing, a user can open the `.sc.yml` file and understand exactly what will happen.

---

### 19.1 CLI Commands

```
sc template run <file>                       # execute a template
sc template run <file> --dry-run             # show what would happen without executing
sc template run <file> --confirm             # skip interactive confirmation prompt
sc template run <file> --var key=value       # pass/override a variable
sc template validate <file>                  # check syntax and references without executing
cat plan.sc.yml | sc template run -          # read template from stdin
```

| Command | Description |
|---|---|
| `run` | Parse, validate, confirm, and execute all operations in order |
| `run --dry-run` | Resolve all variables and references, print the API requests that would be sent, but do not execute |
| `run --confirm` | Skip the interactive confirmation prompt (for scripting) |
| `run --var key=value` | Override a variable declared in the `vars` block; repeatable |
| `validate` | Parse the YAML, check for valid actions/entities/fields/references/variables, report errors |

---

### 19.2 Document Structure

Every `.sc.yml` file has this top-level shape:

```yaml
version: 1

# Optional metadata (informational only — not sent to the API)
meta:
  description: "Sprint 24 setup"
  author: "@alice"

# Optional variables with default values (overridable via --var)
vars:
  sprint_name: "Sprint 24"
  team: "@backend-team"

# Ordered list of operations to execute
operations:
  - action: create
    entity: story
    fields:
      name: "My story"
```

| Top-Level Key | Required | Type | Description |
|---|---|---|---|
| `version` | Yes | integer | Must be `1`. Reserved for future schema evolution. |
| `meta` | No | object | `description` and `author` fields for documentation purposes. |
| `vars` | No | object | Key-value pairs defining variables with default values. |
| `operations` | Yes | array | Ordered list of operations to execute sequentially. |

---

### 19.3 Actions

STL uses a write-only action vocabulary. Templates are for mutations, not queries — if you need to read data, use `sc` commands directly.

| Action | Description | Requires `id`? |
|---|---|---|
| `create` | Create a new entity. Returns the created entity (with its ID available for `$ref`). | No |
| `update` | Update an existing entity. | Yes |
| `delete` | Delete an entity. | Yes |
| `comment` | Add a comment to a story or epic. | Yes (story or epic ID) |
| `link` | Create a story link between two stories. | No (uses `subject_id`/`object_id` in fields) |
| `unlink` | Remove a story link. | Yes (story link ID) |
| `check` | Mark a task complete. | Yes (task ID) |
| `uncheck` | Mark a task incomplete. | Yes (task ID) |

---

### 19.4 Entities

Complete entity vocabulary with their writable fields:

#### `story`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** Story title. |
| `description` | string | Markdown body. |
| `type` | enum | `feature`, `bug`, or `chore`. |
| `owner` | string | Single `@mention` name or UUID. Resolved via `resolve_member_id`. |
| `owners` | string[] | Multiple `@mention` names or UUIDs. |
| `state` | string or i64 | Workflow state name or ID. Resolved via workflow state cache. |
| `epic_id` | i64 or `$ref` | Epic to associate with. |
| `iteration_id` | i64 or `$ref` | Iteration/sprint to assign to. |
| `project_id` | i64 or `$ref` | Project to assign to. |
| `group_id` | string | Group UUID or `@group-mention`. |
| `estimate` | i64 | Story point estimate. |
| `labels` | string[] | Label names (pass-through — API creates if new). |
| `followers` | string[] | `@mention` names or UUIDs. Resolved via `resolve_member_id`. |
| `requested_by` | string | `@mention` name or UUID. |
| `deadline` | string | RFC 3339 datetime (e.g., `2026-03-15T00:00:00Z`). |
| `custom_fields` | object | Map of custom field name → value. Resolved via custom field cache. |
| `tasks` | object[] | Inline task creation (see Section 19.8). |
| `comments` | object[] | Inline comment creation (see Section 19.8). |
| `story_links` | object[] | Inline story link creation (see Section 19.8). |

#### `epic`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `description` | string | Markdown body. |
| `state` | string or i64 | Epic state name or ID. Resolved via `resolve_epic_state_id`. |
| `deadline` | string | RFC 3339 datetime. |
| `owners` | string[] | `@mention` names or UUIDs. |
| `followers` | string[] | `@mention` names or UUIDs. |
| `requested_by` | string | `@mention` name or UUID. |
| `labels` | string[] | Label names. |
| `objective_ids` | i64[] | Associated objective IDs. |
| `milestone_id` | i64 | Associated milestone ID. |
| `group_ids` | string[] | Group UUIDs. |
| `planned_start_date` | string | ISO 8601 date. |

#### `iteration`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `start_date` | string | **Required on create.** ISO 8601 date (YYYY-MM-DD). |
| `end_date` | string | **Required on create.** ISO 8601 date (YYYY-MM-DD). |
| `description` | string | Sprint goal or description. |
| `followers` | string[] | `@mention` names or UUIDs. |
| `labels` | string[] | Label names. |
| `group_ids` | string[] | Group UUIDs. |

#### `label`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `color` | string | Hex color code (e.g., `#e74c3c`). |
| `description` | string | |

#### `objective`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `description` | string | |
| `categories` | i64[] | Category IDs. |

#### `milestone`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `description` | string | |
| `categories` | i64[] | Category IDs. |
| `state` | string | `to do`, `in progress`, or `done`. |

#### `category`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `color` | string | Hex color code. |
| `type` | string | Usually `milestone`. |
| `description` | string | |

#### `group`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `description` | string | |
| `member_ids` | string[] | Member UUIDs or `@mention` names. |
| `mention_name` | string | The group's `@mention` handle. |
| `workflow_ids` | i64[] | Associated workflow IDs. |

#### `document`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `content` | string | Inline Markdown content. |
| `content_file` | string | Path to a file whose contents become the document body. |

#### `project`

| Field | Type | Notes |
|---|---|---|
| `name` | string | **Required on create.** |
| `description` | string | |
| `team_id` | string | Group UUID. |
| `abbreviation` | string | Short project code. |
| `color` | string | Hex color code. |

#### `task`

| Field | Type | Notes |
|---|---|---|
| `story_id` | i64 or `$ref` | **Required.** The parent story. |
| `description` | string | **Required on create.** Task text. |
| `complete` | boolean | `true` to mark done, `false` (default) for open. |
| `owners` | string[] | `@mention` names or UUIDs. |

#### `comment`

| Field | Type | Notes |
|---|---|---|
| `story_id` | i64 or `$ref` | Required (or `epic_id`). The parent entity. |
| `epic_id` | i64 or `$ref` | Required (or `story_id`). The parent entity. |
| `text` | string | Comment body (Markdown). |
| `text_file` | string | Path to a file whose contents become the comment body. |

#### `story_link`

| Field | Type | Notes |
|---|---|---|
| `subject_id` | i64 or `$ref` | **Required.** The "source" story. |
| `object_id` | i64 or `$ref` | **Required.** The "target" story. |
| `verb` | string | `blocks`, `blocked-by`, `duplicates`, or `relates-to`. |

---

### 19.5 Cross-Reference System

Operations can reference results from earlier operations using **aliases** and **`$ref()`**.

#### Defining an alias

Any operation can include an `alias` field — a short name for its result:

```yaml
- action: create
  entity: epic
  alias: platform-epic
  fields:
    name: "Platform Rewrite"
```

#### Referencing an alias

Later operations use `$ref(alias)` to resolve to the created entity's primary ID:

```yaml
- action: create
  entity: story
  fields:
    name: "Design the new architecture"
    epic_id: $ref(platform-epic)
    owner: "@alice"
```

#### Field-level references

Use `$ref(alias.field)` to access a specific field of the created entity:

```yaml
- action: create
  entity: story
  fields:
    description: "Follow-up for epic created at $ref(platform-epic.app_url)"
```

#### Rules

- References are resolved **at execution time**, in operation order.
- An alias is only available after its defining operation has successfully executed.
- If a referenced alias's operation failed, the referencing operation also fails.
- Alias names must be unique within a document.
- Alias names must match `[a-zA-Z][a-zA-Z0-9_-]*`.

---

### 19.6 Variables

Variables allow templates to be parameterized and reusable.

#### Declaring variables

Declare variables in the top-level `vars` block with default values:

```yaml
vars:
  sprint_name: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"
  team: "@backend-team"
```

#### Using variables

Reference variables in any string value using `$var(name)`:

```yaml
fields:
  name: "$var(sprint_name)"
  description: "Focus areas for $var(sprint_name)"
```

Variables support inline interpolation — they can appear within larger strings:

```yaml
fields:
  name: "$var(sprint_name) — Week 1"
  description: "This iteration covers $var(start) through $var(end)."
```

#### Overriding from the command line

```bash
sc template run sprint-setup.sc.yml \
  --var sprint_name="Sprint 25" \
  --var start="2026-03-16" \
  --var end="2026-03-29"
```

CLI `--var` values override the defaults in the `vars` block.

#### Rules

- All variables must be declared in the `vars` block (even if overridden via CLI).
- Referencing an undeclared variable is a validation error.
- Variable names must match `[a-zA-Z][a-zA-Z0-9_]*`.
- Variables are resolved before execution (string substitution pass).

---

### 19.7 Null / Unset Handling

STL follows the Shortcut API's conventions for omitted vs. explicit-null fields:

| Context | Omitted field | Explicit `null` |
|---|---|---|
| `create` | Uses API default | Not meaningful (same as omit) |
| `update` | **No change** (field is not sent) | **Clears/unsets** the field |

Example — remove a story from its epic:

```yaml
- action: update
  entity: story
  id: 123
  fields:
    epic_id: null
```

This sends `{"epic_id": null}` to the API, which clears the epic association. Omitting `epic_id` entirely would leave it unchanged.

---

### 19.8 Inline Nested Entities

On `create`, stories support inline `tasks`, `comments`, and `story_links` arrays. These are created as part of the story in a single API call (the Shortcut `createStory` endpoint accepts them inline).

```yaml
- action: create
  entity: story
  fields:
    name: "Fix login bug"
    type: bug
    owner: "@alice"
    state: "In Progress"
    tasks:
      - description: "Reproduce the issue"
      - description: "Write a regression test"
      - description: "Fix the handler"
        complete: true
    comments:
      - text: "Found this during sprint review"
    story_links:
      - verb: blocks
        object_id: 456
```

Inline nested entities are only available on `create` actions for the `story` entity. For adding tasks or comments to existing stories, use separate `comment` or `create` (entity: `task`) operations.

---

### 19.9 Bulk Shorthand

The `repeat` key creates multiple similar entities from a single operation. Each entry in `repeat` is merged with the shared `fields` block (repeat values override shared fields).

```yaml
- action: create
  entity: story
  repeat:
    - { name: "API endpoint for users", owner: "@alice" }
    - { name: "API endpoint for groups", owner: "@bob" }
    - { name: "API endpoint for roles", owner: "@alice" }
  fields:
    type: feature
    state: "Backlog"
    epic_id: $ref(api-epic)
    labels: [backend]
```

This is equivalent to three separate `create` operations, each with the shared `fields` plus the per-entry overrides. The merge rule is shallow: `repeat` entry values take precedence over `fields` values for the same key.

When `repeat` is present, the operation expands to N sub-operations (one per repeat entry). If any sub-operation fails:
- With `on_error: continue` (operation-level): remaining sub-operations still execute.
- Without `on_error`: the entire repeat sequence stops at the first failure (fail-fast).

Aliases on repeat operations: if the operation has an `alias`, the alias refers to an **array** of created entities. `$ref(alias)` is not valid for array results — use `$ref(alias.0)`, `$ref(alias.1)`, etc. to reference individual results by index.

---

### 19.10 Execution Semantics

#### Operation ordering

Operations execute **sequentially** in declared order. This guarantees that `$ref()` aliases from earlier operations are available to later ones.

#### Error handling

| Scope | Default behavior | `on_error: continue` |
|---|---|---|
| Document-level | **Fail-fast**: stop at the first failed operation. Report which operation failed and list all previously completed operations. | Continue executing remaining operations. Report all failures at the end. |
| Operation-level | Inherit document-level behavior. | Skip this operation's failure and continue with the next one. |

Set document-level error handling at the top level:

```yaml
version: 1
on_error: continue

operations:
  - ...
```

Set per-operation error handling:

```yaml
- action: create
  entity: story
  on_error: continue
  fields:
    name: "This one can fail without stopping the rest"
```

#### Confirmation prompt

Before executing, `sc template run` displays a summary and prompts for confirmation:

```
Template: sprint-setup.sc.yml
Description: Sprint 24 setup

Will execute 8 operations:
  create  1 iteration
  create  1 epic
  create  4 stories
  update  1 story
  comment 1 story

Proceed? [y/N]
```

The `--confirm` flag skips this prompt.

#### Dry-run

`sc template run <file> --dry-run` resolves all variables and references (using placeholder IDs for entities that would be created), then prints each operation's resolved API request:

```
[1/8] POST /api/v3/iterations
  {"name": "Sprint 24", "start_date": "2026-03-02", "end_date": "2026-03-15"}

[2/8] POST /api/v3/epics
  {"name": "Auth Hardening", "state": "In Progress", ...}

[3/8] POST /api/v3/stories
  {"name": "Implement JWT refresh tokens", "epic_id": "<epic from op #2>", ...}

...
```

---

### 19.11 Output

#### Normal execution

Each completed operation prints a result line matching the existing CLI format:

```
[1/8] Created iteration 102 - Sprint 24
[2/8] Created epic 55 - Auth Hardening
[3/8] Created story 200 - Implement JWT refresh tokens
[4/8] Created story 201 - Add rate limiting to login endpoint
[5/8] Created story 202 - Audit session management
[6/8] Updated story 500
[7/8] Added comment to story 500
[8/8] Created story 203 - Write auth documentation

Executed 8/8 operations successfully.
```

#### On failure (fail-fast)

```
[1/3] Created iteration 102 - Sprint 24
[2/3] FAILED: create epic — 422 Unprocessable Entity: name is required

Executed 1/3 operations (1 failed).
Completed: operation 1 (create iteration 102)
```

#### `--json` flag

`sc template run <file> --json` outputs structured JSON results for all operations:

```json
{
  "operations": [
    {
      "index": 0,
      "action": "create",
      "entity": "iteration",
      "status": "success",
      "result": { "id": 102, "name": "Sprint 24" }
    },
    {
      "index": 1,
      "action": "create",
      "entity": "epic",
      "status": "success",
      "result": { "id": 55, "name": "Auth Hardening" }
    }
  ],
  "summary": {
    "total": 8,
    "succeeded": 8,
    "failed": 0
  }
}
```

---

### 19.12 Complete Workflow Example

A full sprint setup template demonstrating variables, cross-references, inline tasks, bulk shorthand, and mixed actions:

```yaml
version: 1

meta:
  description: "Set up Sprint 24 with planned stories"
  author: "@alice"

vars:
  sprint: "Sprint 24"
  start: "2026-03-02"
  end: "2026-03-15"

operations:
  # --- Create the sprint iteration ---
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint)"
      start_date: "$var(start)"
      end_date: "$var(end)"
      description: "Focus on auth hardening and performance"

  # --- Create the auth hardening epic ---
  - action: create
    entity: epic
    alias: auth-epic
    fields:
      name: "Auth Hardening"
      state: "In Progress"
      owners: ["@alice", "@carol"]
      labels: [security, backend]

  # --- Bulk-create stories for the epic ---
  - action: create
    entity: story
    repeat:
      - name: "Implement JWT refresh tokens"
        owner: "@alice"
        estimate: 5
      - name: "Add rate limiting to login endpoint"
        owner: "@carol"
        estimate: 3
      - name: "Audit session management"
        owner: "@alice"
        estimate: 8
    fields:
      type: feature
      state: "Backlog"
      epic_id: $ref(auth-epic)
      iteration_id: $ref(sprint)
      labels: [security]

  # --- Move an existing story into this sprint and close it ---
  - action: update
    entity: story
    id: 500
    fields:
      state: "Done"
      iteration_id: $ref(sprint)

  # --- Add a closing comment to the updated story ---
  - action: comment
    entity: story
    id: 500
    fields:
      text: "Moved to $var(sprint) and marked done — was completed last sprint."
```

---

### 19.13 Implementation Notes

#### Parsing & deserialization

- Parse `.sc.yml` files with the `serde_yaml` crate.
- Deserialize into a typed `Template` struct:

```rust
struct Template {
    version: u32,
    meta: Option<Meta>,
    vars: Option<HashMap<String, serde_yaml::Value>>,
    on_error: Option<ErrorHandling>,
    operations: Vec<Operation>,
}

struct Operation {
    action: Action,
    entity: Entity,
    alias: Option<String>,
    id: Option<IdOrRef>,
    on_error: Option<ErrorHandling>,
    fields: Option<serde_yaml::Mapping>,
    repeat: Option<Vec<serde_yaml::Mapping>>,
}
```

#### Two-pass execution

1. **Validation pass** — parse YAML, check `version`, validate all action/entity combinations, verify field names per entity, check that all `$ref()` targets have corresponding aliases, check that all `$var()` references have corresponding `vars` entries, type-check field values.
2. **Execution pass** — resolve variables (string substitution), execute operations in order, resolve `$ref()` at runtime after the aliased operation completes, apply field resolution (member mentions, state names, etc.).

#### Resolution reuse

Reuse ALL existing resolution logic from the `sc` codebase:
- `resolve_member_id` — for `@mention` → UUID resolution in `owner`, `owners`, `followers`, `requested_by`, `member_ids`
- `resolve_workflow_state_id` — for state name → ID resolution in story `state`
- `resolve_epic_state_id` — for epic state name → ID resolution in epic `state`
- Label pass-through — label names are passed directly to the API (which creates-or-matches)

#### Module structure

- Add `commands/template.rs` with a `TemplateSubcommand` enum (`Run`, `Validate`).
- Add `template/` module with `parser.rs`, `validator.rs`, `executor.rs`, `resolver.rs`.
- The executor creates the appropriate API request structs (reusing existing types from the `shortcut_api` module) and calls the same API client methods as the interactive CLI commands.

#### Validation checks

`sc template validate` verifies:
- Valid YAML syntax
- `version` is `1`
- All `action` values are known
- All `entity` values are known
- All fields per entity are known and correctly typed
- All `$ref()` targets have corresponding `alias` definitions earlier in the operation list
- All `$var()` references have corresponding entries in `vars`
- No duplicate alias names
- Required fields are present for `create` actions (e.g., `name` for stories, epics, iterations)

---

### 19.14 Grammar Summary

A compact reference for the STL syntax. This is designed for AI systems (like Claude) to use as a quick-reference when generating templates:

```
DOCUMENT     = version: 1, meta?: META, vars?: VARS, on_error?: "continue",
               operations: OPERATION[]
META         = description?: string, author?: string
VARS         = { key: default_value, ... }
OPERATION    = action: ACTION, entity: ENTITY, alias?: string, id?: ID,
               on_error?: "continue", fields?: FIELDS, repeat?: FIELDS[]
ACTION       = create | update | delete | comment | link | unlink | check | uncheck
ENTITY       = story | epic | iteration | label | objective | milestone |
               category | group | document | project | task | comment | story_link
ID           = integer | $ref(alias)
FIELDS       = { field_name: VALUE, ... }
VALUE        = string | integer | boolean | null | VALUE[] | $ref(alias) |
               $ref(alias.field) | $var(name) | "text $var(name) text"
RESOLUTION   = @mention → member UUID | state_name → state ID |
               epic_state_name → epic state ID | label_name → pass-through
```

---

### 19.15 Real-World Example Templates

#### 19.15.1 Sprint Planning

Create a new sprint iteration, an epic, and a set of stories with tasks — all cross-referenced.

```yaml
version: 1
meta:
  description: "Weekly sprint planning template"
  author: "@pm"

vars:
  sprint_name: "Sprint 24"
  start_date: "2026-03-02"
  end_date: "2026-03-15"
  team_epic: "Auth Hardening"

operations:
  # Create the sprint
  - action: create
    entity: iteration
    alias: sprint
    fields:
      name: "$var(sprint_name)"
      start_date: "$var(start_date)"
      end_date: "$var(end_date)"
      description: "Sprint goal: harden authentication layer"

  # Create the focus epic
  - action: create
    entity: epic
    alias: epic
    fields:
      name: "$var(team_epic)"
      state: "In Progress"
      owners: ["@alice", "@carol"]
      labels: [security]

  # Create stories with inline tasks
  - action: create
    entity: story
    alias: jwt-story
    fields:
      name: "Implement JWT refresh tokens"
      type: feature
      owner: "@alice"
      estimate: 5
      state: "Backlog"
      epic_id: $ref(epic)
      iteration_id: $ref(sprint)
      labels: [security, backend]
      tasks:
        - description: "Research refresh token best practices"
        - description: "Implement token rotation endpoint"
        - description: "Add refresh token to auth middleware"
        - description: "Write integration tests"

  - action: create
    entity: story
    fields:
      name: "Add rate limiting to login endpoint"
      type: feature
      owner: "@carol"
      estimate: 3
      state: "Backlog"
      epic_id: $ref(epic)
      iteration_id: $ref(sprint)
      labels: [security, backend]
      tasks:
        - description: "Choose rate limiting strategy (sliding window vs token bucket)"
        - description: "Implement middleware"
        - description: "Add monitoring dashboard"
```

#### 19.15.2 Bug Triage Batch

Bulk-create bug stories from a triage session, assigning owners, labels, and priority custom fields.

```yaml
version: 1
meta:
  description: "Bugs triaged on 2026-02-19"
  author: "@triage-lead"

vars:
  sprint: "Sprint 23"

operations:
  - action: create
    entity: story
    repeat:
      - name: "Login crash on Safari 17 with 2FA"
        owner: "@alice"
        estimate: 5
        custom_fields:
          Customer Impact: high
          Risk Level: medium
      - name: "Signup form rejects valid email addresses"
        owner: "@bob"
        estimate: 3
        custom_fields:
          Customer Impact: medium
          Risk Level: low
      - name: "Password reset email not sent for SSO users"
        owner: "@carol"
        estimate: 8
        custom_fields:
          Customer Impact: high
          Risk Level: high
      - name: "Session timeout not respected on mobile web"
        owner: "@alice"
        estimate: 3
        custom_fields:
          Customer Impact: low
          Risk Level: low
    fields:
      type: bug
      state: "Backlog"
      labels: [bug, triage-2026-02-19]
```

#### 19.15.3 Team Onboarding

Create a set of onboarding stories with checklists for a new team member.

```yaml
version: 1
meta:
  description: "Onboarding checklist for new engineers"
  author: "@engineering-manager"

vars:
  new_hire: "@dave"
  buddy: "@alice"
  start_date: "2026-03-01"

operations:
  - action: create
    entity: epic
    alias: onboarding
    fields:
      name: "Onboarding: $var(new_hire)"
      description: "Onboarding tasks for $var(new_hire), starting $var(start_date)"
      state: "In Progress"
      owners: ["$var(new_hire)", "$var(buddy)"]
      labels: [onboarding]

  - action: create
    entity: story
    fields:
      name: "Environment setup"
      type: chore
      owner: "$var(new_hire)"
      epic_id: $ref(onboarding)
      tasks:
        - description: "Clone all team repositories"
        - description: "Install development dependencies"
        - description: "Run the test suite locally"
        - description: "Set up IDE with team settings"
        - description: "Get VPN and staging access"

  - action: create
    entity: story
    fields:
      name: "Team introductions and context"
      type: chore
      owner: "$var(buddy)"
      epic_id: $ref(onboarding)
      tasks:
        - description: "Schedule 1:1s with each team member"
        - description: "Walk through system architecture"
        - description: "Review team norms and on-call rotation"
        - description: "Review open epics and current sprint goals"

  - action: create
    entity: story
    fields:
      name: "First contribution"
      type: feature
      owner: "$var(new_hire)"
      epic_id: $ref(onboarding)
      description: "Pick a small story from the backlog, implement it, and get it reviewed."
      tasks:
        - description: "Pick a story with buddy's help"
        - description: "Implement the change"
        - description: "Write tests"
        - description: "Submit PR and get review"
        - description: "Deploy to staging"
```

#### 19.15.4 Epic Teardown

Move stories out of a defunct epic, then delete it.

```yaml
version: 1
meta:
  description: "Tear down the deprecated Notifications epic"
  author: "@pm"

vars:
  target_epic: 42
  destination_epic: 55

operations:
  # Move stories to the new epic
  - action: update
    entity: story
    id: 200
    fields:
      epic_id: $var(destination_epic)

  - action: update
    entity: story
    id: 201
    fields:
      epic_id: $var(destination_epic)

  - action: update
    entity: story
    id: 202
    fields:
      epic_id: $var(destination_epic)

  # Add a comment explaining the move
  - action: comment
    entity: epic
    id: $var(target_epic)
    fields:
      text: "Tearing down this epic. Stories moved to epic $var(destination_epic)."

  # Delete the old epic
  - action: delete
    entity: epic
    id: $var(target_epic)
```

#### 19.15.5 Release Checklist

Create stories for release steps and link them in a blocking chain.

```yaml
version: 1
meta:
  description: "Release checklist for v2.1.0"
  author: "@release-manager"

vars:
  version: "v2.1.0"

operations:
  - action: create
    entity: epic
    alias: release
    fields:
      name: "Release $var(version)"
      state: "In Progress"
      labels: [release]

  - action: create
    entity: story
    alias: freeze
    fields:
      name: "$var(version): Code freeze"
      type: chore
      state: "Backlog"
      epic_id: $ref(release)
      tasks:
        - description: "Cut release branch"
        - description: "Announce code freeze in Slack"
        - description: "Update version numbers"

  - action: create
    entity: story
    alias: qa
    fields:
      name: "$var(version): QA pass"
      type: chore
      state: "Backlog"
      epic_id: $ref(release)
      tasks:
        - description: "Run full regression suite"
        - description: "Perform manual smoke tests"
        - description: "Verify staging deployment"

  - action: create
    entity: story
    alias: deploy
    fields:
      name: "$var(version): Production deploy"
      type: chore
      state: "Backlog"
      epic_id: $ref(release)
      tasks:
        - description: "Deploy to production"
        - description: "Run post-deploy health checks"
        - description: "Monitor error rates for 30 minutes"

  - action: create
    entity: story
    alias: announce
    fields:
      name: "$var(version): Announce release"
      type: chore
      state: "Backlog"
      epic_id: $ref(release)
      tasks:
        - description: "Write release notes"
        - description: "Post to Slack #engineering"
        - description: "Update public changelog"
        - description: "Notify affected customers"

  # Link stories in a blocking chain: freeze → QA → deploy → announce
  - action: link
    entity: story_link
    fields:
      subject_id: $ref(freeze)
      object_id: $ref(qa)
      verb: blocks

  - action: link
    entity: story_link
    fields:
      subject_id: $ref(qa)
      object_id: $ref(deploy)
      verb: blocks

  - action: link
    entity: story_link
    fields:
      subject_id: $ref(deploy)
      object_id: $ref(announce)
      verb: blocks
```

#### 19.15.6 Sprint Retrospective Cleanup

Close completed stories, add closing comments, and move incomplete work to the next sprint.

```yaml
version: 1
meta:
  description: "Sprint 23 retro cleanup"
  author: "@scrum-master"

vars:
  current_sprint: 101
  next_sprint: 102

operations:
  # Mark completed stories as done
  - action: update
    entity: story
    id: 300
    fields:
      state: "Done"

  - action: update
    entity: story
    id: 301
    fields:
      state: "Done"

  - action: update
    entity: story
    id: 302
    fields:
      state: "Done"

  # Add retro comments to done stories
  - action: comment
    entity: story
    id: 300
    fields:
      text: "Completed in Sprint 23. Shipped to production on 2026-02-28."

  - action: comment
    entity: story
    id: 301
    fields:
      text: "Completed in Sprint 23. Needs monitoring for a week."

  # Move incomplete stories to next sprint
  - action: update
    entity: story
    id: 400
    fields:
      iteration_id: $var(next_sprint)

  - action: update
    entity: story
    id: 401
    fields:
      iteration_id: $var(next_sprint)

  # Add carryover comments
  - action: comment
    entity: story
    id: 400
    fields:
      text: "Carried over from Sprint 23 → Sprint 24. Blocked on API dependency."

  - action: comment
    entity: story
    id: 401
    fields:
      text: "Carried over from Sprint 23 → Sprint 24. Needs design review."
```

---

## Implementation Priority Recommendation

Grouped by impact and effort to help guide sequencing:

### High Impact, Low Effort
1. **`--json` flag** (Section 14.1) — unlocks scripting/automation
2. **`story list`** (Section 1.4) — most-requested missing command
3. **`epic create` / `epic get` / `epic delete`** (Sections 1.1–1.3) — completes epic CRUD
4. **`story delete`** (Section 1.5) — completes story CRUD
5. **Shell completions** (Section 15.1) — major UX win, mostly auto-generated
6. **`--quiet` mode** (Section 14.4) — enables piping patterns

### High Impact, Medium Effort
7. **Iteration management** (Section 2) — core sprint workflow
8. **Search** (Section 6) — find anything fast
9. **Story comments** (Section 5.1) — primary collaboration tool
10. **Label management** (Section 3) — cross-cutting organization
11. **Bulk update** (Section 16.1) — triage and cleanup efficiency
12. **Custom fields display** (Section 12.3) — complete story views
13. **Shortcut Template Language** (Section 19) — AI-driven automation and repeatable workflows; depends on `--json` flag and most CRUD commands being implemented first

### Medium Impact, Low Effort
14. **Cache management** (Section 15.5) — debugging convenience
15. **`--dry-run` flag** (Section 15.4) — safe experimentation
16. **Story links** (Section 4) — dependency tracking
17. **Template list/use** (Sections 18.1–18.3) — templated creation
18. **Color output** (Section 14.3) — visual clarity

### Medium Impact, Medium Effort
19. **Git integration** (Section 17) — developer workflow
20. **Projects** (Section 11) — organizational containers
21. **Groups / Teams** (Section 9) — team-scoped views
22. **Documents** (Section 10) — spec/RFC management
23. **Story history** (Section 13) — audit trail
24. **Interactive mode** (Section 15.2) — guided creation
25. **`--format` templates** (Section 14.2) — custom output layouts

### Lower Priority
26. **Objectives** (Section 7) — strategic alignment (less frequent use)
27. **Milestones & Categories** (Section 8) — high-level planning
28. **Batch create from file** (Section 16.2) — niche but powerful
29. **Configurable defaults** (Section 15.6) — convenience
30. **Progress spinners** (Section 15.3) — polish
31. **Epic comments** (Section 5.2) — less frequent than story comments
32. **Template create/update/delete** (Section 18.4) — template management (web UI is fine)
