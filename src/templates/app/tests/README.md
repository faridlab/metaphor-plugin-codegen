# Tests

This directory contains tests for the `{{APP_NAME}}` application.

## Directory Structure

```
tests/
├── api/                    # API endpoint tests
│   └── (add test files here)
├── integration/            # Integration tests
│   └── (add test files here)
└── README.md              # This file
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run API tests only
cargo test api::

# Run integration tests only
cargo test integration::
```

## Test Organization

- **API Tests**: Test HTTP endpoints and request/response handling
- **Integration Tests**: Test multi-component interactions and database operations

## Writing Tests

```rust
// Example API test
#[tokio::test]
async fn test_health_endpoint() {
    // Setup test client
    // Call endpoint
    // Assert response
}

// Example integration test
#[tokio::test]
async fn test_user_creation_workflow() {
    // Setup database
    // Create user
    // Verify user exists
    // Cleanup
}
```
