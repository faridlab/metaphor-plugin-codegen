# Seed Commands

Database seeding commands for managing initial data, test fixtures, and reference data. Supports both raw SQL and programmatic Rust seeder formats.

---

## Concepts

### Seed Types

| Type | Purpose | Example |
|------|---------|---------|
| `data` | Production-ready initial data | Default admin user, system settings |
| `test` | Development/testing fixtures | Sample users, test orders |
| `reference` | Lookup tables and reference data | Countries, currencies, status codes |

### Seed Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| `sql` | `.sql` | Simple INSERT statements, portable across environments |
| `rust` | `.rs` | Programmatic seeders using metaphor-orm, complex logic |

### Seed Locations

| Scope | Directory |
|-------|-----------|
| Module-level | `libs/modules/<module>/migrations/seeds/` |
| App-level | `apps/<app>/migrations/seeds/` |
| Rust seeders (module) | `libs/modules/<module>/migrations/seeders/` |
| Rust seeders (app) | `apps/<app>/migrations/seeders/` |

---

## seed create

Create a new seed file with timestamped naming.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Seed name (e.g., `initial_users`, `test_data`) |
| `-t`, `--type` | No | `data` | Seed type: `data`, `test`, `reference` |
| `-m`, `--module` | No | -- | Target module (for module-level seeds) |
| `-a`, `--app` | No | `metaphor` | Target app (for app-level seeds) |
| `-f`, `--format` | No | `sql` | Seed format: `sql` or `rust` |

```bash
# SQL seed for a module
metaphor-codegen seed create initial_payments --module payments

# Test data seed
metaphor-codegen seed create test_users --module users --type test

# Reference data
metaphor-codegen seed create currencies --type reference --module payments

# Rust programmatic seeder
metaphor-codegen seed create initial_users --module users --format rust

# App-level seed
metaphor-codegen seed create system_settings --app metaphor
```

### Generated Files (SQL format)

```
libs/modules/payments/migrations/seeds/
  20260414_120000_initial_payments.sql          # Forward seed
  20260414_120000_initial_payments_revert.sql   # Revert seed
```

### Generated Files (Rust format)

```
libs/modules/users/migrations/seeders/
  initial_users_seeder.rs          # Rust seeder implementation
  mod.rs                           # Updated with module declaration
```

### SQL Seed Templates

The generated SQL content varies by seed type:

**Data seeds** -- production-ready INSERT statements with conflict handling:
```sql
INSERT INTO table_name (column1, column2) VALUES ('value1', 'value2')
ON CONFLICT DO NOTHING;
```

**Test seeds** -- development data with clear markers:
```sql
-- Test data for development only
INSERT INTO table_name (column1, column2) VALUES ('test_value1', 'test_value2');
```

**Reference seeds** -- lookup table population:
```sql
-- Reference data: currencies
INSERT INTO table_name (code, name) VALUES ('USD', 'US Dollar')
ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name;
```

### Rust Seeder Template

Generates a seeder struct implementing the `Seeder` trait with `run()` and `revert()` methods:

```rust
pub struct InitialUsersSeeder;

impl Seeder for InitialUsersSeeder {
    async fn run(&self, db: &DatabasePool) -> Result<()> {
        // Insert seed data
    }

    async fn revert(&self, db: &DatabasePool) -> Result<()> {
        // Remove seed data
    }
}
```

---

## seed run

Run database seeds (all or specific).

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | No | all | Specific seed to run (runs all if omitted) |
| `-m`, `--module` | No | -- | Target module |
| `-a`, `--app` | No | `metaphor` | Target app |
| `--force` | No | `false` | Force re-run even if already applied |
| `-f`, `--format` | No | `rust` | Seed format: `sql` or `rust` |

```bash
# Run all SQL seeds for a module
metaphor-codegen seed run --module payments --format sql

# Run specific seed
metaphor-codegen seed run initial_payments --module payments --format sql

# Run Rust seeders
metaphor-codegen seed run --module users --format rust

# Force re-run
metaphor-codegen seed run --module payments --force --format sql
```

