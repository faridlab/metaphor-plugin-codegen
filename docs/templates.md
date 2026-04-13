# Template System

The plugin uses two template processing mechanisms to generate code: **simple placeholder replacement** for `make` commands and **Handlebars-based processing** for `apps` and `module` commands.

---

## Template Directories

All templates are stored under `src/templates/`:

```
src/templates/
  module/              # Module (bounded context) scaffold
  app/                 # Application scaffold (Clean Architecture)
  crud/                # CRUD operation templates
  aggregate/           # Aggregate root templates
  make/                # Make command templates
    command/           # CQRS command templates
    query/             # CQRS query templates
    repository/        # Repository trait + implementation templates
    handler/           # HTTP handler templates
    service/           # Domain service templates
    event/             # Domain event templates (.proto + .rs)
    value_object/      # Value object templates (.proto + .rs)
    specification/     # Specification templates
    migration/         # Migration SQL templates
  jobs/                # Background job templates
```

Additionally, the `make` commands load templates from:
```
crates/metaphor-cli/src/templates/make/
```

---

## Simple Placeholder Replacement

Used by all `make` commands. Templates contain `{{PLACEHOLDER}}` markers that are replaced with actual values via a `HashMap<String, String>`.

### How It Works

1. Template is loaded from disk as a string
2. A `HashMap` of placeholder-to-value mappings is created
3. Each placeholder is replaced using `String::replace()`
4. The processed content is written to the output file
5. Optionally, `mod.rs` is updated with a `pub mod` declaration

### Template Loading

Templates are loaded from the make templates directory:

```rust
// Template path: crates/metaphor-cli/src/templates/make/<type>/<filename>
let template = load_template("command", "{{COMMAND_NAME_SNAKE}}.rs")?;
```

### Placeholder Processing

```rust
let mut replacements = HashMap::new();
replacements.insert("{{COMMAND_NAME}}".to_string(), "CreatePayment".to_string());
replacements.insert("{{COMMAND_NAME_SNAKE}}".to_string(), "create_payment".to_string());
replacements.insert("{{ENTITY_NAME}}".to_string(), "Payment".to_string());

let content = process_template(&template, &replacements);
```

### mod.rs Auto-Update

When a file is generated, the `mod.rs` in the same directory can be automatically updated:

```rust
// Adds this line to mod.rs if not already present:
// pub mod create_payment;
write_generated_file(&output_path, &content, true, "create_payment")?;
```

If `mod.rs` doesn't exist, it is created with a header comment.

---

## Template Context (Module Templates)

The `TemplateContext` struct provides comprehensive placeholder values for module and entity template processing.

### Creating a Context

```rust
// For module creation
let context = TemplateContext::new("payments", "John Doe", Some("Payment processing"));

// For entity creation
let context = TemplateContext::new_for_entity("payments", "Payment", "John Doe", true);

// For aggregate creation
let context = TemplateContext::new_for_aggregate(
    "payments", "Order", "John Doe",
    true,   // with_common_fields
    true,   // with_events
    true,   // with_repository
    Some(vec!["OrderItem".to_string()]),     // entities
    Some(vec!["Money".to_string()]),         // value_objects
);
```

### Context Fields

| Field | Type | Description |
|-------|------|-------------|
| `module_name` | `String` | Original module name |
| `module_name_upper` | `String` | UPPERCASE module name |
| `module_name_lower` | `String` | lowercase module name |
| `author` | `String` | Author name |
| `description` | `Option<String>` | Module description |
| `entity_name` | `Option<String>` | Entity name |
| `entity_name_pascal` | `Option<String>` | Entity in PascalCase |
| `entity_name_snake` | `Option<String>` | Entity in snake_case |
| `entity_plural` | `Option<String>` | Pluralized entity name |
| `with_common_fields` | `bool` | Include timestamp fields |
| `aggregate_name` | `Option<String>` | Aggregate root name |
| `aggregate_name_pascal` | `Option<String>` | Aggregate in PascalCase |
| `aggregate_name_snake` | `Option<String>` | Aggregate in snake_case |
| `aggregate_plural` | `Option<String>` | Pluralized aggregate name |
| `with_events` | `bool` | Generate domain events |
| `with_repository` | `bool` | Generate repository |
| `entities` | `Option<Vec<String>>` | List of entity names |
| `value_objects` | `Option<Vec<String>>` | List of value object names |

### Placeholder Mappings

The `get_replacements()` method generates a complete mapping:

| Placeholder | Source |
|-------------|--------|
| `{{MODULE_NAME}}` | `module_name` |
| `{{MODULE_NAME_PASCAL}}` | PascalCase of module name |
| `{{PascalCaseModuleName}}` | Same as above (alias) |
| `{{MODULE_NAME_SNAKE}}` | snake_case of module name |
| `{{MODULE_NAME_UPPER}}` | UPPERCASE of module name |
| `{{MODULE_NAME_LOWER}}` | lowercase of module name |
| `{{AUTHOR}}` | `author` |
| `{{DESCRIPTION}}` | `description` (if present) |
| `{{ENTITY_NAME}}` | `entity_name` |
| `{{PascalCaseEntity}}` | `entity_name_pascal` |
| `{{ENTITY_NAME_SNAKE}}` | `entity_name_snake` |
| `{{entity_name_snake}}` | Same (lowercase alias for CRUD) |
| `{{ENTITY_NAME_PLURAL}}` | `entity_plural` |
| `{{entity_name_plural}}` | Same (lowercase alias for CRUD) |
| `{{CURRENT_TIMESTAMP}}` | Current UTC time in RFC 3339 |

