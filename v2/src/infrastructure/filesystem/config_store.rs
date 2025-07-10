use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};
use crate::domain::entities::workspace::WorkspaceConfig;

/// Configuration store related errors
#[derive(Debug, Error)]
pub enum ConfigStoreError {
    #[error("Configuration file not found at path: {0}")]
    ConfigFileNotFound(String),
    
    #[error("Invalid configuration file path: {0}")]
    InvalidConfigPath(String),
    
    #[error("Configuration file read failed: {0}")]
    ReadFailed(String),
    
    #[error("Configuration file write failed: {0}")]
    WriteFailed(String),
    
    #[error("YAML parsing failed: {0}")]
    YamlParsingFailed(String),
    
    #[error("YAML serialization failed: {0}")]
    YamlSerializationFailed(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Configuration directory creation failed: {0}")]
    DirectoryCreationFailed(String),
    
    #[error("Configuration backup failed: {0}")]
    BackupFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// Configuration file metadata
#[derive(Debug, Clone)]
pub struct ConfigMetadata {
    /// File path
    pub path: PathBuf,
    
    /// Last modified time
    pub last_modified: std::time::SystemTime,
    
    /// File size in bytes
    pub size: u64,
    
    /// Whether the file exists
    pub exists: bool,
}

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Whether to create backup before write
    pub create_backup: bool,
    
    /// Maximum number of backup files to keep
    pub max_backups: usize,
    
    /// Backup file suffix pattern
    pub backup_suffix: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            create_backup: true,
            max_backups: 5,
            backup_suffix: ".bak".to_string(),
        }
    }
}

/// Schema validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to validate configuration on read
    pub validate_on_read: bool,
    
    /// Whether to validate configuration before write
    pub validate_before_write: bool,
    
    /// Whether to perform strict validation
    pub strict_validation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_on_read: true,
            validate_before_write: true,
            strict_validation: false,
        }
    }
}

/// Extended workspace configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedWorkspaceConfig {
    #[validate(url)]
    pub manifest_url: String,
    
    #[validate(length(min = 1, max = 255))]
    pub manifest_branch: String,
    
    #[serde(default)]
    pub shallow_clones: bool,
    
    #[serde(default)]
    #[validate(length(min = 1))]
    pub repo_groups: Vec<String>,
    
    #[serde(default)]
    pub clone_all_repos: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 255))]
    pub singular_remote: Option<String>,
}

impl From<WorkspaceConfig> for ValidatedWorkspaceConfig {
    fn from(config: WorkspaceConfig) -> Self {
        Self {
            manifest_url: config.manifest_url,
            manifest_branch: config.manifest_branch,
            shallow_clones: config.shallow_clones,
            repo_groups: config.repo_groups,
            clone_all_repos: config.clone_all_repos,
            singular_remote: config.singular_remote,
        }
    }
}

impl From<ValidatedWorkspaceConfig> for WorkspaceConfig {
    fn from(config: ValidatedWorkspaceConfig) -> Self {
        Self {
            manifest_url: config.manifest_url,
            manifest_branch: config.manifest_branch,
            shallow_clones: config.shallow_clones,
            repo_groups: config.repo_groups,
            clone_all_repos: config.clone_all_repos,
            singular_remote: config.singular_remote,
        }
    }
}

/// Configuration store for managing YAML configuration files
pub struct ConfigStore {
    /// Backup configuration
    backup_config: BackupConfig,
    
    /// Validation configuration
    validation_config: ValidationConfig,
}

impl ConfigStore {
    /// Create a new configuration store with default settings
    pub fn new() -> Self {
        Self {
            backup_config: BackupConfig::default(),
            validation_config: ValidationConfig::default(),
        }
    }
    
    /// Create a new configuration store with custom settings
    pub fn with_config(
        backup_config: BackupConfig,
        validation_config: ValidationConfig,
    ) -> Self {
        Self {
            backup_config,
            validation_config,
        }
    }
    
