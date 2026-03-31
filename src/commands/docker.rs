use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

const WUX_SANDBOX_DOCKERFILE: &str = r#"FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    python3.12 \
    python3-pip \
    python3.12-venv \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN curl -OL https://go.dev/dl/go1.24.1.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.24.1.linux-amd64.tar.gz \
    && rm go1.24.1.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

WORKDIR /workspace
"#;

fn to_docker_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    #[cfg(windows)]
    {
        let mut chars = s.chars();
        if let (Some(drive), Some(':')) = (chars.next(), chars.next()) {
            let rest = &s[2..].replace('\\', "/");
            return format!("/{}{}", drive.to_lowercase(), rest);
        }
    }
    s.into_owned()
}

fn check_docker() -> Result<()> {
    let output = std::process::Command::new("docker")
        .arg("--version")
        .output()
        .context("Failed to run docker command")?;

    if !output.status.success() {
        println!(
            "{} Docker not found. Install Docker Desktop from https://docker.com",
            "✗".red()
        );
        std::process::exit(1);
    }
    Ok(())
}

fn check_wux_sandbox_image() -> bool {
    std::process::Command::new("docker")
        .args(["image", "inspect", "wux-sandbox"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn build_wux_sandbox() -> Result<()> {
    let temp_dir = std::env::temp_dir().join("wux-sandbox-build");
    let dockerfile_path = temp_dir.join("Dockerfile");

    std::fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;
    std::fs::write(&dockerfile_path, WUX_SANDBOX_DOCKERFILE)
        .context("Failed to write Dockerfile")?;

    println!(
        "{} Building wux-sandbox image... (one-time, may take a few minutes)",
        "→".cyan()
    );

    let output = std::process::Command::new("docker")
        .args(["build", "-t", "wux-sandbox", &temp_dir.to_string_lossy()])
        .output()
        .context("Failed to build docker image")?;

    let _ = std::fs::remove_dir_all(&temp_dir);

    if !output.status.success() {
        println!("{} Docker build failed:", "✗".red());
        println!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    println!("{} wux-sandbox image built successfully", "✓".green());
    Ok(())
}

pub fn dockersafe() -> Result<()> {
    check_docker()?;

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let docker_path = to_docker_path(&cwd);

    let status = std::process::Command::new("docker")
        .args([
            "run",
            "-it",
            "--rm",
            "--network",
            "none",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!("{}:/workspace:ro", docker_path),
            "ubuntu:24.04",
            "bash",
        ])
        .status()
        .context("Failed to run docker")?;

    std::process::exit(status.code().unwrap_or(1));
}

pub fn dockerrun() -> Result<()> {
    check_docker()?;

    if !check_wux_sandbox_image() {
        build_wux_sandbox()?;
    }

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let docker_path = to_docker_path(&cwd);

    let status = std::process::Command::new("docker")
        .args([
            "run",
            "-it",
            "--rm",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!("{}:/workspace:rw", docker_path),
            "wux-sandbox",
            "bash",
        ])
        .status()
        .context("Failed to run docker")?;

    std::process::exit(status.code().unwrap_or(1));
}
