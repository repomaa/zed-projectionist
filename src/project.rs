use std::path::{Path, PathBuf};

use crate::config::CONFIG_PATHS;

/// Project markers that indicate a project root
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "setup.py",
    "pom.xml",
    "build.gradle",
    "Gemfile",
];

/// Check if any projections config file exists in the given directory
fn has_projections_config(dir: &Path) -> bool {
    CONFIG_PATHS.iter().any(|p| dir.join(p).exists())
}

/// Find the project root by walking up the directory tree
///
/// Priority:
/// 1. Directory containing a projections config file (highest priority)
/// 2. Directory containing a project marker (.git, package.json, etc.)
pub fn find_project_root(starting_path: &Path) -> Option<PathBuf> {
    let starting_dir = if starting_path.is_file() {
        starting_path.parent()?
    } else {
        starting_path
    };

    let mut current = starting_dir;
    let mut marker_root: Option<PathBuf> = None;

    loop {
        // Check for projections config first (highest priority)
        if has_projections_config(current) {
            return Some(current.to_path_buf());
        }

        // Check for project markers
        if marker_root.is_none() {
            for marker in PROJECT_MARKERS {
                if current.join(marker).exists() {
                    marker_root = Some(current.to_path_buf());
                    break;
                }
            }
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) if parent != current => {
                current = parent;
            }
            _ => break,
        }
    }

    // If we found a marker but no projections config, use the marker root
    marker_root
}

/// Find the project root, requiring a projections config to exist
pub fn find_project_root_with_projections(starting_path: &Path) -> Option<PathBuf> {
    let starting_dir = if starting_path.is_file() {
        starting_path.parent()?
    } else {
        starting_path
    };

    let mut current = starting_dir;

    loop {
        if has_projections_config(current) {
            return Some(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) if parent != current => {
                current = parent;
            }
            _ => break,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_project_root_with_projections() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .projections.json
        fs::write(project_root.join(".projections.json"), "{}").unwrap();

        // Create nested directory
        let nested = project_root.join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, project_root);
    }

    #[test]
    fn test_find_project_root_with_git() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .git directory
        fs::create_dir(project_root.join(".git")).unwrap();

        // Create nested directory
        let nested = project_root.join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, project_root);
    }

    #[test]
    fn test_find_project_root_projections_takes_priority() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create both .git and .projections.json in different directories
        fs::create_dir(project_root.join(".git")).unwrap();

        let subproject = project_root.join("subproject");
        fs::create_dir(&subproject).unwrap();
        fs::write(subproject.join(".projections.json"), "{}").unwrap();

        let nested = subproject.join("src");
        fs::create_dir(&nested).unwrap();

        // Starting from nested, should find subproject (with .projections.json)
        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, subproject);
    }

    #[test]
    fn test_find_project_root_with_projections_only() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .git but no .projections.json
        fs::create_dir(project_root.join(".git")).unwrap();

        let nested = project_root.join("src");
        fs::create_dir(&nested).unwrap();

        // Should return None when requiring projections
        let found = find_project_root_with_projections(&nested);
        assert!(found.is_none());

        // Now add .projections.json
        fs::write(project_root.join(".projections.json"), "{}").unwrap();

        let found = find_project_root_with_projections(&nested).unwrap();
        assert_eq!(found, project_root);
    }

    #[test]
    fn test_find_project_root_with_zed_projections() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create .zed/projections.json
        fs::create_dir(project_root.join(".zed")).unwrap();
        fs::write(project_root.join(".zed/projections.json"), "{}").unwrap();

        // Create nested directory
        let nested = project_root.join("src").join("components");
        fs::create_dir_all(&nested).unwrap();

        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, project_root);

        let found = find_project_root_with_projections(&nested).unwrap();
        assert_eq!(found, project_root);
    }

    #[test]
    fn test_root_projections_takes_priority_over_zed() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create both .projections.json and .zed/projections.json
        fs::write(project_root.join(".projections.json"), r#"{"root": true}"#).unwrap();
        fs::create_dir(project_root.join(".zed")).unwrap();
        fs::write(project_root.join(".zed/projections.json"), r#"{"zed": true}"#).unwrap();

        // Both should find project root (priority doesn't matter for root finding)
        let nested = project_root.join("src");
        fs::create_dir(&nested).unwrap();

        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, project_root);
    }
}
