# ADR-0002 — Scaffold modules by cloning a skeleton repo, not embedded templates

- **Status:** Accepted
- **Reader:** Maintainer
- **Supersedes:** the embedded-template approach in `src/templates/module/` + `template_processor.rs`
  path helpers (now dead code)
- **Superseded by:** —

## Context

Originally, `module create` generated a module by walking an embedded template tree
(`src/templates/module/…`), replacing `{{PLACEHOLDER}}` tokens with `template_processor.rs`. That
tied the *module layout* to a *release of this plugin*: changing the skeleton meant editing template
files inside the codegen crate and cutting a new plugin release, and the embedded tree drifted from
what real modules actually looked like.

## Decision

`module create` now **clones a canonical skeleton repository** —
`https://github.com/faridlab/backbone-module` — shallowly (`--depth 1`), then:

1. Detaches it (removes `.git` and `Cargo.lock`).
2. Renames the package in `Cargo.toml` and sets the description.
3. Stamps `__MODULE__` → the schema-module name (Cargo name minus the `backbone-` prefix) across
   every UTF-8 file.
4. Prints next steps (register in `metaphor.yaml`, edit schema, `metaphor schema generate`).

The `backbone-module` repo is the **single source of truth** for module structure.

## Consequences

**Positive**
- Module layout evolves in its own repo, independent of plugin releases. The skeleton *is* a real,
  buildable module, so it can't silently drift from reality.
- The plugin shrinks to a clone-and-stamp operation — simpler, less template-maintenance surface.
- Contributors improve the module shape by PR-ing a normal Rust repo, not opaque template files.

**Negative**
- `module create` now **requires network access and `git` on PATH**; it fails offline. (The handler
  gives an explicit error if `git clone` fails.)
- The generated layout is no longer visible by reading this crate — you must look at the external
  repo.
- **Dead code left behind:** `src/templates/module/` and the `get_*_template_dir()` helpers in
  `template_processor.rs` are now unused (`#![allow(dead_code)]`), and `docs/templates.md`'s claim
  that `module` uses Handlebars templates is stale. Cleanup is outstanding.

## Scope

This ADR originally covered **`module create` only**. As of **v0.2.0**, `apps generate` adopts the
**same** decision: it clones the [`backbone-application`](https://github.com/faridlab/backbone-application)
skeleton and stamps the baked-in package name (`backbone-app` / `backbone_app`) into the new app name,
instead of expanding `src/templates/app/` with Handlebars. Everything above applies to `apps` too,
with these differences:

- The skeleton repo is `backbone-application`, not `backbone-module`.
- The stamped tokens are the literal package names `backbone-app` (kebab) / `backbone_app` (snake),
  not `__MODULE__`.
- `apps generate` no longer edits the workspace `Cargo.toml` — the app is registered in
  `metaphor.yaml` by the developer.

The `make` targets still use embedded `src/templates/make/…` + string replacement — those were not
changed. The Handlebars engine, its helpers, and the `src/templates/app/` tree are now dead code.
