use std::collections::HashMap;
use tempfile::TempDir;
use tokio;
use tsrc::domain::entities::manifest::{FileCopy, FileSymlink, Group, Manifest, ManifestRepo};
use tsrc::infrastructure::filesystem::manifest_store::{
    FileOperationConfig, ManifestProcessingOptions, ManifestStore,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for demonstration
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path();
    let manifest_path = workspace_root.join("manifest.yml");

    println!("=== Manifest Store Usage Example ===\n");

    // 1. Create a sample manifest with file operations
    println!("1. Creating sample manifest...");
    let mut repos = vec![
        ManifestRepo::new("https://github.com/example/core.git", "core"),
        ManifestRepo::new("https://github.com/example/utils.git", "utils"),
    ];

    // Add file copy operations to the first repo
    repos[0].copy = Some(vec![
        FileCopy {
            file: "config/default.yml".to_string(),
            dest: "shared/config.yml".to_string(),
        },
        FileCopy {
            file: "scripts/build.sh".to_string(),
            dest: "tools/build.sh".to_string(),
        },
    ]);

    // Add symlink operations to the second repo
    repos[1].symlink = Some(vec![FileSymlink {
        source: "bin/utils".to_string(),
        target: "../utils/bin/utils".to_string(),
    }]);

    // Create groups
    let mut groups = HashMap::new();
    groups.insert(
        "core".to_string(),
        Group::new(vec!["core".to_string()]).with_description("Core repositories"),
    );
    groups.insert(
        "tools".to_string(),
        Group::new(vec!["utils".to_string()]).with_description("Utility repositories"),
    );

    let manifest = Manifest::new(repos)
        .with_groups(groups)
        .with_default_branch("main");

    // 2. Create ManifestStore with custom options
    println!("2. Creating ManifestStore with custom options...");
    let mut file_config = FileOperationConfig::default();
    file_config.create_backup = true;
    file_config.overwrite_existing = true;
    file_config.max_backups = 3;

    let mut options = ManifestProcessingOptions::default();
    options.file_operation_config = file_config;
    options.validate_manifest = true;

    let mut store = ManifestStore::with_options(options);

    // 3. Write manifest to file
    println!("3. Writing manifest to file...");
    store.write_manifest(&manifest_path, &manifest).await?;
    println!("   Manifest written to: {}", manifest_path.display());

    // 4. Read manifest back
    println!("4. Reading manifest back...");
    let processed_manifest = store.read_manifest(&manifest_path).await?;
    println!(
        "   Read {} repositories",
        processed_manifest.manifest.repos.len()
    );
    println!(
        "   Groups: {:?}",
        store.list_manifest_groups(&processed_manifest.manifest)
    );

    // 5. Get manifest metadata
    println!("5. Getting manifest metadata...");
    let metadata = store.get_manifest_metadata(&manifest_path).await?;
    println!("   File exists: {}", metadata.exists);
    println!("   Size: {} bytes", metadata.size);
    println!("   Repository count: {}", metadata.repo_count);
    println!("   Group count: {}", metadata.group_count);

    // 6. Filter manifest by groups
    println!("6. Filtering manifest by groups...");
    let core_manifest = store.filter_manifest_by_groups(&manifest, &["core".to_string()])?;
    println!(
        "   Filtered manifest has {} repositories",
        core_manifest.repos.len()
    );

    // 7. Create test files for file operations
    println!("7. Creating test files for file operations...");
    let core_dir = workspace_root.join("core");
    tokio::fs::create_dir_all(&core_dir).await?;
    tokio::fs::create_dir_all(core_dir.join("config")).await?;
    tokio::fs::create_dir_all(core_dir.join("scripts")).await?;
    tokio::fs::write(
        core_dir.join("config/default.yml"),
        "# Default config\nkey: value",
    )
    .await?;
    tokio::fs::write(
        core_dir.join("scripts/build.sh"),
        "#!/bin/bash\necho 'Building...'",
    )
    .await?;

    let utils_dir = workspace_root.join("utils");
    tokio::fs::create_dir_all(&utils_dir).await?;
    tokio::fs::create_dir_all(utils_dir.join("bin")).await?;
    tokio::fs::write(
        utils_dir.join("bin/utils"),
        "#!/bin/bash\necho 'Utils tool'",
    )
    .await?;

    // 8. Process file copy operations
    println!("8. Processing file copy operations...");
    let copy_results = store
        .process_copy_operations(&manifest, workspace_root)
        .await?;
    for result in &copy_results {
        if result.success {
            println!(
                "   ✓ Copied {} to {}",
                result.source.display(),
                result.destination.display()
            );
        } else {
            println!(
                "   ✗ Failed to copy {}: {}",
                result.source.display(),
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
    }

    // 9. Process symlink operations
    println!("9. Processing symlink operations...");
    let symlink_results = store
        .process_symlink_operations(&manifest, workspace_root)
        .await?;
    for result in &symlink_results {
        if result.success {
            println!(
                "   ✓ Created symlink {} -> {}",
                result.source.display(),
                result.destination.display()
            );
        } else {
            println!(
                "   ✗ Failed to create symlink {}: {}",
                result.source.display(),
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
    }

    // 10. Process all file operations at once
    println!("10. Processing all file operations...");
    let all_results = store
        .process_all_file_operations(&manifest, workspace_root)
        .await?;
    let successful_ops = all_results.iter().filter(|r| r.success).count();
    let failed_ops = all_results.iter().filter(|r| !r.success).count();
    println!("    Total operations: {}", all_results.len());
    println!("    Successful: {}", successful_ops);
    println!("    Failed: {}", failed_ops);

    // 11. Update manifest and demonstrate backup
    println!("11. Updating manifest and demonstrating backup...");
    let mut updated_manifest = manifest.clone();
    updated_manifest.default_branch = Some("develop".to_string());
    store
        .write_manifest(&manifest_path, &updated_manifest)
        .await?;
    println!("    Manifest updated with backup created");

    // 12. List files to show backups
    println!("12. Listing workspace files...");
    let mut entries = tokio::fs::read_dir(workspace_root).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        if name.to_string_lossy().contains("manifest") {
            println!("    {}", name.to_string_lossy());
        }
    }

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
