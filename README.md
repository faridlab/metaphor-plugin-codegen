# metaphor-plugin-codegen

Code generation and scaffolding plugin for the [Metaphor Framework](https://github.com/faridlab/metaphor-plugin-codegen). Generates boilerplate code following **Domain-Driven Design (DDD)** and **Clean Architecture** principles.

**Binary:** `metaphor-codegen` | **Version:** 0.1.0 | **License:** MIT

## Installation

```bash
# From source
cargo install --path .

# Or build locally
cargo build --release
```

The compiled binary `metaphor-codegen` will be available in `target/release/`.

## Quick Start

```bash
# 1. Create a bounded context module
metaphor-codegen module create payments --description "Payment processing"

# 2. Define your entity schema (schema-first approach)
#    Edit: libs/modules/payments/schema/models/payment.model.yaml

# 3. Generate code from schema
metaphor-codegen make command CreatePayment --module payments --entity Payment

# 4. Generate database migration
metaphor-codegen migration generate Payment payments

# 5. Run migrations
metaphor-codegen migration run --module payments
```

## Command Groups

| Command | Subcommands | Description | Documentation |
|---------|:-----------:|-------------|---------------|
| `make` | 12 | Laravel-style scaffolding for DDD components | [commands-make.md](docs/commands-make.md) |
| `module` | 6 | Bounded context module management | [commands-module.md](docs/commands-module.md) |
| `apps` | 3 | Application generation with Clean Architecture | [commands-apps.md](docs/commands-apps.md) |
| `proto` | 2 | Protocol Buffer operations | [commands-proto.md](docs/commands-proto.md) |
| `migration` | 10 | PostgreSQL migration lifecycle | [commands-migration.md](docs/commands-migration.md) |
| `seed` | 7 | Database seeding and test data | [commands-seed.md](docs/commands-seed.md) |

## Global Flags

| Flag | Description |
|------|-------------|
| `--verbose`, `-v` | Enable verbose output (sets `RUST_LOG=debug`) |
| `--version` | Show version information |
| `--help` | Show help information |

## Generated Module Structure

When you create a module with `metaphor-codegen module create <name>`, the following DDD-aligned directory structure is generated:

```
libs/modules/<name>/
  Cargo.toml
  build.rs
  buf.yaml
  README.md
  schema/                          # Schema-first definitions
    models/
      index.model.yaml             # Entity schema entry point
  proto/                           # Protocol Buffer definitions
    domain/
      entity/
      event/
      value_object/
      repository/
      service/
  src/
    lib.rs
    domain/                        # Domain layer
      entity/                      # Domain entities
      value_object/                # Value objects
      event/                       # Domain events
      repository/                  # Repository traits
      service/                     # Domain services
      specification/               # Business rules
    application/                   # Application layer
      commands/                    # CQRS write operations
      queries/                     # CQRS read operations
    infrastructure/                # Infrastructure layer
      persistence/                 # Database implementations
    presentation/                  # Presentation layer
      http/                        # HTTP handlers
  migrations/                      # Database migrations
  tests/                           # Module tests
  docs/                            # Module documentation
  config/                          # Module configuration
```

## Ecosystem

`metaphor-plugin-codegen` is part of the **Metaphora** ecosystem:

| Component | Description |
|-----------|-------------|
| `metaphor-cli` | Orchestrator CLI that invokes plugins |
| `metaphor-plugin-codegen` | Code generation and scaffolding (this plugin) |
| `metaphor-plugin-dev` | Development workflow tools (lint, test, docs) |
| `metaphor-plugin-schema` | Schema-driven code generation (proto, Rust, SQL, Kotlin, TypeScript) |

## Documentation

- [Architecture & Concepts](docs/architecture.md) - DDD, Clean Architecture, CQRS patterns
- [Make Commands](docs/commands-make.md) - Scaffolding commands reference
- [Module Commands](docs/commands-module.md) - Module management reference
- [Apps Commands](docs/commands-apps.md) - Application generation reference
- [Proto Commands](docs/commands-proto.md) - Protocol Buffer operations reference
- [Migration Commands](docs/commands-migration.md) - Database migration reference
- [Seed Commands](docs/commands-seed.md) - Database seeding reference
- [Template System](docs/templates.md) - Template engine internals
- [Configuration](docs/configuration.md) - Database and environment configuration
