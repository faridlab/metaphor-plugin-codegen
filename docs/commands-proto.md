# Proto Commands

Protocol Buffer operations for generating Rust-native types from `.proto` definitions and validating proto files for correctness.

---

## proto generate

Generate Rust-native types from Protocol Buffer definitions within a module.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `module` | Yes | -- | Target module name |
| `--force` | No | `false` | Force regeneration of existing types |
| `--dry-run` | No | `false` | Preview changes without writing files |

```bash
# Generate types for a module
metaphor-codegen proto generate payments

# Force regeneration
metaphor-codegen proto generate payments --force

# Preview what would be generated
metaphor-codegen proto generate payments --dry-run
```

### How It Works

1. Discovers proto files under `libs/modules/<module>/proto/domain/`
2. Recursively collects all `.proto` files from subdirectories
3. Generates Rust types into `libs/modules/<module>/src/generated/`

### Proto Directory Convention

Proto files are organized by domain concept:

```
libs/modules/<module>/proto/domain/
  entity/                          # Entity message definitions
    payment.proto
    transaction.proto
  event/                           # Domain event messages
    payment_completed.proto
    payment_failed.proto
  value_object/                    # Value object messages
    money.proto
    currency.proto
  repository/                      # Repository service definitions
  service/                         # Domain service definitions
  specification/                   # Specification messages
  usecase/                         # Use case service definitions
```

### Package Naming Convention

Proto files should use the package naming convention `<module>.domain`:

```protobuf
syntax = "proto3";
package payments.domain;

import "google/protobuf/timestamp.proto";
import "buf/validate/validate.proto";

message Payment {
    string id = 1;
    double amount = 2;
    string currency = 3;
    google.protobuf.Timestamp created_at = 4;
}
```

---

## proto lint

Lint and validate all proto files across modules for correctness and style.

| Argument | Required | Default | Description |
|----------|:--------:|---------|-------------|
| `--fix` | No | `false` | Auto-fix issues where possible |

```bash
# Lint all proto files
metaphor-codegen proto lint

# Lint with auto-fix
metaphor-codegen proto lint --fix
```

### How It Works

1. Discovers all modules under `libs/modules/` that contain `proto/domain/` directories
2. Recursively collects all `.proto` files from each module
3. Validates each file against lint rules
4. Reports issues per module and per file

### Lint Rules

The linter checks for:

| Rule | Description |
|------|-------------|
| **Package naming** | Package should match `<module>.domain` convention |
| **Required imports** | Files using `Timestamp` must import `google/protobuf/timestamp.proto` |
| **Validation imports** | Files using `buf.validate` must import `buf/validate/validate.proto` |
| **Syntax check** | Validates proto file structure for suspicious content |

### Example Output

```
Linting proto files...
  Module: payments
    payment.proto                  # pass
    payment_completed.proto        # pass
  Module: users
    user.proto                     # 1 potential issue
    Line 3: Package should be users.domain
Proto files linted successfully
```

### Module Discovery

The linter automatically discovers modules by scanning `libs/modules/` for directories that contain a `proto/domain/` subdirectory. Modules without proto files are skipped.

---

## Proto-to-Rust Type Mapping

When generating Rust types from proto definitions, the following type mappings apply:

| Proto Type | Rust Type |
|------------|-----------|
| `string` | `String` |
| `int32` | `i32` |
| `int64` | `i64` |
| `uint32` | `u32` |
| `uint64` | `u64` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `bytes` | `Vec<u8>` |
| `google.protobuf.Timestamp` | `prost_types::Timestamp` |
| `repeated T` | `Vec<T>` |
| `optional T` | `Option<T>` |
| `map<K, V>` | `HashMap<K, V>` |

### buf.yaml Configuration

Each module includes a `buf.yaml` configuration file for Protocol Buffer management:

```yaml
version: v1
breaking:
  use:
    - FILE
lint:
  use:
    - DEFAULT
```

---

## See Also

- [Architecture & Concepts](architecture.md) -- Proto-first design philosophy
- [Make Commands](commands-make.md) -- Commands that generate proto files (`make event`, `make value-object`)
- [Template System](templates.md) -- How proto templates work
