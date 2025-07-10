use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::application::use_cases::init_workspace::{
    InitWorkspaceUseCase, InitWorkspaceConfig, InitWorkspaceError
};
use crate::domain::value_objects::{git_url::GitUrl, file_path::FilePath};

/// Handler for the init command
pub struct InitCommand {
    pub manifest_url: String,
    pub branch: Option<String>,
    pub groups: Vec<String>,
    pub shallow: bool,
    pub force: bool,
    pub verbose: bool,
}

impl InitCommand {
    pub fn new(
        manifest_url: String,
        branch: Option<String>,
        groups: Vec<String>,
        shallow: bool,
        force: bool,
        verbose: bool,
    ) -> Self {
        Self {
            manifest_url,
            branch,
            groups,
            shallow,
            force,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        let current_dir = env::current_dir()?;
        
        // Parse and validate the manifest URL
        let git_url = GitUrl::new(&self.manifest_url)
            .map_err(|e| anyhow::anyhow!("Invalid manifest URL: {}", e))?;
        
        // Create workspace path
        let workspace_path = FilePath::new_absolute(current_dir.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Invalid workspace path: {}", e))?;
        
        // Prepare groups list
        let groups_list = if self.groups.is_empty() {
            None
        } else {
            Some(self.groups.clone())
        };
        
        // Create configuration
        let config = InitWorkspaceConfig {
            manifest_url: git_url,
            workspace_path,
            branch: self.branch.clone(),
            groups: groups_list,
            shallow: self.shallow,
            force: self.force,
        };
        
        // Execute the use case
        let use_case = InitWorkspaceUseCase::new(config);
        
        println!("{} Initializing workspace...", "::".blue().bold());
        
        match use_case.execute().await {
            Ok(workspace) => {
                println!("{} Workspace initialized successfully!", "✓".green().bold());
                if self.verbose {
                    println!("  Manifest URL: {}", self.manifest_url);
                    println!("  Workspace path: {}", workspace.root_path.display());
                    if let Some(ref branch) = self.branch {
                        println!("  Branch: {}", branch);
                    }
                    if !self.groups.is_empty() {
                        println!("  Groups: {}", self.groups.join(", "));
                    }
                    if self.shallow {
                        println!("  Shallow: enabled");
                    }
                }
                Ok(())
            }
            Err(InitWorkspaceError::WorkspaceAlreadyExists(path)) => {
                Err(anyhow::anyhow!("Workspace already exists at: {}\nUse --force to overwrite", path))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to initialize workspace: {}", e))
            }
        }
    }
}