#[cfg(unix)]
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs as async_fs;

use crate::application::services::manifest_service::{
    ManifestService, ManifestServiceError, ProcessedManifest,
};
use crate::domain::entities::manifest::{FileCopy, FileSymlink, Manifest, ManifestRepo};

/// Manifest store related errors
#[derive(Debug, Error)]
pub enum ManifestStoreError {
    #[error("Manifest file not found at path: {0}")]
    ManifestFileNotFound(String),

    #[error("Invalid manifest file path: {0}")]
    InvalidManifestPath(String),

    #[error("Manifest file read failed: {0}")]
    ReadFailed(String),

    #[error("Manifest file write failed: {0}")]
    WriteFailed(String),

    #[error("YAML parsing failed: {0}")]
    YamlParsingFailed(String),

    #[error("YAML serialization failed: {0}")]
    YamlSerializationFailed(String),

    #[error("Manifest validation failed: {0}")]
    ValidationFailed(String),

    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    #[error("Copy operation failed from {source_path} to {dest_path}: {reason}")]
    CopyOperationFailed {
        source_path: String,
        dest_path: String,
        reason: String,
    },

    #[error("Symlink operation failed from {source_path} to {target_path}: {reason}")]
    SymlinkOperationFailed {
        source_path: String,
        target_path: String,
        reason: String,
    },

    #[error("Directory creation failed: {0}")]
    DirectoryCreationFailed(String),

    #[error("Backup operation failed: {0}")]
    BackupFailed(String),

    #[error("Path validation failed: {0}")]
    PathValidationFailed(String),

