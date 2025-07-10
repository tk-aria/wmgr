use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::application::use_cases::foreach_command::{
    ForeachCommandUseCase, ForeachCommandConfig, ForeachCommandError
};
use crate::domain::entities::workspace::Workspace;

/// Handler for the foreach command
pub struct ForeachCommand {
    pub command: String,
    pub args: Vec<String>,
    pub groups: Vec<String>,
    pub parallel: bool,
    pub jobs: Option<usize>,
    pub continue_on_error: bool,
    pub verbose: bool,
}

impl ForeachCommand {
    pub fn new(
        command: String,
        args: Vec<String>,
        groups: Vec<String>,
        parallel: bool,
        jobs: Option<usize>,
        continue_on_error: bool,
        verbose: bool,
    ) -> Self {
        Self {
            command,
            args,
            groups,
            parallel,
            jobs,
            continue_on_error,
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
        
        // Build the full command
        let full_command = if self.args.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, self.args.join(" "))
        };
        
        // Create configuration
        let config = ForeachCommandConfig {
            command: full_command,
            groups: groups_list,
            parallel: self.parallel,
            max_parallel: self.jobs,
            continue_on_error: self.continue_on_error,
            verbose: self.verbose,
            ..Default::default()
        };
        
        // Execute the use case
        let use_case = ForeachCommandUseCase::new(config);
        
        println!("{} Running command: {}", "::".blue().bold(), self.command);
        
        match use_case.execute(&workspace).await {
            Ok(result) => {
                println!("{} Command execution completed!", "✓".green().bold());
                if self.verbose {
                    println!("  Successful executions: {}", result.success_count);
                    println!("  Failed executions: {}", result.failure_count);
                    println!("  Skipped executions: {}", result.skipped_count);
                    if result.was_parallel {
                        println!("  Executed in parallel");
                    }
                }
                
                // Show any errors
                let failed_results = result.failed_results();
                if !failed_results.is_empty() {
                    println!("{} Some commands failed:", "⚠".yellow().bold());
                    for result in failed_results {
                        let default_error = "Unknown error".to_string();
                        let error_msg = result.error_message.as_ref().unwrap_or(&default_error);
                        println!("  {}: {}", result.dest.bold(), error_msg.red());
                    }
                }
                
                Ok(())
            }
            Err(ForeachCommandError::WorkspaceNotInitialized(path)) => {
                Err(anyhow::anyhow!("Workspace not initialized at: {}\nRun 'tsrc init' first", path))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to execute command: {}", e))
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