### SQL Seed Execution

SQL seeds are executed using `psql`:

1. Loads seed files from the seeds directory
2. Filters by name if specified
3. Resolves database URL from `DATABASE_URL` environment variable or configuration
4. Executes each seed file with `psql -f`
5. Reports success or failure per seed

### Rust Seeder Execution

Rust seeders are run by building and executing the module's seed binary:

1. Loads seeder files from the seeders directory
2. Builds the module crate with `cargo build`
3. Executes the seeder binary

---

## seed revert

Revert applied seeds by running their revert scripts.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | No | all | Specific seed to revert (reverts all if omitted) |
| `-m`, `--module` | No | -- | Target module |
| `-a`, `--app` | No | `metaphor` | Target app |

```bash
# Revert all seeds for a module
metaphor-codegen seed revert --module payments

# Revert specific seed
metaphor-codegen seed revert initial_payments --module payments
```

Executes the corresponding `*_revert.sql` files to undo seed data.

---

## seed status

Show the current status of seeds (applied vs pending).

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `-m`, `--module` | No | -- | Target module |
| `-a`, `--app` | No | `metaphor` | Target app |

```bash
metaphor-codegen seed status --module payments
```

---

## seed history

Show seed execution history with timestamps.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `-m`, `--module` | No | -- | Target module |
| `-a`, `--app` | No | `metaphor` | Target app |
| `-l`, `--limit` | No | `20` | Number of entries to show |

```bash
# Recent history
metaphor-codegen seed history --module payments

# Last 50 entries
metaphor-codegen seed history --module payments --limit 50
```

---

## seed list

List all available seed files.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `-m`, `--module` | No | -- | Target module |
| `-a`, `--app` | No | `metaphor` | Target app |

```bash
metaphor-codegen seed list --module payments
```

Lists all seed files (both SQL and Rust) with their type indicators.

---

## seed run-all

Run seeds for **all** registered modules. Discovers all enabled modules and runs their seeds in order.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `--force` | No | `false` | Force re-run even if already applied |
| `-f`, `--format` | No | `sql` | Seed format: `sql` or `rust` |

```bash
# Run all SQL seeds across all modules
metaphor-codegen seed run-all

# Run all Rust seeders
metaphor-codegen seed run-all --format rust

# Force re-run all
metaphor-codegen seed run-all --force
```

### How It Works

1. Discovers all modules under `libs/modules/`
2. Filters to modules that have seed directories (based on format)
3. Runs seeds for each module in alphabetical order
4. Reports per-module success/failure
5. Prints summary with success and error counts

### Example Output

```
Running SQL seeds for ALL modules...

Found 3 modules with SQL seeds:
   - orders
   - payments
   - users

Module: orders
   Running initial_orders... completed
Module: payments
   Running initial_payments... completed
Module: users
   Running initial_users... completed

Summary:
   Successful: 3
All seeds completed successfully!
```

---

## Best Practices

1. **Use SQL seeds for simple data** -- INSERT statements are portable and easy to review.

2. **Use Rust seeders for complex logic** -- when seeding requires computed values, API calls, or conditional logic.

3. **Make seeds idempotent** -- use `ON CONFLICT DO NOTHING` or `ON CONFLICT DO UPDATE` in SQL seeds so they can be re-run safely.

4. **Always provide revert scripts** -- SQL seeds automatically generate a `_revert.sql` file. Keep it updated to enable clean rollbacks.

5. **Separate test data from production data** -- use `--type test` for development fixtures and `--type data` for production seeds.

6. **Use `run-all` for full environment setup** -- when provisioning a new development environment, `seed run-all` seeds all modules in one command.

7. **Reference data goes in `reference` type** -- currencies, countries, and status codes rarely change. Keep them separate from business data.

---

## See Also

- [Migration Commands](commands-migration.md) -- Database migration lifecycle
- [Configuration](configuration.md) -- Database URL configuration
