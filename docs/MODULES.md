# Module Documentation

This page provides links to the main modules in the wmgr crate.

## Core Modules

- [wmgr](wmgr/index.html) - Main crate documentation
- [Domain](wmgr/domain/index.html) - Core business logic
- [Application](wmgr/application/index.html) - Use cases and services
- [Infrastructure](wmgr/infrastructure/index.html) - External integrations
- [Presentation](wmgr/presentation/index.html) - User interface
- [Common](wmgr/common/index.html) - Shared utilities

## Key Components

### Domain Entities

- [Workspace](wmgr/domain/entities/workspace/struct.Workspace.html)
- [Manifest](wmgr/domain/entities/manifest/struct.Manifest.html)
- [Repository](wmgr/domain/entities/repository/struct.Repository.html)

### Value Objects

- [GitUrl](wmgr/domain/value_objects/git_url/struct.GitUrl.html)
- [BranchName](wmgr/domain/value_objects/branch_name/struct.BranchName.html)
- [FilePath](wmgr/domain/value_objects/file_path/struct.FilePath.html)

### Use Cases

- [InitWorkspace](wmgr/application/use_cases/init_workspace/index.html)
- [SyncRepositories](wmgr/application/use_cases/sync_repositories/index.html)
- [StatusCheck](wmgr/application/use_cases/status_check/index.html)
- [ForeachCommand](wmgr/application/use_cases/foreach_command/index.html)

### Services

- [ManifestService](wmgr/application/services/manifest_service/index.html)

### Error Handling

- [TsrcError](wmgr/common/error/enum.TsrcError.html)
- [Result](wmgr/common/result/type.Result.html)
