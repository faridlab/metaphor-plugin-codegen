# Philosophy & Motivation

> **Reader:** Evaluator. **Mode:** Explanation.
> Read this to decide whether the plugin's worldview matches yours. No commands here.

## The problem

A Domain-Driven, Clean-Architecture Rust module is *mostly the same code every time*. A new
bounded context needs a domain layer, an application layer with CQRS commands and queries, an
infrastructure layer with repository implementations, a presentation layer with HTTP handlers,
proto contracts, migrations, seeders, config, and a test harness — before a single line of
business logic exists. Hand-writing that skeleton is slow, and worse, it *drifts*: every
developer lays out the layers slightly differently, names things slightly differently, and wires
the same plumbing slightly wrong. Six modules in, the codebase has six dialects.

`metaphor-codegen` exists to make the skeleton free and identical. You describe intent — a module
name, an entity schema, a CQRS command — and it emits the conventional structure. You spend your
attention on the part that is actually yours: the behavior inside the `// <<< CUSTOM` regions.

## The worldview

Three convictions shape every decision in this plugin.

**1. Convention over configuration.** There is one right place for a repository trait, one naming
rule for a domain event, one layout for a module. The generator encodes those conventions so that
*every* generated artifact lands in the same place with the same shape. Consistency is the
feature. If you find yourself wanting a flag to put a query somewhere unconventional, the
generator is doing its job by not offering it.

**2. Schema is the source of truth; code is downstream.** Entities are not hand-written — they are
described once in schema YAML, and the domain types, proto contracts, and migrations are generated
from that description. When the schema changes, you regenerate; you do not hand-edit five files and
hope they stay in sync. (This is why `make entity` is deprecated in favor of the schema-first flow
— see [ADR-0003](adr/0003-schema-first-over-make-entity.md).)

**3. Generation must be safe to re-run.** Generated code and hand-written code coexist in the same
files. The `// <<< CUSTOM` marker convention draws the line: everything outside the markers is the
generator's to overwrite, everything inside is yours to keep. Regeneration is a routine act, not a
destructive one. A generator you can only run once is a scaffolding tool; a generator you can run
every day is a framework.

## What it deliberately does *not* do

Trust comes from honest boundaries. This plugin is scaffolding — plumbing, not brains.

- **It does not contain business logic.** Generated services and specifications are empty shells
  with `// <<< CUSTOM` regions. The plugin has no opinion about *your* domain rules and never will.
- **It is not a build tool or a runtime.** It writes files. It does not compile, serve, or deploy
  your code — that is what `metaphor build`, `metaphor dev serve`, and the app's own binary do.
- **It is not a migration engine.** It *generates* SQL migrations and can invoke `sqlx` to run
  them, but the schema semantics and the database live outside it.
- **It is not invoked directly in normal use.** It is a subprocess-dispatched plugin behind the
  `metaphor` CLI ([ADR-0001](adr/0001-subprocess-plugin-dispatch.md)). You type `metaphor make …`;
  the CLI finds and runs `metaphor-codegen`. It ships and versions on its own, but it lives inside
  the workspace's command surface.
- **It is not general-purpose.** It generates *Metaphor* modules, following *Metaphor* conventions.
  It is not a language-agnostic scaffolding engine, and trying to make it one would dilute the
  conventions that make it valuable.

## The test of success

The plugin has done its job when:

- A new module's structure is *identical* to every other module's — a reviewer can navigate it
  blind.
- You never write a repository trait, a CQRS command envelope, or a migration header by hand.
- Regenerating after a schema change touches only generated regions and leaves your custom logic
  untouched.
- The generated skeleton compiles the moment it lands, so your first commit is behavior, not
  boilerplate.

If a proposed feature doesn't move one of those forward, it doesn't belong here.

---

**Next:** [Background & prior art](background.md) — the tools this learned from, and where it
parts ways with them.
