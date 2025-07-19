use std::env;
use std::path::Path;
use colored::Colorize;
use anyhow::Result;

use crate::domain::entities::workspace::Workspace;
use crate::application::services::manifest_service::{ManifestService, ManifestProcessingOptions};
use crate::infrastructure::filesystem::manifest_store::ManifestStore;

/// Handler for the apply-manifest command
pub struct ApplyManifestCommand {
    pub manifest_file: String,
    pub force: bool,
    pub dry_run: bool,
    pub verbose: bool,
}

impl ApplyManifestCommand {
    pub fn new(
        manifest_file: String,
        force: bool,
        dry_run: bool,
        verbose: bool,
    ) -> Self {
        Self {
            manifest_file,
            force,
            dry_run,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        if self.verbose {
            println!("{} Applying manifest from: {}", "::".blue().bold(), self.manifest_file.bold());
        }

        // Check if the manifest file exists
        let manifest_path = Path::new(&self.manifest_file);
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("Manifest file not found: {}", self.manifest_file));
        }

        // Load and parse the new manifest
        let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());
        let new_processed_manifest = manifest_service.parse_from_file(manifest_path).await
            .map_err(|e| anyhow::anyhow!("Failed to parse new manifest: {}", e))?;

        if self.verbose {
            println!("  {} Parsed new manifest successfully", "✓".green());
            println!("  {} Found {} repositories", "->".blue(), new_processed_manifest.manifest.repos.len());
        }

        // Get the current workspace manifest
        let current_manifest_path = workspace.manifest_file_path();
        let current_processed_manifest = if current_manifest_path.exists() {
            Some(manifest_service.parse_from_file(&current_manifest_path).await
                .map_err(|e| anyhow::anyhow!("Failed to parse current manifest: {}", e))?)
        } else {
            None
        };

        // Analyze the differences
        let changes = self.analyze_manifest_changes(
            current_processed_manifest.as_ref().map(|p| &p.manifest),
            &new_processed_manifest.manifest
        );

        if self.verbose || self.dry_run {
            self.print_changes(&changes);
        }

        if changes.is_empty() {
            println!("{} No changes detected in the manifest", "✓".green().bold());
            return Ok(());
        }

        if self.dry_run {
            println!("{} Dry run completed - no changes applied", "::".blue().bold());
            return Ok(());
        }

        if !self.force && !changes.is_empty() {
            return Err(anyhow::anyhow!(
                "Manifest changes detected. Use --force to apply changes or --dry-run to preview"
            ));
        }

        // Apply the new manifest
        if self.verbose {
            println!("{} Applying manifest changes...", "::".blue().bold());
        }

        // Update the manifest file in the workspace
        let manifest_store = ManifestStore::new();
        manifest_store.write_manifest(&current_manifest_path, &new_processed_manifest.manifest).await
            .map_err(|e| anyhow::anyhow!("Failed to save new manifest: {}", e))?;

        println!("{} Manifest applied successfully!", "✓".green().bold());
        
        if self.verbose {
            println!("  {} {} repositories added", "->".green(), changes.added.len());
            println!("  {} {} repositories modified", "->".yellow(), changes.modified.len());
            println!("  {} {} repositories removed", "->".red(), changes.removed.len());
        }

        // Suggest next steps
        if !changes.is_empty() {
            println!();
            println!("{} Next steps:", "::".blue().bold());
            println!("  {} Run 'wmgr sync' to apply repository changes", "1.".bold());
            if !changes.removed.is_empty() {
                println!("  {} Manually clean up removed repositories if needed", "2.".bold());
            }
        }

        Ok(())
    }

    fn analyze_manifest_changes(
        &self,
        current: Option<&crate::domain::entities::manifest::Manifest>,
        new: &crate::domain::entities::manifest::Manifest,
    ) -> ManifestChanges {
        let mut changes = ManifestChanges {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
        };

        match current {
            Some(current_manifest) => {
                // Compare repositories
                let current_repos: std::collections::HashMap<_, _> = current_manifest.repos
                    .iter()
                    .map(|repo| (&repo.dest, repo))
                    .collect();

                let new_repos: std::collections::HashMap<_, _> = new.repos
                    .iter()
                    .map(|repo| (&repo.dest, repo))
                    .collect();

                // Find added repositories
                for (dest, repo) in &new_repos {
                    if !current_repos.contains_key(*dest) {
                        changes.added.push((*repo).clone());
                    }
                }

                // Find modified repositories
                for (dest, new_repo) in &new_repos {
                    if let Some(current_repo) = current_repos.get(*dest) {
                        if self.repo_differs(current_repo, new_repo) {
                            changes.modified.push(ChangeDetail {
                                dest: dest.to_string(),
                                old_repo: Some((*current_repo).clone()),
                                new_repo: (*new_repo).clone(),
                            });
                        }
                    }
                }

                // Find removed repositories
                for (dest, repo) in &current_repos {
                    if !new_repos.contains_key(*dest) {
                        changes.removed.push((*repo).clone());
                    }
                }
            }
            None => {
                // No current manifest, all repositories are new
                changes.added = new.repos.clone();
            }
        }

        changes
    }

    fn repo_differs(
        &self,
        current: &crate::domain::entities::manifest::ManifestRepo,
        new: &crate::domain::entities::manifest::ManifestRepo,
    ) -> bool {
        current.url != new.url ||
        current.branch != new.branch ||
        current.sha1 != new.sha1 ||
        current.tag != new.tag ||
        current.remotes != new.remotes
    }

    fn print_changes(&self, changes: &ManifestChanges) {
        if changes.is_empty() {
            return;
        }

        println!();
        println!("{} Manifest changes:", "::".blue().bold());

        if !changes.added.is_empty() {
            println!("  {} {} repositories to be added:", "+".green().bold(), changes.added.len());
            for repo in &changes.added {
                println!("    {} {} ({})", "+".green(), repo.dest.bold(), repo.url.dimmed());
            }
        }

        if !changes.modified.is_empty() {
            println!("  {} {} repositories to be modified:", "~".yellow().bold(), changes.modified.len());
            for change in &changes.modified {
                println!("    {} {}", "~".yellow(), change.dest.bold());
                if let Some(ref old_repo) = change.old_repo {
                    if old_repo.url != change.new_repo.url {
                        println!("      URL: {} -> {}", old_repo.url.dimmed(), change.new_repo.url.green());
                    }
                    if old_repo.branch != change.new_repo.branch {
                        println!("      Branch: {:?} -> {:?}", old_repo.branch, change.new_repo.branch);
                    }
                    if old_repo.sha1 != change.new_repo.sha1 {
                        println!("      SHA1: {:?} -> {:?}", old_repo.sha1, change.new_repo.sha1);
                    }
                    if old_repo.tag != change.new_repo.tag {
                        println!("      Tag: {:?} -> {:?}", old_repo.tag, change.new_repo.tag);
                    }
                }
            }
        }

        if !changes.removed.is_empty() {
            println!("  {} {} repositories to be removed:", "-".red().bold(), changes.removed.len());
            for repo in &changes.removed {
                println!("    {} {} ({})", "-".red(), repo.dest.bold(), repo.url.dimmed());
            }
        }

        println!();
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
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        use crate::domain::entities::workspace::{WorkspaceStatus, WorkspaceConfig};
        let mut manifest_store = ManifestStore::new();
        
        let processed_manifest = manifest_store.read_manifest(&manifest_file).await
            .map_err(|e| anyhow::anyhow!("Failed to read manifest: {}", e))?;
        
        // Create workspace config from manifest
        let workspace_config = WorkspaceConfig::new(
            "file://".to_string() + &manifest_file.to_string_lossy(),
            processed_manifest.manifest.default_branch.clone().unwrap_or_else(|| "main".to_string())
        );
        
        let workspace = Workspace::new(current_dir, workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);
        
        Ok(workspace)
    }
}

#[derive(Debug, Clone)]
struct ManifestChanges {
    added: Vec<crate::domain::entities::manifest::ManifestRepo>,
    modified: Vec<ChangeDetail>,
    removed: Vec<crate::domain::entities::manifest::ManifestRepo>,
}

#[derive(Debug, Clone)]
struct ChangeDetail {
    dest: String,
    old_repo: Option<crate::domain::entities::manifest::ManifestRepo>,
    new_repo: crate::domain::entities::manifest::ManifestRepo,
}

impl ManifestChanges {
    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
    }
}