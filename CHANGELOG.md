# Changelog

All notable changes to `metaphor-codegen` are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
[0.1.8]: #018
[0.1.7]: #017
[0.1.6]: #016
[0.1.5]: #015
[0.1.4]: #014
[0.1.1]: #011---013
