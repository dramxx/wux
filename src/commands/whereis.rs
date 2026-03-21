use anyhow::{bail, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(file_name: &str) -> Result<()> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        bail!("Please provide a file name to search for.");
    }

    println!("{} Searching for '{}' ...", "→".cyan(), trimmed);

    let matches = find_in_roots(trimmed);

    if matches.is_empty() {
        println!("{} No file named '{}' was found.", "⚠ ".yellow(), trimmed);
        return Ok(());
    }

    for path in matches {
        println!("{}", path.display());
    }

    Ok(())
}

fn find_in_roots(file_name: &str) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    for root in search_roots() {
        find_in_path(&root, file_name, &mut matches);
    }
    matches
}

fn find_in_path(root: &Path, file_name: &str, matches: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let mut stack = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let entry_path = entry.path();
            let metadata = match fs::symlink_metadata(&entry_path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                stack.push(entry_path);
                continue;
            }

            let name = match entry_path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };

            if file_name_matches(name, file_name) {
                matches.push(entry_path);
            }
        }
    }
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
    use super::find_in_path;
    use tempfile::TempDir;

    #[test]
    fn finds_file_in_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("one").join("two");
        std::fs::create_dir_all(&nested).unwrap();
        let target = nested.join("needle.txt");
        std::fs::write(&target, "x").unwrap();

        let mut matches = Vec::new();
        find_in_path(temp_dir.path(), "needle.txt", &mut matches);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], target);
    }

    #[test]
    fn returns_no_matches_for_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("other.txt"), "x").unwrap();

        let mut matches = Vec::new();
        find_in_path(temp_dir.path(), "needle.txt", &mut matches);

        assert!(matches.is_empty());
    }
}
