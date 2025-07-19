use crate::application::use_cases::security_audit::{
    SecurityAuditConfig, SecurityAuditError, SecurityAuditUseCase,
};
use crate::domain::entities::workspace::Workspace;
use crate::presentation::ui::display::DisplayHelper;
use clap::Args;
use std::path::PathBuf;

/// Security audit command arguments
#[derive(Debug, Args)]
pub struct AuditArgs {
    /// Target groups to audit (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub groups: Option<Vec<String>>,

    /// Run audit in parallel
    #[arg(short, long, default_value = "true")]
    pub parallel: bool,

    /// Maximum number of parallel audits
    #[arg(short, long)]
    pub max_parallel: Option<usize>,

    /// Continue on vulnerabilities instead of failing
    #[arg(short, long)]
    pub continue_on_vulnerabilities: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Workspace directory (current directory if not specified)
    #[arg(long)]
    pub workspace_dir: Option<PathBuf>,
}

/// Security audit command implementation
pub struct AuditCommand {
    display: DisplayHelper,
}

impl AuditCommand {
    /// Create a new AuditCommand instance
    pub fn new() -> Self {
        Self {
            display: DisplayHelper::new(true), // Enable colored output
        }
    }

    /// Execute the audit command
    pub async fn execute(&self, args: AuditArgs) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Determine workspace directory
        let workspace_dir = args
            .workspace_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // 2. Load workspace configuration
        let workspace = self.load_workspace(&workspace_dir).await?;

        // 3. Create audit configuration
        let config = SecurityAuditConfig::new()
            .with_groups(args.groups.unwrap_or_default())
            .with_parallel(args.parallel, args.max_parallel)
            .with_fail_on_vulnerabilities(!args.continue_on_vulnerabilities)
            .with_verbose(args.verbose);

        // 4. Execute audit
        let use_case = SecurityAuditUseCase::new(config);

        if args.verbose {
            println!("Starting security audit...");
        }

