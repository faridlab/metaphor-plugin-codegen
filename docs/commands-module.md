# Module Commands

Module commands manage **bounded context modules** -- self-contained business domains with their own domain logic, database persistence, and API layer.

Modules are located at `libs/modules/<name>/` and follow DDD structure.

---

## module create

Create a new module with the full DDD bounded context directory structure.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Module name (lowercase, e.g., `payments`, `analytics`) |
| `--author` | No | `Metaphor Developer` | Author name for generated files |
| `--description` | No | -- | Module description |

```bash
# Basic module
metaphor-codegen module create payments

# With all options
metaphor-codegen module create payments --author "John Doe" --description "Payment processing domain"
```

### Validation Rules

- Module name cannot be empty
- Only alphanumeric characters and underscores allowed
- Maximum 50 characters
- Module must not already exist at `libs/modules/<name>/`

### Generated Directory Structure

```
libs/modules/payments/
  Cargo.toml                       # Crate manifest
  build.rs                         # Build script (proto compilation)
  buf.yaml                         # Protocol Buffer configuration
  README.md                        # Module documentation
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
      specification/
      usecase/
  src/
    lib.rs                         # Library entry point
    domain/                        # Domain layer (pure business logic)
      entity/
      value_object/
      event/
      repository/                  # Repository traits
      service/
      specification/
      state_machine/
      computed_field/
      permission/
      export/
    application/                   # Application layer (use cases)
      commands/
      queries/
    infrastructure/                # Infrastructure layer
      persistence/                 # Database implementations
    presentation/                  # Presentation layer
      http/                        # HTTP handlers
  migrations/                      # Database migration files
    seeds/                         # Database seed files
  seeders/                         # Programmatic Rust seeders
  tests/                           # Module tests
  docs/                            # Module-specific documentation
  config/                          # Module configuration
    application.yml
    application-dev.yml
    application-prod.yml
```

### Next Steps After Creation

The command prints recommended next steps:

1. Define entity schema at `libs/modules/<name>/schema/models/<entity>.model.yaml`
2. Generate code: `metaphor schema generate <name> --target all`
3. Run migrations: `sqlx migrate run`
4. Start development: `metaphor dev:serve`

---

## module list

List all available modules, including both official framework modules and user-created bounded contexts.

```bash
metaphor-codegen module list
```

### Output Sections

**Framework Official Modules:**

Official modules provided by the Metaphor Framework as reusable crate dependencies.

| Module | Description |
|--------|-------------|
| `metaphor-cache` | Redis and in-memory caching support |
| `metaphor-email` | Multi-provider email service (SMTP, SES, Mailgun) |
| `metaphor-queue` | Message queue system (Redis, SQS) |
| `metaphor-search` | Search integration (Elasticsearch, Algolia) |
| `metaphor-storage` | File storage system (S3, local, MinIO) |

**Business Bounded Context Modules:**

User-created modules at `libs/modules/`. Each module shows:
- Cargo.toml status (whether the crate is properly configured)
- Schema directory status (whether schema-first definitions exist)

---

## module info

Show detailed information about a module.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Module name |

```bash
# Official module info
metaphor-codegen module info metaphor-cache

# User module info
metaphor-codegen module info payments
```

### Official Module Info

For official modules, the command displays:
- Description and version
- Key features list
- Cargo.toml dependency line
- Quick start code example
- Documentation links

### User Module Info

For user-created bounded contexts, the command displays:
- Description and version (from `Cargo.toml`)
- Directory structure listing

---

## module enable

Enable a module in the application configuration file.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Module name to enable |

```bash
metaphor-codegen module enable sapiens
```

### How It Works

Updates `apps/metaphor/config/application.yml` to set the module's `enabled` flag to `true` in both the `modules` and `services` sections.

### Supported Modules

Currently supports toggling these known modules:
- `sapiens`
- `postman`
- `bucket`

> **Note:** Restart services after enabling/disabling for changes to take effect.

---

## module disable

Disable a module in the application configuration file.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Module name to disable |

```bash
metaphor-codegen module disable postman
```

Sets the module's `enabled` flag to `false` in `apps/metaphor/config/application.yml`.

---

## module install

Install an external module package as a Cargo dependency.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `package` | Yes | -- | Package name |
| `--production` | No | `false` | Add to `[dependencies]` instead of `[dev-dependencies]` |
| `--version` | No | latest | Version specification (e.g., `1.0.0`, `^2.0`) |
| `--git` | No | `false` | Install from a git repository |

```bash
# From crates.io
metaphor-codegen module install my-custom-module --version "1.0.0"

# From git repository
metaphor-codegen module install https://github.com/org/module.git --git

# Git with specific branch
metaphor-codegen module install https://github.com/org/module.git --git --version main

# As production dependency
metaphor-codegen module install my-module --production --version "^2.0"
```

### How It Works

Uses `cargo add` under the hood to install the package. The command:

1. Validates that `Cargo.toml` exists in the current directory
2. Constructs the appropriate `cargo add` command with flags
3. Executes the installation
4. Prints next steps (import the module, build the project)

---

## Official Modules Reference

These are framework-level crate dependencies, not bounded context modules. Add them to your `Cargo.toml`:

### metaphor-cache

Redis and in-memory caching support.

| Feature | Description |
|---------|-------------|
| Redis backend | Cluster support included |
| In-memory caching | For development environments |
| Generic CacheService trait | Swap backends without code changes |
| Cache invalidation | Multiple strategies supported |
| Metrics | Performance monitoring |

```toml
metaphor-cache = { path = "../../../crates/metaphor-cache" }
```

```rust
use metaphor_cache::{CacheService, RedisCache};
let cache = RedisCache::new(config).await?;
cache.set("key", "value").await?;
```

### metaphor-email

Multi-provider email service.

| Feature | Description |
|---------|-------------|
| SMTP | Self-hosted email support |
| Amazon SES | AWS email integration |
| Mailgun | Mailgun API support |
| Templates | Built-in email template system |
| Async delivery | With retry logic |

```toml
metaphor-email = { path = "../../../crates/metaphor-email" }
```

### metaphor-queue

Message queue system.

| Feature | Description |
|---------|-------------|
| Redis queue | Priority support |
| AWS SQS | SQS integration |
| Batch processing | Process multiple messages |
| Dead letter queue | Failed message handling |

```toml
metaphor-queue = { path = "../../../crates/metaphor-queue" }
```

### metaphor-search

Search integration.

| Feature | Description |
|---------|-------------|
| Elasticsearch | Full-text search backend |
| Algolia | Search-as-a-service |
| Faceting | Advanced filtering and aggregations |
| Autocomplete | Search suggestions |

```toml
metaphor-search = { path = "../../../crates/metaphor-search" }
```

### metaphor-storage

File storage system.

| Feature | Description |
|---------|-------------|
| AWS S3 | Multipart upload support |
| Local filesystem | Development storage |
| MinIO | S3-compatible self-hosted |
| Presigned URLs | Secure temporary access |
| Encryption | File encryption support |

```toml
metaphor-storage = { path = "../../../crates/metaphor-storage" }
```

---

## See Also

- [Architecture & Concepts](architecture.md) -- Bounded context and DDD patterns
- [Make Commands](commands-make.md) -- Scaffolding components within modules
- [Configuration](configuration.md) -- Application YAML configuration
