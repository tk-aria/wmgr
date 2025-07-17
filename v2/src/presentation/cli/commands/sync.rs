use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::application::use_cases::sync_repositories::{
    SyncRepositoriesUseCase, SyncRepositoriesConfig, SyncRepositoriesError
};
use crate::domain::entities::workspace::Workspace;

/// Handler for the sync command
pub struct SyncCommand {
    pub groups: Vec<String>,
    pub force: bool,
    pub no_correct_branch: bool,
    pub jobs: Option<usize>,
    pub verbose: bool,
}

impl SyncCommand {
    pub fn new(
        groups: Vec<String>,
        force: bool,
        no_correct_branch: bool,
        jobs: Option<usize>,
        verbose: bool,
    ) -> Self {
        Self {
            groups,
            force,
            no_correct_branch,
            jobs,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Load workspace
        let mut workspace = self.load_workspace().await?;
        
        // Prepare groups list
        let groups_list = if self.groups.is_empty() {
            None
        } else {
            Some(self.groups.clone())
        };
        
        // Create configuration
        let config = SyncRepositoriesConfig {
            groups: groups_list,
            force: self.force,
            no_correct_branch: self.no_correct_branch,
            parallel_jobs: self.jobs,
            verbose: self.verbose,
        };
        
        // Execute the use case
        let use_case = SyncRepositoriesUseCase::new(config);
        
        println!("{} Synchronizing repositories...", "::".blue().bold());
        
        match use_case.execute(&mut workspace).await {
            Ok(result) => {
                println!("{} Synchronization completed!", "✓".green().bold());
                if self.verbose {
                    println!("  Repositories synced: {}", result.synced_count);
                    println!("  New repositories cloned: {}", result.cloned_count);
                    println!("  Repositories updated: {}", result.updated_count);
                    if result.skipped_count > 0 {
                        println!("  Repositories skipped: {}", result.skipped_count);
                    }
                }
                
                // Show any errors
                if !result.errors.is_empty() {
                    println!("{} Some errors occurred:", "⚠".yellow().bold());
                    for error in result.errors {
                        println!("  {}", error.red());
                    }
                }
                
                Ok(())
            }
            Err(SyncRepositoriesError::WorkspaceNotInitialized(path)) => {
                Err(anyhow::anyhow!("Workspace not initialized at: {}\nManifest file not found", path))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to synchronize repositories: {}", e))
            }
        }
    }

    /// Load workspace from the current directory
    async fn load_workspace(&self) -> Result<Workspace> {
        let current_dir = env::current_dir()?;
        
        // Try to find manifest.yml in current directory first, then .wmgr/
        let manifest_file = if current_dir.join("manifest.yml").exists() {
            current_dir.join("manifest.yml")
        } else if current_dir.join(".wmgr").join("manifest.yml").exists() {
            current_dir.join(".wmgr").join("manifest.yml")
        } else {
            return Err(anyhow::anyhow!("Manifest file not found at: {} or {}", 
                current_dir.join("manifest.yml").display(),
                current_dir.join(".wmgr").join("manifest.yml").display()));
        };
        
        // Load manifest file
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        use crate::domain::entities::workspace::{WorkspaceStatus, WorkspaceConfig};
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store.read_manifest(&manifest_file).await
            .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;
        
        // Create a simple workspace configuration
        let workspace_config = WorkspaceConfig::new(
            &manifest_file.display().to_string(),
            "main"
        );
        
        let workspace = Workspace::new(current_dir, workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);
        Ok(workspace)
    }
}