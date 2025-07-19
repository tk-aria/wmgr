use anyhow::Result;
use colored::Colorize;
use std::env;

use crate::application::use_cases::sync_repositories::{
    SyncRepositoriesConfig, SyncRepositoriesError, SyncRepositoriesUseCase,
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
            Err(SyncRepositoriesError::WorkspaceNotInitialized(path)) => Err(anyhow::anyhow!(
                "Workspace not initialized at: {}\nManifest file not found",
                path
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to synchronize repositories: {}", e)),
        }
    }

    /// Load workspace from the current directory
    async fn load_workspace(&self) -> Result<Workspace> {
        let current_dir = env::current_dir()?;

        let workspace = Workspace::new(current_dir.clone(), WorkspaceConfig::default_local());

        // Use workspace.manifest_file_path() to support wmgr.yml, wmgr.yaml, manifest.yml, manifest.yaml
        let manifest_file = workspace.manifest_file_path();

        if !manifest_file.exists() {
            return Err(anyhow::anyhow!("Manifest file not found. Tried wmgr.yml, wmgr.yaml, manifest.yml, manifest.yaml in current directory and .wmgr/ subdirectory"));
        }

        // Load manifest file
        use crate::domain::entities::workspace::{WorkspaceConfig, WorkspaceStatus};
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store
            .read_manifest(&manifest_file)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

        // Create a simple workspace configuration
        let workspace_config = WorkspaceConfig::new(&manifest_file.display().to_string(), "main");

        let workspace = Workspace::new(current_dir, workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);
        Ok(workspace)
    }
}
