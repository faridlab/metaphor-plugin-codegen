# metaphor-plugin-codegen — Handbook

> The code-generation plugin for the Metaphor CLI. Binary: `metaphor-codegen`.
> One concern: turn intent (a module name, an entity, a schema) into DDD/Clean-Architecture
> Rust scaffolding, migrations, proto, and seed data — so you write behavior, not boilerplate.

This is the handbook. It is organized by *who you are* and *what you came to do*. Start with the
row that matches you.

| You are a… | Start here | Then |
|------------|-----------|------|
| **Evaluator** — deciding whether to adopt | [Philosophy](philosophy.md) | [Background & prior art](background.md) → [Technology & the "why"](technology.md) |
| **App developer** — generating code today | [README quickstart](../README.md) | [Command reference](#command-reference) → [Configuration](configuration.md) |
| **Maintainer** — extending the plugin | [Maintainer guide](maintainer-guide.md) | [Generated-code architecture](architecture.md) → [ADRs](#architecture-decision-records) |
| **Contributor** — opening a PR | [Contribution guide](contributing.md) | [Glossary](glossary.md) |

Unsure of a term? Every capitalized concept in these docs is defined once in the
[Glossary](glossary.md).

---

## The handbook, in order

### Explanation — understand *why* (Evaluator / Maintainer)

1. **[Philosophy & motivation](philosophy.md)** — the problem this refuses to let you solve by
   hand, the worldview (schema-first, convention-over-configuration, regen-safe), and the
   non-goals.
2. **[Background & prior art](background.md)** — Laravel Artisan, Rails generators, Prisma,
   Nx/Turborepo: what this borrows and what it deliberately rejects.
3. **[Technology & the "why"](technology.md)** — the dependency list with a one-line rationale
   and a rejected alternative for each choice.
4. **[Generated-code architecture](architecture.md)** — the DDD 4-layer shape, CQRS, and
   proto-first design that *the output* follows. (This describes the code the plugin emits, not
   the plugin's own internals — for those, see the maintainer guide.)

### How-to & tutorial — get a task done (App developer)

5. **[README quickstart](../README.md)** — install → first module → first generated command in
   under 15 minutes.
6. **Command reference** (see below) — one page per command group.
7. **[Configuration](configuration.md)** — environment variables, `DATABASE_URL`, `.env`
   discovery, `metaphor.yaml` registration.

### Maintenance & contribution (Maintainer / Contributor)

8. **[Maintainer guide](maintainer-guide.md)** — how the plugin dispatches commands, how
   templating actually works (and where it's moving), and a step-by-step walkthrough of adding a
   new `make` target without breaking conventions.
9. **[Contribution guide](contributing.md)** — dev setup, commit conventions, tests, the
   tag-driven release flow, and the PR checklist.

### Reference for everyone

10. **[Glossary](glossary.md)** — the ubiquitous language: one term, one meaning.
11. **[Architecture Decision Records](#architecture-decision-records)** — why the load-bearing
    choices were made.

---

## Command reference

`metaphor-codegen` exposes seven command groups. Run `metaphor-codegen <group> --help` for the
authoritative surface — the pages below explain intent and recipes.

| Group | What it does | Page |
|-------|-------------|------|
| `make` | Laravel-style scaffolding for DDD components (command, query, repository, handler, service, event, value-object, spec, …) | [commands-make.md](commands-make.md) |
| `module` | Scaffold a bounded-context module by cloning the canonical skeleton; list / info / enable / disable / install | [commands-module.md](commands-module.md) |
| `apps` | Generate a runnable Clean-Architecture application | [commands-apps.md](commands-apps.md) |
| `proto` | Protocol Buffer operations (buf / tonic) | [commands-proto.md](commands-proto.md) |
| `migration` | PostgreSQL migration lifecycle (generate, run, status, diff, seed) | [commands-migration.md](commands-migration.md) |
| `seed` | Database seeding and test data | [commands-seed.md](commands-seed.md) |
| `routes` | List HTTP routes discovered in a project (table / list / json / markdown) | [commands-routes.md](commands-routes.md) |

---

## Architecture Decision Records

Load-bearing decisions, one record each. Immutable once accepted — superseded, never edited.

- [ADR-0001 — Subprocess plugin dispatch](adr/0001-subprocess-plugin-dispatch.md)
- [ADR-0002 — Scaffold modules by cloning a skeleton repo, not embedded templates](adr/0002-skeleton-clone-scaffolding.md)
- [ADR-0003 — Schema-first entities over `make entity`](adr/0003-schema-first-over-make-entity.md)

---

## Sources of truth

When a doc and the code disagree, the code wins. These are the sources this handbook is kept
downstream of:

- `src/main.rs` — the clap command tree (the real CLI surface).
- `src/commands/*.rs` — command handlers.
- `src/templates/` and the `backbone-module` skeleton repo — what gets generated.
- `Cargo.toml` — dependencies, version, binary/lib split.
- Workspace `metaphor.yaml` — where generated modules get registered.
- [`CHANGELOG.md`](../CHANGELOG.md) — what changed between versions.

*Handbook version: tracks `metaphor-codegen` 0.1.8. Last reconciled against code 2026-07-13.*
