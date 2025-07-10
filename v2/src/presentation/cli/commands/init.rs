use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::application::use_cases::init_workspace::{
    InitWorkspaceUseCase, InitWorkspaceConfig, InitWorkspaceError
};
use crate::domain::value_objects::{git_url::GitUrl, file_path::FilePath};

/// Handler for the init command
pub struct InitCommand {
    pub manifest_path: String,
    pub groups: Vec<String>,
    pub force: bool,
    pub verbose: bool,
}

impl InitCommand {
    pub fn new(
        manifest_path: String,
        groups: Vec<String>,
        force: bool,
        verbose: bool,
    ) -> Self {
        Self {
            manifest_path,
            groups,
            force,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        use std::path::Path;
        use crate::application::services::manifest_service::{ManifestService, ManifestProcessingOptions};
        use crate::infrastructure::filesystem::{config_store::ConfigStore, manifest_store::ManifestStore};
        use crate::domain::entities::workspace::{Workspace, WorkspaceConfig};
        
        let current_dir = env::current_dir()?;
        
        // Validate manifest file path
        let manifest_file = Path::new(&self.manifest_path);
        if !manifest_file.exists() {
            return Err(anyhow::anyhow!("Manifest file not found: {}", self.manifest_path));
        }
        
        // Check if workspace already exists
        let tsrc_dir = current_dir.join(".tsrc");
        if tsrc_dir.exists() && !self.force {
            return Err(anyhow::anyhow!("Workspace already exists at: {}\nUse --force to overwrite", current_dir.display()));
        }
        
        println!("{} Initializing workspace from local manifest...", "::".blue().bold());
        
        // Parse the manifest file
        let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());
        let processed_manifest = manifest_service.parse_from_file(manifest_file).await
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest file: {}", e))?;
        
        if self.verbose {
            println!("  {} Parsed manifest successfully", "✓".green());
            println!("  {} Found {} repositories", "->" .blue(), processed_manifest.manifest.repos.len());
        }
        
        // Filter by groups if specified
        let filtered_manifest = if !self.groups.is_empty() {
            manifest_service.filter_by_groups(&processed_manifest.manifest, &self.groups)
                .map_err(|e| anyhow::anyhow!("Failed to filter by groups: {}", e))?
        } else {
            processed_manifest.manifest.clone()
        };
        
        // Create .tsrc directory
        std::fs::create_dir_all(&tsrc_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create .tsrc directory: {}", e))?;
        
        // Create workspace configuration
        let workspace_config = WorkspaceConfig::new(
            &self.manifest_path, // Store the local path instead of URL
            "main" // Default branch, not used for local manifests
        );
        
        // Save configuration
        let config_store = ConfigStore::new();
        let config_file = tsrc_dir.join("config.yml");
        config_store.write_workspace_config(&config_file, &workspace_config)
            .map_err(|e| anyhow::anyhow!("Failed to save workspace config: {}", e))?;
        
        // Save manifest
        let manifest_store = ManifestStore::new();
        let manifest_file_path = tsrc_dir.join("manifest.yml");
        manifest_store.write_manifest(&manifest_file_path, &filtered_manifest).await
            .map_err(|e| anyhow::anyhow!("Failed to save manifest: {}", e))?;
        
        // Create workspace object
        let workspace = Workspace::new(current_dir, workspace_config);
        
        println!("{} Workspace initialized successfully!", "✓".green().bold());
        if self.verbose {
            println!("  Manifest file: {}", self.manifest_path);
            println!("  Workspace path: {}", workspace.root_path.display());
            if !self.groups.is_empty() {
                println!("  Groups: {}", self.groups.join(", "));
            }
            println!("  Repositories: {}", filtered_manifest.repos.len());
        }
        
        // Suggest next steps
        println!();
        println!("{} Next steps:", "::".blue().bold());
        println!("  {} Run 'tsrc sync' to clone repositories", "1.".bold());
        
        Ok(())
    }
}