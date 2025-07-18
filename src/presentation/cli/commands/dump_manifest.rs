use std::env;
use colored::Colorize;
use anyhow::Result;

use crate::domain::entities::workspace::Workspace;
use crate::application::services::manifest_service::{ManifestService, ManifestProcessingOptions};

/// Handler for the dump-manifest command
pub struct DumpManifestCommand {
    pub output_format: OutputFormat,
    pub output_file: Option<String>,
    pub pretty: bool,
    pub verbose: bool,
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Yaml,
    Json,
}

impl DumpManifestCommand {
    pub fn new(
        output_format: OutputFormat,
        output_file: Option<String>,
        pretty: bool,
        verbose: bool,
    ) -> Self {
        Self {
            output_format,
            output_file,
            pretty,
            verbose,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        // Load workspace
        let workspace = self.load_workspace().await?;
        
        if self.verbose {
            println!("{} Loading manifest from workspace", "::".blue().bold());
        }

        // Get the manifest file path
        let manifest_path = workspace.manifest_file_path();
        
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("Manifest file not found at: {}", manifest_path.display()));
        }

        // Load and parse the manifest
        let mut manifest_service = ManifestService::new(ManifestProcessingOptions::default());
        let processed_manifest = manifest_service.parse_from_file(&manifest_path).await
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {}", e))?;

        // Serialize the manifest to the requested format
        let output_content = match self.output_format {
            OutputFormat::Yaml => {
                if self.verbose {
                    println!("  {} Dumping manifest as YAML", "->".blue());
                }
                manifest_service.serialize_to_yaml(&processed_manifest.manifest)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize to YAML: {}", e))?
            }
            OutputFormat::Json => {
                if self.verbose {
                    println!("  {} Dumping manifest as JSON", "->".blue());
                }
                let json_str = manifest_service.serialize_to_json(&processed_manifest.manifest)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize to JSON: {}", e))?;
                
                if self.pretty {
                    // Pretty print JSON
                    let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
                    serde_json::to_string_pretty(&json_value)?
                } else {
                    json_str
                }
            }
        };

        // Output the content
        match &self.output_file {
            Some(file_path) => {
                // Write to file
                std::fs::write(file_path, &output_content)?;
                println!("{} Manifest dumped to: {}", "✓".green().bold(), file_path.bold());
                
                if self.verbose {
                    println!("  File size: {} bytes", output_content.len());
                }
            }
            None => {
                // Print to stdout
                if self.verbose {
                    println!("{} Manifest content:", "::".blue().bold());
                    println!();
                }
                println!("{}", output_content);
            }
        }

        Ok(())
    }

    /// Load workspace from the current directory
    async fn load_workspace(&self) -> Result<Workspace> {
        let current_dir = env::current_dir()?;
        
        // Try to find manifest.yml in current directory first, then .wmgr/
        let manifest_file = if current_dir.join("manifest.yml").exists() {
            current_dir.join("manifest.yml")
        } else if current_dir.join(".wmgr").join("manifest.yml").exists() {
            current_dir.join(".wmgr").join("manifest.yml")
        } else {
            return Err(anyhow::anyhow!("Manifest file not found at: {} or {}", 
                current_dir.join("manifest.yml").display(),
                current_dir.join(".wmgr").join("manifest.yml").display()));
        };
        
        // Load manifest file
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;
        use crate::domain::entities::workspace::{WorkspaceStatus, WorkspaceConfig};
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store.read_manifest(&manifest_file).await
            .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;
        
        // Create a simple workspace configuration
        let workspace_config = WorkspaceConfig::new(
            &manifest_file.display().to_string(),
            "main"
        );
        
        let workspace = Workspace::new(current_dir, workspace_config)
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);
        Ok(workspace)
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
            "json" => Ok(OutputFormat::Json),
            _ => Err(anyhow::anyhow!("Invalid output format: {}. Supported formats: yaml, json", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}