# Technology & the "Why"

> **Reader:** Evaluator + Maintainer. **Mode:** Explanation.
> The stack, with a one-line rationale and a named rejected alternative for each choice. Sourced
> from `Cargo.toml` at version 0.1.8 — when the manifest changes, update this page.

## Language & runtime

**Rust (edition 2021).** The plugin generates Rust, so it is written in Rust: the same case-
conversion, naming, and idiom rules apply to generator and output, and the generated code's
correctness assumptions are the author's own. *Rejected:* a scripting language (Python/Node) —
faster to prototype, but it would split the toolchain and lose the single-binary distribution that
[ADR-0001](adr/0001-subprocess-plugin-dispatch.md) depends on.

**Tokio (`full` features), async `main`.** Command handlers are `async` and some shell out
(`git clone`, `sqlx` migration runs). A uniform async surface keeps every handler signature the
same. *Rejected:* synchronous `main` — most commands are IO-light, but `migration run` talks to a
database and a consistent async contract is simpler than mixing sync and async handlers.

## CLI surface

**`clap` v4 (`derive`, `env`, `color`).** The command tree is defined declaratively as enums in
`src/main.rs`; `--help`, version, env-var fallbacks, and colored usage come for free and stay in
sync with the code. This is the *authoritative* command surface — docs are downstream of it.
*Rejected:* hand-rolled arg parsing — total control, but every flag becomes a maintenance liability
and `--help` drifts from reality.

**`colored` v2.** Human-facing status output (`⚡ Metaphor Codegen`, `✅ Module created`) reads as
a friendly tool, not a log dump. *Rejected:* raw ANSI escapes — `colored` handles TTY detection so
piped output stays clean.

## Templating & generation

**`handlebars` v4 + simple `{{PLACEHOLDER}}` string replacement.** Two mechanisms, by design: the
lightweight `make` targets use direct `HashMap`-driven string replacement in
`templates/template_processor.rs` (no logic, just substitution), while richer templates can use
Handlebars for conditionals and loops. *Rejected:* a single heavyweight template engine everywhere —
overkill for a file whose only variable is an entity name.

> **Maintainer note — a live migration.** The `module create` command has *moved past* local
> templating entirely: it now clones the canonical `backbone-module` skeleton repo and stamps names
> in ([ADR-0002](adr/0002-skeleton-clone-scaffolding.md)). Consequently the path helpers in
> `template_processor.rs` (which point at a legacy `crates/metaphor-cli/src/templates/…` location)
> are dead code behind `#![allow(dead_code)]`, and `docs/templates.md`'s claim that `module` uses
> Handlebars templates is **stale**. The `make` targets still use local `src/templates/make/…`.

**`walkdir` v2.** Recursively walks template trees and (for `routes`) source trees. *Rejected:*
hand-rolled recursion — `walkdir` handles symlink and error edge cases correctly.

**`regex` v1 + `once_cell` v1.** The `routes` command scans source files for Axum route patterns
and Backbone CRUD-handler calls; regexes are compiled once via `Lazy`. *Rejected:* a full Rust
parser (`syn`) — accurate but far heavier than pattern-scanning needs, and route macros aren't
always syntactically resolvable anyway.

## Contracts & data

**`tonic-build` v0.12.** Proto-first cross-module contracts mean the plugin must understand and
emit Protocol Buffer scaffolding for domain events, value objects, and services. *Rejected:*
JSON-schema or hand-written traits for cross-service contracts — proto gives backward-compatible,
language-neutral contracts and codegen out of the box.

**`serde` + `serde_json` + `serde_yaml` v0.9.** Schema files and config are YAML; some tooling
speaks JSON (`.sqlx/config.json`, `template-config.json`). Serde is the Rust default. *Rejected:*
bespoke parsers — no reason to reinvent well-trodden serialization.

## Supporting utilities

| Crate | Why it's here | Rejected alternative |
|-------|--------------|---------------------|
| `anyhow` v1 | Ergonomic error propagation in a **binary** (per crate rules, libraries use `thiserror`; this crate ships a binary) | `Box<dyn Error>` — loses context chains |
| `chrono` v0.4 | Timestamps in generated headers, migration names, `{{CURRENT_TIMESTAMP}}` | `time` — either works; `chrono` matches generated code |
| `uuid` v1 (`v4`) | Generated default IDs and seed data | hand-rolled — needless |
| `url` v2 | Parse/validate `DATABASE_URL` for `migration run` | manual string parsing — fragile |
| `dotenvy` v0.15 | Auto-load `.env` (walking up from CWD) so `DATABASE_URL` needn't be exported before every run | `dotenv` — unmaintained fork |

## Distribution

**Independently versioned binary, released by git tag.** `Cargo.toml` declares both a `[[bin]]`
(`metaphor-codegen`) and a `[lib]` (`metaphor_codegen`), with pinned direct versions (no
`workspace.dependencies` inheritance) so the crate is releasable on its own. Pushing a `v*` tag
triggers `.github/workflows/release.yml`, which cross-builds four targets (Linux + macOS, x86_64 +
aarch64) and uploads binaries to a GitHub Release. *Rejected:* publishing to crates.io as the
primary channel — the CLI discovers plugins as binaries on `PATH` / `~/.metaphor/bin/`, so prebuilt
release artifacts are the natural distribution unit.

---

**Next (maintainer):** [Maintainer guide](maintainer-guide.md) — the plugin's own internals and
how to add a generator. **Next (evaluator):** [Generated-code architecture](architecture.md) — the
shape of what it emits.
