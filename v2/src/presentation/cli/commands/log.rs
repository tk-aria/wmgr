use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::domain::entities::workspace::Workspace;
use crate::infrastructure::git::repository::GitRepository;
use crate::domain::value_objects::git_url::GitUrl;

/// Handler for the log command
pub struct LogCommand {
    pub groups: Vec<String>,
    pub oneline: bool,
    pub max_count: Option<usize>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub verbose: bool,
}

impl LogCommand {
    pub fn new(
        groups: Vec<String>,
        oneline: bool,
        max_count: Option<usize>,
        since: Option<String>,
        until: Option<String>,
        verbose: bool,
    ) -> Self {
        Self {
            groups,
            oneline,
            max_count,
            since,
            until,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        // Get repositories to check
        let repositories = self.get_repositories_to_check(&workspace)?;
        
        if repositories.is_empty() {
            println!("{} No repositories found to check log", "⚠".yellow().bold());
            return Ok(());
        }

        println!("{} Showing commit log for {} repositories", "::".blue().bold(), repositories.len());

        // Show log for each repository
        for repo in repositories {
            self.show_repository_log(&repo, &workspace).await?;
        }

        Ok(())
    }

    async fn show_repository_log(&self, repo: &crate::domain::entities::repository::Repository, workspace: &Workspace) -> Result<()> {
        let repo_path = workspace.root_path.join(&repo.dest);
        
        if !repo_path.exists() {
            if self.verbose {
                println!("{} {}: repository not found", "⚠".yellow(), repo.dest);
            }
            return Ok(());
        }

        println!();
        println!("{} {}", "Repository:".bold(), repo.dest.green());
        
        // Try to open the git repository
        let git_repo = match git2::Repository::open(&repo_path) {
            Ok(repo) => repo,
            Err(e) => {
                if self.verbose {
                    println!("  {}: not a git repository - {}", "Error".red(), e);
                }
                return Ok(());
            }
        };

        // Get the current branch
        let head_ref = match git_repo.head() {
            Ok(head) => head,
            Err(e) => {
                if self.verbose {
                    println!("  {}: failed to get HEAD - {}", "Error".red(), e);
                }
                return Ok(());
            }
        };

        let branch_name = head_ref.shorthand().unwrap_or("unknown");
        if self.verbose {
            println!("  {}: {}", "Branch".blue(), branch_name);
        }

        // Walk commits
        let mut revwalk = git_repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut count = 0;
        let max_count = self.max_count.unwrap_or(10);

        for commit_oid in revwalk {
            if count >= max_count {
                break;
            }

            let commit_oid = commit_oid?;
            let commit = git_repo.find_commit(commit_oid)?;

            // Check date filters
            let commit_time = commit.time();
            let commit_timestamp = commit_time.seconds();

            if let Some(ref since) = self.since {
                // For simplicity, we'll just show a placeholder for date parsing
                // In a real implementation, you'd parse the date string
                if self.verbose {
                    println!("  (Date filtering with 'since: {}' not fully implemented)", since);
                }
            }

            if let Some(ref until) = self.until {
                // For simplicity, we'll just show a placeholder for date parsing
                // In a real implementation, you'd parse the date string
                if self.verbose {
                    println!("  (Date filtering with 'until: {}' not fully implemented)", until);
                }
            }

            // Display commit info
            let commit_hash = commit.id().to_string();
            let short_hash = &commit_hash[..7];
            let message = commit.message().unwrap_or("(no message)");
            let summary = message.lines().next().unwrap_or("(no message)");
            
            if self.oneline {
                println!("  {} {}", short_hash.yellow(), summary);
            } else {
                let author = commit.author();
                let author_name = author.name().unwrap_or("unknown");
                let author_email = author.email().unwrap_or("unknown");
                
                // Format time
                let datetime = chrono::DateTime::from_timestamp(commit_timestamp, 0)
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
                let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");

                println!("  {} {}", "commit".yellow(), commit_hash.yellow());
                println!("  {}: {} <{}>", "Author".blue(), author_name, author_email);
                println!("  {}: {}", "Date".blue(), formatted_time);
                println!();
                
                // Show full commit message with indentation
                for line in message.lines() {
                    println!("      {}", line);
                }
                println!();
            }

            count += 1;
        }

        if count == 0 {
            println!("  {}", "No commits found".dimmed());
        }

        Ok(())
    }

    fn get_repositories_to_check(&self, workspace: &Workspace) -> Result<Vec<crate::domain::entities::repository::Repository>> {
        // For now, return a sample repository
        // In a real implementation, this would load from the workspace manifest
        let mut repositories = Vec::new();
        
        // This is a placeholder implementation
        // In a real implementation, you would:
        // 1. Load the manifest from the workspace
        // 2. Filter repositories by groups if specified
        // 3. Return the filtered list
        
        if self.verbose {
            println!("  (Repository filtering by groups not fully implemented)");
        }

        Ok(repositories)
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