### Conditional Placeholders

When `with_common_fields` is `true`, these special placeholders are replaced:

- `TIMESTAMP_FIELDS_PLACEHOLDER` -- replaced with proto timestamp field definitions
- `COMMON_FIELDS_PLACEHOLDER` -- replaced with Rust struct timestamp fields

When `false`, both placeholders are replaced with empty strings.

---

## Directory Processing

For module and entity creation, entire directory trees are copied and processed:

### File Processing Rules

| File Extension | Processing |
|----------------|------------|
| `.rs` | Placeholder replacement in content |
| `.proto` | Placeholder replacement in content |
| `.toml` | Placeholder replacement in content |
| `.yaml` | Placeholder replacement in content |
| `.md` | Placeholder replacement in content |
| Other extensions | Copied as-is (binary files) |

### Filename Processing

Filenames containing placeholders are also processed:

```
# Template filename → Generated filename
{{MODULE_NAME_SNAKE}}_handler.rs  →  payments_handler.rs
{{PascalCaseEntity}}_service.rs   →  Payment_service.rs
{{entity_name_snake}}_model.rs    →  payment_model.rs
```

### Skipped Files

These files/directories are skipped during template copying:

- `target/` -- build artifacts
- `build/` -- build output
- `Cargo.lock` -- lock file (regenerated on build)
- `.git/` -- git metadata

---

## Handlebars Processing (App Generator)

The `AppGenerator` uses the Handlebars template engine for more sophisticated template processing.

### Registered Helpers

| Helper | Usage in Template | Result |
|--------|-------------------|--------|
| `pascal_case` | `{{pascal_case name}}` | `MyService` |
| `snake_case` | `{{snake_case name}}` | `my_service` |
| `kebab_case` | `{{kebab_case name}}` | `my-service` |
| `camel_case` | `{{camel_case name}}` | `myService` |
| `upper_case` | `{{upper_case name}}` | `MY_SERVICE` |
| `title_case` | `{{title_case name}}` | `My Service` |

### Template Variables

The `AppGeneratorConfig` provides these variables to Handlebars templates:

| Variable | Example |
|----------|---------|
| `APP_NAME` | `my-service` |
| `APP_NAME_PASCAL` | `MyService` |
| `APP_NAME_SNAKE` | `my_service` |
| `APP_NAME_KEBAB` | `my-service` |
| `APP_NAME_CAMEL` | `myService` |
| `APP_PORT` | `3000` |
| `DATABASE_TYPE` | `postgresql` |
| `DATABASE_NAME` | `my_service_db` |
| `AUTH_ENABLED` | `true` |
| `METRICS_ENABLED` | `false` |
| `IS_WORKER` | `false` |
| `IS_SCHEDULER` | `false` |

### File Skipping

The app generator skips these files/directories:

- `target/` -- build output
- `.git/` -- git metadata
- `node_modules/` -- npm dependencies
- `.log` files -- log files
- `.DS_Store` -- macOS metadata
- `.env.local` -- local environment

### Workspace Integration

After generating an app, the generator:

1. Searches upward from the output directory for a workspace `Cargo.toml`
2. Adds the app to the `[workspace] members` list
3. Saves the updated `Cargo.toml`

---

## Case Conversion Functions

The template system provides these case conversion utilities:

### PascalCase

Splits by `_` or `-`, capitalizes each word's first letter:
- `payment_gateway` -> `PaymentGateway`
- `my-service` -> `MyService`

### snake_case

Inserts `_` before uppercase letters, lowercases everything:
- `PaymentGateway` -> `payment_gateway`
- `my-service` -> `my_service`

### Pluralization

Simple English pluralization rules:
- Words ending in `s`, `x`, `z`, `ch`, `sh` -> add `es` (e.g., `address` -> `addresses`)
- Words ending in `y` -> replace with `ies` (e.g., `category` -> `categories`)
- Other words -> add `s` (e.g., `payment` -> `payments`)

---

## Template Directories Reference

| Function | Returns |
|----------|---------|
| `get_module_template_dir()` | `crates/metaphor-cli/src/templates/module/` |
| `get_entity_template_dir()` | `crates/metaphor-cli/src/templates/entity/` |
| `get_crud_template_dir()` | `crates/metaphor-cli/src/templates/crud/` |
| `get_aggregate_template_dir()` | `crates/metaphor-cli/src/templates/aggregate/` |
| `get_make_template_dir()` | `crates/metaphor-cli/src/templates/make/` |

---

## See Also

- [Make Commands](commands-make.md) -- Commands using simple placeholder templates
- [Apps Commands](commands-apps.md) -- Commands using Handlebars templates
- [Module Commands](commands-module.md) -- Module creation with directory processing