    #[error("Manifest service error: {0}")]
    ManifestServiceError(#[from] ManifestServiceError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Manifest file metadata
#[derive(Debug, Clone)]
pub struct ManifestMetadata {
    /// File path
    pub path: PathBuf,

    /// Last modified time
    pub last_modified: std::time::SystemTime,

    /// File size in bytes
    pub size: u64,

    /// Whether the file exists
    pub exists: bool,

    /// Number of repositories in the manifest
    pub repo_count: usize,

    /// Number of groups in the manifest
    pub group_count: usize,
}

/// File operation configuration
#[derive(Debug, Clone)]
pub struct FileOperationConfig {
    /// Whether to create backup before file operations
    pub create_backup: bool,

    /// Whether to overwrite existing files
    pub overwrite_existing: bool,

    /// Whether to create parent directories if they don't exist
    pub create_parent_dirs: bool,

    /// Whether to validate file paths before operations
    pub validate_paths: bool,

    /// Maximum number of backup files to keep
    pub max_backups: usize,
}

impl Default for FileOperationConfig {
    fn default() -> Self {
        Self {
            create_backup: true,
            overwrite_existing: false,
            create_parent_dirs: true,
            validate_paths: true,
            max_backups: 5,
        }
    }
}

/// File operation result
#[derive(Debug, Clone)]
pub struct FileOperationResult {
    /// Source path
    pub source: PathBuf,

    /// Destination path
    pub destination: PathBuf,

    /// Operation type (copy, symlink)
    pub operation_type: String,

    /// Whether the operation was successful
    pub success: bool,

    /// Error message if operation failed
    pub error: Option<String>,

    /// Whether a backup was created
    pub backup_created: bool,
}

/// Manifest processing options
#[derive(Debug, Clone)]
pub struct ManifestProcessingOptions {
    /// Whether to process file copy operations
    pub process_copy_operations: bool,

    /// Whether to process symlink operations
    pub process_symlink_operations: bool,

    /// Whether to validate manifest before processing
    pub validate_manifest: bool,

    /// Base directory for resolving relative paths
    pub base_directory: Option<PathBuf>,

    /// File operation configuration
    pub file_operation_config: FileOperationConfig,
}

impl Default for ManifestProcessingOptions {
    fn default() -> Self {
        Self {
            process_copy_operations: true,
            process_symlink_operations: true,
            validate_manifest: true,
            base_directory: None,
            file_operation_config: FileOperationConfig::default(),
        }
    }
}

/// Manifest store for managing manifest files and file operations
pub struct ManifestStore {
    /// Manifest service for parsing and validation
    manifest_service: ManifestService,

    /// Processing options
    options: ManifestProcessingOptions,
}

impl ManifestStore {
    /// Create a new manifest store with default settings
    pub fn new() -> Self {
        Self {
            manifest_service: ManifestService::default(),
            options: ManifestProcessingOptions::default(),
        }
    }

    /// Create a new manifest store with custom settings
    pub fn with_options(options: ManifestProcessingOptions) -> Self {
        Self {
            manifest_service: ManifestService::default(),
            options,
        }
    }

    /// Create a new manifest store with custom manifest service and options
    pub fn with_service_and_options(
        manifest_service: ManifestService,
        options: ManifestProcessingOptions,
    ) -> Self {
        Self {
            manifest_service,
            options,
        }
    }

    /// Read and parse manifest from YAML file
    pub async fn read_manifest<P: AsRef<Path>>(
        &mut self,
        manifest_path: P,
    ) -> Result<ProcessedManifest, ManifestStoreError> {
        let manifest_path = manifest_path.as_ref();

        // Check if file exists
        if !manifest_path.exists() {
            return Err(ManifestStoreError::ManifestFileNotFound(
                manifest_path.display().to_string(),
            ));
        }

        // Parse manifest using the service
        let processed_manifest = self
            .manifest_service
            .parse_from_file(manifest_path)
            .await
            .map_err(ManifestStoreError::ManifestServiceError)?;

        // Additional validation if enabled
        if self.options.validate_manifest {
            self.validate_file_operations(&processed_manifest.manifest, manifest_path)?;
        }

        Ok(processed_manifest)
    }

    /// Read manifest from URL
    pub async fn read_manifest_from_url(
        &mut self,
        url: &str,
    ) -> Result<ProcessedManifest, ManifestStoreError> {
        let processed_manifest = self
            .manifest_service
            .parse_from_url(url)
            .await
            .map_err(ManifestStoreError::ManifestServiceError)?;

        // Additional validation if enabled
        if self.options.validate_manifest {
            self.validate_file_operations(&processed_manifest.manifest, Path::new("."))?;
        }

        Ok(processed_manifest)
    }

    /// Write manifest to YAML file
    pub async fn write_manifest<P: AsRef<Path>>(
        &self,
        manifest_path: P,
        manifest: &Manifest,
    ) -> Result<(), ManifestStoreError> {
        let manifest_path = manifest_path.as_ref();

        // Validate manifest before writing
        if self.options.validate_manifest {
            self.manifest_service
                .validate_manifest(manifest)
                .map_err(ManifestStoreError::ManifestServiceError)?;
        }

        // Create backup if enabled and file exists
        if self.options.file_operation_config.create_backup && manifest_path.exists() {
            self.create_backup(manifest_path).await?;
        }

        // Ensure parent directory exists
        if let Some(parent) = manifest_path.parent() {
            if self.options.file_operation_config.create_parent_dirs && !parent.exists() {
                async_fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ManifestStoreError::DirectoryCreationFailed(e.to_string()))?;
            }
        }

        // Serialize manifest to YAML
        let yaml_content = self
            .manifest_service
            .serialize_to_yaml(manifest)
            .map_err(ManifestStoreError::ManifestServiceError)?;

        // Write to file
        async_fs::write(manifest_path, yaml_content)
            .await
            .map_err(|e| ManifestStoreError::WriteFailed(e.to_string()))?;

        Ok(())
    }

    /// Process file copy operations from manifest
    pub async fn process_copy_operations<P: AsRef<Path>>(
        &self,
        manifest: &Manifest,
        workspace_root: P,
    ) -> Result<Vec<FileOperationResult>, ManifestStoreError> {
        if !self.options.process_copy_operations {
            return Ok(Vec::new());
        }

        let workspace_root = workspace_root.as_ref();
        let mut results = Vec::new();

        for repo in &manifest.repos {
            if let Some(copy_operations) = &repo.copy {
                for copy_op in copy_operations {
                    let result = self
                        .execute_copy_operation(copy_op, repo, workspace_root)
                        .await;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Process symlink operations from manifest
    pub async fn process_symlink_operations<P: AsRef<Path>>(
        &self,
        manifest: &Manifest,
        workspace_root: P,
    ) -> Result<Vec<FileOperationResult>, ManifestStoreError> {
        if !self.options.process_symlink_operations {
            return Ok(Vec::new());
        }

        let workspace_root = workspace_root.as_ref();
        let mut results = Vec::new();

        for repo in &manifest.repos {
            if let Some(symlink_operations) = &repo.symlink {
                for symlink_op in symlink_operations {
                    let result = self
                        .execute_symlink_operation(symlink_op, repo, workspace_root)
                        .await;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    /// Process all file operations from manifest
    pub async fn process_all_file_operations<P: AsRef<Path>>(
        &self,
        manifest: &Manifest,
        workspace_root: P,
    ) -> Result<Vec<FileOperationResult>, ManifestStoreError> {
        let workspace_root = workspace_root.as_ref();
        let mut results = Vec::new();

        // Process copy operations
        let mut copy_results = self
            .process_copy_operations(manifest, workspace_root)
            .await?;
        results.append(&mut copy_results);

        // Process symlink operations
        let mut symlink_results = self
            .process_symlink_operations(manifest, workspace_root)
            .await?;
        results.append(&mut symlink_results);

        Ok(results)
    }

    /// Get manifest file metadata
    pub async fn get_manifest_metadata<P: AsRef<Path>>(
        &mut self,
        manifest_path: P,
    ) -> Result<ManifestMetadata, ManifestStoreError> {
        let manifest_path = manifest_path.as_ref();

        let mut metadata = ManifestMetadata {
            path: manifest_path.to_path_buf(),
            last_modified: std::time::SystemTime::UNIX_EPOCH,
            size: 0,
            exists: false,
            repo_count: 0,
            group_count: 0,
        };

        if manifest_path.exists() {
            let fs_metadata = async_fs::metadata(manifest_path)
                .await
                .map_err(|e| ManifestStoreError::ReadFailed(e.to_string()))?;

            metadata.last_modified = fs_metadata
                .modified()
                .map_err(|e| ManifestStoreError::ReadFailed(e.to_string()))?;
            metadata.size = fs_metadata.len();
            metadata.exists = true;

            // Read manifest to get repo and group counts
            if let Ok(processed_manifest) = self.read_manifest(manifest_path).await {
                metadata.repo_count = processed_manifest.manifest.repos.len();
                metadata.group_count = processed_manifest
                    .manifest
                    .groups
                    .as_ref()
                    .map(|g| g.len())
                    .unwrap_or(0);
            }
        }

        Ok(metadata)
    }

    /// Check if manifest file exists
    pub fn manifest_exists<P: AsRef<Path>>(&self, manifest_path: P) -> bool {
        manifest_path.as_ref().exists()
    }

    /// Delete manifest file
    pub async fn delete_manifest<P: AsRef<Path>>(
        &self,
        manifest_path: P,
    ) -> Result<(), ManifestStoreError> {
        let manifest_path = manifest_path.as_ref();

        if !manifest_path.exists() {
            return Ok(()); // Already deleted
        }

        // Create backup before deletion if enabled
        if self.options.file_operation_config.create_backup {
            self.create_backup(manifest_path).await?;
        }

        async_fs::remove_file(manifest_path)
            .await
            .map_err(|e| ManifestStoreError::WriteFailed(e.to_string()))?;

        Ok(())
    }

    /// Filter manifest by groups
    pub fn filter_manifest_by_groups(
        &self,
        manifest: &Manifest,
        group_names: &[String],
    ) -> Result<Manifest, ManifestStoreError> {
        self.manifest_service
            .filter_by_groups(manifest, group_names)
            .map_err(ManifestStoreError::ManifestServiceError)
    }

    /// List groups in manifest
    pub fn list_manifest_groups(&self, manifest: &Manifest) -> Vec<String> {
        self.manifest_service.list_groups(manifest)
    }

    // Private helper methods

    /// Execute a copy operation
    async fn execute_copy_operation(
        &self,
        copy_op: &FileCopy,
        repo: &ManifestRepo,
        workspace_root: &Path,
    ) -> FileOperationResult {
        let source_path = workspace_root.join(&repo.dest).join(&copy_op.file);
        let dest_path = workspace_root.join(&copy_op.dest);

        let mut result = FileOperationResult {
            source: source_path.clone(),
            destination: dest_path.clone(),
            operation_type: "copy".to_string(),
            success: false,
            error: None,
            backup_created: false,
        };

        // Validate paths if enabled
        if self.options.file_operation_config.validate_paths {
            if let Err(e) = self.validate_copy_paths(&source_path, &dest_path) {
                result.error = Some(e.to_string());
                return result;
            }
        }

        // Create backup if enabled and destination exists
        if self.options.file_operation_config.create_backup && dest_path.exists() {
            if let Err(e) = self.create_backup(&dest_path).await {
                result.error = Some(format!("Backup failed: {}", e));
                return result;
            }
            result.backup_created = true;
        }

        // Create parent directory if needed
        if self.options.file_operation_config.create_parent_dirs {
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = async_fs::create_dir_all(parent).await {
                    result.error = Some(format!("Failed to create parent directory: {}", e));
                    return result;
                }
            }
        }

        // Check if destination exists and overwrite is disabled
        if dest_path.exists() && !self.options.file_operation_config.overwrite_existing {
            result.error = Some("Destination exists and overwrite is disabled".to_string());
            return result;
        }

        // Perform the copy operation
        match async_fs::copy(&source_path, &dest_path).await {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result.error = Some(e.to_string());
            }
        }

        result
    }

    /// Execute a symlink operation
    async fn execute_symlink_operation(
        &self,
        symlink_op: &FileSymlink,
        _repo: &ManifestRepo,
        workspace_root: &Path,
    ) -> FileOperationResult {
        let source_path = workspace_root.join(&symlink_op.source);
        let target_path = Path::new(&symlink_op.target);

        let mut result = FileOperationResult {
            source: source_path.clone(),
            destination: target_path.to_path_buf(),
            operation_type: "symlink".to_string(),
            success: false,
            error: None,
            backup_created: false,
        };

        // Validate paths if enabled
        if self.options.file_operation_config.validate_paths {
            if let Err(e) = self.validate_symlink_paths(&source_path, target_path) {
                result.error = Some(e.to_string());
                return result;
            }
        }

        // Create backup if enabled and source exists
        if self.options.file_operation_config.create_backup && source_path.exists() {
            if let Err(e) = self.create_backup(&source_path).await {
                result.error = Some(format!("Backup failed: {}", e));
                return result;
            }
            result.backup_created = true;
        }

        // Create parent directory if needed
        if self.options.file_operation_config.create_parent_dirs {
            if let Some(parent) = source_path.parent() {
                if let Err(e) = async_fs::create_dir_all(parent).await {
                    result.error = Some(format!("Failed to create parent directory: {}", e));
                    return result;
                }
            }
        }

        // Check if source exists and overwrite is disabled
        if source_path.exists() && !self.options.file_operation_config.overwrite_existing {
            result.error = Some("Source exists and overwrite is disabled".to_string());
            return result;
        }

        // Remove existing symlink if it exists
        if source_path.is_symlink() {
            if let Err(e) = async_fs::remove_file(&source_path).await {
                result.error = Some(format!("Failed to remove existing symlink: {}", e));
                return result;
            }
        }

        // Perform the symlink operation
        #[cfg(unix)]
        let symlink_result = unix_fs::symlink(target_path, &source_path);

        #[cfg(windows)]
        let symlink_result = {
            use std::os::windows::fs::symlink_file;
            symlink_file(target_path, &source_path)
        };

        match symlink_result {
            Ok(_) => {
                result.success = true;
            }
            Err(e) => {
                result.error = Some(e.to_string());
            }
        }

        result
    }

    /// Validate file operations in manifest
    fn validate_file_operations(
        &self,
        manifest: &Manifest,
        base_path: &Path,
    ) -> Result<(), ManifestStoreError> {
        let base_dir = base_path.parent().unwrap_or(Path::new("."));

        for repo in &manifest.repos {
            // Validate copy operations
            if let Some(copy_operations) = &repo.copy {
                for copy_op in copy_operations {
                    let source_path = base_dir.join(&repo.dest).join(&copy_op.file);
                    let dest_path = base_dir.join(&copy_op.dest);
                    self.validate_copy_paths(&source_path, &dest_path)?;
                }
            }

            // Validate symlink operations
            if let Some(symlink_operations) = &repo.symlink {
                for symlink_op in symlink_operations {
                    let source_path = base_dir.join(&symlink_op.source);
                    let target_path = Path::new(&symlink_op.target);
                    self.validate_symlink_paths(&source_path, target_path)?;
                }
            }
        }

        Ok(())
    }

    /// Validate copy operation paths
    fn validate_copy_paths(
        &self,
        source_path: &Path,
        dest_path: &Path,
    ) -> Result<(), ManifestStoreError> {
        // Check for path traversal attacks
        if source_path.to_string_lossy().contains("..") {
            return Err(ManifestStoreError::PathValidationFailed(format!(
                "Source path contains path traversal: {}",
                source_path.display()
            )));
        }

        if dest_path.to_string_lossy().contains("..") {
            return Err(ManifestStoreError::PathValidationFailed(format!(
                "Destination path contains path traversal: {}",
                dest_path.display()
            )));
        }

        // Check that source and destination are not the same
        if source_path == dest_path {
            return Err(ManifestStoreError::PathValidationFailed(
                "Source and destination paths are the same".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate symlink operation paths
    fn validate_symlink_paths(
        &self,
        source_path: &Path,
        target_path: &Path,
    ) -> Result<(), ManifestStoreError> {
        // Check for path traversal attacks
        if source_path.to_string_lossy().contains("..") {
            return Err(ManifestStoreError::PathValidationFailed(format!(
                "Source path contains path traversal: {}",
                source_path.display()
            )));
        }

        // Allow relative target paths for symlinks, but validate them
        if target_path.is_absolute() && target_path.to_string_lossy().contains("..") {
            return Err(ManifestStoreError::PathValidationFailed(format!(
                "Target path contains path traversal: {}",
                target_path.display()
            )));
        }

        Ok(())
    }

    /// Create backup of a file
    async fn create_backup<P: AsRef<Path>>(&self, file_path: P) -> Result<(), ManifestStoreError> {
        let file_path = file_path.as_ref();

        if !file_path.exists() {
            return Ok(()); // Nothing to backup
        }

        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = file_path.with_file_name(format!(
            "{}.bak_{}",
            file_path.file_name().unwrap().to_string_lossy(),
            timestamp
        ));

        // Copy file to backup location
        async_fs::copy(file_path, &backup_path)
            .await
            .map_err(|e| ManifestStoreError::BackupFailed(e.to_string()))?;

        // Clean up old backups
        self.cleanup_old_backups(file_path).await?;

        Ok(())
    }

    /// Clean up old backup files
    async fn cleanup_old_backups<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(), ManifestStoreError> {
        let file_path = file_path.as_ref();
        let parent = file_path.parent().unwrap_or(Path::new("."));
        let base_name = file_path.file_name().ok_or_else(|| {
            ManifestStoreError::InvalidManifestPath(file_path.display().to_string())
        })?;

        let mut backups = Vec::new();

        if let Ok(mut entries) = async_fs::read_dir(parent).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(&format!("{}.bak_", base_name.to_string_lossy())) {
                        backups.push(path);
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_time = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        // Keep only the maximum number of backups
        if backups.len() > self.options.file_operation_config.max_backups {
            for backup in backups
                .iter()
                .skip(self.options.file_operation_config.max_backups)
            {
                let _ = async_fs::remove_file(backup).await; // Ignore errors for cleanup
            }
        }

        Ok(())
    }
}

impl Default for ManifestStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::manifest::Group;
    use std::collections::HashMap;
    use tempfile::TempDir;

    async fn create_test_manifest() -> Manifest {
        let mut repos = vec![
            ManifestRepo::new("https://github.com/example/repo1.git", "repo1"),
            ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
        ];

        // Add copy and symlink operations to the first repo
        repos[0].copy = Some(vec![FileCopy {
            file: "config.yml".to_string(),
            dest: "shared/config.yml".to_string(),
        }]);

        repos[0].symlink = Some(vec![FileSymlink {
            source: "bin/tool".to_string(),
            target: "../repo1/bin/tool".to_string(),
        }]);

        let mut groups = HashMap::new();
        groups.insert(
            "core".to_string(),
            Group::new(vec!["repo1".to_string()]).with_description("Core repositories"),
        );

        Manifest::new(repos)
            .with_groups(groups)
            .with_default_branch("main")
    }

    #[tokio::test]
    async fn test_manifest_store_creation() {
        let store = ManifestStore::new();
        assert!(store.options.process_copy_operations);
        assert!(store.options.process_symlink_operations);
        assert!(store.options.validate_manifest);
    }

    #[tokio::test]
    async fn test_write_and_read_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.yml");
        let mut store = ManifestStore::new();
        let original_manifest = create_test_manifest().await;

        // Write manifest
        let result = store
            .write_manifest(&manifest_path, &original_manifest)
            .await;
        assert!(result.is_ok());
        assert!(manifest_path.exists());

        // Read manifest back
        let read_result = store.read_manifest(&manifest_path).await;
        assert!(read_result.is_ok());

        let processed_manifest = read_result.unwrap();
        assert_eq!(processed_manifest.manifest.repos.len(), 2);
        assert_eq!(processed_manifest.manifest.repos[0].dest, "repo1");
        assert_eq!(processed_manifest.manifest.repos[1].dest, "repo2");
    }

    #[tokio::test]
    async fn test_read_nonexistent_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("nonexistent.yml");
        let mut store = ManifestStore::new();

        let result = store.read_manifest(&manifest_path).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestStoreError::ManifestFileNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_manifest_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.yml");
        let mut store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        // Write manifest
        store
            .write_manifest(&manifest_path, &manifest)
            .await
            .unwrap();

        // Get metadata
        let metadata = store.get_manifest_metadata(&manifest_path).await.unwrap();
        assert!(metadata.exists);
        assert!(metadata.size > 0);
        assert_eq!(metadata.repo_count, 2);
        assert_eq!(metadata.group_count, 1);
        assert_eq!(metadata.path, manifest_path);
    }

    #[tokio::test]
    async fn test_file_operations_processing() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();
        let store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        // Create source files for testing
        let repo1_dir = workspace_root.join("repo1");
        async_fs::create_dir_all(&repo1_dir).await.unwrap();
        async_fs::write(repo1_dir.join("config.yml"), "test config")
            .await
            .unwrap();

        let bin_dir = repo1_dir.join("bin");
        async_fs::create_dir_all(&bin_dir).await.unwrap();
        async_fs::write(bin_dir.join("tool"), "#!/bin/bash\necho 'test'")
            .await
            .unwrap();

        // Process copy operations
        let copy_results = store
            .process_copy_operations(&manifest, workspace_root)
            .await
            .unwrap();
        assert_eq!(copy_results.len(), 1);
        assert!(copy_results[0].success);
        assert_eq!(copy_results[0].operation_type, "copy");

        // Check that file was copied
        assert!(workspace_root.join("shared/config.yml").exists());

        // Process symlink operations
        let symlink_results = store
            .process_symlink_operations(&manifest, workspace_root)
            .await
            .unwrap();
        assert_eq!(symlink_results.len(), 1);
        assert!(symlink_results[0].success);
        assert_eq!(symlink_results[0].operation_type, "symlink");

        // Check that symlink was created
        let symlink_path = workspace_root.join("bin/tool");
        assert!(symlink_path.exists());
        assert!(symlink_path.is_symlink());
    }

    #[tokio::test]
    async fn test_process_all_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path();
        let store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        // Create source files for testing
        let repo1_dir = workspace_root.join("repo1");
        async_fs::create_dir_all(&repo1_dir).await.unwrap();
        async_fs::write(repo1_dir.join("config.yml"), "test config")
            .await
            .unwrap();

        let bin_dir = repo1_dir.join("bin");
        async_fs::create_dir_all(&bin_dir).await.unwrap();
        async_fs::write(bin_dir.join("tool"), "#!/bin/bash\necho 'test'")
            .await
            .unwrap();

        // Process all file operations
        let results = store
            .process_all_file_operations(&manifest, workspace_root)
            .await
            .unwrap();
        assert_eq!(results.len(), 2); // 1 copy + 1 symlink

        // Check that both operations succeeded
        assert!(results.iter().all(|r| r.success));

        // Verify files exist
        assert!(workspace_root.join("shared/config.yml").exists());
        assert!(workspace_root.join("bin/tool").is_symlink());
    }

    #[tokio::test]
    async fn test_filter_manifest_by_groups() {
        let store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        let filtered = store
            .filter_manifest_by_groups(&manifest, &["core".to_string()])
            .unwrap();

        assert_eq!(filtered.repos.len(), 1);
        assert_eq!(filtered.repos[0].dest, "repo1");

        let groups = filtered.groups.unwrap();
        assert!(groups.contains_key("core"));
    }

    #[tokio::test]
    async fn test_list_manifest_groups() {
        let store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        let groups = store.list_manifest_groups(&manifest);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], "core");
    }

    #[tokio::test]
    async fn test_delete_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.yml");
        let mut store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        // Write manifest
        store
            .write_manifest(&manifest_path, &manifest)
            .await
            .unwrap();
        assert!(manifest_path.exists());

        // Delete manifest
        store.delete_manifest(&manifest_path).await.unwrap();
        assert!(!manifest_path.exists());
    }

    #[tokio::test]
    async fn test_backup_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.yml");
        let mut store = ManifestStore::new();
        let manifest = create_test_manifest().await;

        // Write initial manifest
        store
            .write_manifest(&manifest_path, &manifest)
            .await
            .unwrap();

        // Modify and write again (should create backup)
        let mut modified_manifest = manifest.clone();
        modified_manifest.default_branch = Some("develop".to_string());
        store
            .write_manifest(&manifest_path, &modified_manifest)
            .await
            .unwrap();

        // Check that backup was created
        let parent_dir = temp_dir.path();
        let mut backup_found = false;
        if let Ok(mut entries) = async_fs::read_dir(parent_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name();
                if name.to_string_lossy().contains("manifest.yml.bak_") {
                    backup_found = true;
                    break;
                }
            }
        }
        assert!(backup_found);
    }

    #[tokio::test]
    async fn test_path_validation() {
        let store = ManifestStore::new();

        // Test path traversal validation
        let bad_source = Path::new("../etc/passwd");
        let dest = Path::new("config.txt");
        let result = store.validate_copy_paths(bad_source, dest);
        assert!(result.is_err());

        // Test same path validation
        let same_path = Path::new("config.txt");
        let result = store.validate_copy_paths(same_path, same_path);
        assert!(result.is_err());

        // Test valid paths
        let source = Path::new("src/config.txt");
        let dest = Path::new("dest/config.txt");
        let result = store.validate_copy_paths(source, dest);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_options() {
        let mut options = ManifestProcessingOptions::default();
        options.process_copy_operations = false;
        options.file_operation_config.overwrite_existing = true;

        let store = ManifestStore::with_options(options);

        assert!(!store.options.process_copy_operations);
        assert!(store.options.file_operation_config.overwrite_existing);
    }
}
