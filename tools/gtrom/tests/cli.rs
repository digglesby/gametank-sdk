use std::process::Command;
use tempfile::TempDir;

fn gtrom() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gtrom"))
}

#[test]
fn test_init_creates_project() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("my_game");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap()])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success(), "gtrom init failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify project structure
    assert!(project_dir.join("rom/Cargo.toml").exists());
    assert!(project_dir.join("rom/src/main.rs").exists());
    assert!(project_dir.join("rom/build.rs").exists());
}

#[test]
fn test_init_with_custom_name() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("folder");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap(), "--name", "custom-game"])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success());
    
    let cargo_toml = std::fs::read_to_string(project_dir.join("rom/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"custom-game\""));
}

#[test]
fn test_init_with_audiofw_src() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("audio_game");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap(), "--with-audiofw-src"])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success());
    assert!(project_dir.join("audiofw-src").exists());
}

#[test]
fn test_init_fails_on_existing_project() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("existing");
    
    // Create project first time
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap()])
        .output()
        .expect("failed to run gtrom init");
    assert!(output.status.success());
    
    // Try to init again - should fail
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap()])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(!output.status.success(), "init should fail on existing directory");
}

#[test]
fn test_help_shows_commands() {
    let output = gtrom()
        .arg("--help")
        .output()
        .expect("failed to run gtrom --help");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("build"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("convert"));
}

#[test]
fn test_version() {
    let output = gtrom()
        .arg("--version")
        .output()
        .expect("failed to run gtrom --version");
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gtrom"));
}

#[test]
fn test_build_fails_without_project() {
    let temp = TempDir::new().unwrap();
    
    let output = gtrom()
        .arg("build")
        .current_dir(temp.path())
        .output()
        .expect("failed to run gtrom build");
    
    // Should fail because there's no project
    assert!(!output.status.success());
}

#[test]
fn test_convert_fails_with_missing_file() {
    let output = gtrom()
        .args(["convert", "/nonexistent/file.elf"])
        .output()
        .expect("failed to run gtrom convert");
    
    assert!(!output.status.success());
}

#[test]
fn test_init_sanitizes_name_with_spaces() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("test1");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap(), "--name", "My Cool Game"])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success());
    
    let cargo_toml = std::fs::read_to_string(project_dir.join("rom/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"my-cool-game\""), "name should be sanitized: {}", cargo_toml);
}

#[test]
fn test_init_sanitizes_name_with_underscores() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("test2");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap(), "--name", "my_cool_game"])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success());
    
    let cargo_toml = std::fs::read_to_string(project_dir.join("rom/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"my-cool-game\""), "underscores should become hyphens: {}", cargo_toml);
}

#[test]
fn test_init_sanitizes_name_starting_with_number() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("test3");
    
    let output = gtrom()
        .args(["init", project_dir.to_str().unwrap(), "--name", "123game"])
        .output()
        .expect("failed to run gtrom init");
    
    assert!(output.status.success());
    
    let cargo_toml = std::fs::read_to_string(project_dir.join("rom/Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"game-123game\""), "should prefix with 'game-': {}", cargo_toml);
}
