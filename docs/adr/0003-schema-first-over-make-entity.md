# ADR-0003 — Schema-first entities over `make entity`

- **Status:** Accepted
- **Reader:** Maintainer + App developer
- **Supersedes:** —
- **Superseded by:** —

## Context

Entities can be created two ways: imperatively, by running `make entity User --module payments`
(which scaffolds a hand-editable entity file + proto), or declaratively, by describing the entity in
`schema/models/*.model.yaml` and generating domain types, proto, and migrations from that schema.

Supporting both makes the entity's *definition* ambiguous: is the truth the Rust file, the proto,
or the schema? When they disagree — and they will, once someone hand-edits the generated Rust — the
system has no principled way to reconcile them. Two sources of truth is zero sources of truth.

## Decision

The **schema YAML is the single source of truth** for entities. `make entity` is **deprecated**.
Entities are defined in `schema/models/*.model.yaml` and generated via the `metaphor-plugin-schema`
plugin (`metaphor schema generate`), which emits the Rust domain types, proto contracts, and SQL
migrations together, in sync.

## Consequences

**Positive**
- One artifact defines an entity; the derived Rust/proto/SQL cannot drift from each other because
  they are all regenerated from the same schema.
- Schema changes are a single edit + regenerate, not a hand-coordinated edit across several files.
- Cleanly divides responsibility: `metaphor-plugin-schema` owns entity generation;
  `metaphor-plugin-codegen` owns scaffolding of *everything else* (modules, CQRS components, apps,
  proto tooling, migrations, seeds).

**Negative**
- Developers must learn the schema DSL rather than editing a familiar Rust struct.
- `make entity` still exists in the CLI surface (for backward compatibility) but should not be used;
  its presence can mislead newcomers. Documentation must actively steer them to schema-first.

## Notes

`docs/architecture.md` already carries the deprecation notice and an example schema. The other
`make` targets (command, query, repository, handler, service, event, value-object, spec) are **not**
deprecated — they scaffold components *around* schema-defined entities, and remain the intended way
to add those.
