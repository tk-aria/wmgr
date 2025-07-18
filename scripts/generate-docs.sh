#!/bin/bash
# Documentation generation script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DOCS_DIR="target/doc"
OUTPUT_DIR=${1:-"docs"}

echo -e "${BLUE}Generating API documentation for tsrc...${NC}"

# Clean previous docs
echo -e "${YELLOW}Cleaning previous documentation...${NC}"
rm -rf "${DOCS_DIR}"
rm -rf "${OUTPUT_DIR}"

# Generate rustdoc
echo -e "${YELLOW}Generating rustdoc...${NC}"
RUSTDOCFLAGS="--cfg docsrs" cargo doc \
    --no-deps \
    --all-features \
    --document-private-items

# Check if generation was successful
if [ ! -d "${DOCS_DIR}" ]; then
    echo -e "${RED}Documentation generation failed!${NC}"
    exit 1
fi

# Copy generated docs to output directory
echo -e "${YELLOW}Copying documentation to ${OUTPUT_DIR}...${NC}"
cp -r "${DOCS_DIR}" "${OUTPUT_DIR}"

# Create index.html that redirects to the main crate
cat > "${OUTPUT_DIR}/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>tsrc Documentation</title>
    <meta http-equiv="refresh" content="0; url=tsrc/index.html">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 2rem;
            text-align: center;
        }
        .loading {
            margin-top: 2rem;
        }
        a {
            color: #007bff;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <h1>tsrc Documentation</h1>
    <div class="loading">
        <p>Redirecting to documentation...</p>
        <p>If you are not redirected automatically, <a href="tsrc/index.html">click here</a>.</p>
    </div>
</body>
</html>
EOF

# Generate additional documentation files
echo -e "${YELLOW}Generating additional documentation...${NC}"

# Create a simple API overview
cat > "${OUTPUT_DIR}/API_OVERVIEW.md" << 'EOF'
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
EOF

# Generate module documentation
echo -e "${YELLOW}Generating module documentation index...${NC}"

cat > "${OUTPUT_DIR}/MODULES.md" << 'EOF'
# Module Documentation

This page provides links to the main modules in the tsrc crate.

## Core Modules

- [tsrc](tsrc/index.html) - Main crate documentation
- [Domain](tsrc/domain/index.html) - Core business logic
- [Application](tsrc/application/index.html) - Use cases and services
- [Infrastructure](tsrc/infrastructure/index.html) - External integrations
- [Presentation](tsrc/presentation/index.html) - User interface
- [Common](tsrc/common/index.html) - Shared utilities

## Key Components

### Domain Entities

- [Workspace](tsrc/domain/entities/workspace/struct.Workspace.html)
- [Manifest](tsrc/domain/entities/manifest/struct.Manifest.html)
- [Repository](tsrc/domain/entities/repository/struct.Repository.html)

### Value Objects

- [GitUrl](tsrc/domain/value_objects/git_url/struct.GitUrl.html)
- [BranchName](tsrc/domain/value_objects/branch_name/struct.BranchName.html)
- [FilePath](tsrc/domain/value_objects/file_path/struct.FilePath.html)

### Use Cases

- [InitWorkspace](tsrc/application/use_cases/init_workspace/index.html)
- [SyncRepositories](tsrc/application/use_cases/sync_repositories/index.html)
- [StatusCheck](tsrc/application/use_cases/status_check/index.html)
- [ForeachCommand](tsrc/application/use_cases/foreach_command/index.html)

### Services

- [ManifestService](tsrc/application/services/manifest_service/index.html)

### Error Handling

- [TsrcError](tsrc/common/error/enum.TsrcError.html)
- [Result](tsrc/common/result/type.Result.html)
EOF

# Generate examples documentation
echo -e "${YELLOW}Generating examples documentation...${NC}"

cat > "${OUTPUT_DIR}/EXAMPLES.md" << 'EOF'
# Code Examples

This page contains practical examples of using the tsrc library.

## Basic Workspace Operations

### Initialize a Workspace

```rust
use tsrc::application::use_cases::init_workspace::{
    InitWorkspaceUseCase, InitWorkspaceConfig
};
use tsrc::domain::value_objects::{git_url::GitUrl, file_path::FilePath};

async fn init_workspace() -> tsrc::Result<()> {
    let config = InitWorkspaceConfig {
        manifest_url: GitUrl::new("https://github.com/example/manifest.git")?,
        workspace_path: FilePath::new_absolute("/path/to/workspace")?,
        branch: Some("main".to_string()),
        groups: None,
        shallow: false,
        force: false,
    };
    
    let use_case = InitWorkspaceUseCase::new(config);
    let workspace = use_case.execute().await?;
    
    println!("Workspace initialized at: {:?}", workspace.root_path);
    Ok(())
}
```

### Sync Repositories

```rust
use tsrc::application::use_cases::sync_repositories::{
    SyncRepositoriesUseCase, SyncRepositoriesConfig
};

async fn sync_repositories() -> tsrc::Result<()> {
    let config = SyncRepositoriesConfig::new(".")
        .with_groups(vec!["web".to_string()])
        .with_force(false)
        .with_parallel_jobs(4);
    
    let use_case = SyncRepositoriesUseCase::new(config);
    let result = use_case.execute().await?;
    
    println!("Synced {} repositories successfully", result.successful);
    println!("Failed: {}", result.failed);
    
    Ok(())
}
```

### Check Status

```rust
use tsrc::application::use_cases::status_check::{
    StatusCheckUseCase, StatusCheckConfig
};

async fn check_status() -> tsrc::Result<()> {
    let config = StatusCheckConfig::new(".")
        .with_show_branch(true)
        .with_compact(false);
    
    let use_case = StatusCheckUseCase::new(config);
    let result = use_case.execute().await?;
    
    for repo_status in result.repositories {
        println!("{}: {:?}", repo_status.repository.dest, repo_status.status);
    }
    
    Ok(())
}
```

### Execute Commands

```rust
use tsrc::application::use_cases::foreach_command::{
    ForeachCommandUseCase, ForeachCommandConfig
};

async fn run_foreach() -> tsrc::Result<()> {
    let config = ForeachCommandConfig::new(".", "git status")
        .with_groups(vec!["core".to_string()])
        .with_parallel(true)
        .with_max_parallel(Some(4));
    
    let use_case = ForeachCommandUseCase::new(config);
    let result = use_case.execute().await?;
    
    for execution in result.executions {
        println!("Repository: {}", execution.repository.dest);
        println!("Command: {}", execution.command);
        println!("Exit code: {}", execution.exit_code);
        if !execution.stdout.is_empty() {
            println!("Output: {}", execution.stdout);
        }
    }
    
    Ok(())
}
```

## Working with Manifests

### Parse Manifest from File

```rust
use tsrc::application::services::manifest_service::ManifestService;
use std::path::Path;

async fn parse_manifest() -> tsrc::Result<()> {
    let service = ManifestService::new();
    let processed = service.parse_from_file(Path::new("manifest.yml")).await?;
    
    println!("Manifest contains {} repositories", processed.manifest.repos.len());
    
    for repo in &processed.manifest.repos {
        println!("Repository: {} -> {}", repo.dest, repo.url);
        if let Some(groups) = &repo.groups {
            println!("  Groups: {:?}", groups);
        }
    }
    
    if !processed.warnings.is_empty() {
        println!("Warnings:");
        for warning in &processed.warnings {
            println!("  - {}", warning);
        }
    }
    
    Ok(())
}
```

### Parse Manifest from URL

```rust
use tsrc::application::services::manifest_service::ManifestService;

async fn parse_remote_manifest() -> tsrc::Result<()> {
    let service = ManifestService::new();
    let processed = service.parse_from_url("https://example.com/manifest.yml").await?;
    
    println!("Remote manifest loaded with {} repositories", processed.manifest.repos.len());
    
    Ok(())
}
```

## Error Handling

### Basic Error Handling

```rust
use tsrc::{TsrcError, Result};

fn handle_errors() -> Result<()> {
    match some_operation() {
        Ok(result) => {
            println!("Success: {:?}", result);
            Ok(())
        }
        Err(TsrcError::GitOperationFailed { operation, error }) => {
            eprintln!("Git operation '{}' failed: {}", operation, error);
            Err(TsrcError::GitOperationFailed { operation, error })
        }
        Err(TsrcError::ManifestParsingFailed { path, error }) => {
            eprintln!("Failed to parse manifest at '{}': {}", path, error);
            Err(TsrcError::ManifestParsingFailed { path, error })
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            Err(e)
        }
    }
}

fn some_operation() -> Result<String> {
    // Some operation that might fail
    Ok("Success".to_string())
}
```

### Using ? Operator

```rust
use tsrc::Result;

async fn chained_operations() -> Result<()> {
    // These operations will automatically propagate errors
    let workspace = load_workspace(".")?;
    let manifest = parse_manifest("manifest.yml").await?;
    let result = sync_repositories(&workspace, &manifest).await?;
    
    println!("All operations completed successfully");
    Ok(())
}

fn load_workspace(path: &str) -> Result<Workspace> {
    // Implementation
    unimplemented!()
}

async fn parse_manifest(path: &str) -> Result<Manifest> {
    // Implementation
    unimplemented!()
}

async fn sync_repositories(workspace: &Workspace, manifest: &Manifest) -> Result<SyncResult> {
    // Implementation
    unimplemented!()
}
```

## Value Objects

### Working with GitUrl

```rust
use tsrc::domain::value_objects::git_url::GitUrl;

fn git_url_examples() -> tsrc::Result<()> {
    // Create URLs
    let https_url = GitUrl::new("https://github.com/owner/repo.git")?;
    let ssh_url = GitUrl::new("git@github.com:owner/repo.git")?;
    
    // Extract information
    println!("Repository name: {:?}", https_url.repo_name());
    println!("Organization: {:?}", https_url.organization());
    
    // Convert formats
    println!("SSH URL: {}", https_url.to_ssh_url());
    
    // Check if URLs point to same repository
    assert!(https_url.is_same_repo(&ssh_url));
    
    Ok(())
}
```

### Working with BranchName

```rust
use tsrc::domain::value_objects::branch_name::BranchName;

fn branch_name_examples() -> tsrc::Result<()> {
    let branch = BranchName::new("feature/new-functionality")?;
    
    println!("Branch name: {}", branch.as_str());
    println!("Is default branch: {}", branch.is_default_branch());
    println!("Is feature branch: {}", branch.is_feature_branch());
    
    Ok(())
}
```
EOF

echo -e "${GREEN}Documentation generation completed!${NC}"
echo -e "${BLUE}Generated files:${NC}"
echo "  - API documentation: ${OUTPUT_DIR}/tsrc/index.html"
echo "  - Main index: ${OUTPUT_DIR}/index.html"
echo "  - API overview: ${OUTPUT_DIR}/API_OVERVIEW.md"
echo "  - Module index: ${OUTPUT_DIR}/MODULES.md"
echo "  - Code examples: ${OUTPUT_DIR}/EXAMPLES.md"

echo -e "${YELLOW}To view the documentation locally:${NC}"
echo "  python3 -m http.server -d ${OUTPUT_DIR} 8000"
echo "  # Then open http://localhost:8000 in your browser"

echo -e "${GREEN}Documentation generation completed successfully!${NC}"