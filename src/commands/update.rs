use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_OWNER: &str = "dramxx";
const REPO_NAME: &str = "wux";

pub fn run() -> Result<()> {
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

    std::fs::rename(&exe_path, &backup_path).context("Failed to backup old exe")?;

    let download_url = format!(
        "https://github.com/{}/{}/releases/download/v{}/wux.exe",
        REPO_OWNER, REPO_NAME, version
    );

    let output = Command::new("curl")
        .args(["-L", "-o", exe_path.to_str().unwrap(), &download_url])
        .output()
        .context("Failed to download update")?;

    if !output.status.success() {
        std::fs::rename(&backup_path, &exe_path).ok();
        return Ok(false);
    }

    std::fs::remove_file(backup_path).ok();
    Ok(true)
}
