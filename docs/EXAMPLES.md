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
