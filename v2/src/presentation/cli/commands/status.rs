use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::application::use_cases::status_check::{
    StatusCheckUseCase, StatusCheckConfig, StatusCheckError, RepositoryState
};
use crate::domain::entities::workspace::Workspace;

/// Handler for the status command
pub struct StatusCommand {
    pub groups: Vec<String>,
    pub show_branch: bool,
    pub compact: bool,
    pub verbose: bool,
}

impl StatusCommand {
    pub fn new(
        groups: Vec<String>,
        show_branch: bool,
        compact: bool,
        verbose: bool,
    ) -> Self {
        Self {
            groups,
            show_branch,
            compact,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        // Prepare groups list
        let groups_list = if self.groups.is_empty() {
            None
        } else {
            Some(self.groups.clone())
        };
        
        // Create configuration
        let config = StatusCheckConfig {
            groups: groups_list,
            show_branch: self.show_branch,
            compact: self.compact,
            verbose: self.verbose,
        };
        
        // Execute the use case
        let use_case = StatusCheckUseCase::new(config);
        
        match use_case.execute(&workspace).await {
            Ok(status) => {
                if self.compact {
                    self.print_compact_status(&status);
                } else {
                    self.print_detailed_status(&status);
                }
                Ok(())
            }
            Err(StatusCheckError::WorkspaceNotInitialized(path)) => {
                Err(anyhow::anyhow!("Workspace not initialized at: {}\nRun 'tsrc init' first", path))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to check status: {}", e))
            }
        }
    }

    fn print_compact_status(&self, status: &crate::application::use_cases::status_check::StatusResult) {
        for repo_status in &status.repositories {
            let state_char = match repo_status.state {
                RepositoryState::Clean => "✓".green(),
                RepositoryState::Dirty => "M".yellow(),
                RepositoryState::Missing => "?".red(),
                RepositoryState::WrongBranch => "B".cyan(),
                RepositoryState::OutOfSync => "S".magenta(),
                RepositoryState::Error => "E".red(),
            };
            println!("{} {}", state_char, repo_status.dest);
        }
    }
    
    fn print_detailed_status(&self, status: &crate::application::use_cases::status_check::StatusResult) {
        for repo_status in &status.repositories {
            let state_text = match repo_status.state {
                RepositoryState::Clean => "clean".green(),
                RepositoryState::Dirty => "dirty".yellow(),
                RepositoryState::Missing => "missing".red(),
                RepositoryState::WrongBranch => "wrong branch".cyan(),
                RepositoryState::OutOfSync => "out of sync".magenta(),
                RepositoryState::Error => "error".red(),
            };
            
            print!("{}: {}", repo_status.dest.bold(), state_text);
            
            if self.show_branch {
                if let Some(ref current_branch) = repo_status.current_branch {
                    print!(" ({})", current_branch.blue());
                }
            }
            
            if repo_status.state == RepositoryState::Dirty {
                let mut changes = Vec::new();
                if repo_status.modified_files > 0 {
                    changes.push(format!("{}M", repo_status.modified_files));
                }
                if repo_status.staged_files > 0 {
                    changes.push(format!("{}S", repo_status.staged_files));
                }
                if repo_status.untracked_files > 0 {
                    changes.push(format!("{}U", repo_status.untracked_files));
                }
                if !changes.is_empty() {
                    print!(" [{}]", changes.join(" "));
                }
            }
            
            println!();
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