    /// Read workspace configuration from YAML file
    pub fn read_workspace_config<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<WorkspaceConfig, ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        // Check if file exists
        if !config_path.exists() {
            return Err(ConfigStoreError::ConfigFileNotFound(
                config_path.display().to_string()
            ));
        }
        
        // Read file contents
        let contents = fs::read_to_string(config_path)
            .map_err(|e| ConfigStoreError::ReadFailed(e.to_string()))?;
        
        // Parse YAML
        let validated_config: ValidatedWorkspaceConfig = serde_yaml::from_str(&contents)
            .map_err(|e| ConfigStoreError::YamlParsingFailed(e.to_string()))?;
        
        // Validate if enabled
        if self.validation_config.validate_on_read {
            self.validate_workspace_config(&validated_config)?;
        }
        
        Ok(validated_config.into())
    }
    
    /// Write workspace configuration to YAML file
    pub fn write_workspace_config<P: AsRef<Path>>(
        &self,
        config_path: P,
        config: &WorkspaceConfig,
    ) -> Result<(), ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        // Convert to validated config
        let validated_config: ValidatedWorkspaceConfig = config.clone().into();
        
        // Validate before write if enabled
        if self.validation_config.validate_before_write {
            self.validate_workspace_config(&validated_config)?;
        }
        
        // Create backup if enabled and file exists
        if self.backup_config.create_backup && config_path.exists() {
            self.create_backup(config_path)?;
        }
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigStoreError::DirectoryCreationFailed(e.to_string()))?;
        }
        
        // Serialize to YAML
        let yaml_content = serde_yaml::to_string(&validated_config)
            .map_err(|e| ConfigStoreError::YamlSerializationFailed(e.to_string()))?;
        
        // Write to file
        fs::write(config_path, yaml_content)
            .map_err(|e| ConfigStoreError::WriteFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Read any configuration type from YAML file
    pub fn read_config<T, P>(&self, config_path: P) -> Result<T, ConfigStoreError>
    where
        T: for<'de> Deserialize<'de>,
        P: AsRef<Path>,
    {
        let config_path = config_path.as_ref();
        
        if !config_path.exists() {
            return Err(ConfigStoreError::ConfigFileNotFound(
                config_path.display().to_string()
            ));
        }
        
        let contents = fs::read_to_string(config_path)
            .map_err(|e| ConfigStoreError::ReadFailed(e.to_string()))?;
        
        serde_yaml::from_str(&contents)
            .map_err(|e| ConfigStoreError::YamlParsingFailed(e.to_string()))
    }
    
    /// Write any configuration type to YAML file
    pub fn write_config<T, P>(
        &self,
        config_path: P,
        config: &T,
    ) -> Result<(), ConfigStoreError>
    where
        T: Serialize,
        P: AsRef<Path>,
    {
        let config_path = config_path.as_ref();
        
        // Create backup if enabled and file exists
        if self.backup_config.create_backup && config_path.exists() {
            self.create_backup(config_path)?;
        }
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigStoreError::DirectoryCreationFailed(e.to_string()))?;
        }
        
        // Serialize to YAML
        let yaml_content = serde_yaml::to_string(config)
            .map_err(|e| ConfigStoreError::YamlSerializationFailed(e.to_string()))?;
        
        // Write to file
        fs::write(config_path, yaml_content)
            .map_err(|e| ConfigStoreError::WriteFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get configuration file metadata
    pub fn get_config_metadata<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<ConfigMetadata, ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        if !config_path.exists() {
            return Ok(ConfigMetadata {
                path: config_path.to_path_buf(),
                last_modified: std::time::SystemTime::UNIX_EPOCH,
                size: 0,
                exists: false,
            });
        }
        
        let metadata = fs::metadata(config_path)?;
        
        Ok(ConfigMetadata {
            path: config_path.to_path_buf(),
            last_modified: metadata.modified()?,
            size: metadata.len(),
            exists: true,
        })
    }
    
    /// Check if configuration file exists
    pub fn config_exists<P: AsRef<Path>>(&self, config_path: P) -> bool {
        config_path.as_ref().exists()
    }
    
    /// Delete configuration file
    pub fn delete_config<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<(), ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        if !config_path.exists() {
            return Ok(()); // Already deleted
        }
        
        // Create backup before deletion if enabled
        if self.backup_config.create_backup {
            self.create_backup(config_path)?;
        }
        
        fs::remove_file(config_path)
            .map_err(|e| ConfigStoreError::WriteFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Validate YAML schema without reading the full config
    pub fn validate_yaml_schema<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<(), ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        if !config_path.exists() {
            return Err(ConfigStoreError::ConfigFileNotFound(
                config_path.display().to_string()
            ));
        }
        
        let contents = fs::read_to_string(config_path)
            .map_err(|e| ConfigStoreError::ReadFailed(e.to_string()))?;
        
        // Try to parse as YAML to verify syntax
        let _: serde_yaml::Value = serde_yaml::from_str(&contents)
            .map_err(|e| ConfigStoreError::YamlParsingFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// List all backup files for a configuration
    pub fn list_backups<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<Vec<PathBuf>, ConfigStoreError> {
        let config_path = config_path.as_ref();
        let parent = config_path.parent().unwrap_or(Path::new("."));
        let base_name = config_path.file_name()
            .ok_or_else(|| ConfigStoreError::InvalidConfigPath(
                config_path.display().to_string()
            ))?;
        
        let mut backups = Vec::new();
        
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(&base_name.to_string_lossy().to_string()) 
                        && name_str.contains(&self.backup_config.backup_suffix) {
                        backups.push(path);
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });
        
        Ok(backups)
    }
    
    // Private helper methods
    
    /// Validate workspace configuration
    fn validate_workspace_config(
        &self,
        config: &ValidatedWorkspaceConfig,
    ) -> Result<(), ConfigStoreError> {
        config.validate()
            .map_err(|e| ConfigStoreError::ValidationFailed(format!("{:?}", e)))?;
        
        // Additional custom validation
        if self.validation_config.strict_validation {
            self.strict_validate_workspace_config(config)?;
        }
        
        Ok(())
    }
    
    /// Perform strict validation on workspace configuration
    fn strict_validate_workspace_config(
        &self,
        config: &ValidatedWorkspaceConfig,
    ) -> Result<(), ConfigStoreError> {
        // Validate manifest URL format
        if !config.manifest_url.starts_with("http") && !config.manifest_url.starts_with("git@") {
            return Err(ConfigStoreError::ValidationFailed(
                "Manifest URL must be a valid HTTP or SSH URL".to_string()
            ));
        }
        
        // Validate branch name format
        if config.manifest_branch.contains("..") || config.manifest_branch.starts_with('/') {
            return Err(ConfigStoreError::ValidationFailed(
                "Invalid branch name format".to_string()
            ));
        }
        
        // Validate repo groups are not empty strings
        for group in &config.repo_groups {
            if group.trim().is_empty() {
                return Err(ConfigStoreError::ValidationFailed(
                    "Repository group names cannot be empty".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Create backup of configuration file
    fn create_backup<P: AsRef<Path>>(&self, config_path: P) -> Result<(), ConfigStoreError> {
        let config_path = config_path.as_ref();
        
        // Generate backup filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = config_path.with_file_name(
            format!(
                "{}{}_{}", 
                config_path.file_name().unwrap().to_string_lossy(),
                self.backup_config.backup_suffix,
                timestamp
            )
        );
        
        // Copy file to backup location
        fs::copy(config_path, &backup_path)
            .map_err(|e| ConfigStoreError::BackupFailed(e.to_string()))?;
        
        // Clean up old backups
        self.cleanup_old_backups(config_path)?;
        
        Ok(())
    }
    
    /// Clean up old backup files
    fn cleanup_old_backups<P: AsRef<Path>>(&self, config_path: P) -> Result<(), ConfigStoreError> {
        let mut backups = self.list_backups(config_path)?;
        
        // Keep only the maximum number of backups
        if backups.len() > self.backup_config.max_backups {
            backups.truncate(self.backup_config.max_backups);
            
            // Remove older backups
            for backup in backups.iter().skip(self.backup_config.max_backups) {
                let _ = fs::remove_file(backup); // Ignore errors for cleanup
            }
        }
        
        Ok(())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    fn create_test_config() -> WorkspaceConfig {
        WorkspaceConfig::new("git@github.com:example/manifest.git", "main")
            .with_repo_groups(vec!["group1".to_string()])
            .with_shallow_clones(true)
    }
    
    #[test]
    fn test_config_store_creation() {
        let store = ConfigStore::new();
        assert!(store.backup_config.create_backup);
        assert!(store.validation_config.validate_on_read);
    }
    
    #[test]
    fn test_write_and_read_workspace_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        let store = ConfigStore::new();
        let original_config = create_test_config();
        
        // Write config
        let result = store.write_workspace_config(&config_path, &original_config);
        assert!(result.is_ok());
        assert!(config_path.exists());
        
        // Read config back
        let read_result = store.read_workspace_config(&config_path);
        assert!(read_result.is_ok());
        
        let read_config = read_result.unwrap();
        assert_eq!(read_config.manifest_url, original_config.manifest_url);
        assert_eq!(read_config.manifest_branch, original_config.manifest_branch);
        assert_eq!(read_config.shallow_clones, original_config.shallow_clones);
    }
    
    #[test]
    fn test_read_nonexistent_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.yml");
        let store = ConfigStore::new();
        
        let result = store.read_workspace_config(&config_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigStoreError::ConfigFileNotFound(_)));
    }
    
    #[test]
    fn test_config_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        let store = ConfigStore::new();
        let config = create_test_config();
        
        // Write config
        store.write_workspace_config(&config_path, &config).unwrap();
        
        // Get metadata
        let metadata = store.get_config_metadata(&config_path).unwrap();
        assert!(metadata.exists);
        assert!(metadata.size > 0);
        assert_eq!(metadata.path, config_path);
    }
    
    #[test]
    fn test_config_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        
        // Write invalid YAML manually
        fs::write(&config_path, "invalid: yaml: content: [").unwrap();
        
        let store = ConfigStore::new();
        let result = store.validate_yaml_schema(&config_path);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_backup_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        let store = ConfigStore::new();
        let config = create_test_config();
        
        // Write initial config
        store.write_workspace_config(&config_path, &config).unwrap();
        
        // Modify and write again (should create backup)
        let mut modified_config = config.clone();
        modified_config.manifest_branch = "develop".to_string();
        store.write_workspace_config(&config_path, &modified_config).unwrap();
        
        // Check that backup was created
        let backups = store.list_backups(&config_path).unwrap();
        assert!(!backups.is_empty());
    }
    
    #[test]
    fn test_delete_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        let store = ConfigStore::new();
        let config = create_test_config();
        
        // Write config
        store.write_workspace_config(&config_path, &config).unwrap();
        assert!(config_path.exists());
        
        // Delete config
        store.delete_config(&config_path).unwrap();
        assert!(!config_path.exists());
    }
    
    #[test]
    fn test_strict_validation() {
        let validation_config = ValidationConfig {
            validate_on_read: true,
            validate_before_write: true,
            strict_validation: true,
        };
        let store = ConfigStore::with_config(BackupConfig::default(), validation_config);
        
        // Test invalid URL
        let mut invalid_config = create_test_config();
        invalid_config.manifest_url = "invalid-url".to_string();
        
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yml");
        
        let result = store.write_workspace_config(&config_path, &invalid_config);
        assert!(result.is_err());
    }
}

// TODO: Add support for configuration file migration between versions
// TODO: Add support for configuration file encryption
// TODO: Add support for remote configuration storage
// TODO: Add configuration file change monitoring/watching
// TODO: Add support for configuration file templates
// TODO: Add support for environment variable substitution in configs
// TODO: Add configuration file diff/merge capabilities
// TODO: Add support for multiple configuration file formats (JSON, TOML)
// TODO: Add configuration file compression for large configs
// TODO: Add audit logging for configuration changes