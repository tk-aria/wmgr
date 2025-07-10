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
                Err(anyhow::anyhow!("Workspace not initialized at: {}\nRun 'tsrc init' first", path))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to synchronize repositories: {}", e))
            }
        }
    }

    /// Load workspace from the current directory
    async fn load_workspace(&self) -> Result<Workspace> {
        let current_dir = env::current_dir()?;
        
        // Check if .tsrc directory exists
        let tsrc_dir = current_dir.join(".tsrc");
        if !tsrc_dir.exists() {
            return Err(anyhow::anyhow!("Workspace not initialized. Run 'tsrc init' first."));
        }
        
        // Load configuration
        let config_file = tsrc_dir.join("config.yml");
        if !config_file.exists() {
            return Err(anyhow::anyhow!("Workspace configuration not found. Run 'tsrc init' first."));
        }
        
        // For now, create a basic workspace - in a real implementation, this would load from config
        let workspace_config = crate::domain::entities::workspace::WorkspaceConfig::new(
            "https://example.com/manifest.git", // This would be loaded from config
            "main"
        );
        
        let workspace = Workspace::new(current_dir, workspace_config);
        Ok(workspace)
    }
}