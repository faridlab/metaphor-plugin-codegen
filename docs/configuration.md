# Configuration

This document covers database configuration, environment variables, and YAML configuration files used by `metaphor-codegen`.

---

## Database URL Resolution

All commands that connect to the database (migrations, seeds) use a centralized resolution function. The database URL is resolved in this priority order:

### Priority 1: `.env` File

The `.env` file in the project root is checked first.

**Option A -- Direct URL:**

```env
DATABASE_URL=postgresql://user:password@localhost:5432/mydb
```

**Option B -- Individual variables (composed into URL):**

```env
POSTGRES_DB=mydb
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=password
```

When `POSTGRES_DB` is found, the URL is constructed as:
```
postgresql://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{POSTGRES_HOST}:{POSTGRES_PORT}/{POSTGRES_DB}
```

Default values if individual variables are missing:

| Variable | Default |
|----------|---------|
| `POSTGRES_HOST` | `localhost` |
| `POSTGRES_PORT` | `5432` |
| `POSTGRES_USER` | `postgres` |
| `POSTGRES_PASSWORD` | `password` |

### Priority 2: `apps/metaphor/config/application.yml`

```yaml
database:
  url: "postgresql://${POSTGRES_USER:postgres}:${POSTGRES_PASSWORD:password}@${POSTGRES_HOST:localhost}:${POSTGRES_PORT:5432}/${POSTGRES_DB:metaphor}"
```

### Priority 3: `config/application.yml` (Fallback)

Same format as above, used when the app-specific config is not found.

---

## Environment Variable Expansion

YAML configuration values support environment variable placeholders:

### Syntax

```
${VARIABLE_NAME}          # Expands to env var, or empty string if not set
${VARIABLE_NAME:default}  # Expands to env var, or "default" if not set
```

### Examples

```yaml
database:
  url: "postgresql://${DB_USER:postgres}:${DB_PASS:secret}@${DB_HOST:localhost}:${DB_PORT:5432}/${DB_NAME:myapp}"

server:
  port: ${APP_PORT:3000}
  host: ${APP_HOST:0.0.0.0}
```

The expansion is handled by the `expand_env_vars()` utility function using a compiled regex pattern.

---

## Application Configuration

### Module Configuration

Modules are managed in `apps/metaphor/config/application.yml`:

```yaml
modules:
  sapiens:
    enabled: true
  postman:
    enabled: false
  bucket:
    enabled: true

services:
  sapiens:
    enabled: true
  postman:
    enabled: false
  bucket:
    enabled: true
```

The `module enable` and `module disable` commands update both the `modules` and `services` sections.

### Environment-Specific Configuration

Each module can have environment-specific overrides:

```
libs/modules/<module>/config/
  application.yml           # Base configuration
  application-dev.yml       # Development overrides
  application-prod.yml      # Production overrides
```

---

## Environment Variables

| Variable | Used By | Description |
|----------|---------|-------------|
| `DATABASE_URL` | migration, seed | Full PostgreSQL connection URL |
| `POSTGRES_DB` | migration, seed | Database name |
| `POSTGRES_HOST` | migration, seed | Database host (default: `localhost`) |
| `POSTGRES_PORT` | migration, seed | Database port (default: `5432`) |
| `POSTGRES_USER` | migration, seed | Database user (default: `postgres`) |
| `POSTGRES_PASSWORD` | migration, seed | Database password (default: `password`) |
| `RUST_LOG` | all | Log level (set to `debug` with `--verbose`) |

---

## URL Sanitization

When displaying database URLs in output, passwords are automatically masked:

```
postgresql://user:secret@localhost:5432/mydb
→ postgresql://user:***@localhost:5432/mydb
```

Supported URL schemes: `postgresql://`, `postgres://`, `mysql://`, `sqlite://`

---

## Validation Rules

### Module Names

- Cannot be empty
- Only alphanumeric characters and underscores
- Maximum 50 characters

### Entity Names

- Cannot be empty
- Must be PascalCase (first character uppercase)
- Maximum 100 characters

### App Names

- Must be 3-50 characters
- kebab-case only (lowercase, numbers, hyphens)
- Cannot start or end with a hyphen
- No consecutive hyphens
- Reserved names: `metaphor`, `framework`, `cli`, `test`, `demo`

---

## Utility Functions

The `utils.rs` module provides these helpers:

| Function | Description |
|----------|-------------|
| `get_database_url()` | Centralized database URL resolution |
| `expand_env_vars(input)` | Replace `${VAR:default}` patterns |
| `get_env_value(file, key)` | Extract value from `.env` file |
| `path_exists(path)` | Check if path exists |
| `ensure_dir_exists(path)` | Create directory with parents |
| `timestamp()` | Current UTC timestamp as string |
| `generate_uuid()` | Random UUID v4 |
| `validate_module_name(name)` | Validate module name format |
| `validate_entity_name(name)` | Validate entity name format |
| `sanitize_db_url(url)` | Mask password in connection URL |

---

## See Also

- [Migration Commands](commands-migration.md) -- Database migration operations
- [Seed Commands](commands-seed.md) -- Database seeding operations
- [Module Commands](commands-module.md) -- Module enable/disable configuration
