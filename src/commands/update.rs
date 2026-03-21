use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_OWNER: &str = "dramxx";
const REPO_NAME: &str = "wux";

pub fn run() -> Result<()> {
    if let Some(repo_root) = find_repo_root(&std::env::current_dir()?) {
        return update_from_repo(&repo_root);
    }

    println!("{} Checking for updates...", "→".cyan());

    let latest = get_latest_version()?;

    if latest == CURRENT_VERSION {
        println!("{} wux is up to date (v{})", "✓".green(), CURRENT_VERSION);
        return Ok(());
    }

    if is_newer(&latest, CURRENT_VERSION) {
        println!(
            "{} New version available: v{} -> v{}",
            "→".cyan(),
            CURRENT_VERSION,
            latest.yellow()
        );
        println!("Downloading v{}...", latest);

        if download_update(&latest)? {
            println!(
                "{} Updated to v{}. Restart to use new version.",
                "✓".green(),
                latest
            );
        } else {
            println!(
                "{} Update failed. Download manually from GitHub releases.",
                "✗".red()
            );
        }
    } else {
        println!("{} wux is up to date (v{})", "✓".green(), CURRENT_VERSION);
    }

    Ok(())
}

fn update_from_repo(repo_root: &Path) -> Result<()> {
    println!("{} Git repo detected. Pulling latest changes...", "→".cyan());

    let output = Command::new("git")
        .arg("pull")
        .current_dir(repo_root)
        .output()
        .context("Failed to run git pull")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git pull failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("{}", stdout.trim());
    }

    spawn_reinstall(repo_root)?;
    println!(
        "{} Pulled latest changes. Reinstall started. Restart to use the updated version.",
        "✓".green()
    );

    Ok(())
}

fn get_latest_version() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "-s",
            &format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                REPO_OWNER, REPO_NAME
            ),
        ])
        .output()
        .context("Failed to check for updates")?;

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse release info")?;

    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or(CURRENT_VERSION);

    Ok(tag.trim_start_matches('v').to_string())
}

fn is_newer(latest: &str, current: &str) -> bool {
    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }
    false
}

fn download_update(version: &str) -> Result<bool> {
    let exe_path = std::env::current_exe().context("Failed to get exe path")?;
    let backup_path = exe_path.with_extension("old");
    let asset_name = exe_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("Failed to determine executable file name")?;

    std::fs::rename(&exe_path, &backup_path).context("Failed to backup old exe")?;

    let download_url = format!(
        "https://github.com/{}/{}/releases/download/v{}/{}",
        REPO_OWNER, REPO_NAME, version, asset_name
    );

    let exe_path_str = exe_path.to_string_lossy().into_owned();

    let output = Command::new("curl")
        .args(["-L", "-o", &exe_path_str, &download_url])
        .output()
        .context("Failed to download update")?;

    if !output.status.success() {
        std::fs::rename(&backup_path, &exe_path).ok();
        return Ok(false);
    }

    std::fs::remove_file(backup_path).ok();
    Ok(true)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        if dir.join(".git").exists()
            && dir.join("Cargo.toml").exists()
            && dir.join("install").exists()
        {
            return Some(dir.to_path_buf());
        }
    }

    None
}

#[cfg(windows)]
fn spawn_reinstall(repo_root: &Path) -> Result<()> {
    let install_script = repo_root.join("install").join("install.ps1");
    let install_script = install_script.to_string_lossy().replace('"', "\"\"");
    let command = format!(
        "Start-Sleep -Seconds 1; & \"{}\" -Force",
        install_script
    );

    Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &command])
        .spawn()
        .context("Failed to start installer")?;

    Ok(())
}

#[cfg(unix)]
fn spawn_reinstall(repo_root: &Path) -> Result<()> {
    let install_script = repo_root.join("install").join("install.sh");
    let install_script = shell_escape(&install_script.to_string_lossy());
    let command = format!("sleep 1; {} --force", install_script);

    Command::new("sh")
        .args(["-c", &command])
        .spawn()
        .context("Failed to start installer")?;

    Ok(())
}

#[cfg(unix)]
fn shell_escape(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::find_repo_root;
    use tempfile::TempDir;

    #[test]
    fn finds_repo_root_from_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::create_dir(temp_dir.path().join(".git")).unwrap();
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname='wux'\nversion='0.1.0'\n").unwrap();
        std::fs::create_dir(temp_dir.path().join("install")).unwrap();
        let nested = temp_dir.path().join("src").join("commands");
        std::fs::create_dir_all(&nested).unwrap();

        let found = find_repo_root(&nested).unwrap();

        assert_eq!(found, temp_dir.path());
    }

    #[test]
    fn returns_none_outside_repo() {
        let temp_dir = TempDir::new().unwrap();

        assert!(find_repo_root(temp_dir.path()).is_none());
    }
}
