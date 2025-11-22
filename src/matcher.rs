use crate::config::{ProjectionConfig, Projections};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Result of matching a file against projections
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The glob pattern that matched
    pub glob: String,
    /// The configuration for this pattern
    pub config: ProjectionConfig,
    /// Extracted placeholders ('' key contains the stem)
    pub placeholders: HashMap<String, String>,
}

/// Handles matching files against projection patterns
pub struct Matcher {
    projections: Projections,
}

impl Matcher {
    pub fn new(projections: Projections) -> Self {
        Self { projections }
    }

    /// Convert a projectionist glob pattern to a regex that captures the stem
    ///
    /// The `*` in glob patterns captures the "stem" - the variable portion
    /// between the prefix and suffix.
    ///
    /// Examples:
    /// - `src/*.ts` → `^src/(.*)\.ts$`
    /// - `app/models/*.rb` → `^app/models/(.*)\.rb$`
    fn glob_to_regex(glob: &str) -> Regex {
        let star_index = glob.find('*');

        match star_index {
            None => {
                // No *, escape and return exact match
                let escaped = regex::escape(glob);
                Regex::new(&format!("^{}$", escaped)).unwrap()
            }
            Some(index) => {
                let prefix = &glob[..index];
                let suffix = &glob[index + 1..];

                let escaped_prefix = regex::escape(prefix);
                let escaped_suffix = regex::escape(suffix);

                let pattern = format!("^{}(.*){}$", escaped_prefix, escaped_suffix);
                Regex::new(&pattern).unwrap()
            }
        }
    }

    /// Match a file path against all projections and return matching results
    ///
    /// The file path should be relative to the project root.
    pub fn match_file(&self, relative_path: &str) -> Vec<MatchResult> {
        // Normalize path separators to forward slashes
        let normalized_path = relative_path.replace('\\', "/");

        let mut matches = Vec::new();

        for (glob, config) in &self.projections {
            let regex = Self::glob_to_regex(glob);

            if let Some(captures) = regex.captures(&normalized_path) {
                let mut placeholders = HashMap::new();

                // The first capture group contains the "stem"
                if let Some(stem) = captures.get(1) {
                    placeholders.insert(String::new(), stem.as_str().to_string());
                } else {
                    placeholders.insert(String::new(), String::new());
                }

                matches.push(MatchResult {
                    glob: glob.clone(),
                    config: config.clone(),
                    placeholders,
                });
            }
        }

        matches
    }

    /// Match an absolute file path using the project root
    pub fn match_absolute_path(&self, file_path: &Path, project_root: &Path) -> Vec<MatchResult> {
        if let Ok(relative) = file_path.strip_prefix(project_root) {
            let relative_str = relative.to_string_lossy().replace('\\', "/");
            self.match_file(&relative_str)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProjectionConfig;

    fn create_test_projections() -> Projections {
        let mut projections = Projections::new();

        projections.insert(
            "src/*.ts".to_string(),
            ProjectionConfig {
                alternate: Some("test/{}.test.ts".to_string()),
                file_type: Some("source".to_string()),
                ..Default::default()
            },
        );

        projections.insert(
            "test/*.test.ts".to_string(),
            ProjectionConfig {
                alternate: Some("src/{}.ts".to_string()),
                file_type: Some("test".to_string()),
                ..Default::default()
            },
        );

        projections.insert(
            "app/models/*.rb".to_string(),
            ProjectionConfig {
                alternate: Some("spec/models/{}_spec.rb".to_string()),
                file_type: Some("model".to_string()),
                ..Default::default()
            },
        );

        projections
    }

    #[test]
    fn test_glob_to_regex_simple() {
        let regex = Matcher::glob_to_regex("src/*.ts");
        assert!(regex.is_match("src/utils.ts"));
        assert!(regex.is_match("src/foo/bar.ts"));
        assert!(!regex.is_match("test/utils.ts"));
        assert!(!regex.is_match("src/utils.tsx"));
    }

    #[test]
    fn test_glob_to_regex_captures_stem() {
        let regex = Matcher::glob_to_regex("src/*.ts");
        let captures = regex.captures("src/utils.ts").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "utils");

        let captures = regex.captures("src/foo/bar.ts").unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "foo/bar");
    }

    #[test]
    fn test_match_file() {
        let projections = create_test_projections();
        let matcher = Matcher::new(projections);

        let matches = matcher.match_file("src/utils.ts");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].glob, "src/*.ts");
        assert_eq!(matches[0].placeholders.get("").unwrap(), "utils");
    }

    #[test]
    fn test_match_file_nested_path() {
        let projections = create_test_projections();
        let matcher = Matcher::new(projections);

        let matches = matcher.match_file("src/components/Button.ts");
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].placeholders.get("").unwrap(),
            "components/Button"
        );
    }

    #[test]
    fn test_match_file_test_pattern() {
        let projections = create_test_projections();
        let matcher = Matcher::new(projections);

        let matches = matcher.match_file("test/utils.test.ts");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].glob, "test/*.test.ts");
        assert_eq!(matches[0].placeholders.get("").unwrap(), "utils");
    }

    #[test]
    fn test_match_file_no_match() {
        let projections = create_test_projections();
        let matcher = Matcher::new(projections);

        let matches = matcher.match_file("other/file.js");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_match_file_ruby_model() {
        let projections = create_test_projections();
        let matcher = Matcher::new(projections);

        let matches = matcher.match_file("app/models/user.rb");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].glob, "app/models/*.rb");
        assert_eq!(matches[0].placeholders.get("").unwrap(), "user");
    }
}
