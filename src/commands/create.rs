use crate::config::load_projections_from_root;
use crate::matcher::Matcher;
use crate::project::find_project_root;
use crate::transform::expand_placeholders;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("No project root found")]
    NoProjectRoot,
    #[error("Failed to load projections: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("No template found for: {0}")]
    NoTemplateFound(String),
    #[error("Failed to create file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("File already exists: {0}")]
    FileExists(String),
}

/// Result of creating a file
#[derive(Debug)]
#[allow(dead_code)]
pub struct CreateResult {
    /// The path of the created file
    pub path: PathBuf,
    /// The content that was written
    pub content: String,
    /// The project root used
    pub project_root: PathBuf,
}

/// Create a new file from a template
///
/// The file path is matched against projections to find a template.
/// Parent directories are created if needed.
pub fn create_file(file_path: &Path, force: bool) -> Result<CreateResult, CreateError> {
    // Check if file already exists
    if file_path.exists() && !force {
        return Err(CreateError::FileExists(
            file_path.to_string_lossy().to_string(),
        ));
    }

    // Find project root (use parent of the file path)
    let parent = file_path.parent().unwrap_or(file_path);
    let project_root = find_project_root(parent).ok_or(CreateError::NoProjectRoot)?;

    // Load projections
    let projections = load_projections_from_root(&project_root)?;

    // Create matcher and match the (potential) file path
    let matcher = Matcher::new(projections);
    let matches = matcher.match_absolute_path(file_path, &project_root);

    // Find first match with a template
    for m in matches {
        if let Some(template_lines) = &m.config.template {
            // Expand placeholders in each template line
            let expanded_lines: Vec<String> = template_lines
                .iter()
                .map(|line| expand_placeholders(line, &m.placeholders))
                .collect();

            let content = expanded_lines.join("\n");

            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write the file
            fs::write(file_path, &content)?;

            return Ok(CreateResult {
                path: file_path.to_path_buf(),
                content,
                project_root,
            });
        }
    }

    Err(CreateError::NoTemplateFound(
        file_path.to_string_lossy().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = r#"{
            "src/*.ts": {
                "template": [
                    "export class {} {",
                    "  constructor() {",
                    "    // TODO: implement",
                    "  }",
                    "}"
                ]
            },
            "src/components/*.tsx": {
                "template": [
                    "import React from 'react';",
                    "",
                    "interface {}Props {}",
                    "",
                    "export const {}: React.FC<{}Props> = (props) => {",
                    "  return <div>{}</div>;",
                    "};"
                ]
            }
        }"#;
        fs::write(root.join(".projections.json"), config).unwrap();
        fs::create_dir(root.join("src")).unwrap();

        temp_dir
    }

    #[test]
    fn test_create_file_with_template() {
        let temp_dir = setup_test_project();
        let new_file = temp_dir.path().join("src/utils.ts");

        let result = create_file(&new_file, false).unwrap();

        assert!(new_file.exists());
        assert!(result.content.contains("export class utils"));
        assert!(result.content.contains("constructor()"));
    }

    #[test]
    fn test_create_file_creates_directories() {
        let temp_dir = setup_test_project();
        let new_file = temp_dir.path().join("src/components/Button.tsx");

        let result = create_file(&new_file, false).unwrap();

        assert!(new_file.exists());
        assert!(result.content.contains("export const Button"));
    }

    #[test]
    fn test_create_file_exists_error() {
        let temp_dir = setup_test_project();
        let existing_file = temp_dir.path().join("src/existing.ts");
        fs::write(&existing_file, "existing content").unwrap();

        let result = create_file(&existing_file, false);

        assert!(matches!(result, Err(CreateError::FileExists(_))));
    }

    #[test]
    fn test_create_file_force_overwrite() {
        let temp_dir = setup_test_project();
        let existing_file = temp_dir.path().join("src/existing.ts");
        fs::write(&existing_file, "existing content").unwrap();

        let result = create_file(&existing_file, true).unwrap();

        assert!(result.content.contains("export class existing"));
    }
}
