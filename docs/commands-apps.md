# Apps Commands

Application generation commands for creating new Metaphor Framework applications. Each generated app is scaffolded from the canonical **`backbone-application`** skeleton and follows **Clean Architecture** with pre-configured routing, middleware, and configuration.

> **How generation works (v0.2.0):** `apps generate` **clones** the
> [`backbone-application`](https://github.com/faridlab/backbone-application) skeleton repo and stamps
> the app name in — it no longer expands local Handlebars templates. The skeleton repo is the single
> source of truth for app/service structure. This requires `git` on PATH and network access, and
> **fails offline**. See [ADR-0002](adr/0002-skeleton-clone-scaffolding.md) for the rationale (the
> same decision that governs `module`).

---

## apps generate

Generate a new Metaphor Framework application by cloning the `backbone-application` skeleton.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Application name in kebab-case (e.g., `my-service`) |
| `-t`, `--app-type` | No | `api` | Application type: `api`, `auth`, `worker`, `scheduler` |
| `-p`, `--port` | No | auto | Server port (auto-detected by type) |
| `-d`, `--database` | No | `postgresql` | Database: `postgresql`, `mongodb`, `sqlite`, `none` |
| `-m`, `--description` | No | auto | Application description |
| `--auth` | No | `false` | Enable authentication features |
| `--metrics` | No | `false` | Enable metrics collection |
| `-o`, `--output` | No | `apps` | Output directory |
| `--author` | No | -- | Author name |
| `--email` | No | -- | Author email |

> **Note:** since generation now clones a fixed skeleton, only `name` and `--output` change the files
> written to disk. The `--app-type`, `--port`, `--database`, `--auth`, and `--metrics` flags are still
> validated and shown in the configuration summary, but no longer alter the generated app — tune those
> in the generated project's config after scaffolding.

```bash
# Basic API service
metaphor-codegen apps generate my-service

# Auth service with all features
metaphor-codegen apps generate auth-service --app-type auth --auth --metrics

# Worker with custom port and MongoDB
metaphor-codegen apps generate data-processor --app-type worker --port 4000 --database mongodb

# Scheduler in custom directory
metaphor-codegen apps generate cron-jobs --app-type scheduler --output services
```

### App Name Validation Rules

- Must be 3-50 characters long
- Must be kebab-case (lowercase letters, numbers, and hyphens only)
- Cannot start or end with a hyphen
- Cannot contain consecutive hyphens (`--`)
- Cannot use reserved names: `metaphor`, `framework`, `cli`, `test`, `demo`

### App Types and Default Ports

| Type | Default Port | Description |
|------|:------------:|-------------|
| `api` | 3000 | Standard HTTP API with REST endpoints |
| `auth` | 3002 | User authentication and authorization service |
| `worker` | 3003 | Asynchronous background job processor |
| `scheduler` | 3004 | Cron-based task scheduling service |

### Generated App Structure

The layout is whatever the [`backbone-application`](https://github.com/faridlab/backbone-application)
skeleton currently ships — read that repo for the authoritative structure. At the time of writing it
is a Clean Architecture app roughly shaped like:

```
apps/<name>/
  Cargo.toml                       # Crate manifest (package renamed to <name>)
  docker-compose.yml               # Docker setup
  README.md                        # App documentation
  config/                          # Configuration files
  tests/                           # Application tests
  src/
    main.rs                        # Entry point
    config/                        # App configuration
    middleware/                    # HTTP middleware
    routes/                        # Route definitions
    shared/                        # Shared utilities
    application/                   # Application layer
    domain/                        # Domain layer
    infrastructure/                # Infrastructure layer
    presentation/                  # Presentation layer
```

The generator detaches the clone from the skeleton (removes `.git` and `Cargo.lock`) and stamps the
skeleton's baked-in package name — `backbone-app` (kebab) and `backbone_app` (snake) — to your app
name across every UTF-8 text file (Cargo.toml, `src/main.rs`, Dockerfiles, config, deployment).

### Workspace Integration

The generator **no longer edits the workspace `Cargo.toml`**. After scaffolding, register the app in
`metaphor.yaml` yourself:

```yaml
# metaphor.yaml
projects:
  - name: my-service
    type: backend-service
```

### Configuration Summary

Before generating, the command displays a configuration summary:

```
Configuration:
  - App Name: my-service
  - Type: api
  - Port: 3000
  - Database: postgresql
  - Auth: Disabled
  - Metrics: Disabled
  - Description: My Service service
  - Author: Developer <dev@example.com>
  - Database Name: my_service_db
```

### Next Steps After Generation

```bash
# 1. Register the app in metaphor.yaml (name: my-service / type: backend-service)
cd apps/my-service
cargo build           # Build the application
cargo run             # Start the application
# Visit http://localhost:3000
# Health check: http://localhost:3000/health
# API: http://localhost:3000/api/v1
```

---

## apps list

List all available application templates.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `-d`, `--detailed` | No | `false` | Show detailed information |

```bash
# Brief listing
metaphor-codegen apps list

# Detailed listing
metaphor-codegen apps list --detailed
```

### Available Templates

| Template | Description | Details |
|----------|-------------|---------|
| `api` | REST API service | Standard HTTP API with REST endpoints |
| `auth` | Authentication service | User authentication and authorization |
| `worker` | Background worker | Asynchronous background job processor |
| `scheduler` | Task scheduler | Cron-based task scheduling service |

---

## apps validate

Validate an application name for correctness and availability.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Application name to validate |

```bash
metaphor-codegen apps validate my-service
```

### Checks Performed

1. **Format validation** -- checks kebab-case, length, and character rules
2. **Availability check** -- checks if `apps/<name>/` already exists

### Example Output

```
Validation passed: my-service is a valid app name
App name available: my-service is available for creation
```

Or with issues:

```
Validation failed: My-Service: App name must contain only lowercase letters, numbers, and hyphens
```

---

## Name Stamping

The generator does not use a template engine. It replaces the skeleton's literal package name with
your app name across every UTF-8 text file in the clone:

| Skeleton token | Replaced with | Example (`my-service`) |
|----------------|---------------|------------------------|
| `backbone-app` (kebab) | your app name | `my-service` |
| `backbone_app` (snake) | snake_cased app name | `my_service` |

Everything else — layout, dependencies, config keys, Clean Architecture layers — comes verbatim from
the skeleton. To change the generated shape, edit the `backbone-application` repo, not this plugin.

---

## See Also

- [Architecture & Concepts](architecture.md) -- Clean Architecture layers
- [Template System](templates.md) -- how `make`/`module`/`apps` generation works
- [ADR-0002](adr/0002-skeleton-clone-scaffolding.md) -- why scaffolding clones a skeleton repo
- [Configuration](configuration.md) -- Database and environment configuration
