//! # wmgr - Git Repository Manager
//!
//! `wmgr` is a command-line tool for managing multiple git repositories organized in workspaces.
//! It provides a clean, efficient way to synchronize, check status, and run commands across
//! collections of repositories.
//!
//! ## Features
//!
//! - **Repository Management**: Clone, sync, and manage multiple git repositories
//! - **Group Organization**: Organize repositories into logical groups
//! - **Parallel Operations**: Execute commands across repositories in parallel
//! - **Status Checking**: Get an overview of all repository states
//! - **Manifest-based Configuration**: Define your workspace using YAML manifests
//!
//! ## Quick Start
//!
//! 1. Create a manifest file (`manifest.yml`):
//!
//! ```yaml
//! repos:
//!   - dest: "frontend"
//!     url: "https://github.com/example/frontend.git"
//!     groups: ["web"]
//!   - dest: "backend"
//!     url: "https://github.com/example/backend.git"
//!     groups: ["api"]
//! ```
//!
//! 2. Initialize your workspace:
//!
//! ```bash
//! wmgr init manifest.yml
//! ```
//!
//! 3. Sync all repositories:
//!
//! ```bash
//! wmgr sync
//! ```
//!
//! ## Architecture
//!
//! The crate is organized using clean architecture principles:
//!
//! - [`domain`]: Core business logic and entities
//! - [`application`]: Use cases and business workflows  
//! - [`infrastructure`]: External dependencies and I/O operations
//! - [`presentation`]: CLI interface and user interaction
//! - [`common`]: Shared utilities and error handling
//!
//! ## Domain Model
//!
//! The core domain entities include:
//!
//! - [`domain::entities::workspace::Workspace`]: Represents a workspace containing multiple repositories
//! - [`domain::entities::manifest::Manifest`]: Configuration defining repositories and their organization
//! - [`domain::entities::repository::Repository`]: Individual git repository with its configuration
//! - [`domain::value_objects::git_url::GitUrl`]: Type-safe Git URL representation
//! - [`domain::value_objects::branch_name::BranchName`]: Type-safe branch name representation
//!
//! ## Use Cases
//!
//! The main application use cases are:
//!
//! - [`application::use_cases::init_workspace`]: Initialize a new workspace from a manifest
//! - [`application::use_cases::sync_repositories`]: Synchronize repositories in a workspace
//! - [`application::use_cases::status_check`]: Check the status of repositories
//! - [`application::use_cases::foreach_command`]: Execute commands across repositories
//!
//! ## Infrastructure
//!
//! Infrastructure components provide concrete implementations:
//!
//! - [`infrastructure::git`]: Git operations using libgit2
//! - [`infrastructure::filesystem`]: File system operations and configuration storage
//! - [`infrastructure::process`]: Process execution for external commands
//!
//! ## Error Handling
//!
//! The crate uses a comprehensive error handling system:
//!
//! - [`common::error::TsrcError`]: Main error type with detailed context
//! - [`common::result::TsrcResult`]: Type alias for `Result<T, TsrcError>`
//!
//! ## Examples
//!
//! ### Using the Library
//!
//! ```rust,no_run
//! use wmgr::application::use_cases::sync_repositories::{
//!     SyncRepositoriesUseCase, SyncRepositoriesConfig
//! };
//! use wmgr::domain::entities::workspace::Workspace;
//!
//! # async fn example() -> wmgr::Result<()> {
//! // Load workspace
//! let workspace = Workspace::load_from_path(".")?;
//!
//! // Create sync configuration
//! let config = SyncRepositoriesConfig::new(".")
//!     .with_groups(vec!["web".to_string()])
//!     .with_force(false);
//!
//! // Execute sync
//! let use_case = SyncRepositoriesUseCase::new(config);
//! let result = use_case.execute().await?;
//!
//! println!("Synced {} repositories", result.successful);
//! # Ok(())
//! # }
//! ```
//!
//! ### Working with Manifests
//!
//! ```rust,no_run
//! use wmgr::application::services::manifest_service::ManifestService;
//! use std::path::Path;
//!
//! # async fn example() -> wmgr::Result<()> {
//! let manifest_service = ManifestService::new();
//! let processed = manifest_service.parse_from_file(Path::new("manifest.yml")).await?;
//!
//! println!("Found {} repositories", processed.manifest.repos.len());
//!
//! for repo in &processed.manifest.repos {
//!     println!("Repository: {} -> {}", repo.dest, repo.url);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! Currently all features are enabled by default. Future versions may include optional features
//! for different backends or additional functionality.

// Documentation attributes
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod application;
pub mod common;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

// Re-export commonly used types for convenience
pub use crate::common::error::TsrcError;
pub use crate::common::result::TsrcResult as Result;
