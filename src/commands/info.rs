use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;

    let mut total_size: u64 = 0;
    let mut file_count: usize = 0;
    let mut folder_count: usize = 0;
    let mut largest_path = cwd.clone();
    let mut largest_size: u64 = 0;
    let mut newest_path = cwd.clone();
    let mut newest_time: u64 = 0;
    let mut oldest_path = cwd.clone();
    let mut oldest_time: u64 = u64::MAX;

    traverse(
        &cwd,
        &mut total_size,
        &mut file_count,
        &mut folder_count,
        &mut largest_path,
        &mut largest_size,
        &mut newest_path,
        &mut newest_time,
        &mut oldest_path,
        &mut oldest_time,
    )?;

    let folder_name = cwd.to_string_lossy();
    println!("{} {}", "📁".cyan(), folder_name.white());

    println!(
        "   {:<12} {}",
        "Size".dimmed(),
        format_size(total_size).white()
    );
    println!(
        "   {:<12} {}",
        "Files".dimmed(),
        file_count.to_string().white()
    );
    println!(
        "   {:<12} {}",
        "Folders".dimmed(),
        folder_count.to_string().white()
    );

    let largest_name = largest_path
        .strip_prefix(&cwd)
        .unwrap_or(&largest_path)
        .to_string_lossy();
    println!(
        "   {:<12} {:.<50} {}",
        "Largest".dimmed(),
        largest_name,
        format_size(largest_size).white()
    );

    let newest_name = newest_path
        .strip_prefix(&cwd)
        .unwrap_or(&newest_path)
        .to_string_lossy();
    println!(
        "   {:<12} {:.<50} {}",
        "Newest".dimmed(),
        newest_name,
        format_relative_time(newest_time).white()
    );

    let oldest_name = oldest_path
        .strip_prefix(&cwd)
        .unwrap_or(&oldest_path)
        .to_string_lossy();
    println!(
        "   {:<12} {:.<50} {}",
        "Oldest".dimmed(),
        oldest_name,
        format_relative_time(oldest_time).white()
    );

    Ok(())
}

fn traverse(
    path: &Path,
    total_size: &mut u64,
    file_count: &mut usize,
    folder_count: &mut usize,
    largest_path: &mut std::path::PathBuf,
    largest_size: &mut u64,
    newest_path: &mut std::path::PathBuf,
    newest_time: &mut u64,
    oldest_path: &mut std::path::PathBuf,
    oldest_time: &mut u64,
) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if metadata.is_dir() {
            *folder_count += 1;
            traverse(
                &entry.path(),
                total_size,
                file_count,
                folder_count,
                largest_path,
                largest_size,
                newest_path,
                newest_time,
                oldest_path,
                oldest_time,
            )?;
        } else {
            *file_count += 1;
            let size = metadata.len();
            *total_size += size;
            if size > *largest_size {
                *largest_size = size;
                *largest_path = entry.path();
            }
            if modified > *newest_time {
                *newest_time = modified;
                *newest_path = entry.path();
            }
            if modified > 0 && modified < *oldest_time {
                *oldest_time = modified;
                *oldest_path = entry.path();
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_relative_time(seconds_ago: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(seconds_ago);

    if diff < 60 {
        return "just now".to_string();
    }
    let minutes = diff / 60;
    if minutes < 60 {
        return format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        );
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    }
    let days = hours / 24;
    if days < 30 {
        return format!("{} day{} ago", days, if days == 1 { "" } else { "s" });
    }
    let months = days / 30;
    if months < 12 {
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    }
    let years = months / 12;
    format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
}
