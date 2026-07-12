# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Botifactory is a Rust web service built with Axum that manages software release distribution. It provides REST APIs for projects, channels, and releases with file upload capabilities. The service uses SQLite for data persistence and includes structured logging with tracing.

## Architecture

- **Web Framework**: Axum with modular routing structure
- **Database**: SQLite with sqlx for async database operations  
- **Configuration**: Hierarchical config system (base.toml + environment-specific)
- **Logging**: Structured logging with tracing and bunyan formatting
- **File Storage**: Local filesystem storage for release binaries
- **API Structure**: REST endpoints organized by domain (projects, channels, releases)

The main application modules are:
- `routes/`: API endpoints grouped by domain (project, channel, release, health_check)
- `configuration.rs`: Environment-based config loading with validation
- `startup.rs`: Application initialization and server setup
- `telemetry.rs`: Logging and tracing configuration

## Development Setup

### Prerequisites
Install required tools:
```shell
cargo install tracing-bunyan-formatter sqlx-cli
```

### Environment Variables
Set these variables (direnv recommended):
```shell
export DATABASE_URL="sqlite://$PWD/develop.db"
export BOTIFACTORY_APP__DATABASE__URL="file://$PWD/develop.db"
```

### Database Setup
```shell
sqlx database setup
```

## Common Commands

### Build and Run
```shell
cargo build
cargo run
```

### Run with formatted logs
```shell
cargo run | bunyan
```

### Testing
```shell
cargo test
```

### Database Operations
```shell
sqlx database setup     # Initialize database and run migrations
sqlx migrate run        # Run pending migrations
```

### Code Quality
```shell
cargo clippy           # Linting
cargo fmt             # Code formatting
cargo check           # Type checking
```

## Configuration System

The application uses a layered configuration approach:
1. `configuration/base.toml` - Base settings
2. `configuration/{environment}.toml` - Environment overrides
3. Environment variables with `BOTIFACTORY_APP__` prefix

Environment is controlled by `BOTIFACTORY_APP_ENVIRONMENT` (defaults to "development").

## API Endpoints

The service provides REST APIs for:
- **Projects**: Create and manage software projects
- **Channels**: Manage release channels (stable, unstable, etc.)
- **Releases**: Upload and distribute release binaries
- **Health Check**: Service status endpoint

## Dependencies

Key external dependencies:
- `botifactory-types`: Type definitions (local path or git)
- `botifactory-common`: Shared utilities (git dependency)
- `axum`: Web framework with routing and middleware
- `sqlx`: Async SQL with compile-time verification
- `tracing`: Structured logging and instrumentation