use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("No projections.json found (looked for .projections.json and .zed/projections.json)")]
    NotFound,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectionConfig {
    /// Primary alternate file pattern
    pub alternate: Option<String>,
    /// Array of related file patterns
    pub related: Option<Vec<String>>,
    /// File type identifier for categorization
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    /// Template lines for new file creation
    pub template: Option<Vec<String>>,
    /// Search pattern for auto-scroll in related files
    pub define: Option<String>,
    /// Additional properties (vim-projectionist compatibility)
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Map of glob patterns to their configurations
pub type Projections = HashMap<String, ProjectionConfig>;

/// Load projections from a .projections.json file
pub fn load_projections(path: &Path) -> Result<Projections, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound);
    }
    let content = fs::read_to_string(path)?;
    let projections: Projections = serde_json::from_str(&content)?;
    Ok(projections)
}

/// Configuration file paths to check, in priority order
pub const CONFIG_PATHS: &[&str] = &[".projections.json", ".zed/projections.json"];

/// Load projections from a project root directory
/// Checks for config files in priority order: .projections.json, .zed/projections.json
pub fn load_projections_from_root(project_root: &Path) -> Result<Projections, ConfigError> {
    for config_path in CONFIG_PATHS {
        let full_path = project_root.join(config_path);
        if full_path.exists() {
            return load_projections(&full_path);
        }
    }
    Err(ConfigError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_projections() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".projections.json");

        let config_content = r#"{
            "src/*.ts": {
                "alternate": "test/{}.test.ts",
                "type": "source"
            },
            "test/*.test.ts": {
                "alternate": "src/{}.ts",
                "type": "test"
            }
        }"#;

        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let projections = load_projections_from_root(temp_dir.path()).unwrap();
        assert_eq!(projections.len(), 2);
        assert!(projections.contains_key("src/*.ts"));
        assert!(projections.contains_key("test/*.test.ts"));
    }

    #[test]
    fn test_projection_config_parsing() {
        let json = r#"{
            "alternate": "test/{}.test.ts",
            "related": ["docs/{}.md"],
            "type": "source",
            "template": ["export class {} {", "}"],
            "define": "class {}"
        }"#;

        let config: ProjectionConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.alternate, Some("test/{}.test.ts".to_string()));
        assert_eq!(config.related, Some(vec!["docs/{}.md".to_string()]));
        assert_eq!(config.file_type, Some("source".to_string()));
        assert_eq!(
            config.template,
            Some(vec!["export class {} {".to_string(), "}".to_string()])
        );
        assert_eq!(config.define, Some("class {}".to_string()));
    }

    #[test]
    fn test_load_projections_from_zed_directory() {
        let temp_dir = TempDir::new().unwrap();
        let zed_dir = temp_dir.path().join(".zed");
        fs::create_dir(&zed_dir).unwrap();

        let config_content = r#"{
            "src/*.rs": {
                "alternate": "tests/{}.rs",
                "type": "source"
            }
        }"#;

        fs::write(zed_dir.join("projections.json"), config_content).unwrap();

        let projections = load_projections_from_root(temp_dir.path()).unwrap();
        assert_eq!(projections.len(), 1);
        assert!(projections.contains_key("src/*.rs"));
    }

    #[test]
    fn test_root_projections_takes_priority_over_zed() {
        let temp_dir = TempDir::new().unwrap();

        // Create .zed/projections.json first
        let zed_dir = temp_dir.path().join(".zed");
        fs::create_dir(&zed_dir).unwrap();
        fs::write(
            zed_dir.join("projections.json"),
            r#"{"zed/*.ts": {"type": "zed"}}"#,
        )
        .unwrap();

        // Create .projections.json (should take priority)
        fs::write(
            temp_dir.path().join(".projections.json"),
            r#"{"root/*.ts": {"type": "root"}}"#,
        )
        .unwrap();

        let projections = load_projections_from_root(temp_dir.path()).unwrap();
        assert_eq!(projections.len(), 1);
        assert!(projections.contains_key("root/*.ts"));
        assert!(!projections.contains_key("zed/*.ts"));
    }
}
