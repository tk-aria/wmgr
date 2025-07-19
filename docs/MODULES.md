# Module Documentation

This page provides links to the main modules in the tsrc crate.

## Core Modules

- [tsrc](tsrc/index.html) - Main crate documentation
- [Domain](tsrc/domain/index.html) - Core business logic
- [Application](tsrc/application/index.html) - Use cases and services
- [Infrastructure](tsrc/infrastructure/index.html) - External integrations
- [Presentation](tsrc/presentation/index.html) - User interface
- [Common](tsrc/common/index.html) - Shared utilities

## Key Components

### Domain Entities

- [Workspace](tsrc/domain/entities/workspace/struct.Workspace.html)
- [Manifest](tsrc/domain/entities/manifest/struct.Manifest.html)
- [Repository](tsrc/domain/entities/repository/struct.Repository.html)

### Value Objects

- [GitUrl](tsrc/domain/value_objects/git_url/struct.GitUrl.html)
- [BranchName](tsrc/domain/value_objects/branch_name/struct.BranchName.html)
- [FilePath](tsrc/domain/value_objects/file_path/struct.FilePath.html)

### Use Cases

- [InitWorkspace](tsrc/application/use_cases/init_workspace/index.html)
- [SyncRepositories](tsrc/application/use_cases/sync_repositories/index.html)
- [StatusCheck](tsrc/application/use_cases/status_check/index.html)
- [ForeachCommand](tsrc/application/use_cases/foreach_command/index.html)

### Services

- [ManifestService](tsrc/application/services/manifest_service/index.html)

### Error Handling

- [TsrcError](tsrc/common/error/enum.TsrcError.html)
- [Result](tsrc/common/result/type.Result.html)
