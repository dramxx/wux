use std::fs;
use tempfile::TempDir;
use wux::commands::nuke;

fn nuke_path(path: &str) -> anyhow::Result<()> {
    nuke::run(path, false, true)
}

fn nuke_dry_run(path: &str) -> anyhow::Result<()> {
    nuke::run(path, true, false)
}

#[test]
fn nuke_deletes_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    assert!(file_path.exists());
    nuke_path(file_path.to_str().unwrap()).unwrap();
    assert!(!file_path.exists());
}

#[test]
fn nuke_deletes_directory() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir(&dir_path).unwrap();
    fs::write(dir_path.join("file1.txt"), "content1").unwrap();
    fs::write(dir_path.join("file2.txt"), "content2").unwrap();
    fs::create_dir(dir_path.join("subdir")).unwrap();
    fs::write(dir_path.join("subdir").join("file3.txt"), "content3").unwrap();

    assert!(dir_path.exists());
    nuke_path(dir_path.to_str().unwrap()).unwrap();
    assert!(!dir_path.exists());
}

#[test]
fn nuke_dry_run_does_not_delete() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    assert!(file_path.exists());
    nuke_dry_run(file_path.to_str().unwrap()).unwrap();
    assert!(file_path.exists(), "File should still exist after dry run");
}

#[test]
fn nuke_aborts_on_nonexistent_path() {
    let temp_dir = TempDir::new().unwrap();
    let missing_path = temp_dir.path().join("missing-path-12345");
    let result = nuke_path(missing_path.to_str().unwrap());
    if let Err(e) = &result {
        eprintln!("Error: {:?}", e);
    }
    assert!(result.is_ok());
}

#[test]
fn nuke_refuses_cwd() {
    let cwd = std::env::current_dir().unwrap();
    let result = nuke_path(cwd.to_str().unwrap());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("current working directory"));
}

#[test]
fn nuke_refuses_root() {
    #[cfg(windows)]
    let root = "C:\\";
    #[cfg(unix)]
    let root = "/";

    let result = nuke_path(root);
    assert!(result.is_err());
}
