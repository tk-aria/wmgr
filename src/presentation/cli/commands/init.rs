use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use std::env;
use crate::common::templates::TemplateProcessor;

/// Initialize a new wmgr workspace
pub struct InitCommand {
    /// Path where to create the wmgr.yaml file
    pub path: Option<PathBuf>,
    /// Force overwrite existing file
    pub force: bool,
    /// Use manifest.yaml instead of wmgr.yaml
    pub use_manifest_name: bool,
}

impl InitCommand {
    pub fn new(path: Option<PathBuf>, force: bool, use_manifest_name: bool) -> Self {
        Self {
            path,
            force,
            use_manifest_name,
        }
    }

    /// Execute the init command
    pub async fn execute(&self) -> Result<()> {
        let current_dir = env::current_dir()?;
        let target_dir = self.path.as_ref().unwrap_or(&current_dir);
        
        // Determine filename based on use_manifest_name flag
        let filename = if self.use_manifest_name {
            "manifest.yaml"
        } else {
            "wmgr.yaml"
        };
        
        let target_file = target_dir.join(filename);
        
        // Check if file already exists
        if target_file.exists() && !self.force {
            return Err(anyhow::anyhow!(
                "File {} already exists. Use --force to overwrite.",
                target_file.display()
            ));
        }
        
        // Create target directory if it doesn't exist
        if let Some(parent) = target_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Generate template content
        let processor = TemplateProcessor::new();
        let template_content = processor.get_default_wmgr_template();
        
        // Write template to file
        fs::write(&target_file, template_content)?;
        
        println!("✅ Successfully created {} template file", filename);
        println!("📁 Location: {}", target_file.display());
        println!();
        println!("📝 Next steps:");
        println!("   1. Edit the {} file to configure your repositories", filename);
        println!("   2. Run 'wmgr sync' to clone and sync repositories");
        println!("   3. Use 'wmgr status' to check repository status");
        
        Ok(())
    }
}