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

### From Source

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr/v2
cargo build --release
```

The binary will be available at `target/release/tsrc`.

### Prerequisites

- Rust 1.85.0 or later
- Git

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

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

This project is licensed under the BSD-3-Clause License.