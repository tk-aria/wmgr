# API Overview

This document provides a high-level overview of the tsrc API structure.

## Modules

### Domain Layer (`tsrc::domain`)

The domain layer contains the core business logic and entities:

- `entities/`: Core business entities
  - `workspace::Workspace`: Represents a workspace containing repositories
  - `manifest::Manifest`: Configuration defining repositories
  - `repository::Repository`: Individual repository configuration

- `value_objects/`: Type-safe value objects
  - `git_url::GitUrl`: Git URL representation
  - `branch_name::BranchName`: Branch name representation
  - `file_path::FilePath`: File path representation

### Application Layer (`tsrc::application`)

The application layer contains use cases and services:

- `use_cases/`: Main application workflows
  - `init_workspace`: Initialize workspaces
  - `sync_repositories`: Synchronize repositories
  - `status_check`: Check repository status
  - `foreach_command`: Execute commands across repositories

- `services/`: Application services
  - `manifest_service`: Manifest parsing and validation

### Infrastructure Layer (`tsrc::infrastructure`)

The infrastructure layer provides concrete implementations:

- `git/`: Git operations using libgit2
- `filesystem/`: File system operations and storage
- `process/`: External process execution

### Presentation Layer (`tsrc::presentation`)

The presentation layer handles user interaction:

- `cli/`: Command-line interface implementation

### Common (`tsrc::common`)

Shared utilities and cross-cutting concerns:

- `error`: Error handling system
- `result`: Type aliases for results
- `executor`: Task execution framework

## Key Types

### Error Handling

All operations return `tsrc::Result<T>` which is an alias for `Result<T, TsrcError>`.

### Configuration

Most use cases accept configuration structs that can be built using builder patterns.

### Async Operations

Many operations are async and return futures that need to be awaited.

## Usage Patterns

### Basic Pattern

1. Create configuration object
2. Instantiate use case with configuration
3. Execute use case
4. Handle results

### Error Handling

Use the `?` operator for propagating errors or match on results for custom handling.

### Parallel Operations

Many operations support parallel execution for better performance.
