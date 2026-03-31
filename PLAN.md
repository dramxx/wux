# wux docker commands — implementation plan

## Overview

Add two new built-in commands to wux:
- `wux dockersafe` — spins up a read-only, no-network Ubuntu container for inspecting suspicious repos
- `wux dockerrun` — spins up a writable, networked multi-language container for building/running code

Both commands mount the current working directory into `/workspace` inside the container.

---

## New built-in commands

### `wux dockersafe`
- Image: `ubuntu:24.04`
- Mount: current dir → `/workspace` (read-only, `:ro`)
- Network: none (`--network none`)
- Caps: `--cap-drop ALL`
- Security: `--security-opt no-new-privileges`
- Flags: `-it --rm`
- Shell: `bash`

### `wux dockerrun`
- Image: `wux-sandbox` (custom, see below)
- Mount: current dir → `/workspace` (read-write)
- Network: default (internet access)
- Caps: `--cap-drop ALL`
- Security: `--security-opt no-new-privileges`
- Flags: `-it --rm`
- Shell: `bash`
- Before running: check if `wux-sandbox` image exists locally. If not, build it automatically from the embedded Dockerfile (see below).

---

## Path conversion

Docker on Windows requires paths in the format `/c/Users/foo/bar` instead of `C:\Users\foo\bar`.

Add a helper function `to_docker_path`:

```rust
fn to_docker_path(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    let mut chars = s.chars();
    if let (Some(drive), Some(':')) = (chars.next(), chars.next()) {
        let rest = &s[2..].replace('\\', "/");
        format!("/{}{}", drive.to_lowercase(), rest)
    } else {
        s.replace('\\', "/")
    }
}
```

Use `std::env::current_dir()` to get the current path, pass through `to_docker_path`, then use the result in the `-v` mount argument.

---

## wux-sandbox Docker image

The `dockerun` command requires a custom multi-language image called `wux-sandbox`.

Embed the following Dockerfile as a string constant in the wux source:

```dockerfile
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Base tools
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Node.js 22
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Python 3.12
RUN apt-get update && apt-get install -y \
    python3.12 \
    python3-pip \
    python3.12-venv \
    && rm -rf /var/lib/apt/lists/*

# Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Go
RUN curl -OL https://go.dev/dl/go1.24.1.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.24.1.linux-amd64.tar.gz \
    && rm go1.24.1.linux-amd64.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

WORKDIR /workspace
```

### Auto-build logic for `wux dockerun`

Before running the container, check if the image exists:

```rust
let image_exists = std::process::Command::new("docker")
    .args(["image", "inspect", "wux-sandbox"])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false);

if !image_exists {
    println!("wux-sandbox image not found, building... (one-time, may take a few minutes)");
    // build it
}
```

Write the embedded Dockerfile string to a temp directory using `tempfile` or `std::env::temp_dir()`, then run:

```
docker build -t wux-sandbox <temp_dir>
```

Print progress to stdout so user knows it's working. After build, proceed with `docker run`.

---

## Implementation location

Follow the existing pattern for built-in commands in wux (wherever `free`, `nuke`, `list` etc. are matched). Add `"dockersafe"` and `"dockerrun"` as new match arms.

---

## Error handling

- If `docker` binary is not found: print `"Docker not found. Install Docker Desktop from https://docker.com"` and exit with non-zero code.
- If `docker build` fails: print the build output and exit with error.
- If `current_dir()` fails: print error and exit.

---

## Summary of changes

1. Add `to_docker_path()` helper function
2. Add `wux dockersafe` built-in command
3. Add `wux dockerrun` built-in command  
4. Add embedded Dockerfile string constant for `wux-sandbox`
5. Add auto-build logic that runs once before first `dockerrun`
