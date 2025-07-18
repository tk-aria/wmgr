# Contributing to tsrc

Thank you for your interest in contributing to tsrc! This document provides guidelines and information for contributors.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Contributing Guidelines](#contributing-guidelines)
4. [Code Standards](#code-standards)
5. [Testing](#testing)
6. [Documentation](#documentation)
7. [Submitting Changes](#submitting-changes)
8. [Release Process](#release-process)

## Getting Started

### Prerequisites

- Rust 1.85.0 or later
- Git 2.0 or later
- Basic familiarity with Rust and Git

### Areas for Contribution

We welcome contributions in these areas:

- **Bug Fixes**: Fix issues reported in GitHub Issues
- **Features**: Implement new functionality
- **Documentation**: Improve user guides, API docs, and examples
- **Testing**: Add tests and improve test coverage
- **Performance**: Optimize operations and reduce resource usage
- **Platform Support**: Improve support for different operating systems
- **CI/CD**: Enhance build and deployment processes

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/your-username/wmgr.git
cd wmgr/v2

# Add upstream remote
git remote add upstream https://github.com/tk-aria/wmgr.git
```

### 2. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install required components
rustup component add rustfmt clippy rust-src

# Install additional targets for cross-compilation (optional)
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-msvc
```

### 3. Build and Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run clippy for linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all
```

### 4. Development Workflow

We provide several convenience scripts:

```bash
# Run full development validation
./scripts/dev-build.sh all

# Run specific checks
./scripts/dev-build.sh clippy
./scripts/dev-build.sh test
./scripts/dev-build.sh fmt

# Using Make
make dev      # Format + clippy + test
make all      # Full validation pipeline
make help     # Show all available targets
```

## Contributing Guidelines

### Code of Conduct

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all interactions.

### Issue Guidelines

#### Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Minimal steps to reproduce the issue
3. **Expected Behavior**: What you expected to happen
4. **Actual Behavior**: What actually happened
5. **Environment**: OS, Rust version, tsrc version
6. **Manifest**: Minimal manifest file that reproduces the issue (if applicable)
7. **Logs**: Relevant log output with `TSRC_LOG=debug`

Example bug report:

```markdown
## Bug Description
tsrc sync fails with permission error on Windows

## Steps to Reproduce
1. Create manifest with SSH URL
2. Run `tsrc init manifest.yml`
3. Run `tsrc sync`

## Expected Behavior
Repositories should sync successfully

## Actual Behavior
Error: Permission denied (publickey)

## Environment
- OS: Windows 11
- Rust: 1.85.0
- tsrc: 0.1.0

## Logs
```
TSRC_LOG=debug tsrc sync
[DEBUG] Attempting to clone repository...
[ERROR] Permission denied (publickey)
```
```

#### Feature Requests

When requesting features, please include:

1. **Use Case**: Why this feature would be useful
2. **Description**: Detailed description of the feature
3. **Examples**: How the feature would be used
4. **Alternatives**: Alternative solutions you've considered

### Pull Request Guidelines

1. **One Feature Per PR**: Keep PRs focused on a single feature or fix
2. **Branch Naming**: Use descriptive branch names (e.g., `feature/parallel-sync`, `fix/windows-path-handling`)
3. **Commit Messages**: Write clear, descriptive commit messages
4. **Tests**: Include tests for new functionality
5. **Documentation**: Update documentation for user-facing changes
6. **Changelog**: Add entry to CHANGELOG.md for significant changes

## Code Standards

### Rust Style

Follow the standard Rust style guidelines:

```bash
# Format code according to Rust standards
cargo fmt --all

# Check for common issues
cargo clippy --all-targets --all-features -- -D warnings
```

### Architecture Principles

Our codebase follows clean architecture principles:

1. **Domain Layer** (`src/domain/`): Core business logic
   - Entities: Core business objects
   - Value Objects: Immutable objects with validation
   - No external dependencies

2. **Application Layer** (`src/application/`): Use cases and workflows
   - Use Cases: Business workflows
   - Services: Application-specific services
   - Depends only on domain layer

3. **Infrastructure Layer** (`src/infrastructure/`): External integrations
   - Git operations
   - File system operations
   - Network operations
   - Depends on application and domain layers

4. **Presentation Layer** (`src/presentation/`): User interface
   - CLI implementation
   - User interaction
   - Depends on application layer

### Error Handling

- Use `TsrcError` for all errors
- Provide meaningful error messages with context
- Include relevant information (file paths, repository names, etc.)
- Use `Result` type consistently

```rust
// Good
fn parse_manifest(path: &Path) -> Result<Manifest, TsrcError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| TsrcError::filesystem_error_with_source(
            "Failed to read manifest file",
            Some(path.to_path_buf()),
            e
        ))?;
    
    serde_yaml::from_str(&content)
        .map_err(|e| TsrcError::manifest_error_with_source(
            "Failed to parse manifest YAML",
            Some(path.to_path_buf()),
            e
        ))
}
```

### Documentation

- Add rustdoc comments for public APIs
- Include examples in documentation
- Use `//!` for module-level documentation
- Use `///` for function/struct documentation

```rust
/// Represents a Git repository in a workspace.
///
/// A repository contains information about where to clone from,
/// where to place the code locally, and which branch/tag/commit to use.
///
/// # Examples
///
/// ```rust
/// use tsrc::domain::entities::repository::{Repository, Remote};
///
/// let repo = Repository::new("backend/api", vec![
///     Remote::new("origin", "https://github.com/example/api.git")
/// ]);
/// ```
pub struct Repository {
    // ...
}
```

### Testing Standards

#### Unit Tests

- Test individual functions and methods in isolation
- Use descriptive test names that explain what is being tested
- Follow the Arrange-Act-Assert pattern
- Use test fixtures for consistent test data

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_git_url_parses_https_urls_correctly() {
        // Arrange
        let url_string = "https://github.com/owner/repo.git";
        
        // Act
        let result = GitUrl::new(url_string);
        
        // Assert
        assert!(result.is_ok());
        let git_url = result.unwrap();
        assert_eq!(git_url.host(), "github.com");
        assert_eq!(git_url.organization(), Some("owner"));
        assert_eq!(git_url.repo_name(), Some("repo"));
    }
}
```

#### Integration Tests

- Test complete workflows and component interactions
- Use temporary directories for file system operations
- Clean up resources after tests
- Test both success and failure scenarios

```rust
#[tokio::test]
async fn test_workspace_initialization_workflow() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let manifest = create_test_manifest();
    
    // Act
    let result = initialize_workspace(&temp_dir, &manifest).await;
    
    // Assert
    assert!(result.is_ok());
    assert!(temp_dir.path().join(".tsrc").exists());
    assert!(temp_dir.path().join("repo1").exists());
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_git_url_parsing

# Run tests in parallel
cargo test --jobs 4
```

### Test Coverage

We aim for high test coverage:

```bash
# Generate coverage report
cargo tarpaulin --out html --output-dir target/coverage

# Open coverage report
open target/coverage/tarpaulin-report.html
```

### Performance Testing

```bash
# Run benchmarks
cargo bench

# Profile performance
cargo build --release
perf record target/release/tsrc sync
perf report
```

## Documentation

### User Documentation

- **README.md**: Overview and quick start
- **docs/USER_GUIDE.md**: Comprehensive user guide
- **docs/examples/**: Example configurations and workflows

### Developer Documentation

- **docs/ARCHITECTURE.md**: System architecture overview
- **docs/API.md**: API documentation
- **Rustdoc**: Generated from code comments

### Documentation Guidelines

1. **Clarity**: Write for your audience (users vs. developers)
2. **Examples**: Include practical examples
3. **Maintenance**: Keep documentation up to date with code changes
4. **Testing**: Test code examples to ensure they work

### Building Documentation

```bash
# Build API documentation
cargo doc --open --all-features

# Generate complete documentation
./scripts/generate-docs.sh

# Serve documentation locally
make docs-open
```

## Submitting Changes

### Commit Guidelines

Follow conventional commit format:

```
type(scope): description

Detailed explanation of the change if needed.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(sync): add parallel repository synchronization

Add support for syncing multiple repositories in parallel to improve
performance on large workspaces.

Fixes #42

fix(windows): resolve path handling on Windows

Replace Unix-specific path operations with cross-platform alternatives.

Fixes #58
```

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Changes**
   - Implement your feature or fix
   - Add tests
   - Update documentation
   - Run validation pipeline

3. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

4. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   # Create pull request on GitHub
   ```

5. **PR Requirements**
   - All tests pass
   - Code is formatted (`cargo fmt`)
   - No clippy warnings
   - Documentation is updated
   - Changelog entry added (for significant changes)

### Review Process

1. **Automated Checks**: CI runs tests, linting, and builds
2. **Code Review**: Maintainers review code quality and design
3. **Testing**: Changes are tested in different environments
4. **Approval**: At least one maintainer approval required
5. **Merge**: Squash and merge after approval

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Steps

1. **Update Version**
   ```bash
   # Update Cargo.toml
   sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml
   ```

2. **Update Changelog**
   - Move unreleased changes to new version section
   - Add release date
   - Update unreleased section

3. **Create Release Commit**
   ```bash
   git add .
   git commit -m "chore: release v0.2.0"
   git tag v0.2.0
   ```

4. **Push Release**
   ```bash
   git push origin main
   git push origin v0.2.0
   ```

5. **Automated Release**
   - GitHub Actions builds binaries
   - Creates GitHub release
   - Publishes to crates.io
   - Updates Homebrew formula

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Discord**: Real-time chat (link in README)

### Mentorship

New contributors are welcome! We provide mentorship for:

- First-time contributors to Rust
- First-time contributors to open source
- Understanding the codebase architecture
- Implementing specific features

Look for issues labeled `good first issue` or `mentor available`.

## Recognition

Contributors are recognized in:

- **CONTRIBUTORS.md**: List of all contributors
- **Release Notes**: Major contributions highlighted
- **GitHub**: Contributor graphs and statistics

Thank you for contributing to tsrc! Your efforts help make the tool better for everyone.