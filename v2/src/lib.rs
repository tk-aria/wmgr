//! # tsrc
//!
//! A command-line tool for managing groups of git repositories.
//!
//! ## Overview
//!
//! `tsrc` helps you manage multiple git repositories organized in groups. It provides
//! a clean, efficient way to synchronize, check status, and run commands across
//! collections of repositories.
//!
//! ## Architecture
//!
//! The codebase follows clean architecture principles:
//!
//! - **Domain**: Core business logic and entities
//! - **Application**: Use cases and application services
//! - **Infrastructure**: External system integrations (Git, filesystem, etc.)
//! - **Presentation**: CLI interface and user interaction
//!
//! ## Examples
//!
//! ```no_run
//! use tsrc::application::use_cases::init_workspace::InitWorkspaceUseCase;
//! use tsrc::infrastructure::filesystem::manifest_store::ManifestStore;
//!
//! // Initialize a workspace from a manifest
//! let manifest_store = ManifestStore::new();
//! let init_use_case = InitWorkspaceUseCase::new(manifest_store);
//! ```

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
pub mod common;