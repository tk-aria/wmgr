# tsrc

Manage groups of git repositories with ease.

## Overview

`tsrc` is a command-line tool that helps you manage multiple git repositories organized in groups. It provides a clean, efficient way to synchronize, check status, and run commands across collections of repositories.

## Features

- **Repository Management**: Clone, sync, and manage multiple git repositories
- **Group Organization**: Organize repositories into logical groups
- **Parallel Operations**: Execute commands across repositories in parallel
- **Status Checking**: Get an overview of all repository states
- **Manifest-based Configuration**: Define your workspace using YAML manifests

## Installation

### From Releases

Download the latest binary from the [releases page](https://github.com/tk-aria/wmgr/releases):

```bash
# Linux/macOS quick install
curl -L https://github.com/tk-aria/wmgr/releases/latest/download/install.sh | bash

# Or manually download and install
wget https://github.com/tk-aria/wmgr/releases/latest/download/tsrc-linux-x86_64.tar.gz
tar xzf tsrc-linux-x86_64.tar.gz
sudo mv tsrc /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr/v2
cargo build --release
sudo cp target/release/tsrc /usr/local/bin/
```

### Using Cargo

```bash
cargo install --git https://github.com/tk-aria/wmgr.git tsrc
```

### Prerequisites

- Rust 1.85.0 or later (for building from source)
- Git 2.0 or later

## Quick Start

1. **Create a manifest file** (`manifest.yml`):

```yaml
repos:
  - dest: "frontend"
    url: "https://github.com/example/frontend.git"
    branch: "main"
    groups: ["web"]
  
  - dest: "backend"
    url: "https://github.com/example/backend.git"
    branch: "main"
    groups: ["api"]
  
  - dest: "shared"
    url: "https://github.com/example/shared.git"
    branch: "main"
    groups: ["web", "api"]
```

2. **Initialize your workspace**:

```bash
tsrc init manifest.yml
```

3. **Sync all repositories**:

```bash
tsrc sync
```

4. **Check status**:

```bash
tsrc status
```

## Commands

### `tsrc init <manifest_path>`

Initialize a new workspace from a manifest file.

**Options:**
- `--group <GROUP>`: Only initialize repositories from specific group(s)
- `--force`: Force initialization even if workspace already exists

**Example:**
```bash
tsrc init manifest.yml --group web
```

### `tsrc sync`

Synchronize all repositories in the workspace.

**Options:**
- `--group <GROUP>`: Only sync repositories from specific group(s)
- `--force`: Force sync even if there are uncommitted changes
- `--no-correct-branch`: Skip branch synchronization

**Example:**
```bash
tsrc sync --group api --force
```

### `tsrc status`

Show the status of all repositories in the workspace.

**Options:**
- `--branch`: Show branch information
- `--compact`: Use compact output format

**Example:**
```bash
tsrc status --branch
```

### `tsrc foreach <command>`

Execute a command in each repository.

**Options:**
- `--group <GROUP>`: Only execute in repositories from specific group(s)
- `--parallel`: Execute commands in parallel

**Example:**
```bash
tsrc foreach "git pull" --parallel
tsrc foreach "npm install" --group web
```

### `tsrc log`

Show commit logs across repositories.

### `tsrc dump-manifest`

Output the current workspace manifest.

### `tsrc apply-manifest <manifest_path>`

Apply a new manifest to the current workspace.

## Configuration

The workspace configuration is stored in `.tsrc/config.yml` in your workspace root.

### Example Configuration

```yaml
manifest_path: "manifest.yml"
groups: ["web", "api"]
workspace_path: "/path/to/workspace"
```

## Manifest Format

The manifest file defines your repositories and their organization:

```yaml
repos:
  - dest: "local-directory-name"
    url: "https://github.com/owner/repo.git"
    branch: "main"                    # optional, defaults to "main"
    groups: ["group1", "group2"]      # optional
    tag: "v1.0.0"                     # optional, use specific tag
    sha1: "abc123..."                 # optional, use specific commit
    remotes:                          # optional, additional remotes
      - name: "upstream"
        url: "https://github.com/upstream/repo.git"
```

### Repository Options

- `dest`: Local directory name (required)
- `url`: Git repository URL (required)
- `branch`: Branch to checkout (optional, defaults to "main")
- `groups`: List of groups this repository belongs to (optional)
- `tag`: Specific tag to checkout (optional)
- `sha1`: Specific commit to checkout (optional)
- `remotes`: Additional remotes to configure (optional)

## Working with Groups

Groups allow you to organize repositories logically and operate on subsets:

```bash
# Initialize only web repositories
tsrc init manifest.yml --group web

# Sync only API repositories
tsrc sync --group api

# Run tests only in core repositories
tsrc foreach "npm test" --group core
```

## Advanced Usage

### Parallel Operations

Most commands support parallel execution for better performance:

```bash
# Sync all repositories in parallel
tsrc sync --parallel

# Run commands in parallel across repositories
tsrc foreach "git pull" --parallel
```

### Environment Variables

The `foreach` command sets helpful environment variables:

- `TSRC_REPO_NAME`: Name of the current repository
- `TSRC_REPO_PATH`: Full path to the repository
- `TSRC_WORKSPACE_PATH`: Path to the workspace root

### Example Workflow

```bash
# Clone and set up workspace
tsrc init manifest.yml

# Check what needs to be done
tsrc status

# Sync everything
tsrc sync

# Run tests across all repositories
tsrc foreach "make test" --parallel

# Update only web components
tsrc sync --group web

# Check status with branch info
tsrc status --branch
```

## Troubleshooting

### Common Issues

1. **Repository clone failures**: Check network connectivity and repository URLs
2. **Permission issues**: Ensure you have proper git credentials configured
3. **Merge conflicts**: Use `--force` flag carefully, or resolve conflicts manually

### Getting Help

```bash
tsrc --help
tsrc <command> --help
```

## Development

### Building from Source

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr/v2

# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run tests with all features
cargo test --all-features

# Check code with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

### Development Scripts

Use the provided development scripts for common tasks:

```bash
# Full development validation pipeline
./scripts/dev-build.sh all

# Run specific checks
./scripts/dev-build.sh clippy
./scripts/dev-build.sh test
./scripts/dev-build.sh release

# Cross-compilation for releases
./scripts/build-releases.sh
```

### Using Make

```bash
# Show available targets
make help

# Development workflow
make dev          # fmt + clippy + test
make all          # Full validation pipeline
make cross-build  # Build for all platforms
```

### Architecture

The project follows a clean architecture pattern:

- **Domain Layer**: Core business logic and entities (`src/domain/`)
- **Application Layer**: Use cases and business workflows (`src/application/`)
- **Infrastructure Layer**: External dependencies and I/O (`src/infrastructure/`)
- **Presentation Layer**: CLI interface and user interaction (`src/presentation/`)

### Testing

The project includes comprehensive testing:

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **Test Helpers**: Reusable testing utilities

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test '*'

# Run with coverage
cargo tarpaulin --out html --output-dir target/coverage
```

## Performance

### Benchmarks

Run benchmarks to measure performance:

```bash
cargo bench
```

### Optimization Tips

- Use `--parallel` flag for operations on multiple repositories
- Configure appropriate `parallel_jobs` in your configuration
- Use groups to operate on subsets of repositories
- Enable logging only when debugging: `RUST_LOG=debug tsrc sync`

## Configuration Reference

### Workspace Configuration (`.tsrc/config.yml`)

```yaml
# Path to the manifest file (relative to workspace root)
manifest_path: "manifest.yml"

# Default groups to use when none specified
groups: ["core", "web"]

# Workspace root path
workspace_path: "/path/to/workspace"

# Git configuration
git:
  # Default branch name for new repositories
  default_branch: "main"
  
  # Whether to use shallow clones by default
  shallow_clones: false
  
  # Default remote name
  default_remote: "origin"

# Parallel execution settings
parallel:
  # Maximum number of parallel jobs (default: number of CPU cores)
  max_jobs: 4
  
  # Timeout for individual operations (in seconds)
  timeout: 300
```

### Manifest Schema

Complete manifest file example:

```yaml
# Global settings
defaults:
  branch: "main"
  shallow: false

# Repository definitions
repos:
  - dest: "backend/api"
    url: "https://github.com/example/api-server.git"
    branch: "develop"
    groups: ["backend", "core"]
    remotes:
      - name: "upstream"
        url: "https://github.com/upstream/api-server.git"
  
  - dest: "frontend/web"
    url: "git@github.com:example/web-app.git"
    tag: "v2.1.0"
    groups: ["frontend", "web"]
    shallow: true
  
  - dest: "tools/scripts"
    url: "https://github.com/example/build-tools.git"
    sha1: "abc123def456"
    groups: ["tools"]

# Group definitions (optional)
groups:
  core:
    repos: ["backend/api", "shared/common"]
    description: "Core infrastructure components"
  
  frontend:
    repos: ["frontend/web", "frontend/mobile"]
    description: "User-facing applications"
  
  tools:
    repos: ["tools/scripts", "tools/ci"]
    description: "Development and CI tools"
```

## Environment Variables

### Runtime Configuration

- `TSRC_LOG`: Set log level (`error`, `warn`, `info`, `debug`, `trace`)
- `TSRC_CONFIG_PATH`: Override default config file location
- `TSRC_PARALLEL_JOBS`: Override parallel job count
- `TSRC_TIMEOUT`: Set operation timeout in seconds

### Command Environment

During `foreach` execution, these variables are available:

- `TSRC_REPO_NAME`: Current repository name
- `TSRC_REPO_PATH`: Full path to current repository
- `TSRC_WORKSPACE_PATH`: Workspace root path
- `TSRC_REPO_URL`: Repository URL
- `TSRC_REPO_BRANCH`: Current branch name
- `TSRC_REPO_GROUPS`: Comma-separated list of groups

### Examples

```bash
# Enable debug logging
TSRC_LOG=debug tsrc sync

# Limit parallel jobs
TSRC_PARALLEL_JOBS=2 tsrc sync --parallel

# Use custom config
TSRC_CONFIG_PATH=/custom/config.yml tsrc status
```

## CI/CD Integration

### GitHub Actions

Example workflow for using tsrc in CI:

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
      run: tsrc init manifest.yml
    
    - name: Run tests
      run: tsrc foreach "make test" --parallel
    
    - name: Check status
      run: tsrc status
```

### Docker

Example Dockerfile:

```dockerfile
FROM rust:1.85 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tsrc /usr/local/bin/
ENTRYPOINT ["tsrc"]
```

## Troubleshooting

### Common Issues

1. **Repository clone failures**
   - Check network connectivity and repository URLs
   - Verify SSH keys are configured for private repositories
   - Ensure repository URLs are accessible

2. **Permission issues**
   - Configure git credentials: `git config --global credential.helper store`
   - Set up SSH keys for GitHub/GitLab: `ssh-keygen -t ed25519 -C "your_email@example.com"`
   - Check repository permissions

3. **Merge conflicts during sync**
   - Use `--force` flag to discard local changes (destructive)
   - Resolve conflicts manually before syncing
   - Use `tsrc status` to identify repositories with conflicts

4. **Performance issues**
   - Use `--parallel` flag for faster operations
   - Reduce `parallel_jobs` if system becomes unresponsive
   - Consider using shallow clones for large repositories

5. **Manifest parsing errors**
   - Validate YAML syntax: `yamllint manifest.yml`
   - Check required fields are present
   - Verify repository URLs are valid

### Debug Mode

Enable verbose logging to troubleshoot issues:

```bash
RUST_LOG=debug tsrc sync
RUST_LOG=trace tsrc init manifest.yml  # Very verbose
```

### Getting Help

```bash
tsrc --help                    # General help
tsrc <command> --help         # Command-specific help
tsrc --version                # Show version information
```

## Contributing

We welcome contributions! Here's how to get started:

### Development Setup

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/wmgr.git`
3. Create a feature branch: `git checkout -b feature/amazing-feature`
4. Install development dependencies: `cargo build`

### Development Workflow

1. Make your changes
2. Run the full validation pipeline: `./scripts/dev-build.sh all`
3. Write tests for new functionality
4. Update documentation if needed
5. Commit your changes with a clear message
6. Push to your fork and submit a pull request

### Code Standards

- Follow Rust best practices and idioms
- Write comprehensive tests for new features
- Include documentation for public APIs
- Run `cargo fmt` and `cargo clippy` before committing
- Ensure all CI checks pass

### Testing Guidelines

- Unit tests should test individual components in isolation
- Integration tests should test real-world scenarios
- Use mock services for external dependencies
- Aim for high test coverage

### Documentation

- Update README.md for user-facing changes
- Add rustdoc comments for public APIs
- Include examples in documentation
- Update changelog for releases

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a complete list of changes in each version.

## License

This project is licensed under the BSD-3-Clause License. See [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by [tsrc](https://github.com/TankerHQ/tsrc) (Python implementation)
- Built with the amazing Rust ecosystem
- Thanks to all contributors and users