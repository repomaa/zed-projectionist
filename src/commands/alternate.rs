use crate::config::load_projections_from_root;
use crate::matcher::Matcher;
use crate::project::find_project_root_with_projections;
use crate::transform::expand_placeholders;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlternateError {
    #[error("No project root found (no .projections.json in parent directories)")]
    NoProjectRoot,
    #[error("Failed to load projections: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

/// Result of finding alternate files
#[derive(Debug)]
pub struct AlternateResult {
    /// List of potential alternate file paths (absolute)
    pub paths: Vec<PathBuf>,
    /// Which paths actually exist on the filesystem
    pub existing: Vec<PathBuf>,
    /// The project root used
    pub project_root: PathBuf,
}

/// Find alternate files for a given file path
pub fn find_alternate(file_path: &Path) -> Result<AlternateResult, AlternateError> {
    // Find project root with .projections.json
    let project_root = find_project_root_with_projections(file_path)
        .ok_or(AlternateError::NoProjectRoot)?;

    // Load projections
    let projections = load_projections_from_root(&project_root)?;

    // Create matcher and match file
    let matcher = Matcher::new(projections);
    let matches = matcher.match_absolute_path(file_path, &project_root);

    let mut all_paths = Vec::new();
    let mut existing_paths = Vec::new();

    for m in matches {
        if let Some(alternate_pattern) = &m.config.alternate {
            let expanded = expand_placeholders(alternate_pattern, &m.placeholders);
            let full_path = project_root.join(&expanded);

            if !all_paths.contains(&full_path) {
                all_paths.push(full_path.clone());

                if full_path.exists() {
                    existing_paths.push(full_path);
                }
            }
        }
    }

    Ok(AlternateResult {
        paths: all_paths,
        existing: existing_paths,
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

        // Create .projections.json
        let config = r#"{
            "src/*.ts": {
                "alternate": "test/{}.test.ts"
            },
            "test/*.test.ts": {
                "alternate": "src/{}.ts"
            }
        }"#;
        fs::write(root.join(".projections.json"), config).unwrap();

        // Create directories
        fs::create_dir(root.join("src")).unwrap();
        fs::create_dir(root.join("test")).unwrap();

        // Create source file
        fs::write(root.join("src/utils.ts"), "// source").unwrap();

        // Create test file
        fs::write(root.join("test/utils.test.ts"), "// test").unwrap();

        temp_dir
    }

    #[test]
    fn test_find_alternate_source_to_test() {
        let temp_dir = setup_test_project();
        let source_file = temp_dir.path().join("src/utils.ts");

        let result = find_alternate(&source_file).unwrap();

        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.existing.len(), 1);
        assert!(result.paths[0].ends_with("test/utils.test.ts"));
    }

    #[test]
    fn test_find_alternate_test_to_source() {
        let temp_dir = setup_test_project();
        let test_file = temp_dir.path().join("test/utils.test.ts");

        let result = find_alternate(&test_file).unwrap();

        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.existing.len(), 1);
        assert!(result.paths[0].ends_with("src/utils.ts"));
    }

    #[test]
    fn test_find_alternate_nonexistent() {
        let temp_dir = setup_test_project();
        let source_file = temp_dir.path().join("src/newfile.ts");
        fs::write(&source_file, "// new file").unwrap();

        let result = find_alternate(&source_file).unwrap();

        // Path should be computed but not exist
        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.existing.len(), 0);
        assert!(result.paths[0].ends_with("test/newfile.test.ts"));
    }
}
