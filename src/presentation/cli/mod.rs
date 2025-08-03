pub mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use std::env;
use std::process::exit;

use crate::application::use_cases::{
    foreach_command::{ForeachCommandConfig, ForeachCommandError, ForeachCommandUseCase},
    status_check::{StatusCheckConfig, StatusCheckError, StatusCheckUseCase},
    sync_repositories::{SyncRepositoriesConfig, SyncRepositoriesError, SyncRepositoriesUseCase},
};

use crate::domain::entities::workspace::Workspace;

use crate::domain::value_objects::{file_path::FilePath, git_url::GitUrl};

/// Output format options for status command
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output (default)
    Text,
    /// JSON output
    Json,
    /// YAML output
    Yaml,
}

/// wmgr - A tool for managing multiple git repositories
#[derive(Parser)]
#[command(name = "wmgr")]
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
    /// Initialize a new wmgr workspace
    Init {
        /// Directory where to create the wmgr.yaml file (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,

        /// Use manifest.yaml instead of wmgr.yaml
        #[arg(long)]
        manifest: bool,
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

        /// Disable recursive sync of child workspaces
        #[arg(long)]
        no_recursive: bool,
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

        /// Output format (text, json, yaml)
        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputFormat,
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

    /// Security audit for dependencies
    Audit {
        /// Groups to audit (if not specified, all groups will be audited)
        #[arg(short, long)]
        group: Vec<String>,

        /// Run audit in parallel
        #[arg(short, long, default_value = "true")]
        parallel: bool,

        /// Maximum number of parallel audits
        #[arg(short, long)]
        jobs: Option<usize>,

        /// Continue on vulnerabilities instead of failing
        #[arg(short, long)]
        continue_on_vulnerabilities: bool,
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

    /// Apply a new manifest to the workspace
    ApplyManifest {
        /// Path to the new manifest file
        manifest_file: String,

        /// Force apply changes without confirmation
        #[arg(short, long)]
        force: bool,

        /// Show what would be changed without applying
        #[arg(long)]
        dry_run: bool,
    },
}

/// CLI application runner
pub struct CliApp {
    cli: Cli,
}

impl CliApp {
    pub fn new() -> Self {
        Self { cli: Cli::parse() }
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
                path,
                force,
                manifest,
            } => {
                self.handle_init_command(path.as_ref(), *force, *manifest)
                    .await
            }
            Commands::Sync {
                group,
                force,
                no_correct_branch,
                jobs,
                no_recursive,
            } => {
                self.handle_sync_command(group, *force, *no_correct_branch, *jobs, *no_recursive)
                    .await
            }
            Commands::Status {
                branch,
                compact,
                group,
                output,
            } => self.handle_status_command(*branch, *compact, group, output.clone()).await,
            Commands::Foreach {
                command,
                args,
                group,
                parallel,
                jobs,
                continue_on_error,
            } => {
                self.handle_foreach_command(
                    command,
                    args,
                    group,
                    *parallel,
                    *jobs,
                    *continue_on_error,
                )
                .await
            }
            Commands::Audit {
                group,
                parallel,
                jobs,
                continue_on_vulnerabilities,
            } => {
                self.handle_audit_command(group, *parallel, *jobs, *continue_on_vulnerabilities)
                    .await
            }
            Commands::Log {
                group,
                oneline,
                max_count,
                since,
                until,
            } => {
                self.handle_log_command(group, *oneline, *max_count, since, until)
                    .await
            }
            Commands::DumpManifest {
                format,
                output,
                pretty,
            } => {
                self.handle_dump_manifest_command(format, output, *pretty)
                    .await
            }
            Commands::ApplyManifest {
                manifest_file,
                force,
                dry_run,
            } => {
                self.handle_apply_manifest_command(manifest_file, *force, *dry_run)
                    .await
            }
        }
    }

    async fn handle_init_command(
        &self,
        path: Option<&String>,
        force: bool,
        use_manifest_name: bool,
    ) -> anyhow::Result<()> {
        use crate::presentation::cli::commands::init::InitCommand;

        let target_path = path.map(|p| std::path::PathBuf::from(p));
        let init_cmd = InitCommand::new(target_path, force, use_manifest_name);
        init_cmd.execute().await
    }

    async fn handle_sync_command(
        &self,
        groups: &[String],
        force: bool,
        no_correct_branch: bool,
        jobs: Option<usize>,
        no_recursive: bool,
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
            recursive: !no_recursive,
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
            Err(SyncRepositoriesError::WorkspaceNotInitialized(path)) => Err(anyhow::anyhow!(
                "Workspace not initialized at: {}\nManifest file not found",
                path
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to synchronize repositories: {}", e)),
        }
    }

    async fn handle_status_command(
        &self,
        show_branch: bool,
        compact: bool,
        groups: &[String],
        output_format: OutputFormat,
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
                match output_format {
                    OutputFormat::Json => self.print_json_status(&status)?,
                    OutputFormat::Yaml => self.print_yaml_status(&status)?,
                    OutputFormat::Text => {
                        if compact {
                            self.print_compact_status(&status);
                        } else {
                            self.print_detailed_status(&status, show_branch);
                        }
                    }
                }
                Ok(())
            }
            Err(StatusCheckError::WorkspaceNotInitialized(path)) => Err(anyhow::anyhow!(
                "Workspace not initialized at: {}\nManifest file not found",
                path
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to check status: {}", e)),
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
            Err(ForeachCommandError::WorkspaceNotInitialized(path)) => Err(anyhow::anyhow!(
                "Workspace not initialized at: {}\nManifest file not found",
                path
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to execute command: {}", e)),
        }
    }

    async fn handle_audit_command(
        &self,
        groups: &[String],
        parallel: bool,
        jobs: Option<usize>,
        continue_on_vulnerabilities: bool,
    ) -> anyhow::Result<()> {
        use crate::presentation::cli::commands::audit::{AuditArgs, AuditCommand};
        use std::path::PathBuf;

        let args = AuditArgs {
            groups: if groups.is_empty() {
                None
            } else {
                Some(groups.to_vec())
            },
            parallel,
            max_parallel: jobs,
            continue_on_vulnerabilities,
            verbose: self.cli.verbose,
            workspace_dir: None, // Use current directory
        };

        let command = AuditCommand::new();
        command
            .execute(args)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))
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
        use crate::presentation::cli::commands::dump_manifest::{
            DumpManifestCommand, OutputFormat,
        };

        let output_format: OutputFormat = format
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid format: {}", e))?;

        let command =
            DumpManifestCommand::new(output_format, output_file.clone(), pretty, self.cli.verbose);

        command.execute().await
    }

    async fn handle_apply_manifest_command(
        &self,
        manifest_file: &str,
        force: bool,
        dry_run: bool,
    ) -> anyhow::Result<()> {
        use crate::presentation::cli::commands::apply_manifest::ApplyManifestCommand;

        let command =
            ApplyManifestCommand::new(manifest_file.to_string(), force, dry_run, self.cli.verbose);

        command.execute().await
    }

    /// Load workspace from the current directory or any parent directory
    async fn load_workspace(&self) -> anyhow::Result<Workspace> {
        let current_dir = env::current_dir()?;

        // Discover workspace root by searching upward for manifest files
        let workspace_root = if let Some(root) = Workspace::discover_workspace_root(&current_dir) {
            root
        } else {
            return Err(anyhow::anyhow!(
                "No wmgr workspace found. Searched upward from {} for wmgr.yml, wmgr.yaml, manifest.yml, or manifest.yaml files.",
                current_dir.display()
            ));
        };

        // Create workspace and find manifest file
        let workspace = Workspace::new(workspace_root.clone(), WorkspaceConfig::default_local());
        let manifest_file = workspace.manifest_file_path();

        // Load manifest file
        use crate::domain::entities::workspace::{WorkspaceConfig, WorkspaceStatus};
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store
            .read_manifest(&manifest_file)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

        // Create a workspace configuration
        let workspace_config = WorkspaceConfig::new(&manifest_file.display().to_string(), "main");

        let workspace = Workspace::new(workspace_root, workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);
        Ok(workspace)
    }

    fn print_compact_status(
        &self,
        status: &crate::application::use_cases::status_check::StatusResult,
    ) {
        for repo_status in &status.repositories {
            let state_char = match repo_status.state {
                crate::application::use_cases::status_check::RepositoryState::Clean => "✓".green(),
                crate::application::use_cases::status_check::RepositoryState::Dirty => "M".yellow(),
                crate::application::use_cases::status_check::RepositoryState::Missing => "?".red(),
                crate::application::use_cases::status_check::RepositoryState::WrongBranch => {
                    "B".cyan()
                }
                crate::application::use_cases::status_check::RepositoryState::OutOfSync => {
                    "S".magenta()
                }
                crate::application::use_cases::status_check::RepositoryState::Error => "E".red(),
            };
            println!("{} {}", state_char, repo_status.dest);
        }
    }

    fn print_detailed_status(
        &self,
        status: &crate::application::use_cases::status_check::StatusResult,
        show_branch: bool,
    ) {
        for repo_status in &status.repositories {
            let state_text = match repo_status.state {
                crate::application::use_cases::status_check::RepositoryState::Clean => {
                    "clean".green()
                }
                crate::application::use_cases::status_check::RepositoryState::Dirty => {
                    "dirty".yellow()
                }
                crate::application::use_cases::status_check::RepositoryState::Missing => {
                    "missing".red()
                }
                crate::application::use_cases::status_check::RepositoryState::WrongBranch => {
                    "wrong branch".cyan()
                }
                crate::application::use_cases::status_check::RepositoryState::OutOfSync => {
                    "out of sync".magenta()
                }
                crate::application::use_cases::status_check::RepositoryState::Error => {
                    "error".red()
                }
            };

            print!("{}: {}", repo_status.dest.bold(), state_text);

            if show_branch {
                if let Some(ref current_branch) = repo_status.current_branch {
                    print!(" ({})", current_branch.blue());
                }
            }

            if repo_status.state
                == crate::application::use_cases::status_check::RepositoryState::Dirty
            {
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

    fn print_json_status(
        &self,
        status: &crate::application::use_cases::status_check::StatusResult,
    ) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(status)?;
        println!("{}", json);
        Ok(())
    }

    fn print_yaml_status(
        &self,
        status: &crate::application::use_cases::status_check::StatusResult,
    ) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(status)?;
        print!("{}", yaml);
        Ok(())
    }
}
