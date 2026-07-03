# ADR-0001 — Subprocess plugin dispatch

- **Status:** Accepted
- **Reader:** Maintainer
- **Supersedes:** —
- **Superseded by:** —

## Context

The Metaphor CLI needs many capabilities — codegen, schema, dev tooling, agent management. Bundling
all of them into one monolithic `metaphor` binary would mean every capability shares a release
cadence, a dependency tree, and a compile time. Codegen in particular pulls heavy dependencies
(`tonic-build`, `handlebars`) that most other commands don't need, and it evolves on its own
schedule.

## Decision

Ship code generation as a **separate binary** (`metaphor-codegen`) that the `metaphor` CLI
discovers and invokes as a **subprocess**. Discovery order is `$PATH` →
`$METAPHOR_PLUGIN_BIN_DIR` → `~/.metaphor/bin/`. The plugin declares both a `[[bin]]` and a `[lib]`
in its own `Cargo.toml`, pins its direct dependency versions (no workspace inheritance), and
releases independently via git tags and prebuilt cross-platform binaries.

## Consequences

**Positive**
- The plugin versions and releases on its own; a codegen fix doesn't require a CLI release.
- Its dependency weight is isolated — nothing else pays for `tonic-build`.
- New plugins can be added to the ecosystem without modifying the CLI core.
- Prebuilt release binaries (four target triples) make distribution a download, not a compile.

**Negative**
- Cross-process boundary: the CLI passes intent as argv, not typed calls; contract drift is caught
  at runtime, not compile time.
- Discovery depends on the binary being on a known path — a missing/wrong-version plugin is a
  runtime failure, mitigated by `metaphor doctor`.
- Shared concepts (naming, conventions) must be kept consistent across repos by discipline and
  documentation rather than a shared library.

## Notes

The plugin remains directly runnable (`metaphor-codegen make …`) for development and testing, but
the intended entry point is `metaphor make …`. See [technology.md](../technology.md#distribution)
for the release mechanics.
