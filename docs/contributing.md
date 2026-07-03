# Contribution Guide

> **Reader:** Contributor. **Mode:** How-to.
> Everything you need to land a correct PR on the first try. Assumes you can read the
> [Maintainer guide](maintainer-guide.md) for internals.

## Dev setup

```bash
git clone https://github.com/faridlab/metaphor-plugin-codegen
cd metaphor-plugin-codegen
cargo build          # inside a Metaphor workspace, prefer: metaphor dev build
cargo test           #                                       metaphor dev test
```

Requirements: a stable Rust toolchain (edition 2021), `git` on `PATH` (the `module create` command
shells out to it), and — only for exercising `migration run` — a reachable PostgreSQL and a
`DATABASE_URL` (a `.env` at or above CWD is auto-loaded; see [configuration.md](configuration.md)).

> This crate follows the workspace rules in the root `CLAUDE.md`: inside a Metaphor workspace use
> `metaphor dev build` / `metaphor dev test`, **never** `cargo build`/`test` from the workspace
> *root*. Running raw `cargo` **inside this project directory** is fine — it has its own
> `Cargo.toml`.

## Before you write code

1. Read the [Philosophy](philosophy.md) — a change that adds business logic, breaks regeneration
   safety, or introduces an unconventional layout will be rejected on principle, not on style.
2. Read the relevant `docs/commands-*.md` and the handler in `src/commands/`.
3. If you're adding a generator, follow the [add-a-`make`-target walkthrough](maintainer-guide.md#walkthrough-add-a-new-make-target).

## Commit conventions

**Conventional Commits**, and **no signatures of any kind** — no `Co-Authored-By`, no "Generated
with", no trailer. This is a hard workspace rule.

```
feat: add make projection target
fix(migration): prefer top-level migrations/ over legacy postgres/ subfolder
chore: bump version to 0.1.8
docs: document the routes command
```

Format: `type(scope): summary`. Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`.
Keep the summary one line and imperative. Group changes by functionality — small related files in
one commit, large files on their own; don't dump unrelated changes into a single commit.

## Tests

- Unit tests live inline (`#[cfg(test)] mod tests`) — see `template_processor.rs` for the pattern.
  Test the *pure* logic (case conversion, placeholder replacement), not the filesystem.
- Run the full suite before opening a PR:

  ```bash
  cargo test
  cargo build            # must be warning-clean for code you touch
  ```

- New generators: add a case-conversion / context test if you introduced placeholder logic, and
  manually run the command against a scratch module to confirm the output compiles.

## PR checklist

Before you open the PR, confirm:

- [ ] `cargo build` and `cargo test` pass locally.
- [ ] Commits are Conventional Commits with **no signature/trailer**.
- [ ] The change keeps generation **regen-safe** — `// <<< CUSTOM` regions are preserved on re-run.
- [ ] Output lands in the correct Clean-Architecture layer; no new top-level layers invented.
- [ ] New/changed CLI flags are reflected in **both** the CLI enum and the handler enum where the
      two-type split exists (`module`, `migration`).
- [ ] Docs updated: the relevant `docs/commands-*.md`, the [handbook index](index.md), and a
      [glossary](glossary.md) entry for any new concept.
- [ ] `--help` output still reads correctly (it's generated from your clap definitions).

## Review expectations

- **Correctness first:** does the generated code compile, and does re-running preserve custom
  regions? A generator that clobbers hand-written code fails review regardless of anything else.
- **Convention conformance:** naming, layer placement, and status-output voice match the existing
  commands.
- **Scope discipline:** one concern per PR. Refactors ride separately from features.
- **Docs are part of the diff**, not a follow-up. An undocumented command is an incomplete feature
  (the `routes` command is the standing example of this gap —
  [see the maintainer guide](maintainer-guide.md#undocumented-surface)).

## Releasing (maintainers)

1. Bump `version` in `Cargo.toml` (`chore: bump version to X.Y.Z`).
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. `.github/workflows/release.yml` builds the four target triples and publishes the GitHub Release.
4. Update this handbook's "tracks version" footer in [index.md](index.md) if behavior changed.

---

**See also:** [Maintainer guide](maintainer-guide.md) · [Glossary](glossary.md)
