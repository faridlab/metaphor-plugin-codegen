# Changelog

All notable changes to `metaphor-codegen` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2]

### Fixed

- `migration run-all` no longer depends on alphabetical luck. Modules were applied
  in name order, which is not migration order: a module's migrations can need
  database objects another module creates, and nothing in the manifest says so —
  an accounting module's audit triggers call a function an auditlog module owns,
  and auditlog sorts after it. On any database where the dependency had not been
  applied yet, the earlier module simply failed. The sweep now runs what is
  pending and retries whatever failed for as long as the previous pass moved
  something forward, so a module blocked only by a sibling is applied once that
  sibling has run. When a pass advances nothing, the remaining failures are
  reported and the command exits non-zero. Retrying is safe because applied
  migrations are recorded and skipped. The summary names any module that needed a
  retry, which is how an undeclared dependency becomes visible.
- A project whose `migrations/` holds no SQL of its own — only a `manual/`
  directory, for instance — is now skipped instead of counted as a failure, so a
  sweep no longer reports an error that no action could clear.
- The crate's unit tests compile again. A helper moved to `crate::utils` without
  the test module's import following it, and because that broke the whole test
  binary, every test in the crate stopped running rather than one failing
  visibly.


## [0.2.0]

### Changed
- `apps generate` now clones the `backbone-application` skeleton from GitHub and stamps the app name
  in, instead of expanding local Handlebars templates. The skeleton repo is the single source of
  truth for runnable app/service structure, so generated apps stay in lockstep with it. Requires
  `git` on PATH and network access; fails offline.

### Removed
- The Handlebars template engine and its helpers (`pascal_case`, `snake_case`, …) are gone from the
  app generator, along with the `src/templates/app/` template tree and the automatic
  workspace-`Cargo.toml` member insertion. Register the generated app in `metaphor.yaml` yourself.

## [0.1.9]

### Added
- `migration run-all --target <env>` routes migrations to `metaphor deploy migrate <env>`, running
  them **remotely** on that env's stack over SSH with a production confirmation gate. Add `--yes` to
  skip the gate in CI. Without `--target`, `run-all` behaves as before and runs against the local
  database.

## [0.1.8]

### Changed
- `module` scaffolding now clones the `backbone-module` skeleton instead of expanding local
  templates, so generated modules stay in lockstep with the canonical skeleton.

## [0.1.7]

### Fixed
- Migration discovery prefers a module's top-level `migrations/` directory over the legacy
  `migrations/postgres/` subfolder.

## [0.1.6]

### Added
- `routes` command lists the application's HTTP routes, now wired into the CLI.

## [0.1.5]

### Fixed
- Seed paths are resolved via the `metaphor.yaml` workspace manifest rather than assumed layout.

## [0.1.4]

### Added
- `.env` is auto-loaded from the current working directory at startup.

### Changed
- Database configuration prefers `config/application.yml` over the `apps/metaphor` fallback.

## [0.1.1] - [0.1.3]

### Added
- Modules are discovered via the `metaphor.yaml` workspace manifest.

[Unreleased]: #unreleased
[0.2.0]: #020
[0.1.9]: #019
[0.1.8]: #018
[0.1.7]: #017
[0.1.6]: #016
[0.1.5]: #015
[0.1.4]: #014
[0.1.1]: #011---013
