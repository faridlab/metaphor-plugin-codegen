# Make Commands

Laravel-inspired scaffolding commands for quickly generating DDD components. Each command creates boilerplate code following Metaphor Framework conventions.

## Prerequisites

Most `make` commands require an existing module. Create one first:

```bash
metaphor-codegen module create payments
```

Module path: `libs/modules/<name>/`

---

## make module

Create a new module with bounded context structure. Delegates to [`module create`](commands-module.md#module-create).

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Module name (lowercase, e.g., `payments`) |
| `--description` | No | -- | Module description |

```bash
metaphor-codegen make module payments --description "Payment processing domain"
```

**Output:** Full DDD module structure at `libs/modules/payments/`. See [module create](commands-module.md#module-create) for the complete generated directory tree.

---

## make entity

> **DEPRECATED:** This command now uses the schema-first approach. Instead of generating entities directly, create a schema YAML file and use `metaphor schema generate`.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Entity name (PascalCase, e.g., `Payment`) |
| `-m`, `--module` | Yes | -- | Target module |
| `--soft-delete` | No | `false` | Include soft delete support |
| `--versioned` | No | `false` | Include versioning for optimistic locking |

### Recommended Workflow

1. Create the schema file:
   ```
   libs/modules/<module>/schema/models/<entity>.model.yaml
   ```

2. Define the entity:
   ```yaml
   models:
     - name: Payment
       collection: payments
       fields:
         id:
           type: uuid
           attributes: ["@id", "@default(uuid)"]
         # Add your fields here...
   ```

3. Generate code from schema:
   ```bash
   metaphor schema generate payments --target proto,rust,sql
   ```

---

## make command

Create a CQRS command (write operation). Commands represent intentions to change state.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Command name in PascalCase (e.g., `CreateUser`) |
| `-m`, `--module` | Yes | -- | Target module |
| `-e`, `--entity` | Yes | -- | Entity this command operates on |

```bash
metaphor-codegen make command CreatePayment --module payments --entity Payment
```

**Generated files:**
```
libs/modules/payments/
  src/application/commands/
    create_payment.rs              # Command implementation
    mod.rs                         # Updated with: pub mod create_payment;
```

**Template placeholders used:**
- `{{COMMAND_NAME}}` -> `CreatePayment`
- `{{COMMAND_NAME_SNAKE}}` -> `create_payment`
- `{{COMMAND_DESCRIPTION}}` -> `create payment`
- `{{ENTITY_NAME}}` -> `Payment`

### Naming Conventions

Use imperative verb + entity name:
- `CreateUser`, `UpdatePayment`, `DeleteOrder`
- `ApproveRequest`, `CancelSubscription`, `RestoreUser`

---

## make query

Create a CQRS query (read operation). Queries retrieve data without side effects.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Query name in PascalCase (e.g., `GetUser`) |
| `-m`, `--module` | Yes | -- | Target module |
| `-e`, `--entity` | Yes | -- | Entity this query returns |

```bash
metaphor-codegen make query ListPayments --module payments --entity Payment
```

**Generated files:**
```
libs/modules/payments/
  src/application/queries/
    list_payments.rs               # Query implementation
    mod.rs                         # Updated with: pub mod list_payments;
```

**Template placeholders used:**
- `{{QUERY_NAME}}` -> `ListPayments`
- `{{QUERY_NAME_SNAKE}}` -> `list_payments`
- `{{ENTITY_NAME}}` -> `Payment`

### Naming Conventions

Use retrieval verb + entity name:
- `GetUser`, `ListPayments`, `SearchOrders`
- `FindByEmail`, `CountActiveUsers`

---

## make repository

Create a repository interface (trait) and its database implementation.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Entity name (e.g., `Payment`) |
| `-m`, `--module` | Yes | -- | Target module |
| `--database` | No | `postgres` | Database type: `postgres` or `mongodb` |

```bash
# PostgreSQL (default)
metaphor-codegen make repository Payment --module payments

# MongoDB
metaphor-codegen make repository Payment --module payments --database mongodb
```

**Generated files (PostgreSQL):**
```
libs/modules/payments/
  src/domain/repository/
    payment_repository.rs          # Repository trait (interface)
    mod.rs                         # Updated
  src/infrastructure/persistence/
    postgres_payment_repository.rs # PostgreSQL implementation
    mod.rs                         # Updated
```

**Generated files (MongoDB):**
```
libs/modules/payments/
  src/domain/repository/
    payment_repository.rs          # Repository trait (interface)
    mod.rs                         # Updated
  src/infrastructure/persistence/
    mongo_payment_repository.rs    # MongoDB implementation
    mod.rs                         # Updated
```

**Template placeholders used:**
- `{{ENTITY_NAME}}` -> `Payment`
- `{{ENTITY_NAME_SNAKE}}` -> `payment`
- `{{COLLECTION}}` -> `payments`

---

## make handler

Create an HTTP handler for a resource.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Handler name (e.g., `payment`) |
| `-m`, `--module` | Yes | -- | Target module |
| `--crud` | No | `false` | Generate all CRUD handler methods |

```bash
# Standard handler
metaphor-codegen make handler payment --module payments

# Full CRUD handler (create, read, update, delete, list)
metaphor-codegen make handler payment --module payments --crud
```

**Generated files:**
```
libs/modules/payments/
  src/presentation/http/
    payment_handler.rs             # HTTP handler (or payment_crud_handler.rs with --crud)
    mod.rs                         # Updated with: pub mod payment_handler;
```

**Template placeholders used:**
- `{{HANDLER_NAME}}` -> `Payment`
- `{{HANDLER_NAME_SNAKE}}` -> `payment`
- `{{COLLECTION}}` -> `payments`

The `--crud` flag generates a handler with endpoints for all standard CRUD operations instead of a minimal handler stub.

---

## make service

Create a domain service for cross-entity business logic.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Service name in PascalCase (e.g., `PaymentProcessor`) |
| `-m`, `--module` | Yes | -- | Target module |

```bash
metaphor-codegen make service PaymentProcessor --module payments
```

**Generated files:**
```
libs/modules/payments/
  src/domain/service/
    payment_processor.rs           # Domain service implementation
    mod.rs                         # Updated with: pub mod payment_processor;
```

**Template placeholders used:**
- `{{SERVICE_NAME}}` -> `PaymentProcessor`
- `{{SERVICE_NAME_SNAKE}}` -> `payment_processor`
- `{{SERVICE_DESCRIPTION}}` -> `payment processor`

### When to Use Domain Services

Use a domain service when business logic spans multiple entities or doesn't naturally belong to a single entity. Examples:
- `PaymentProcessor` -- coordinates payment validation, charging, and notification
- `PricingCalculator` -- applies discounts, taxes, and promotions across products
- `InventoryAllocator` -- manages stock allocation across warehouses

---

## make event

Create a domain event with both a Protocol Buffer definition and a Rust handler.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Event name in PascalCase (e.g., `PaymentCompleted`) |
| `-m`, `--module` | Yes | -- | Target module |
| `-e`, `--entity` | No | `entity` | Entity this event relates to |

```bash
metaphor-codegen make event PaymentCompleted --module payments --entity Payment
```

**Generated files:**
```
libs/modules/payments/
  proto/domain/event/
    payment_completed.proto        # Protocol Buffer definition
  src/domain/event/
    payment_completed.rs           # Rust event handler
    mod.rs                         # Updated with: pub mod payment_completed;
```

**Template placeholders used:**
- `{{EVENT_NAME}}` -> `PaymentCompleted`
- `{{EVENT_NAME_SNAKE}}` -> `payment_completed`
- `{{ENTITY_NAME_SNAKE}}` -> `payment`
- `{{MODULE_NAME}}` -> `payments`

### Naming Conventions

Domain events are named in **past tense** -- they describe something that already happened:
- `UserCreated`, `PaymentProcessed`, `OrderCancelled`
- `InventoryUpdated`, `EmailSent`, `SubscriptionExpired`

---

## make test

Create a test file. Delegates to the `metaphor-dev` plugin.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Test name or entity name |
| `-m`, `--module` | Yes | -- | Target module |
| `--type` | No | `unit` | Test type: `unit`, `integration`, or `e2e` |

```bash
metaphor-codegen make test Payment --module payments --type integration
```

This command prints instructions to use the `metaphor-dev` plugin:

```
metaphor-dev test generate --module payments --entity Payment --integration
```

> **Note:** The `metaphor-plugin-dev` package must be installed separately for test generation.

---

## make migration

Create a timestamped database migration with up and down files.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Migration name (e.g., `create_payments_table`) |
| `-m`, `--module` | Yes | -- | Target module |
| `--create` | No | -- | Table name for CREATE TABLE migration |
| `--table` | No | -- | Table name for ALTER TABLE migration |

```bash
# Generic migration
metaphor-codegen make migration add_status_to_payments --module payments

# Create table migration
metaphor-codegen make migration create_payments_table --module payments --create payments

# Alter table migration
metaphor-codegen make migration add_email_to_users --module users --table users
```

**Generated files:**
```
libs/modules/payments/
  migrations/
    20260414120000_create_payments_table.up.sql     # Forward migration
    20260414120000_create_payments_table.down.sql    # Rollback migration
```

The timestamp prefix (`YYYYMMDDHHMMSS`) ensures migrations run in chronological order.

> **Tip:** For entity-aware migration generation (with fields automatically mapped to SQL types), use [`migration generate`](commands-migration.md#migration-generate) instead.

---

## make value-object

Create a value object with both a Protocol Buffer definition and a Rust implementation.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Value object name in PascalCase (e.g., `Email`, `Money`) |
| `-m`, `--module` | Yes | -- | Target module |

```bash
metaphor-codegen make value-object Money --module payments
```

**Generated files:**
```
libs/modules/payments/
  proto/domain/value_object/
    money.proto                    # Protocol Buffer definition
  src/domain/value_object/
    money.rs                       # Rust implementation
    mod.rs                         # Updated with: pub mod money;
```

**Template placeholders used:**
- `{{VALUE_OBJECT_NAME}}` -> `Money`
- `{{VALUE_OBJECT_NAME_SNAKE}}` -> `money`
- `{{MODULE_NAME}}` -> `payments`

### When to Use Value Objects

Value objects are immutable and defined by their attributes, not identity:
- `Email` -- validated email address
- `Money` -- amount + currency pair
- `Address` -- street, city, country
- `DateRange` -- start date + end date
- `PhoneNumber` -- validated phone number

---

## make spec

Create a specification (business rule) as a composable predicate.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `name` | Yes | -- | Specification name in PascalCase (e.g., `UserIsActive`) |
| `-m`, `--module` | Yes | -- | Target module |

```bash
metaphor-codegen make spec PaymentIsValid --module payments
```

**Generated files:**
```
libs/modules/payments/
  src/domain/specification/
    payment_is_valid.rs            # Specification implementation
    mod.rs                         # Updated with: pub mod payment_is_valid;
```

**Template placeholders used:**
- `{{SPEC_NAME}}` -> `PaymentIsValid`
- `{{SPEC_NAME_SNAKE}}` -> `payment_is_valid`

### When to Use Specifications

Specifications encapsulate business rules that can be composed:
- `UserIsActive` -- user account is not suspended or deleted
- `PaymentIsValid` -- payment amount is positive and currency is supported
- `OrderCanBeCancelled` -- order is in a cancellable state
- `UserHasPermission` -- user has required role/permission

---

## Template Placeholders Reference

All `make` commands use template-based code generation with placeholder replacement:

| Placeholder | Description | Example Value |
|-------------|-------------|---------------|
| `{{COMMAND_NAME}}` | Command name in PascalCase | `CreatePayment` |
| `{{COMMAND_NAME_SNAKE}}` | Command name in snake_case | `create_payment` |
| `{{COMMAND_DESCRIPTION}}` | Human-readable description | `create payment` |
| `{{QUERY_NAME}}` | Query name in PascalCase | `ListPayments` |
| `{{QUERY_NAME_SNAKE}}` | Query name in snake_case | `list_payments` |
| `{{ENTITY_NAME}}` | Entity name in PascalCase | `Payment` |
| `{{ENTITY_NAME_SNAKE}}` | Entity name in snake_case | `payment` |
| `{{HANDLER_NAME}}` | Handler name in PascalCase | `Payment` |
| `{{HANDLER_NAME_SNAKE}}` | Handler name in snake_case | `payment` |
| `{{SERVICE_NAME}}` | Service name in PascalCase | `PaymentProcessor` |
| `{{SERVICE_NAME_SNAKE}}` | Service name in snake_case | `payment_processor` |
| `{{EVENT_NAME}}` | Event name in PascalCase | `PaymentCompleted` |
| `{{EVENT_NAME_SNAKE}}` | Event name in snake_case | `payment_completed` |
| `{{VALUE_OBJECT_NAME}}` | Value object name in PascalCase | `Money` |
| `{{VALUE_OBJECT_NAME_SNAKE}}` | Value object name in snake_case | `money` |
| `{{SPEC_NAME}}` | Specification name in PascalCase | `PaymentIsValid` |
| `{{SPEC_NAME_SNAKE}}` | Specification name in snake_case | `payment_is_valid` |
| `{{MODULE_NAME}}` | Module name | `payments` |
| `{{COLLECTION}}` | Pluralized entity name | `payments` |
| `{{TABLE_NAME}}` | Table name in snake_case | `payments` |
| `{{MIGRATION_NAME}}` | Timestamped migration name | `20260414_create_payments` |

Templates are loaded from `crates/metaphor-cli/src/templates/make/`.

---

## Best Practices

1. **Create the module first** before using `make` commands:
   ```bash
   metaphor-codegen module create payments
   ```

2. **Use schema-first for entities** -- the `make entity` command is deprecated. Define entities in YAML schema files and generate code from there.

3. **Follow naming conventions** -- PascalCase for commands/queries/events/specifications, snake_case for modules and file names.

4. **One command per action** -- each CQRS command should represent a single business action. Prefer `CreatePayment` and `UpdatePaymentStatus` over a generic `ManagePayment`.

5. **Use `--crud` for standard resources** -- when creating handlers for entities with standard CRUD operations, use the `--crud` flag to generate all endpoints at once.

6. **Pair events with commands** -- when a command changes state, create a corresponding domain event to notify other parts of the system.

## See Also

- [Architecture & Concepts](architecture.md) -- DDD and Clean Architecture patterns
- [Module Commands](commands-module.md) -- Creating and managing modules
- [Migration Commands](commands-migration.md) -- Entity-aware migration generation
- [Template System](templates.md) -- Template internals and customization
