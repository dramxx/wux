use anyhow::{bail, Result};
use colored::Colorize;
use jwalk::{Parallelism, WalkDir};
use std::path::{Path, PathBuf};

const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "__pycache__",
    ".svn",
    ".hg",
];

pub fn run(file_name: &str) -> Result<()> {
    run_with_ignores(file_name, None, false)
}

pub fn run_with_ignores(
    file_name: &str,
    custom_ignores: Option<Vec<String>>,
    no_ignore: bool,
) -> Result<()> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        bail!("Please provide a file name to search for.");
    }

    let ignore_patterns: Vec<String> = if no_ignore {
        Vec::new()
    } else {
        match custom_ignores {
            Some(custom) => custom,
            None => DEFAULT_IGNORE_PATTERNS.iter().map(|s| s.to_string()).collect(),
        }
    };

    println!("{} Searching for '{}' ...", "→".cyan(), trimmed);

    let matches = find_in_roots(trimmed, &ignore_patterns);

    if matches.is_empty() {
        println!("{} No file named '{}' was found.", "⚠ ".yellow(), trimmed);
        return Ok(());
    }

    for path in matches {
        println!("{}", path.display());
    }

    Ok(())
}

fn find_in_roots(file_name: &str, ignore_patterns: &[String]) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    for root in search_roots() {
        find_in_path(&root, file_name, ignore_patterns, &mut matches);
    }
    matches
}

fn find_in_path(root: &Path, file_name: &str, ignore_patterns: &[String], matches: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let parallelism = Parallelism::RayonNewPool(
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    );

    for entry in WalkDir::new(root)
        .parallelism(parallelism)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if should_ignore(&path, ignore_patterns) {
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if file_name_matches(name, file_name) {
            matches.push(path.to_path_buf());
        }
    }
}

fn should_ignore(path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                for pattern in patterns {
                    if name_str == pattern {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(windows)]
fn file_name_matches(candidate: &str, requested: &str) -> bool {
    candidate.eq_ignore_ascii_case(requested)
}

#[cfg(not(windows))]
fn file_name_matches(candidate: &str, requested: &str) -> bool {
    candidate == requested
}

#[cfg(windows)]
fn search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    for drive in b'A'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", drive as char));
        if root.exists() {
            roots.push(root);
        }
    }

    if roots.is_empty() {
        roots.push(PathBuf::from("C:\\"));
    }

    roots
}

#[cfg(not(windows))]
fn search_roots() -> Vec<PathBuf> {
    vec![PathBuf::from("/")]
}

#[cfg(test)]
mod tests {
    use super::{find_in_path, should_ignore};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn finds_file_in_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("one").join("two");
        std::fs::create_dir_all(&nested).unwrap();
        let target = nested.join("needle.txt");
        std::fs::write(&target, "x").unwrap();

        let mut matches = Vec::new();
        find_in_path(temp_dir.path(), "needle.txt", &[], &mut matches);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], target);
    }

    #[test]
    fn returns_no_matches_for_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("other.txt"), "x").unwrap();

        let mut matches = Vec::new();
        find_in_path(temp_dir.path(), "needle.txt", &[], &mut matches);

        assert!(matches.is_empty());
    }

    #[test]
    fn ignores_node_modules_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let target = node_modules.join("package.json");
        std::fs::write(&target, "{}").unwrap();

        let mut matches = Vec::new();
        let default_patterns: Vec<String> = super::DEFAULT_IGNORE_PATTERNS.iter().map(|s| s.to_string()).collect();
        find_in_path(temp_dir.path(), "package.json", &default_patterns, &mut matches);

        assert!(matches.is_empty(), "Should not find files in node_modules");
    }

    #[test]
    fn ignores_git_directory_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        let target = git_dir.join("config");
        std::fs::write(&target, "x").unwrap();

        let mut matches = Vec::new();
        let default_patterns: Vec<String> = super::DEFAULT_IGNORE_PATTERNS.iter().map(|s| s.to_string()).collect();
        find_in_path(temp_dir.path(), "config", &default_patterns, &mut matches);

        assert!(matches.is_empty(), "Should not find files in .git");
    }

    #[test]
    fn no_ignore_finds_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let target = node_modules.join("package.json");
        std::fs::write(&target, "{}").unwrap();

        let mut matches = Vec::new();
        find_in_path(temp_dir.path(), "package.json", &[], &mut matches);

        assert_eq!(matches.len(), 1, "Should find files when ignoring is disabled");
    }

    #[test]
    fn custom_ignore_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let custom_dir = temp_dir.path().join("skip_this");
        std::fs::create_dir_all(&custom_dir).unwrap();
        let target = custom_dir.join("secret.txt");
        std::fs::write(&target, "x").unwrap();

        let mut matches = Vec::new();
        let patterns = vec!["skip_this".to_string()];
        find_in_path(temp_dir.path(), "secret.txt", &patterns, &mut matches);

        assert!(matches.is_empty(), "Should not find files in custom ignored directory");
    }

    #[test]
    fn should_ignore_returns_true_for_matching_pattern() {
        let path = PathBuf::from("/home/user/node_modules/package.json");
        let patterns = vec!["node_modules".to_string()];
        assert!(should_ignore(path.as_path(), &patterns));
    }

    #[test]
    fn should_ignore_returns_false_for_non_matching_pattern() {
        let path = PathBuf::from("/home/user/src/index.js");
        let patterns = vec!["node_modules".to_string()];
        assert!(!should_ignore(path.as_path(), &patterns));
    }

    #[test]
    fn should_ignore_returns_false_for_empty_patterns() {
        let path = PathBuf::from("/home/user/node_modules/package.json");
        let patterns: Vec<String> = vec![];
        assert!(!should_ignore(path.as_path(), &patterns));
    }
}
