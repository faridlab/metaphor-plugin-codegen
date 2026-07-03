# `routes` Command

> **Reader:** App developer. **Mode:** Reference + How-to.
> List the HTTP routes defined in a project by scanning its source. Read-only — it never edits code.

The `routes` command answers "what endpoints does this project actually expose?" by scanning `.rs`
files for route definitions and printing them. It reads source text; it does not compile or run the
project, so it works on code that doesn't build yet.

## Usage

```
metaphor-codegen routes [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--path <PATH>` | `src` | Directory to scan recursively for `.rs` files |
| `--format <FORMAT>` | `table` | Output format: `table`, `list`, `json`, `markdown` |
| `--filter <FILTER>` | — | Only show routes whose **path** contains this substring |

Routes are sorted by path, then method. Symlinks are not followed. If the path doesn't exist the
command errors; if no routes are found it prints a warning and exits successfully.

## What it detects

Two patterns are recognized:

**1. Raw Axum routes** — `.route("/path", method(handler))`, including chained methods:

```rust
Router::new()
    .route("/health", get(health))
    .route("/users/:id", get(get_user).delete(delete_user))
```

Each method in a chain becomes its own row. A `.route(...)` whose method can't be parsed is reported
with method `ANY` and no handler.

**2. Backbone CRUD handlers** — `BackboneCrudHandler::<_>::routes(service, "/base")`. A single call
**expands to the full 15-endpoint CRUD surface** mounted at the base path:

| Method | Path suffix | Meaning | Method | Path suffix | Meaning |
|--------|-------------|---------|--------|-------------|---------|
| GET | `` | list | GET | `/trash` | list deleted |
| POST | `` | create | POST | `/:id/restore` | restore |
| GET | `/:id` | get by id | DELETE | `/empty` | empty trash |
| PUT | `/:id` | full update | GET | `/:id/deleted` | get deleted by id |
| PATCH | `/:id` | partial update | DELETE | `/trash/:id` | permanent delete |
| DELETE | `/:id` | soft delete | GET | `/count` | count active |
| POST | `/bulk` | bulk create | GET | `/trash/count` | count deleted |
| POST | `/upsert` | upsert | | | |

> This table mirrors `BACKBONE_CRUD_ENDPOINTS` in `src/commands/routes.rs`, kept in sync with
> `backbone-core`. If the CRUD surface changes upstream, that constant (and this table) must follow.

## Output formats

### `table` (default)

Human-readable, color-coded by method (GET green, POST yellow, PUT/PATCH blue, DELETE red), with a
total count. Given a file containing the two patterns above plus a `/payments` CRUD mount:

```
METHOD  PATH                   HANDLER            SOURCE
------  ---------------------  -----------------  ------
GET     /health                health             api.rs
GET     /payments              list               api.rs
POST    /payments              create             api.rs
DELETE  /payments/:id          soft delete        api.rs
...
DELETE  /users/:id             delete_user        api.rs
GET     /users/:id             get_user           api.rs

Total: 18 routes
```

### `list`

Terse `METHOD path`, one per line — good for piping/grep:

```bash
metaphor-codegen routes --format list
```

### `markdown`

A Markdown table (Method / Path / Handler / Source) — paste straight into a PR or doc.

### `json`

Machine-readable array; each object has `method`, `path`, `handler` (nullable), `source`:

```bash
metaphor-codegen routes --format json --filter users
```

```json
[
  { "handler": "delete_user", "method": "DELETE", "path": "/users/:id", "source": "api.rs" },
  { "handler": "get_user",    "method": "GET",    "path": "/users/:id", "source": "api.rs" }
]
```

## Recipes

```bash
# All routes in the current project
metaphor-codegen routes

# Only the payments endpoints
metaphor-codegen routes --filter payments

# Scan a specific module and emit Markdown for a PR description
metaphor-codegen routes --path libs/modules/payments/src --format markdown

# Feed the route list into jq
metaphor-codegen routes --format json | jq '.[] | select(.method=="POST") | .path'
```

## Limitations

- **Nested routers are not followed.** `Router::nest("/prefix", inner)` is not resolved — inner
  routes are reported at their literal mount path, without the `/prefix`. Compose the final path
  mentally until this is supported.
- **Static scan, not semantic.** Routes built dynamically (loops, macros, computed paths) or through
  helper functions the regex doesn't recognize won't appear. What you see is what's literally in the
  source text.
- Only `.rs` files are scanned; symlinks are skipped.

---

**See also:** [Command index](index.md#command-reference) · the CRUD surface lives in
`backbone-core` (see [glossary: Repository](glossary.md#domain-driven-design)).
