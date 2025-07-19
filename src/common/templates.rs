/// Templates module for embedded YAML templates
/// This module provides access to embedded template files using include_str! macro

/// Get the default wmgr.yaml template content
/// The template is embedded at compile time using include_str! macro
pub fn get_wmgr_template() -> &'static str {
    include_str!("../../templates/wmgr.yaml")
}

/// Template replacement functionality
pub struct TemplateProcessor;

impl TemplateProcessor {
    /// Create a new template processor
    pub fn new() -> Self {
        Self
    }
    
    /// Process template with optional replacements
    /// For now, returns the template as-is, but can be extended for variable substitution
    pub fn process(&self, template: &str, _replacements: Option<std::collections::HashMap<String, String>>) -> String {
        // Future enhancement: implement variable substitution like {{workspace_name}}, {{description}}, etc.
        template.to_string()
    }
    
    /// Get the default wmgr.yaml template with optional processing
    pub fn get_default_wmgr_template(&self) -> String {
        let template = get_wmgr_template();
        self.process(template, None)
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_wmgr_template() {
        let template = get_wmgr_template();
        assert!(!template.is_empty());
        assert!(template.contains("wmgr configuration file template"));
    }

    #[test]
    fn test_template_processor() {
        let processor = TemplateProcessor::new();
        let template = processor.get_default_wmgr_template();
        assert!(!template.is_empty());
        assert!(template.contains("groups:"));
        assert!(template.contains("repositories:"));
    }
}