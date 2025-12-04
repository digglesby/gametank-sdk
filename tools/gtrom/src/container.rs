//! Container orchestration for builds
//! 
//! Manages the podman container lifecycle for llvm-mos toolchain access.

use std::path::Path;
use std::process::Command;

/// Check if we're running inside a container
pub fn is_in_container() -> bool {
    Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || std::env::var("container").is_ok()
}

/// Find the workspace root (where .git or root Cargo.toml is)
pub fn find_workspace_root() -> Result<std::path::PathBuf, String> {
    let mut current = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    loop {
        // Check for .git directory (repo root)
        if current.join(".git").exists() {
            return Ok(current);
        }
        // Check for workspace Cargo.toml with [workspace] section
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Ok(current);
                }
            }
        }
        
        if !current.pop() {
            break;
        }
    }
    
    // Fallback to current dir
    std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))
}

/// Ensure the build container is running
pub fn ensure_container() -> Result<std::path::PathBuf, String> {
    let workspace_root = find_workspace_root()?;
    
    // Check if container is already running
    let output = Command::new("podman")
        .args(["ps", "--filter", "name=gametank", "--filter", "status=running", "--format", "{{.Names}}"])
        .output()
        .map_err(|e| format!("Failed to check container status: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("gametank") {
        return Ok(workspace_root);
    }

    // Start the container
    println!("Starting build container...");
    let status = Command::new("podman")
        .args([
            "run", "-d",
            "--name", "gametank",
            "-v", &format!("{}:/workspace:z", workspace_root.display()),
            "--replace",
            "rust-mos:gte",
            "sleep", "infinity"
        ])
        .status()
        .map_err(|e| format!("Failed to start container: {}", e))?;

    if status.success() {
        Ok(workspace_root)
    } else {
        Err("Failed to start build container".to_string())
    }
}

/// Execute a command inside the container
pub fn podman_exec(workdir: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new("podman")
        .args(["exec", "-t", "-w", workdir, "gametank"])
        .args(args)
        .status()
        .map_err(|e| format!("Failed to exec in container: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Command failed: {:?}", args))
    }
}
