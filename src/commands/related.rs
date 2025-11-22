use crate::config::load_projections_from_root;
use crate::matcher::Matcher;
use crate::project::find_project_root_with_projections;
use crate::transform::expand_placeholders;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelatedError {
    #[error("No project root found (no .projections.json in parent directories)")]
    NoProjectRoot,
    #[error("Failed to load projections: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

/// Information about a related file
#[derive(Debug, Clone)]
pub struct RelatedFile {
    /// Full path to the related file
    pub path: PathBuf,
    /// Optional search pattern for auto-scroll (from `define`)
    pub search_pattern: Option<String>,
    /// Whether the file exists
    pub exists: bool,
}

/// Result of finding related files
#[derive(Debug)]
pub struct RelatedResult {
    /// List of related files with metadata
    pub files: Vec<RelatedFile>,
    /// The project root used
    pub project_root: PathBuf,
}

/// Find related files for a given file path
pub fn find_related(file_path: &Path) -> Result<RelatedResult, RelatedError> {
    // Find project root with .projections.json
    let project_root =
        find_project_root_with_projections(file_path).ok_or(RelatedError::NoProjectRoot)?;

    // Load projections
    let projections = load_projections_from_root(&project_root)?;

    // Create matcher and match file
    let matcher = Matcher::new(projections);
    let matches = matcher.match_absolute_path(file_path, &project_root);

    let mut related_files = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for m in matches {
        // Process related patterns
        if let Some(related_patterns) = &m.config.related {
            for pattern in related_patterns {
                let expanded = expand_placeholders(pattern, &m.placeholders);
                let full_path = project_root.join(&expanded);

                if seen_paths.insert(full_path.clone()) {
                    let search_pattern = m
                        .config
                        .define
                        .as_ref()
                        .map(|d| expand_placeholders(d, &m.placeholders));

                    related_files.push(RelatedFile {
                        exists: full_path.exists(),
                        path: full_path,
                        search_pattern,
                    });
                }
            }
        }

        // Also include alternate as a related file
        if let Some(alternate_pattern) = &m.config.alternate {
            let expanded = expand_placeholders(alternate_pattern, &m.placeholders);
            let full_path = project_root.join(&expanded);

            if seen_paths.insert(full_path.clone()) {
                let search_pattern = m
                    .config
                    .define
                    .as_ref()
                    .map(|d| expand_placeholders(d, &m.placeholders));

                related_files.push(RelatedFile {
                    exists: full_path.exists(),
                    path: full_path,
                    search_pattern,
                });
            }
        }
    }

    Ok(RelatedResult {
        files: related_files,
        project_root,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = r#"{
            "src/components/*.tsx": {
                "alternate": "src/components/{}.test.tsx",
                "related": ["src/components/{}.module.css", "src/components/{}.stories.tsx"],
                "define": "export const {}"
            }
        }"#;
        fs::write(root.join(".projections.json"), config).unwrap();

        // Create directories
        fs::create_dir_all(root.join("src/components")).unwrap();

        // Create files
        fs::write(root.join("src/components/Button.tsx"), "export const Button").unwrap();
        fs::write(root.join("src/components/Button.module.css"), ".button {}").unwrap();

        temp_dir
    }

    #[test]
    fn test_find_related() {
        let temp_dir = setup_test_project();
        let source_file = temp_dir.path().join("src/components/Button.tsx");

        let result = find_related(&source_file).unwrap();

        // Should find: Button.module.css (exists), Button.stories.tsx (not exists), Button.test.tsx (not exists)
        assert_eq!(result.files.len(), 3);

        let css_file = result
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("module.css"));
        assert!(css_file.is_some());
        assert!(css_file.unwrap().exists);

        // Check search pattern is expanded
        for file in &result.files {
            assert_eq!(file.search_pattern, Some("export const Button".to_string()));
        }
    }
}
