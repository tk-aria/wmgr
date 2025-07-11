# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete rewrite in Rust for better performance and reliability
- Clean architecture with domain-driven design
- Comprehensive test suite with unit and integration tests
- Parallel operations for improved performance
- Advanced error handling and reporting
- Support for shallow clones
- Enhanced manifest validation
- Built-in shell completion support
- Cross-platform binaries for Linux, macOS, and Windows
- Docker container support
- Comprehensive documentation and user guide

### Features
- **Repository Management**: Clone, sync, and manage multiple git repositories
- **Group Organization**: Organize repositories into logical groups
- **Parallel Operations**: Execute commands across repositories in parallel
- **Status Checking**: Get comprehensive overview of repository states
- **Manifest-based Configuration**: Define workspaces using YAML manifests
- **Remote Manifest Support**: Load manifests from remote URLs
- **Advanced Git Operations**: Support for tags, SHA commits, and multiple remotes
- **Environment Variables**: Rich environment context for command execution
- **CI/CD Integration**: Built for automation and continuous integration

### Commands
- `init`: Initialize workspace from manifest
- `sync`: Synchronize repositories with upstream
- `status`: Check status of all repositories
- `foreach`: Execute commands across repositories
- `log`: View commit logs across repositories
- `dump-manifest`: Export current workspace manifest
- `apply-manifest`: Apply new manifest to existing workspace

### Infrastructure
- Clean architecture with separated layers (domain, application, infrastructure, presentation)
- Comprehensive error handling with detailed context
- Type-safe value objects for Git URLs, branch names, and file paths
- Async/await support for non-blocking operations
- Extensive logging and debugging capabilities
- Memory-safe operations with Rust's ownership system

### Testing
- 145+ unit tests covering core functionality
- Integration tests for end-to-end workflows
- Test helpers and mock services for reliable testing
- Property-based testing for value objects
- Performance benchmarks

### Build and Distribution
- Cross-compilation for multiple platforms
- Optimized release builds with LTO and strip
- GitHub Actions CI/CD pipeline
- Automated release packaging and distribution
- Homebrew formula (coming soon)
- Docker images (coming soon)

### Documentation
- Comprehensive README with examples
- API documentation with rustdoc
- User guide with detailed workflows
- Architecture documentation
- Contributing guidelines
- Security guidelines

## [0.1.0] - 2024-01-XX

### Added
- Initial Rust implementation
- Basic workspace management
- Manifest parsing and validation
- Repository synchronization
- Command execution across repositories
- Group-based operations
- Parallel execution support

---

## Migration from Python Version

If you're migrating from the Python version of tsrc, here are the key differences:

### Compatibility
- Manifest format is largely compatible
- Command-line interface maintains similar patterns
- Some advanced features may have different syntax

### Improvements
- **Performance**: Significantly faster due to Rust's efficiency
- **Memory Usage**: Lower memory footprint
- **Error Handling**: More detailed error messages and context
- **Parallel Operations**: Better parallel execution with work-stealing
- **Type Safety**: Compile-time guarantees prevent many runtime errors

### Breaking Changes
- Some command flags may have changed names
- Configuration file format may differ slightly
- Error message formats have been improved
- Some edge cases may behave differently

### Migration Steps
1. Backup your existing workspace
2. Install the Rust version of tsrc
3. Test with `tsrc status` to verify compatibility
4. Update any automation scripts to use new flag names
5. Review error handling in CI/CD pipelines

For detailed migration guidance, see the [Migration Guide](MIGRATION.md).

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE](LICENSE) file for details.