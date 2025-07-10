# tsrc User Guide

Comprehensive guide for using tsrc to manage multiple git repositories.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Commands Reference](#commands-reference)
3. [Manifest Configuration](#manifest-configuration)
4. [Working with Groups](#working-with-groups)
5. [Advanced Usage](#advanced-usage)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Getting Started

### Installation

```bash
# Clone and build from source
git clone https://github.com/tk-aria/wmgr.git
cd wmgr/v2
cargo build --release
```

### Your First Workspace

1. **Create a manifest file** (`manifest.yml`):

```yaml
repos:
  - dest: "frontend"
    url: "https://github.com/myorg/frontend.git"
    branch: "main"
    groups: ["web"]
  
  - dest: "backend"
    url: "https://github.com/myorg/backend.git"
    branch: "main"
    groups: ["api"]
```

2. **Initialize your workspace**:

```bash
mkdir my-workspace
cd my-workspace
tsrc init manifest.yml
```

3. **Verify the setup**:

```bash
tsrc status
```

## Commands Reference

### `tsrc init`

Initialize a new workspace from a manifest file.

**Syntax:**
```bash
tsrc init <manifest_path> [OPTIONS]
```

**Options:**
- `--group <GROUP>`: Only initialize repositories from specific group(s)
- `--force`: Force initialization even if workspace already exists

**Examples:**

```bash
# Initialize all repositories
tsrc init manifest.yml

# Initialize only web repositories
tsrc init manifest.yml --group web

# Initialize multiple groups
tsrc init manifest.yml --group web --group api

# Force re-initialization
tsrc init manifest.yml --force
```

**What it does:**
- Creates `.tsrc/config.yml` in your workspace
- Clones all specified repositories
- Sets up configured remotes
- Checks out specified branches

### `tsrc sync`

Synchronize all repositories in the workspace.

**Syntax:**
```bash
tsrc sync [OPTIONS]
```

**Options:**
- `--group <GROUP>`: Only sync repositories from specific group(s)
- `--force`: Force sync even if there are uncommitted changes
- `--no-correct-branch`: Skip branch synchronization

**Examples:**

```bash
# Sync all repositories
tsrc sync

# Sync only API repositories
tsrc sync --group api

# Force sync with uncommitted changes
tsrc sync --force

# Sync without switching branches
tsrc sync --no-correct-branch
```

**What it does:**
- Fetches latest changes from remotes
- Merges changes (fast-forward only)
- Switches to configured branch
- Updates remote configurations

### `tsrc status`

Show the status of all repositories in the workspace.

**Syntax:**
```bash
tsrc status [OPTIONS]
```

**Options:**
- `--branch`: Show branch information
- `--compact`: Use compact output format

**Examples:**

```bash
# Basic status
tsrc status

# Status with branch information
tsrc status --branch

# Compact output
tsrc status --compact
```

**Output example:**
```
frontend: clean (main)
backend: dirty (feature/auth) - 2 files modified
shared: ahead 3 commits (develop)
```

### `tsrc foreach`

Execute a command in each repository.

**Syntax:**
```bash
tsrc foreach <command> [OPTIONS]
```

**Options:**
- `--group <GROUP>`: Only execute in repositories from specific group(s)
- `--parallel`: Execute commands in parallel

**Examples:**

```bash
# Pull all repositories
tsrc foreach "git pull"

# Install dependencies in web repositories
tsrc foreach "npm install" --group web

# Run tests in parallel
tsrc foreach "make test" --parallel

# Check Git status
tsrc foreach "git status --porcelain"
```

**Environment Variables:**
The foreach command sets these environment variables:
- `TSRC_REPO_NAME`: Name of the current repository
- `TSRC_REPO_PATH`: Full path to the repository
- `TSRC_WORKSPACE_PATH`: Path to the workspace root

### `tsrc log`

Show commit logs across repositories.

**Syntax:**
```bash
tsrc log [OPTIONS]
```

**Examples:**

```bash
# Show recent commits
tsrc log

# Show commits for specific group
tsrc log --group web
```

### `tsrc dump-manifest`

Output the current workspace manifest.

**Syntax:**
```bash
tsrc dump-manifest
```

**Use cases:**
- Backup current configuration
- Generate manifest templates
- Debug configuration issues

### `tsrc apply-manifest`

Apply a new manifest to the current workspace.

**Syntax:**
```bash
tsrc apply-manifest <manifest_path>
```

**Examples:**

```bash
# Apply updated manifest
tsrc apply-manifest updated-manifest.yml
```

## Manifest Configuration

### Basic Structure

```yaml
repos:
  - dest: "local-directory"
    url: "https://github.com/owner/repo.git"
    branch: "main"
    groups: ["group1", "group2"]
```

### Advanced Configuration

```yaml
repos:
  - dest: "complex-repo"
    url: "https://github.com/owner/repo.git"
    branch: "develop"
    groups: ["core", "backend"]
    tag: "v2.1.0"                    # Use specific tag
    sha1: "abc123def456..."          # Use specific commit
    remotes:
      - name: "upstream"
        url: "https://github.com/upstream/repo.git"
      - name: "fork"
        url: "https://github.com/myuser/repo.git"
```

### Repository Options

| Option | Required | Description | Example |
|--------|----------|-------------|---------|
| `dest` | Yes | Local directory name | `"frontend"` |
| `url` | Yes | Git repository URL | `"https://github.com/org/repo.git"` |
| `branch` | No | Branch to checkout | `"main"` (default) |
| `groups` | No | Groups this repo belongs to | `["web", "core"]` |
| `tag` | No | Specific tag to checkout | `"v1.0.0"` |
| `sha1` | No | Specific commit to checkout | `"abc123..."` |
| `remotes` | No | Additional remotes | See example above |

### Validation Rules

- `dest` must be a valid directory name
- `url` must be a valid Git URL
- `branch` must be a valid branch name
- `groups` must be an array of strings
- `tag` and `sha1` cannot be used together
- `remotes` must have unique names

## Working with Groups

Groups allow you to organize repositories logically and operate on subsets.

### Defining Groups

```yaml
repos:
  - dest: "frontend"
    url: "https://github.com/org/frontend.git"
    groups: ["web", "client"]
  
  - dest: "backend"
    url: "https://github.com/org/backend.git"
    groups: ["api", "server"]
  
  - dest: "shared"
    url: "https://github.com/org/shared.git"
    groups: ["web", "api", "common"]
```

### Using Groups

```bash
# Initialize only web repositories
tsrc init manifest.yml --group web

# Sync only API repositories
tsrc sync --group api

# Run tests in client repositories
tsrc foreach "npm test" --group client

# Multiple groups
tsrc sync --group web --group api
```

### Group Strategies

**By Technology:**
```yaml
groups: ["react", "node", "python"]
```

**By Team:**
```yaml
groups: ["frontend-team", "backend-team", "devops-team"]
```

**By Feature:**
```yaml
groups: ["authentication", "payments", "analytics"]
```

**By Environment:**
```yaml
groups: ["development", "staging", "production"]
```

## Advanced Usage

### Parallel Operations

Most commands support parallel execution:

```bash
# Parallel sync
tsrc sync --parallel

# Parallel command execution
tsrc foreach "git pull" --parallel

# Parallel with specific group
tsrc foreach "make build" --group web --parallel
```

### Complex Workflows

**Multi-stage deployment:**
```bash
# Stage 1: Update all repositories
tsrc sync

# Stage 2: Build services
tsrc foreach "make build" --group services --parallel

# Stage 3: Run tests
tsrc foreach "make test" --parallel

# Stage 4: Deploy
tsrc foreach "make deploy" --group production
```

**Development workflow:**
```bash
# Check current state
tsrc status --branch

# Create feature branches
tsrc foreach "git checkout -b feature/new-feature" --group core

# Work on changes...

# Check status before commit
tsrc status

# Commit changes
tsrc foreach "git add . && git commit -m 'Add new feature'"

# Push changes
tsrc foreach "git push origin feature/new-feature"
```

### Environment Variables in foreach

```bash
# Use environment variables in commands
tsrc foreach 'echo "Working on $TSRC_REPO_NAME in $TSRC_REPO_PATH"'

# Conditional execution
tsrc foreach 'if [ -f package.json ]; then npm install; fi'

# Generate reports
tsrc foreach 'echo "$TSRC_REPO_NAME,$(git rev-parse HEAD)" >> ../report.csv'
```

## Best Practices

### Repository Organization

1. **Use meaningful group names**
   ```yaml
   groups: ["web-ui", "api-services", "data-processing"]
   ```

2. **Keep manifests focused**
   - Create separate manifests for different purposes
   - Use descriptive names: `dev-manifest.yml`, `prod-manifest.yml`

3. **Document your groups**
   ```yaml
   # Groups:
   # - web: Frontend applications
   # - api: Backend services
   # - tools: Development tools
   repos:
     - dest: "frontend"
       groups: ["web"]
   ```

### Workflow Best Practices

1. **Regular synchronization**
   ```bash
   # Daily sync routine
   tsrc sync
   tsrc status
   ```

2. **Use groups for targeted operations**
   ```bash
   # Deploy only production services
   tsrc foreach "make deploy" --group production
   ```

3. **Parallel execution for speed**
   ```bash
   # Faster operations
   tsrc sync --parallel
   tsrc foreach "make test" --parallel
   ```

### Manifest Management

1. **Version control your manifests**
   ```bash
   git add manifest.yml
   git commit -m "Update manifest: add new repository"
   ```

2. **Use meaningful commit messages**
   ```bash
   git commit -m "manifest: add payment service to api group"
   ```

3. **Test manifest changes**
   ```bash
   # Test in a separate directory
   mkdir test-workspace
   cd test-workspace
   tsrc init ../new-manifest.yml
   ```

## Troubleshooting

### Common Issues

#### Repository Clone Failures

**Problem:** Repository fails to clone
```
Error: Failed to clone repository: authentication required
```

**Solutions:**
1. Check Git credentials: `git config --global credential.helper`
2. Verify repository URL: `git ls-remote <url>`
3. Check network connectivity
4. Use SSH instead of HTTPS if needed

#### Permission Errors

**Problem:** Permission denied during operations
```
Error: Permission denied (publickey)
```

**Solutions:**
1. Set up SSH keys: `ssh-keygen -t ed25519 -C "your-email@example.com"`
2. Add key to SSH agent: `ssh-add ~/.ssh/id_ed25519`
3. Add key to GitHub/GitLab account
4. Test connection: `ssh -T git@github.com`

#### Merge Conflicts

**Problem:** Sync fails due to merge conflicts
```
Error: Cannot fast-forward, merge conflicts detected
```

**Solutions:**
1. Use `--force` flag to override (careful!)
2. Resolve conflicts manually in each repository
3. Use `tsrc status` to identify problematic repositories

#### Workspace Corruption

**Problem:** Workspace appears corrupted
```
Error: Invalid workspace configuration
```

**Solutions:**
1. Check `.tsrc/config.yml` file
2. Re-initialize workspace: `tsrc init manifest.yml --force`
3. Verify manifest file syntax

### Debugging Commands

**Check workspace state:**
```bash
tsrc status --branch
cat .tsrc/config.yml
```

**Verbose output:**
```bash
RUST_LOG=debug tsrc sync
```

**Manual verification:**
```bash
# Check individual repository
cd repository-name
git status
git remote -v
git branch -a
```

### Performance Issues

**Slow sync operations:**
1. Use `--parallel` flag
2. Check network connectivity
3. Consider using shallow clones for large repositories

**Memory usage:**
1. Limit parallel operations
2. Use `--group` to work with subsets
3. Monitor system resources

### Getting Help

**Command-line help:**
```bash
tsrc --help
tsrc <command> --help
```

**Check version:**
```bash
tsrc --version
```

**Enable debug logging:**
```bash
export RUST_LOG=debug
tsrc sync
```