# Template System

> ⚠️ **Accuracy note (v0.2.0):** both `module create` and `apps generate` **no longer use local
> templates**. They clone an external skeleton repo (`backbone-module` / `backbone-application`
> respectively) and stamp names in — see [ADR-0002](adr/0002-skeleton-clone-scaffolding.md) and the
> [Maintainer guide](maintainer-guide.md#how-templating-actually-works). Only the **`make`** targets
> still use local templates (string replacement). References below to `module` or `apps` using local
> templates / Handlebars are legacy; the `src/templates/app/` tree, the Handlebars helpers, and the
> `template_processor.rs` path helpers are dead code.

The plugin now uses a single local template mechanism — **simple placeholder replacement** for `make`
commands. `module` and `apps` are clone-and-stamp (no template engine).

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

## Skeleton-clone (App Generator) — the current reality

> **Legacy:** the `AppGenerator` used to expand `src/templates/app/` with the Handlebars engine and
> register `pascal_case` / `snake_case` / … helpers. As of **v0.2.0** it no longer does. The
> Handlebars engine, its helpers, the `src/templates/app/` tree, and the automatic workspace-member
> insertion are all removed.

`apps generate` now works exactly like `module create` (see below): it clones the canonical
[`backbone-application`](https://github.com/faridlab/backbone-application) skeleton and stamps names
in. Concretely, `AppGenerator::generate_app`:

1. Bails if the target `apps/<name>/` already exists (won't clobber).
2. `git clone --depth 1 https://github.com/faridlab/backbone-application <name>` — the skeleton repo
   is the single source of truth for app structure.
3. Removes `.git` and `Cargo.lock` (detach from the skeleton; resolve deps fresh).
4. Stamps the skeleton's baked-in package name across every UTF-8 file via `replace_token_in_tree`:
   `backbone-app` → `<name>` (kebab) and `backbone_app` → `<name_snake>` (snake). Binary/non-UTF-8
   files and `.git` / `target` are skipped.
5. Prints next steps, including **register the app in `metaphor.yaml`** (the generator no longer edits
   any workspace `Cargo.toml`).

Requires `git` on PATH and network access; it fails offline with an explicit error. See
[ADR-0002](adr/0002-skeleton-clone-scaffolding.md) for the rationale.

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
- [Apps Commands](commands-apps.md) -- App generation via skeleton clone
- [Module Commands](commands-module.md) -- Module creation via skeleton clone
