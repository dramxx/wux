use anyhow::{Context, Result};
use std::process::Command;

pub fn find_pid_on_port(port: u16) -> Result<Option<(u32, String)>> {
    let output = Command::new("netstat")
        .args(["-ano"])
        .output()
        .context("Failed to run netstat")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let port_str = format!(":{}", port);

    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.contains(&port_str) || !trimmed.contains("LISTENING") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let local_addr = parts[1];
        if !local_addr.ends_with(&port_str) {
            continue;
        }

        if let Some(pid_str) = parts.last() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                let process_name = get_process_name(pid).unwrap_or_else(|| "unknown".to_string());
                return Ok(Some((pid, process_name)));
            }
        }
    }

    Ok(None)
}

fn get_process_name(pid: u32) -> Option<String> {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?;
    first_line
        .split(',')
        .next()
        .map(|s| s.trim_matches('"').to_string())
}

pub fn kill_pid(pid: u32) -> Result<()> {
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
        .context("Failed to run taskkill")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Access is denied") || stderr.contains("Permission") {
            anyhow::bail!("Permission denied. Try running wux as administrator.");
        }
        if stderr.contains("not found") || stderr.contains("not find") {
            anyhow::bail!("Process {} not found - it may have already exited.", pid);
        }
        anyhow::bail!("Failed to kill process {}: {}", pid, stderr.trim());
    }

    Ok(())
}
