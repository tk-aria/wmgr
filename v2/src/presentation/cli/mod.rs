pub mod commands;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::exit;
use std::env;

use crate::application::use_cases::{
    init_workspace::{InitWorkspaceUseCase, InitWorkspaceConfig, InitWorkspaceError},
    sync_repositories::{SyncRepositoriesUseCase, SyncRepositoriesConfig, SyncRepositoriesError},
    status_check::{StatusCheckUseCase, StatusCheckConfig, StatusCheckError},
    foreach_command::{ForeachCommandUseCase, ForeachCommandConfig, ForeachCommandError},
};

use crate::domain::entities::workspace::Workspace;

use crate::domain::value_objects::{
    git_url::GitUrl,
    file_path::FilePath,
};

/// tsrc - A tool for managing multiple git repositories
#[derive(Parser)]
#[command(name = "tsrc")]
#[command(about = "A tool for managing multiple git repositories")]
#[command(version)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Working directory (defaults to current directory)
    #[arg(short = 'C', long, global = true)]
    pub directory: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new workspace
    Init {
        /// Manifest repository URL
        manifest_url: String,
        
        /// Branch to use for the manifest repository
        #[arg(short, long)]
        branch: Option<String>,
        
        /// Groups to clone (if not specified, all groups will be cloned)
        #[arg(short, long)]
        group: Vec<String>,
        
        /// Use shallow clones
        #[arg(long)]
        shallow: bool,
        
        /// Force initialization even if workspace already exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Synchronize repositories
    Sync {
        /// Groups to sync (if not specified, all groups will be synced)
        #[arg(short, long)]
        group: Vec<String>,
        
        /// Force sync, ignoring local changes
        #[arg(short, long)]
        force: bool,
        
        /// Don't switch to correct branch
        #[arg(long)]
        no_correct_branch: bool,
        
        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
    },
    
    /// Show repository status
    Status {
        /// Show branch information
        #[arg(short, long)]
        branch: bool,
        
        /// Use compact output format
        #[arg(short, long)]
        compact: bool,
        
        /// Groups to check (if not specified, all groups will be checked)
        #[arg(short, long)]
        group: Vec<String>,
    },
    
    /// Run a command in each repository
    Foreach {
        /// Command to run
        command: String,
        
        /// Arguments for the command
        args: Vec<String>,
        
        /// Groups to run command in (if not specified, all groups will be used)
        #[arg(short, long)]
        group: Vec<String>,
        
        /// Run commands in parallel
        #[arg(short, long)]
        parallel: bool,
        
        /// Maximum number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
        
        /// Continue execution even if some commands fail
        #[arg(long)]
        continue_on_error: bool,
    },
    
    /// Show commit log for repositories
    Log {
        /// Groups to show log for (if not specified, all groups will be used)
        #[arg(short, long)]
        group: Vec<String>,
        
        /// Show one line per commit
        #[arg(long)]
        oneline: bool,
        
        /// Maximum number of commits to show
        #[arg(short = 'n', long)]
        max_count: Option<usize>,
        
        /// Show commits since date
        #[arg(long)]
        since: Option<String>,
        
        /// Show commits until date
        #[arg(long)]
        until: Option<String>,
    },
    
    /// Dump the workspace manifest
    DumpManifest {
        /// Output format (yaml or json)
        #[arg(short, long, default_value = "yaml")]
        format: String,
        
        /// Output file path (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Pretty print JSON output
        #[arg(long)]
        pretty: bool,
    },
}

/// CLI application runner
pub struct CliApp {
    cli: Cli,
}

impl CliApp {
    pub fn new() -> Self {
        Self {
            cli: Cli::parse(),
        }
    }
    
