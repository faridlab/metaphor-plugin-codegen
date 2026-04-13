# Apps Commands

Application generation commands for creating new Metaphor Framework applications from templates. Each generated app follows **Clean Architecture** with pre-configured routing, middleware, and configuration.

---

## apps generate

Generate a new Metaphor Framework application from a template.

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

```
apps/<name>/
  Cargo.toml                       # Crate manifest
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

### Workspace Integration

After generation, the app is automatically added to the workspace `Cargo.toml` members list. The generator searches upward for the workspace root (a `Cargo.toml` containing `[workspace]`).

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

## Handlebars Template Helpers

The app generator uses Handlebars for template processing and registers these helpers:

| Helper | Input | Output |
|--------|-------|--------|
| `pascal_case` | `my-service` | `MyService` |
| `snake_case` | `my-service` | `my_service` |
| `kebab_case` | `my_service` | `my-service` |
| `camel_case` | `my-service` | `myService` |
| `upper_case` | `my-service` | `MY_SERVICE` |
| `title_case` | `my-service` | `My Service` |

These helpers can be used in template files as `{{pascal_case APP_NAME}}`.

### Template Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `APP_NAME` | Original app name | `my-service` |
| `APP_NAME_PASCAL` | PascalCase | `MyService` |
| `APP_NAME_SNAKE` | snake_case | `my_service` |
| `APP_NAME_KEBAB` | kebab-case | `my-service` |
| `APP_NAME_CAMEL` | camelCase | `myService` |
| `APP_PORT` | Server port | `3000` |
| `DATABASE_TYPE` | Database type | `postgresql` |
| `DATABASE_NAME` | Database name | `my_service_db` |
| `AUTH_ENABLED` | Auth flag | `true`/`false` |
| `METRICS_ENABLED` | Metrics flag | `true`/`false` |
| `IS_WORKER` | Worker type flag | `true`/`false` |
| `IS_SCHEDULER` | Scheduler type flag | `true`/`false` |
| `CREATION_DATE` | Generation date | `2026-04-14` |
| `CREATION_DATETIME` | Generation timestamp | `2026-04-14 12:00:00` |
| `AUTHOR_NAME` | Author name | `Developer` |
| `AUTHOR_EMAIL` | Author email | `dev@example.com` |

---

## See Also

- [Architecture & Concepts](architecture.md) -- Clean Architecture layers
- [Template System](templates.md) -- Handlebars template processing details
- [Configuration](configuration.md) -- Database and environment configuration
