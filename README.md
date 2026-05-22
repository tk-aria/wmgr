# wmgr

**Workspace Manager** — A fast CLI tool for managing multi-repository workspaces.

[![CI](https://github.com/tk-aria/wmgr/actions/workflows/ci.yml/badge.svg)](https://github.com/tk-aria/wmgr/actions/workflows/ci.yml)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](LICENSE)

> [日本語版 README はこちら](README_ja.md)

---

## Overview

`wmgr` lets you define, sync, and operate on collections of repositories using a single YAML manifest. It supports **Git, SVN, Perforce, Mercurial, HTTP archives, S3, Google Drive**, and **symlinks** — all from one unified workflow.

```
wmgr.yml          wmgr sync           your workspace
+-----------+      ---------->      +------------------+
| repos:    |                       | frontend/        |
|  - frontend|                      | backend/         |
|  - backend |                      | shared-assets/   |
|  - assets  |                      | design-files/    |
+-----------+                       +------------------+
```

## Features

- **Multi-SCM support** — Git, SVN, Perforce, Mercurial, HTTP, S3, Google Drive, Symlink
- **Group filtering** — Organize repos into groups; sync or operate on subsets
- **Parallel execution** — Run commands across repos concurrently
- **Credential management** — 4-level credential resolution (env, manifest, profile, system)
- **Recursive workspaces** — Nested workspace support with child manifest discovery
- **Multiple output formats** — Text, JSON, YAML for CI/scripting integration

## Installation

### Quick Install (Recommended)

```bash
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | sh
```

This auto-detects your OS and architecture, downloads the correct binary, and installs to `/usr/local/bin`.

### Options

```bash
# Install a specific version
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | WMGR_VERSION=v1.0.0 sh

# Install to a custom path
curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | WMGR_INSTALL_PATH=$HOME/.local/bin sh
```

### Manual Download

Download from [Releases](https://github.com/tk-aria/wmgr/releases) and place in your `PATH`:

| Platform | File |
|---|---|
| Linux x86_64 | `wmgr-linux-x86_64.tar.gz` |
| Linux ARM64 | `wmgr-linux-aarch64.tar.gz` |
| macOS Intel | `wmgr-darwin-x86_64.tar.gz` |
| macOS Apple Silicon | `wmgr-darwin-aarch64.tar.gz` |
| Windows x86_64 | `wmgr-windows-x86_64.tar.gz` |

```bash
# Example: macOS Apple Silicon
curl -L https://github.com/tk-aria/wmgr/releases/latest/download/wmgr-darwin-aarch64.tar.gz | tar xz
sudo mv wmgr /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr
cargo build --release
sudo cp target/release/wmgr /usr/local/bin/
```

**Requirements:** Rust 1.70+, Git 2.0+

---

## Quick Start

**1. Create a manifest** (`wmgr.yml`):

```yaml
repos:
  - dest: frontend
    url: https://github.com/example/frontend.git
    branch: main
    groups: [web]

  - dest: backend
    url: https://github.com/example/backend.git
    branch: main
    groups: [api]

  - dest: shared-assets
    url: s3://my-bucket/assets
    scm: s3
    scm_options:
      type: S3
      region: ap-northeast-1
```

**2. Sync everything:**

```bash
wmgr sync
```

**3. Check status:**

```bash
wmgr status
```

---

## Commands

| Command | Description |
|---|---|
| `wmgr init <manifest>` | Initialize workspace from a manifest file |
| `wmgr sync` | Synchronize all repositories |
| `wmgr status` | Show repository status |
| `wmgr foreach <cmd>` | Run a command in each repository |
| `wmgr log` | Show commit logs across repositories |
| `wmgr dump-manifest` | Output the current manifest |
| `wmgr apply-manifest <file>` | Apply a new manifest |
| `wmgr audit` | Run security audit on the workspace |

### Examples

```bash
# Sync only web group
wmgr sync --group web

# Run tests in parallel across all repos
wmgr foreach "make test" --parallel

# Export status as JSON (useful for CI)
wmgr status --output json

# Force re-sync
wmgr sync --force
```

---

## Manifest Format

### Basic (flat list)

```yaml
repos:
  - dest: my-app
    url: https://github.com/org/my-app.git
    branch: main
```

### Multi-SCM workspace

```yaml
repos:
  # Git repository
  - dest: backend
    url: https://github.com/org/backend.git
    branch: develop
    groups: [core]

  # HTTP archive download (auto-extracted)
  - dest: tools/sdk
    url: https://releases.example.com/sdk-v2.0.tar.gz
    scm: http

  # S3 bucket sync
  - dest: assets/textures
    url: s3://game-assets/textures
    scm: s3
    scm_options:
      type: S3
      region: us-east-1

  # Google Drive via rclone
  - dest: design-files
    url: "gdrive-remote:Design/ProjectX"
    scm: gdrive
    scm_options:
      type: GDrive
      rclone_remote: gdrive-remote

  # Local symlink
  - dest: shared-config
    url: /opt/shared/config
    scm: symlink
```

### Repository Options

| Field | Required | Description |
|---|---|---|
| `dest` | Yes | Local directory name |
| `url` | Yes | Repository URL or path |
| `branch` | No | Branch to checkout (default: `main`) |
| `groups` | No | Groups this repo belongs to |
| `scm` | No | SCM type: `git`, `svn`, `p4`, `hg`, `http`, `s3`, `gdrive`, `symlink` |
| `scm_options` | No | SCM-specific configuration |
| `tag` | No | Specific tag to checkout |
| `sha1` | No | Specific commit hash |
| `credential` | No | Credential profile name |

---

## Credential Management

wmgr supports a 4-level credential resolution system:

1. **Environment variables** — `WMGR_AWS_ACCESS_KEY_ID`, etc.
2. **Manifest inline** — credentials directly in `wmgr.yml`
3. **Profile reference** — `~/.config/wmgr/credential.yml`
4. **System credential helpers** — git credential helpers, AWS CLI config

```yaml
# ~/.config/wmgr/credential.yml
profiles:
  my-s3:
    type: S3
    access_key_id: AKIA...
    secret_access_key: ...
    region: ap-northeast-1
```

```yaml
# wmgr.yml
repos:
  - dest: assets
    url: s3://my-bucket/assets
    scm: s3
    credential: my-s3
```

---

## CI/CD Integration

### GitHub Actions

```yaml
- name: Install wmgr
  run: |
    curl -sSLf https://raw.githubusercontent.com/tk-aria/wmgr/main/scripts/install.sh | sh

- name: Sync workspace
  run: wmgr sync

- name: Run tests
  run: wmgr foreach "make test" --parallel
```

---

## Project Structure

```
wmgr/
├── crates/
│   ├── wmgr/          # Library crate (domain, application, infrastructure)
│   └── wmgr-cli/      # Binary crate (CLI interface)
├── config/            # Embedded templates
├── scripts/           # Install & build scripts
├── templates/         # User-facing YAML templates
├── tests/             # Integration tests
└── docs/              # Documentation
```

## Development

```bash
git clone https://github.com/tk-aria/wmgr.git
cd wmgr
cargo build            # Dev build
cargo test             # Run all tests
cargo clippy           # Lint
cargo fmt              # Format
```

## License

[BSD-3-Clause](LICENSE)

## Acknowledgments

- Inspired by [tsrc](https://github.com/TankerHQ/tsrc) (Python)
- Built with Rust
