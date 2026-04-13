# {{title_case APP_NAME}}

{{APP_DESCRIPTION}}

🦀 **{{title_case APP_TYPE}} Service** - Metaphor Framework application

## 🎯 Overview

This is the main Metaphor Framework application that serves as a **Laravel-like monolith orchestrator** for managing Domain-Driven Design (DDD) bounded context modules. It provides the structure and tooling to build scalable backend applications with the simplicity of a monolith and the power of modular architecture.

## 🏗️ Architecture

```
apps/metaphor/                     # 🚀 MAIN APPLICATION (Laravel-like)
├── src/
│   ├── main.rs                    # Application entry point
│   ├── config/                    # ⚙️ Configuration management
│   ├── middleware/                # 🛡️ HTTP middleware
│   ├── routes/                    # 🛣️ Route definitions
│   └── shared/                    # 🔧 Shared utilities
├── config/                        # 📋 Configuration files
│   ├── application.yml            # Main configuration
│   ├── application-dev.yml        # Development overrides
│   └── application-prod.yml       # Production settings
├── migrations/                    # 🗄️ Database migrations
├── docker-compose.yml             # 🐳 Development environment
└── README.md                      # This file
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Docker & Docker Compose

### Development Setup

1. **Start Supporting Services**
   ```bash
   cd apps/metaphor
   docker-compose up -d postgres redis mailhog
   ```

2. **Set Environment Variables**
   ```bash
   export DATABASE_URL="postgresql://postgres:password@localhost:5432/bersihirdb"
   export JWT_SECRET="your-super-secret-jwt-key"
   export LOG_LEVEL="debug"
   ```

3. **Run the Application**
   ```bash
   # From monorepo root
   cargo run --bin metaphor

   # Or from apps/metaphor
   cargo run
   ```

4. **Verify Installation**
   ```bash
   curl http://localhost:3001/health
   curl http://localhost:3001/api/v1/status
   ```

## 📋 Available Endpoints

### Core Application
- `GET /` - Welcome message and service information
- `GET /health` - Basic health check
- `GET /health/detailed` - Detailed health status
- `GET /api/v1/status` - API status and available modules

### Administration
- `GET /admin` - Admin dashboard overview
- `GET /admin/modules` - Module management interface
- `GET /admin/config` - Configuration overview
- `GET /admin/health` - Detailed health check

### Development Services
- **MailHog UI**: http://localhost:8025 (Email testing)
- **Redis**: localhost:6379
- **PostgreSQL**: localhost:5432

## 🔧 Configuration

The application uses a Laravel-inspired configuration system with environment variable overrides:

### Main Configuration
See `config/application.yml` for all available settings.

### Environment-Specific Overrides
- `config/application-dev.yml` - Development settings
- `config/application-prod.yml` - Production settings

### Environment Variables
```bash
# Application
APP_ENV=development
HOST=0.0.0.0
PORT=3001

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/dbname

# Security
JWT_SECRET=your-super-secret-key
CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# Modules
SAPIENS_ENABLED=true
POSTMAN_ENABLED=true
BUCKET_ENABLED=true

# Email (Postman)
SMTP_HOST=localhost
SMTP_PORT=1025

# File Storage (Bucket)
STORAGE_TYPE=local
LOCAL_STORAGE_PATH=./storage
```

## 🏛️ Module Integration

The application is designed to orchestrate DDD bounded context modules:

### Current Modules
- **Sapiens** - User management bounded context
- **Postman** - Email notification bounded context
- **Bucket** - File storage bounded context

### Module Registration
Modules are automatically discovered and registered through the configuration system:

```yaml
modules:
  sapiens:
    enabled: true
    database_url: "${DATABASE_URL}"
    features:
      registration: true
      email_verification: true
```

### Future Module Integration
When modules are created using the Metaphor CLI:

```bash
# Create a new module
metaphor module create payments --description "Payment processing bounded context"

# Register in the application
# TODO: Implement automatic module discovery
```

## 🧪 Testing

### Run Tests
```bash
# Run all tests
cargo test

# Run application tests
cargo test --package metaphor-app

# Run with coverage
cargo test --features coverage
```

### Test Database
Tests use a separate test database:

```bash
export TEST_DATABASE_URL="postgresql://postgres:password@localhost:5432/metaphor_test"
cargo test integration
```

## 📊 Monitoring & Observability

### Health Checks
```bash
# Basic health
curl http://localhost:3001/health

