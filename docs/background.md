# Background & Prior Art

> **Reader:** Evaluator. **Mode:** Explanation.
> What came before, what this borrows, and where it deliberately diverges. Prior art is credited,
> not strawmanned.

Code generators are old and well-explored. `metaphor-codegen` is not novel in *that* it generates
code — it is opinionated in *what* it generates (DDD 4-layer Rust modules) and *how* it stays safe
to re-run. Here is the lineage.

## Laravel Artisan `make:*` — the ergonomics

Laravel's `php artisan make:controller`, `make:model`, `make:migration` set the standard for
"scaffolding as a verb." A developer types an intent and a conventional file appears. This plugin's
entire `make` command group is a deliberate homage: `make command`, `make query`, `make handler`,
`make repository`, `make event`, `make value-object`, `make spec`.

**Borrowed:** the verb-noun ergonomics, the convention-driven file placement, the "you shouldn't
have to think about where this goes" feel.

**Rejected:** Artisan generates for a dynamically-typed MVC framework. Its output is a near-empty
class you fill in freely. Metaphor generates for a statically-typed, layered architecture where the
*relationships between files* (domain trait ↔ infrastructure impl ↔ proto contract) matter as much
as the files themselves. So the generators are relational, not one-file-at-a-time.

## Rails generators & scaffolding — and its cautionary tale

Rails popularized `rails generate scaffold`, which emits a full model-view-controller-migration
stack from one command. It also taught the community scaffolding's failure mode: **generated code
you can never regenerate.** Rails scaffold output is a one-shot dump — run it, then own the files
forever; re-running clobbers your edits.

**Borrowed:** the ambition to generate a *whole vertical slice*, not just one layer.

**Rejected:** the one-shot model. Metaphor's `// <<< CUSTOM` marker convention exists precisely so
regeneration is routine rather than destructive (see [Philosophy](philosophy.md) and the
[Maintainer guide](maintainer-guide.md)). Generation you can only do once is a strictly weaker tool.

## Prisma & schema-first ORMs — the source of truth

Prisma made a `schema.prisma` file the single source of truth from which client types and
migrations are generated. The developer edits the schema, runs generate/migrate, and the derived
artifacts follow.

**Borrowed:** schema-as-SSoT. Metaphor entities are described in schema YAML
(`schema/models/*.model.yaml`); domain types, proto, and migrations are generated *from* that
schema. This is why `make entity` is deprecated in favor of the schema-first flow
([ADR-0003](adr/0003-schema-first-over-make-entity.md)) — hand-writing an entity would create a
second, competing source of truth.

**Rejected:** Prisma owns the runtime query layer too. Metaphor stops at generation — it emits the
domain types and migrations and then gets out of the way. The runtime (repositories, SQLx queries)
is the module's own code, not a generated client you call through.

> **Division of labor:** in Metaphor, schema *definition* and schema-driven generation belong to
> the `metaphor-plugin-schema` plugin (`metaphor schema generate`). `metaphor-codegen` owns
> *scaffolding* — the module skeleton, the `make` components, apps, proto tooling, migrations, and
> seeds. The two plugins compose; this one does not parse model schemas into domain types itself.

## Nx / Turborepo generators — the workspace context

Monorepo tools like Nx ship generators (`nx generate`) that scaffold libraries and apps *into a
known workspace*, then wire them into the workspace graph. Generation is aware of the repo it lands
in.

**Borrowed:** workspace-awareness. A generated Metaphor module isn't a free-floating crate — it is
meant to be registered in the workspace `metaphor.yaml` and discovered by the `metaphor` CLI. The
`module create` command ends by telling you to register the module and run `metaphor schema
generate`.

**Rejected:** Nx generators are TypeScript plugins loaded in-process by a Node-based orchestrator.
Metaphor's plugins are *separate compiled binaries* dispatched as subprocesses
([ADR-0001](adr/0001-subprocess-plugin-dispatch.md)), so a plugin can be written, versioned, and
released independently of the CLI core.

## Where this lands

`metaphor-codegen` is Artisan's ergonomics + Prisma's schema-first discipline + Rails' whole-slice
ambition + Nx's workspace-awareness — minus each tool's principal drawback, and specialized hard
for one target: idiomatic, layered, statically-typed Rust modules that stay safe to regenerate.

---

**Next:** [Technology & the "why"](technology.md) — the concrete stack and the reasoning behind
each dependency.