        match use_case.execute(&workspace).await {
            Ok(result) => {
                self.display_audit_result(&result, args.verbose);

                if result.has_critical_or_high_vulnerabilities()
                    && !args.continue_on_vulnerabilities
                {
                    std::process::exit(1);
                }

                Ok(())
            }
            Err(SecurityAuditError::NoRustProjectsFound) => {
                self.display.warning("No Rust projects found in workspace");
                Ok(())
            }
            Err(e) => {
                eprintln!("Security audit failed: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Load workspace from directory
    async fn load_workspace(
        &self,
        workspace_dir: &PathBuf,
    ) -> Result<Workspace, Box<dyn std::error::Error>> {
        let workspace = Workspace::new(workspace_dir.clone(), WorkspaceConfig::default_local());

        // Use workspace.manifest_file_path() to support wmgr.yml, wmgr.yaml, manifest.yml, manifest.yaml
        let manifest_file = workspace.manifest_file_path();

        if !manifest_file.exists() {
            return Err("Manifest file not found. Tried wmgr.yml, wmgr.yaml, manifest.yml, manifest.yaml in current directory and .wmgr/ subdirectory".into());
        }

        // Load manifest file
        use crate::domain::entities::workspace::{WorkspaceConfig, WorkspaceStatus};
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        let mut manifest_store = ManifestStore::new();

        let processed_manifest = manifest_store
            .read_manifest(&manifest_file)
            .await
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        // Create workspace config from manifest
        let workspace_config = WorkspaceConfig::new(
            "file://".to_string() + &manifest_file.to_string_lossy(),
            processed_manifest
                .manifest
                .default_branch
                .clone()
                .unwrap_or_else(|| "main".to_string()),
        );

        let workspace = Workspace::new(workspace_dir.clone(), workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);

        Ok(workspace)
    }

    /// Display audit results
    fn display_audit_result(
        &self,
        result: &crate::application::use_cases::security_audit::WorkspaceAuditResult,
        verbose: bool,
    ) {
        // Summary
        println!("\n=== Security Audit Summary ===");
        println!("Total repositories: {}", result.total_count());
        println!("Audited (Rust projects): {}", result.audited_count);
        println!("Skipped (non-Rust): {}", result.skipped_count);
        println!("Errors: {}", result.error_count);
        println!("With vulnerabilities: {}", result.vulnerable_count);

        // Vulnerability details
        if result.has_vulnerabilities() {
            println!("\n=== Vulnerabilities Found ===");

            let vulnerable_repos = result.vulnerable_results();
            for repo_result in &vulnerable_repos {
                if let Some(audit_result) = &repo_result.audit_result {
                    let count = &audit_result.warning_count;

                    // Repository header
                    if count.has_critical_or_high() {
                        eprintln!("📛 {}: {} vulnerabilities", repo_result.dest, count.total());
                    } else {
                        self.display.warning(&format!(
                            "⚠️  {}: {} vulnerabilities",
                            repo_result.dest,
                            count.total()
                        ));
                    }

                    // Severity breakdown
                    if count.critical > 0 {
                        println!("    🔴 Critical: {}", count.critical);
                    }
                    if count.high > 0 {
                        println!("    🟠 High: {}", count.high);
                    }
                    if count.medium > 0 {
                        println!("    🟡 Medium: {}", count.medium);
                    }
                    if count.low > 0 {
                        println!("    🔵 Low: {}", count.low);
                    }

                    // Detailed vulnerability information if verbose
                    if verbose && !audit_result.vulnerabilities.is_empty() {
                        println!("    Vulnerabilities:");
                        for vuln in &audit_result.vulnerabilities {
                            println!(
                                "      - {} ({}): {}",
                                vuln.id,
                                format!("{:?}", vuln.severity).to_lowercase(),
                                vuln.description.chars().take(80).collect::<String>()
                            );
                            if let Some(url) = &vuln.url {
                                println!("        More info: {}", url);
                            }
                        }
                    }
                }
            }

            // Recommendations
            println!("\n=== Recommendations ===");
            if result.has_critical_or_high_vulnerabilities() {
                eprintln!("❗ Critical or high severity vulnerabilities found!");
                println!("   • Review and update affected dependencies immediately");
                println!("   • Run 'cargo update' in affected repositories");
                println!("   • Check for security advisories and patches");
            } else {
                self.display
                    .warning("⚠️  Lower severity vulnerabilities found");
                println!("   • Consider updating dependencies when convenient");
                println!("   • Monitor for security updates");
            }

            println!("   • Run 'cargo audit' in individual repositories for more details");
            println!("   • Use 'cargo audit fix' to attempt automatic fixes");
        } else if result.audited_count > 0 {
            self.display.success("✅ No vulnerabilities found!");
        }

        // Error details
        if result.error_count > 0 {
            println!("\n=== Audit Errors ===");
            let failed_repos = result.failed_results();
            for repo_result in failed_repos {
                if let Some(error) = &repo_result.error {
                    eprintln!("❌ {}: {}", repo_result.dest, error);
                }
            }
        }

        // Final status
        println!();
        if result.has_critical_or_high_vulnerabilities() {
            eprintln!("🚨 Security audit completed with critical/high vulnerabilities");
        } else if result.has_vulnerabilities() {
            self.display
                .warning("⚠️  Security audit completed with minor vulnerabilities");
        } else if result.is_success() && result.audited_count > 0 {
            self.display
                .success("✅ Security audit completed successfully - no vulnerabilities found");
        } else {
            println!("ℹ️  Security audit completed");
        }
    }
}

impl Default for AuditCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_audit_args_parsing() {
        // Test with groups
        let args = AuditArgs {
            groups: Some(vec!["group1".to_string(), "group2".to_string()]),
            parallel: true,
            max_parallel: Some(4),
            continue_on_vulnerabilities: false,
            verbose: true,
            workspace_dir: None,
        };

        assert_eq!(
            args.groups,
            Some(vec!["group1".to_string(), "group2".to_string()])
        );
        assert!(args.parallel);
        assert_eq!(args.max_parallel, Some(4));
        assert!(!args.continue_on_vulnerabilities);
        assert!(args.verbose);
    }

    #[test]
    fn test_audit_command_creation() {
        let command = AuditCommand::new();
        // Just verify it can be created without errors
        assert!(std::ptr::addr_of!(command.display) as *const _ != std::ptr::null());
    }

    #[tokio::test]
    async fn test_load_workspace_not_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let command = AuditCommand::new();

        let result = command.load_workspace(&temp_dir.path().to_path_buf()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Manifest file not found"));
    }

    #[tokio::test]
    async fn test_load_workspace_initialized() {
        let temp_dir = TempDir::new().unwrap();

        // Create minimal manifest file
        let manifest_content = r#"
repos:
  - url: https://github.com/example/repo.git
    dest: repo
"#;
        fs::write(temp_dir.path().join("manifest.yml"), manifest_content).unwrap();

        let command = AuditCommand::new();
        let result = command.load_workspace(&temp_dir.path().to_path_buf()).await;

        assert!(result.is_ok());
        let workspace = result.unwrap();
        assert!(workspace.is_initialized());
    }
}
