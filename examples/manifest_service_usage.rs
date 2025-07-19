use tsrc::application::services::manifest_service::{ManifestProcessingOptions, ManifestService};
use tsrc::domain::entities::manifest::{Group, Manifest, ManifestRepo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a manifest service with default options
    let mut service = ManifestService::default();

    // Example YAML manifest content
    let manifest_yaml = r#"
repos:
  - dest: core/app
    url: https://github.com/example/core-app.git
    branch: main
  - dest: libs/utils
    url: https://github.com/example/utils.git
    branch: develop
  - dest: docs/wiki
    url: https://github.com/example/wiki.git
    shallow: true

groups:
  core:
    repos:
      - core/app
      - libs/utils
    description: "Core application components"
  
  documentation:
    repos:
      - docs/wiki
    description: "Project documentation"

default_branch: main
"#;

    // Parse the manifest
    println!("Parsing manifest...");
    let result = service.parse_from_string(manifest_yaml, None).await?;

    // Display basic information
    println!("✅ Manifest parsed successfully!");
    println!("📦 Total repositories: {}", result.manifest.repos.len());

    if !result.warnings.is_empty() {
        println!("⚠️  Warnings: {}", result.warnings.len());
        for warning in &result.warnings {
            println!("   - {}", warning);
        }
    }

    // List all groups
    let groups = service.list_groups(&result.manifest);
    println!("📂 Groups: {:?}", groups);

    // Filter by specific group
    println!("\nFiltering by 'core' group...");
    let filtered = service.filter_by_groups(&result.manifest, &["core".to_string()])?;
    println!("📦 Repositories in 'core' group: {}", filtered.repos.len());
    for repo in &filtered.repos {
        println!("   - {} -> {}", repo.dest, repo.url);
    }

    // Get detailed group information
    if let Some((group, repos)) = service.get_group_info(&result.manifest, "core") {
        println!("\n📋 Group 'core' details:");
        if let Some(desc) = &group.description {
            println!("   Description: {}", desc);
        }
        println!("   Repositories: {}", repos.len());
    }

    // Serialize back to YAML
    println!("\nSerializing filtered manifest to YAML:");
    let yaml_output = service.serialize_to_yaml(&filtered)?;
    println!("{}", yaml_output);

    Ok(())
}