# Detailed health
curl http://localhost:3001/health/detailed

# Admin health
curl http://localhost:3001/admin/health
```

### Logging
- Development: Pretty console output
- Production: Structured JSON logging
- Configurable log levels (trace, debug, info, warn, error)

### Metrics (Optional)
```bash
# Start with monitoring profile
docker-compose --profile monitoring up

# Prometheus: http://localhost:9091
# Grafana: http://localhost:3000 (admin/admin)
```

## 🐳 Docker Development

### Complete Development Stack
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f metaphor

# Stop services
docker-compose down
```

### Service Profiles
```bash
# With Elasticsearch
docker-compose --profile search up

# With monitoring
docker-compose --profile monitoring up

# With storage (MinIO)
docker-compose --profile storage up

# All services
docker-compose --profile search --profile monitoring --profile storage up
```

## 🚀 Deployment

### Production Deployment

1. **Environment Setup**
   ```bash
   export APP_ENV=production
   export DATABASE_URL="postgresql://prod_user:prod_pass@db-host:5432/prod_db"
   export JWT_SECRET="production-super-secret-key"
   ```

2. **Build Application**
   ```bash
   cargo build --release
   ```

3. **Database Migration**
   ```bash
   # TODO: Add migration command
   # ./target/release/metaphor migrate
   ```

4. **Run Application**
   ```bash
   ./target/release/metaphor
   ```

### Docker Deployment
```bash
# Build Docker image
docker build -t metaphor-app .

# Run with production config
docker run -p 3001:3001 \
  -e DATABASE_URL="$DATABASE_URL" \
  -e JWT_SECRET="$JWT_SECRET" \
  metaphor-app
```

## 🔒 Security Features

- **JWT Authentication** - Secure token-based authentication
- **CORS Support** - Configurable cross-origin resource sharing
- **Rate Limiting** - Configurable per-endpoint rate limits
- **Input Validation** - Request validation and sanitization
- **Security Headers** - HSTS, CSP, X-Frame-Options
- **Environment Variable Overrides** - Sensitive config via environment

## 🛠️ Development Workflow

### Adding New Modules
```bash
# 1. Create module using CLI
metaphor module create mymodule --description "My new bounded context"

# 2. Add module to application configuration
# Edit config/application.yml

# 3. Add module routes to the application
# Edit src/routes/mod.rs

# 4. Test integration
cargo test
```

### Adding New Routes
```bash
# 1. Add route handler
# Edit src/routes/mod.rs

# 2. Add middleware if needed
# Edit src/middleware/mod.rs

# 3. Test endpoint
curl http://localhost:3001/my-new-endpoint
```

### Configuration Changes
```bash
# 1. Update configuration files
# Edit config/application.yml or environment-specific files

# 2. Test configuration
cargo run --bin metaphor

# 3. Verify with health check
curl http://localhost:3001/admin/config
```

## 📚 API Documentation

### REST API Conventions
- **Base URL**: `http://localhost:3001/api/v1`
- **Authentication**: JWT Bearer tokens (when modules are implemented)
- **Content-Type**: `application/json`
- **Error Format**: Structured error responses

### Response Format
```json
{
  "success": true,
  "data": { ... }
}
```

### Error Format
```json
{
  "success": false,
  "error": {
    "message": "Error description",
    "code": 400
  }
}
```

## 🤝 Contributing

### Development Setup
1. Fork the repository
2. Create feature branch
3. Make changes with tests
4. Ensure all tests pass
5. Submit pull request

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test

# Check documentation
cargo doc --no-deps
```

## 🔗 Related Components

- **[Metaphor CLI](../../../crates/metaphor-cli/README.md)** - Code generation and module management
- **[Metaphor Core](../../../crates/metaphor-core/README.md)** - Core framework traits and utilities
- **[DDD Modules](../../../libs/modules/README.md)** - Domain-Driven Design bounded contexts

## 📄 License

This application is part of the Metaphor Framework and is licensed under the MIT License.

## 🆘 Support

For support and questions:

- **Documentation**: Check inline docs and README files
- **Issues**: Create an issue in the repository
- **Discussions**: Join community discussions
- **Examples**: Check the examples directory

---

🦀 **Built with Metaphor Framework** - The Modular Monolith Powerhouse