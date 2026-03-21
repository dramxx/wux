use crate::prompt;
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(path: &str, dry_run: bool, skip_prompt: bool) -> Result<()> {
    let path = Path::new(path);
    let abs_path = resolve_path(path)?;

    if !abs_path.exists() {
        println!(
            "{} Nothing at '{}' — already gone?",
            "⚠ ".yellow(),
            path.display()
        );
        return Ok(());
    }

    let cwd = std::env::current_dir()?.canonicalize()?;
    if abs_path == cwd {
        anyhow::bail!("Refusing to nuke your current working directory.");
    }

    if let Some(parent) = abs_path.parent() {
        if parent == abs_path {
            anyhow::bail!("Refusing to nuke a filesystem root. That would be bad.");
        }
    }

    let path_str = abs_path.to_string_lossy().to_lowercase();
    if path_str == "c:\\" || path_str == "c:/" {
        anyhow::bail!("Refusing to nuke a filesystem root. That would be bad.");
    }

    let is_dir = abs_path.is_dir();
    let (file_count, dir_count) = if is_dir && dry_run {
        count_contents(&abs_path)?
    } else {
        (0, 0)
    };

    if dry_run {
        if is_dir {
            println!(
                "{} Would delete directory: {} ({} files, {} subdirectories)",
                "→".cyan(),
                abs_path.display(),
                file_count,
                dir_count
            );
        } else {
            println!("{} Would delete file: {}", "→".cyan(), abs_path.display());
        }
        return Ok(());
    }

    if !skip_prompt {
        let confirmed = if is_dir {
            prompt::confirm(&format!(
                "Do you really want to remove '{}' and all its contents? [y/N]",
                path.display()
            ))
        } else {
            prompt::confirm(&format!(
                "Do you really want to delete the file '{}'? [y/N]",
                path.display()
            ))
        };

        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
    }

    if is_dir {
        fs::remove_dir_all(&abs_path).context("Failed to delete directory")?;
    } else {
        fs::remove_file(&abs_path).context("Failed to delete file")?;
    }

    println!("{} Nuked. {} is gone.", "✓".green(), abs_path.display());
    Ok(())
}

fn resolve_path(path: &Path) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;

    if path.is_absolute() {
        if path.exists() {
            return path.canonicalize().context("Failed to resolve path");
        }
        return Ok(path.to_path_buf());
    }

    let abs = cwd.join(path);
    if abs.exists() {
        abs.canonicalize().context("Failed to resolve path")
    } else {
        Ok(abs)
    }
}

fn count_contents(path: &std::path::Path) -> Result<(usize, usize)> {
    let mut file_count = 0;
    let mut dir_count = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                dir_count += 1;
                let (fc, dc) = count_contents(&entry.path())?;
                file_count += fc;
                dir_count += dc;
            } else {
                file_count += 1;
            }
        }
    }

    Ok((file_count, dir_count))
}