    pub async fn run(self) -> anyhow::Result<()> {
        // Set up colored output
        if !self.cli.no_color {
            colored::control::set_override(true);
        } else {
            colored::control::set_override(false);
        }
        
        // Change directory if specified
        if let Some(ref dir) = self.cli.directory {
            env::set_current_dir(dir)?;
        }
        
        // Handle the command
        match self.handle_command().await {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                exit(1);
            }
        }
    }
    
    async fn handle_command(&self) -> anyhow::Result<()> {
        match &self.cli.command {
            Commands::Init { 
                manifest_url, 
                branch, 
                group, 
                shallow, 
                force 
            } => {
                self.handle_init_command(manifest_url, branch, group, *shallow, *force).await
            }
            Commands::Sync { 
                group, 
                force, 
                no_correct_branch, 
                jobs 
            } => {
                self.handle_sync_command(group, *force, *no_correct_branch, *jobs).await
            }
            Commands::Status { 
                branch, 
                compact, 
                group 
            } => {
                self.handle_status_command(*branch, *compact, group).await
            }
            Commands::Foreach { 
                command, 
                args, 
                group, 
                parallel, 
                jobs, 
                continue_on_error 
            } => {
                self.handle_foreach_command(command, args, group, *parallel, *jobs, *continue_on_error).await
            }
            Commands::Log {
                group,
                oneline,
                max_count,
                since,
                until,
            } => {
                self.handle_log_command(group, *oneline, *max_count, since, until).await
            }
            Commands::DumpManifest {
                format,
                output,
                pretty,
            } => {
                self.handle_dump_manifest_command(format, output, *pretty).await
            }
        }
    }
    
    async fn handle_init_command(
        &self,
        manifest_url: &str,
        branch: &Option<String>,
        groups: &[String],
        shallow: bool,
        force: bool,
    ) -> anyhow::Result<()> {
        let current_dir = env::current_dir()?;
        
        // Parse and validate the manifest URL
        let git_url = GitUrl::new(manifest_url)
            .map_err(|e| anyhow::anyhow!("Invalid manifest URL: {}", e))?;
        
        // Create workspace path
        let workspace_path = FilePath::new_absolute(current_dir.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Invalid workspace path: {}", e))?;
        
        // Prepare groups list
        let groups_list = if groups.is_empty() {
            None
        } else {
            Some(groups.to_vec())
        };
        
        // Create configuration
        let config = InitWorkspaceConfig {
            manifest_url: git_url,
            workspace_path,
            branch: branch.clone(),
            groups: groups_list,
            shallow,
            force,
        };
        
        // Execute the use case
        let use_case = InitWorkspaceUseCase::new(config);
        
        println!("{} Initializing workspace...", "::".blue().bold());
        
        match use_case.execute().await {
            Ok(workspace) => {
                println!("{} Workspace initialized successfully!", "✓".green().bold());
                if self.cli.verbose {
                    println!("  Manifest URL: {}", manifest_url);
                    println!("  Workspace path: {}", workspace.root_path.display());
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
    
    async fn handle_sync_command(
        &self,
        groups: &[String],
        force: bool,
        no_correct_branch: bool,
        jobs: Option<usize>,
    ) -> anyhow::Result<()> {
        // Load workspace
        let mut workspace = self.load_workspace().await?;
        
        // Prepare groups list
        let groups_list = if groups.is_empty() {
            None
        } else {
            Some(groups.to_vec())
        };
        
        // Create configuration
        let config = SyncRepositoriesConfig {
            groups: groups_list,
            force,
            no_correct_branch,
            parallel_jobs: jobs,
            verbose: self.cli.verbose,
        };
        
        // Execute the use case
        let use_case = SyncRepositoriesUseCase::new(config);
        
        println!("{} Synchronizing repositories...", "::".blue().bold());
        
        match use_case.execute(&mut workspace).await {
            Ok(result) => {
                println!("{} Synchronization completed!", "✓".green().bold());
                if self.cli.verbose {
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
    
    async fn handle_status_command(
        &self,
        show_branch: bool,
        compact: bool,
        groups: &[String],
    ) -> anyhow::Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        // Prepare groups list
        let groups_list = if groups.is_empty() {
            None
        } else {
            Some(groups.to_vec())
        };
        
        // Create configuration
        let config = StatusCheckConfig {
            groups: groups_list,
            show_branch,
            compact,
            verbose: self.cli.verbose,
        };
        
        // Execute the use case
        let use_case = StatusCheckUseCase::new(config);
        
        match use_case.execute(&workspace).await {
            Ok(status) => {
                if compact {
                    self.print_compact_status(&status);
                } else {
                    self.print_detailed_status(&status, show_branch);
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
    
    async fn handle_foreach_command(
        &self,
        command: &str,
        args: &[String],
        groups: &[String],
        parallel: bool,
        jobs: Option<usize>,
        continue_on_error: bool,
    ) -> anyhow::Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        // Prepare groups list
        let groups_list = if groups.is_empty() {
            None
        } else {
            Some(groups.to_vec())
        };
        
        // Build the full command
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };
        
        // Create configuration
        let config = ForeachCommandConfig {
            command: full_command,
            groups: groups_list,
            parallel,
            max_parallel: jobs,
            continue_on_error,
            verbose: self.cli.verbose,
            ..Default::default()
        };
        
        // Execute the use case
        let use_case = ForeachCommandUseCase::new(config);
        
        println!("{} Running command: {}", "::".blue().bold(), command);
        
        match use_case.execute(&workspace).await {
            Ok(result) => {
                println!("{} Command execution completed!", "✓".green().bold());
                if self.cli.verbose {
                    println!("  Successful executions: {}", result.success_count);
                    println!("  Failed executions: {}", result.failure_count);
                    println!("  Skipped executions: {}", result.skipped_count);
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
    
    async fn handle_log_command(
        &self,
        groups: &[String],
        oneline: bool,
        max_count: Option<usize>,
        since: &Option<String>,
        until: &Option<String>,
    ) -> anyhow::Result<()> {
        use crate::presentation::cli::commands::log::LogCommand;
        
        let command = LogCommand::new(
            groups.to_vec(),
            oneline,
            max_count,
            since.clone(),
            until.clone(),
            self.cli.verbose,
        );
        
        command.execute().await
    }
    
    async fn handle_dump_manifest_command(
        &self,
        format: &str,
        output_file: &Option<String>,
        pretty: bool,
    ) -> anyhow::Result<()> {
        use crate::presentation::cli::commands::dump_manifest::{DumpManifestCommand, OutputFormat};
        
        let output_format: OutputFormat = format.parse()
            .map_err(|e| anyhow::anyhow!("Invalid format: {}", e))?;
        
        let command = DumpManifestCommand::new(
            output_format,
            output_file.clone(),
            pretty,
            self.cli.verbose,
        );
        
        command.execute().await
    }
    
    /// Load workspace from the current directory
    async fn load_workspace(&self) -> anyhow::Result<Workspace> {
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
    
    fn print_compact_status(&self, status: &crate::application::use_cases::status_check::StatusResult) {
        for repo_status in &status.repositories {
            let state_char = match repo_status.state {
                crate::application::use_cases::status_check::RepositoryState::Clean => "✓".green(),
                crate::application::use_cases::status_check::RepositoryState::Dirty => "M".yellow(),
                crate::application::use_cases::status_check::RepositoryState::Missing => "?".red(),
                crate::application::use_cases::status_check::RepositoryState::WrongBranch => "B".cyan(),
                crate::application::use_cases::status_check::RepositoryState::OutOfSync => "S".magenta(),
                crate::application::use_cases::status_check::RepositoryState::Error => "E".red(),
            };
            println!("{} {}", state_char, repo_status.dest);
        }
    }
    
    fn print_detailed_status(&self, status: &crate::application::use_cases::status_check::StatusResult, show_branch: bool) {
        for repo_status in &status.repositories {
            let state_text = match repo_status.state {
                crate::application::use_cases::status_check::RepositoryState::Clean => "clean".green(),
                crate::application::use_cases::status_check::RepositoryState::Dirty => "dirty".yellow(),
                crate::application::use_cases::status_check::RepositoryState::Missing => "missing".red(),
                crate::application::use_cases::status_check::RepositoryState::WrongBranch => "wrong branch".cyan(),
                crate::application::use_cases::status_check::RepositoryState::OutOfSync => "out of sync".magenta(),
                crate::application::use_cases::status_check::RepositoryState::Error => "error".red(),
            };
            
            print!("{}: {}", repo_status.dest.bold(), state_text);
            
            if show_branch {
                if let Some(ref current_branch) = repo_status.current_branch {
                    print!(" ({})", current_branch.blue());
                }
            }
            
            if repo_status.state == crate::application::use_cases::status_check::RepositoryState::Dirty {
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
}