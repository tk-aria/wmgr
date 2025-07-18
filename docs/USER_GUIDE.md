# tsrc User Guide

This comprehensive guide covers everything you need to know about using tsrc effectively for managing multiple Git repositories.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Concepts](#core-concepts)
3. [Configuration](#configuration)
4. [Basic Operations](#basic-operations)
5. [Advanced Usage](#advanced-usage)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)
8. [Examples](#examples)

## Getting Started

### Installation

#### Option 1: Download Pre-built Binary

The easiest way to install tsrc is to download a pre-built binary:

```bash
# Linux/macOS quick install
curl -L https://github.com/tk-aria/wmgr/releases/latest/download/install.sh | bash

# Manual installation
# Download the appropriate binary for your platform from:
# https://github.com/tk-aria/wmgr/releases/latest

# For Linux x86_64:
wget https://github.com/tk-aria/wmgr/releases/latest/download/tsrc-linux-x86_64.tar.gz
tar xzf tsrc-linux-x86_64.tar.gz
sudo mv tsrc /usr/local/bin/

# For macOS:
wget https://github.com/tk-aria/wmgr/releases/latest/download/tsrc-macos-x86_64.tar.gz
tar xzf tsrc-macos-x86_64.tar.gz
sudo mv tsrc /usr/local/bin/
```

#### Option 2: Install from Source

If you have Rust installed:

```bash
# Install via cargo
cargo install --git https://github.com/tk-aria/wmgr.git tsrc

# Or clone and build manually
git clone https://github.com/tk-aria/wmgr.git
cd wmgr/v2
cargo build --release
sudo cp target/release/tsrc /usr/local/bin/
```

#### Option 3: Using Package Managers

```bash
# Homebrew (macOS/Linux)
brew tap tk-aria/tsrc
brew install tsrc

# Coming soon: apt, yum, pacman packages
```

### Verification

Verify your installation:

```bash
tsrc --version
tsrc --help
```

## Core Concepts

### Workspace

A **workspace** is a directory that contains multiple Git repositories organized according to a manifest file. It includes:

- Multiple Git repositories in subdirectories
- A `.tsrc/` directory containing configuration and metadata
- A manifest file (usually `manifest.yml`) defining repository structure

### Manifest

A **manifest** is a YAML file that defines:

- Which repositories to include
- Where to clone them locally
- Which branches/tags/commits to use
- How to organize repositories into groups
- Additional remotes to configure

### Groups

**Groups** allow you to organize repositories logically and operate on subsets:

- Logical organization (e.g., "frontend", "backend", "tools")
- Selective operations (sync only certain groups)
- Simplified workflows for different teams

### Repository Configuration

Each repository in a manifest can specify:

- **Destination**: Local directory name
- **URL**: Git repository URL (HTTPS or SSH)
- **Branch**: Default branch to checkout
- **Groups**: Which groups this repository belongs to
- **Remotes**: Additional Git remotes to configure
- **Tags/SHA**: Specific versions to checkout

## Configuration

### Manifest File Format

Here's a comprehensive manifest example:

```yaml
# Global defaults (optional)
defaults:
  branch: "main"
  shallow: false

# Repository definitions
repos:
  # Basic repository
  - dest: "frontend/web"
    url: "https://github.com/example/web-frontend.git"
    groups: ["frontend", "web"]

  # Repository with specific branch
  - dest: "backend/api"
    url: "git@github.com:example/api-server.git"
    branch: "develop"
    groups: ["backend", "core"]

  # Repository with additional remotes
  - dest: "shared/common"
    url: "https://github.com/example/shared-lib.git"
    groups: ["shared", "core"]
    remotes:
      - name: "upstream"
        url: "https://github.com/upstream/shared-lib.git"
      - name: "fork"
        url: "https://github.com/myname/shared-lib.git"

  # Repository pinned to specific tag
  - dest: "tools/build"
    url: "https://github.com/example/build-tools.git"
    tag: "v2.1.0"
    groups: ["tools"]

  # Repository pinned to specific commit
  - dest: "vendor/legacy"
    url: "https://github.com/example/legacy-code.git"
    sha1: "abc123def456789"
    groups: ["vendor"]

  # Shallow clone for large repositories
  - dest: "docs/wiki"
    url: "https://github.com/example/documentation.git"
    shallow: true
    groups: ["docs"]

# Group definitions (optional but recommended)
groups:
  frontend:
    repos: ["frontend/web", "frontend/mobile"]
    description: "User-facing applications"

  backend:
    repos: ["backend/api", "backend/worker"]
    description: "Server-side services"

  core:
    repos: ["backend/api", "shared/common"]
    description: "Core infrastructure components"

  tools:
    repos: ["tools/build", "tools/ci"]
    description: "Development and build tools"

  docs:
    repos: ["docs/wiki", "docs/api"]
    description: "Documentation repositories"
```

### Workspace Configuration

The workspace configuration is stored in `.tsrc/config.yml`:

```yaml
# Path to manifest file (relative to workspace root)
manifest_path: "manifest.yml"

# Default groups (used when no groups specified)
default_groups: ["core"]

# Workspace metadata
workspace_path: "/path/to/workspace"
manifest_url: "https://github.com/example/manifest.git"

# Git settings
git:
  default_branch: "main"
  shallow_clones: false
  default_remote: "origin"

# Parallel execution settings
parallel:
  max_jobs: 4
  timeout: 300

# Command aliases (optional)
aliases:
  test: "npm test"
  lint: "npm run lint"
  build: "npm run build"
```

## Basic Operations

### Initializing a Workspace

Start by creating a new workspace from a manifest:

```bash
# Initialize from local manifest file
tsrc init manifest.yml

# Initialize from remote manifest
tsrc init https://github.com/example/manifest.git

# Initialize specific groups only
tsrc init manifest.yml --group frontend --group tools

# Force initialization (overwrite existing workspace)
tsrc init manifest.yml --force
```

### Syncing Repositories

Keep your repositories up to date:

```bash
# Sync all repositories
tsrc sync

# Sync specific groups
tsrc sync --group frontend --group backend

# Force sync (discard local changes)
tsrc sync --force

# Sync in parallel for faster operation
tsrc sync --parallel

# Sync without correcting branches
tsrc sync --no-correct-branch
```

### Checking Status

Get an overview of repository states:

```bash
# Basic status
tsrc status

# Include branch information
tsrc status --branch

# Compact output
tsrc status --compact

# Check specific groups
tsrc status --group core
```

### Executing Commands

Run commands across multiple repositories:

```bash
# Run git status in all repositories
tsrc foreach "git status"

# Run commands in parallel
tsrc foreach "git pull" --parallel

# Run in specific groups
tsrc foreach "npm test" --group frontend

# Run with environment variables
tsrc foreach "echo \$TSRC_REPO_NAME: \$TSRC_REPO_PATH"
```

### Managing Manifests

Work with manifest files:

```bash
# Show current manifest
tsrc dump-manifest

# Export in different formats
tsrc dump-manifest --format json --pretty

# Save to file
tsrc dump-manifest --output current-manifest.yml

# Apply new manifest to existing workspace
tsrc apply-manifest new-manifest.yml

# Preview changes without applying
tsrc apply-manifest new-manifest.yml --dry-run
```

## Advanced Usage

### Working with Groups

Groups are a powerful way to organize and operate on repository subsets:

```bash
# List available groups
tsrc status --group all

# Initialize workspace with multiple groups
tsrc init manifest.yml --group frontend --group backend --group tools

# Sync only development repositories
tsrc sync --group core --group tools

# Run tests on frontend components
tsrc foreach "npm test" --group frontend

# Check status of production services
tsrc status --group backend --group database
```

### Parallel Operations

Improve performance with parallel execution:

```bash
# Sync repositories in parallel (default: number of CPU cores)
tsrc sync --parallel

# Limit parallel jobs
tsrc sync --parallel --jobs 2

# Parallel command execution
tsrc foreach "git pull" --parallel --max-parallel 4

# Set via environment variable
TSRC_PARALLEL_JOBS=8 tsrc sync --parallel
```

### Environment Variables

#### Runtime Configuration

Control tsrc behavior with environment variables:

```bash
# Enable debug logging
TSRC_LOG=debug tsrc sync

# Set custom config path
TSRC_CONFIG_PATH=/custom/path/config.yml tsrc status

# Override parallel job count
TSRC_PARALLEL_JOBS=2 tsrc sync

# Set operation timeout
TSRC_TIMEOUT=600 tsrc foreach "long-running-command"
```

#### Command Environment

During `foreach` execution, these variables are available in your commands:

- `TSRC_REPO_NAME`: Current repository name
- `TSRC_REPO_PATH`: Full path to current repository
- `TSRC_WORKSPACE_PATH`: Workspace root path
- `TSRC_REPO_URL`: Repository URL
- `TSRC_REPO_BRANCH`: Current branch
- `TSRC_REPO_GROUPS`: Comma-separated list of groups

Example usage:

```bash
# Create custom scripts using environment variables
tsrc foreach 'echo "Processing $TSRC_REPO_NAME in $TSRC_REPO_PATH"'

# Conditional operations based on groups
tsrc foreach 'if [[ $TSRC_REPO_GROUPS == *"frontend"* ]]; then npm test; fi'

# Use in build scripts
tsrc foreach 'make build REPO_NAME=$TSRC_REPO_NAME'
```

### Advanced Manifest Features

#### Include External Manifests

```yaml
# Include repositories from other manifests
includes:
  - url: "https://github.com/team/shared-manifest.git"
    file: "core-services.yml"
  - url: "https://github.com/team/tools-manifest.git"
    file: "dev-tools.yml"
    groups: ["tools"]

repos:
  # Local repositories
  - dest: "app/main"
    url: "https://github.com/example/main-app.git"
```

#### Conditional Repository Inclusion

```yaml
repos:
  # Include only for specific environments
  - dest: "tools/dev"
    url: "https://github.com/example/dev-tools.git"
    condition: "development"
    groups: ["tools"]

  # Platform-specific repositories
  - dest: "native/ios"
    url: "https://github.com/example/ios-app.git"
    condition: "ios"
    groups: ["mobile"]

  - dest: "native/android"
    url: "https://github.com/example/android-app.git"
    condition: "android"
    groups: ["mobile"]
```

### Git Workflow Integration

#### Branch Management

```bash
# Check out feature branches across repositories
tsrc foreach "git checkout -b feature/new-feature"

# Sync specific branch
tsrc foreach "git checkout develop && git pull"

# Create pull requests (using gh CLI)
tsrc foreach "gh pr create --title 'Feature: \$TSRC_REPO_NAME updates'"
```

#### Release Management

```bash
# Tag release across repositories
tsrc foreach "git tag v1.2.0"

# Create release branches
tsrc foreach "git checkout -b release/1.2.0"

# Push tags
tsrc foreach "git push origin --tags"
```

### CI/CD Integration

#### GitHub Actions

```yaml
name: Multi-repo CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install tsrc
      run: |
        curl -L https://github.com/tk-aria/wmgr/releases/latest/download/tsrc-linux-x86_64.tar.gz | tar xz
        sudo mv tsrc /usr/local/bin/
    
    - name: Initialize workspace
      run: tsrc init manifest.yml --group ci

    - name: Run tests
      run: tsrc foreach "make test" --group testable --parallel

    - name: Build artifacts
      run: tsrc foreach "make build" --group buildable
```

#### GitLab CI

```yaml
stages:
  - setup
  - test
  - build

setup:
  stage: setup
  script:
    - curl -L https://github.com/tk-aria/wmgr/releases/latest/download/tsrc-linux-x86_64.tar.gz | tar xz
    - mv tsrc /usr/local/bin/
    - tsrc init $CI_PROJECT_URL/manifest.git

test:
  stage: test
  script:
    - tsrc foreach "npm test" --group frontend --parallel
    - tsrc foreach "cargo test" --group backend --parallel

build:
  stage: build
  script:
    - tsrc foreach "npm run build" --group frontend
    - tsrc foreach "cargo build --release" --group backend
```

## Best Practices

### Repository Organization

1. **Logical Grouping**: Organize repositories by function, team, or deployment unit
2. **Consistent Naming**: Use clear, consistent naming conventions for destinations
3. **Group Strategy**: Create groups that match your team structure and workflows
4. **Shallow Clones**: Use shallow clones for large repositories you don't modify

### Manifest Management

1. **Version Control**: Keep your manifest file in version control
2. **Documentation**: Document group purposes and repository relationships
3. **Validation**: Regularly validate your manifest structure
4. **Modular Design**: Split large manifests using includes

### Workflow Recommendations

1. **Regular Syncing**: Sync frequently to stay up to date
2. **Parallel Operations**: Use parallel flags for better performance
3. **Group-based Operations**: Operate on relevant groups rather than all repositories
4. **Environment Variables**: Use tsrc environment variables in scripts

### Performance Optimization

1. **Parallel Execution**: Always use `--parallel` for multi-repository operations
2. **Selective Groups**: Work with specific groups instead of all repositories
3. **Shallow Clones**: Use shallow clones for repositories you don't modify
4. **Resource Limits**: Adjust `--max-parallel` based on your system capabilities

### Security Considerations

1. **SSH Keys**: Use SSH keys for private repositories
2. **Credential Management**: Configure Git credential helpers appropriately
3. **Access Control**: Ensure proper repository access permissions
4. **Manifest Security**: Protect your manifest repository with appropriate access controls

## Troubleshooting

### Common Issues

#### Authentication Problems

```bash
# Problem: Git authentication failures
# Solution: Configure Git credentials
git config --global credential.helper store

# For SSH keys
ssh-keygen -t ed25519 -C "your_email@example.com"
ssh-add ~/.ssh/id_ed25519

# Test SSH connection
ssh -T git@github.com
```

#### Repository Clone Failures

```bash
# Problem: Repository doesn't exist or access denied
# Solutions:
# 1. Check repository URL and access permissions
# 2. Use correct protocol (HTTPS vs SSH)
# 3. Verify network connectivity

# Debug with verbose logging
TSRC_LOG=debug tsrc sync
```

#### Merge Conflicts During Sync

```bash
# Problem: Local changes conflict with remote updates
# Solutions:
# 1. Resolve conflicts manually
git status  # Check conflicted files
git add .   # Stage resolved files
git commit  # Commit resolution

# 2. Force sync (discards local changes - be careful!)
tsrc sync --force

# 3. Stash changes before syncing
tsrc foreach "git stash"
tsrc sync
tsrc foreach "git stash pop"
```

#### Performance Issues

```bash
# Problem: Operations are slow
# Solutions:
# 1. Use parallel execution
tsrc sync --parallel

# 2. Reduce parallel job count if system becomes unresponsive
tsrc sync --parallel --jobs 2

# 3. Use shallow clones for large repositories
# Add to manifest:
# shallow: true

# 4. Work with specific groups instead of all repositories
tsrc sync --group core
```

#### Manifest Parsing Errors

```bash
# Problem: YAML syntax errors in manifest
# Solutions:
# 1. Validate YAML syntax
yamllint manifest.yml

# 2. Check required fields are present
# Required: dest, url
# Optional: branch, groups, remotes, etc.

# 3. Verify repository URLs are accessible
curl -I https://github.com/example/repo.git
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Debug level logging
TSRC_LOG=debug tsrc sync

# Trace level (very verbose)
TSRC_LOG=trace tsrc init manifest.yml

# Log to file
TSRC_LOG=debug tsrc sync 2>&1 | tee tsrc.log
```

### Getting Help

```bash
# General help
tsrc --help

# Command-specific help
tsrc sync --help
tsrc foreach --help

# Version information
tsrc --version
```

## Examples

### Example 1: Web Development Team

**Scenario**: A web development team with frontend, backend, and shared components.

**Manifest** (`manifest.yml`):

```yaml
repos:
  - dest: "frontend/web"
    url: "git@github.com:company/web-frontend.git"
    groups: ["frontend", "web"]

  - dest: "frontend/mobile"
    url: "git@github.com:company/mobile-app.git"
    groups: ["frontend", "mobile"]

  - dest: "backend/api"
    url: "git@github.com:company/api-server.git"
    groups: ["backend", "core"]

  - dest: "backend/worker"
    url: "git@github.com:company/background-worker.git"
    groups: ["backend"]

  - dest: "shared/ui-components"
    url: "git@github.com:company/ui-library.git"
    groups: ["shared", "frontend"]

  - dest: "shared/utils"
    url: "git@github.com:company/shared-utils.git"
    groups: ["shared", "core"]

groups:
  frontend:
    repos: ["frontend/web", "frontend/mobile", "shared/ui-components"]
    description: "Frontend applications and components"

  backend:
    repos: ["backend/api", "backend/worker"]
    description: "Backend services"

  core:
    repos: ["backend/api", "shared/utils"]
    description: "Core infrastructure"
```

**Workflow**:

```bash
# Setup workspace
tsrc init manifest.yml

# Frontend developer workflow
tsrc sync --group frontend
tsrc foreach "npm install" --group frontend
tsrc foreach "npm test" --group frontend

# Backend developer workflow
tsrc sync --group backend
tsrc foreach "cargo test" --group backend

# Full team integration
tsrc sync
tsrc foreach "make test" --parallel
```

### Example 2: Microservices Architecture

**Scenario**: A company with multiple microservices, infrastructure, and tooling.

**Manifest** (`manifest.yml`):

```yaml
repos:
  # Core services
  - dest: "services/user-service"
    url: "https://github.com/company/user-service.git"
    groups: ["services", "core"]

  - dest: "services/auth-service"
    url: "https://github.com/company/auth-service.git"
    groups: ["services", "core"]

  - dest: "services/payment-service"
    url: "https://github.com/company/payment-service.git"
    groups: ["services", "payment"]

  - dest: "services/notification-service"
    url: "https://github.com/company/notification-service.git"
    groups: ["services", "communication"]

  # Infrastructure
  - dest: "infrastructure/kubernetes"
    url: "https://github.com/company/k8s-configs.git"
    groups: ["infrastructure", "deployment"]

  - dest: "infrastructure/terraform"
    url: "https://github.com/company/terraform-modules.git"
    groups: ["infrastructure", "provisioning"]

  # Tools and utilities
  - dest: "tools/cli"
    url: "https://github.com/company/cli-tools.git"
    groups: ["tools"]

  - dest: "tools/monitoring"
    url: "https://github.com/company/monitoring-setup.git"
    groups: ["tools", "observability"]

groups:
  services:
    repos: ["services/*"]
    description: "All microservices"

  core:
    repos: ["services/user-service", "services/auth-service"]
    description: "Core business services"

  infrastructure:
    repos: ["infrastructure/*"]
    description: "Infrastructure as code"

  tools:
    repos: ["tools/*"]
    description: "Development and operational tools"
```

**Workflow**:

```bash
# DevOps workflow
tsrc init manifest.yml --group infrastructure --group tools
tsrc foreach "terraform plan" --group provisioning
tsrc foreach "kubectl apply -f ." --group deployment

# Service development
tsrc sync --group services
tsrc foreach "docker build -t \$TSRC_REPO_NAME ." --group services
tsrc foreach "docker run \$TSRC_REPO_NAME npm test" --group services

# Production deployment
tsrc sync --group core
tsrc foreach "make deploy-prod" --group core
```

### Example 3: Open Source Project

**Scenario**: An open source project with multiple repositories and contributors.

**Manifest** (`manifest.yml`):

```yaml
repos:
  # Main application
  - dest: "core/main"
    url: "https://github.com/project/main.git"
    remotes:
      - name: "upstream"
        url: "https://github.com/project/main.git"
    groups: ["core"]

  # Plugins and extensions
  - dest: "plugins/auth"
    url: "https://github.com/project/auth-plugin.git"
    groups: ["plugins"]

  - dest: "plugins/storage"
    url: "https://github.com/project/storage-plugin.git"
    groups: ["plugins"]

  # Documentation
  - dest: "docs/user-guide"
    url: "https://github.com/project/user-docs.git"
    groups: ["docs"]

  - dest: "docs/api"
    url: "https://github.com/project/api-docs.git"
    groups: ["docs"]

  # Examples and tutorials
  - dest: "examples/basic"
    url: "https://github.com/project/basic-examples.git"
    groups: ["examples"]

  - dest: "examples/advanced"
    url: "https://github.com/project/advanced-examples.git"
    groups: ["examples"]

groups:
  development:
    repos: ["core/main", "plugins/*"]
    description: "Core development repositories"

  documentation:
    repos: ["docs/*", "examples/*"]
    description: "Documentation and examples"
```

**Workflow**:

```bash
# Contributor setup
tsrc init manifest.yml --group development
tsrc foreach "git remote add fork git@github.com:username/\$TSRC_REPO_NAME.git"

# Documentation contributor
tsrc init manifest.yml --group documentation
tsrc foreach "git checkout -b update-docs"

# Maintainer workflow
tsrc sync
tsrc foreach "git pull upstream main" --group development
tsrc foreach "make test" --group development --parallel
```

This user guide provides comprehensive coverage of tsrc functionality, from basic usage to advanced workflows. Users can refer to specific sections based on their needs and experience level.