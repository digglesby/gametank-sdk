//! Assembly compilation
//!
//! Handles assembling .asm files into libasm.a using llvm-mc and llvm-ar.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::container::podman_exec;

/// Recursively find all .asm files in a directory, excluding target directory
fn find_asm_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut asm_files = Vec::new();
    
    fn visit_dirs(dir: &Path, asm_files: &mut Vec<PathBuf>) -> Result<(), String> {
        if dir.is_dir() {
            // Skip target directory
            if dir.file_name().map_or(false, |n| n == "target") {
                return Ok(());
            }
            
            for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                
                if path.is_dir() {
                    visit_dirs(&path, asm_files)?;
                } else if path.extension().map_or(false, |ext| ext == "asm") {
                    asm_files.push(path);
                }
            }
        }
        Ok(())
    }
    
    visit_dirs(dir, &mut asm_files)?;
    Ok(asm_files)
}


/// Build assembly files into libasm.a (runs directly)
pub fn build_asm(workdir: &str) -> Result<(), String> {
    println!("Assembling .asm files...");
    
    let workdir_path = Path::new(workdir);
    let target_dir = workdir_path.join("target/asm");
    
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target/asm: {}", e))?;

    // Find and assemble all .asm files recursively in workdir
    let asm_files = find_asm_files(workdir_path)?;
    
    for path in asm_files {
        let filename = path.file_stem().unwrap().to_string_lossy();
        println!("  Assembling {}...", filename);
        
        let status = Command::new("llvm-mc")
            .args([
                "--filetype=obj",
                "-triple=mos",
                "-mcpu=mosw65c02",
                path.to_str().unwrap(),
                "-o",
                &format!("{}/target/asm/{}.o", workdir, filename),
            ])
            .status()
            .map_err(|e| format!("Failed to assemble {}: {}", filename, e))?;

        if !status.success() {
            return Err(format!("Failed to assemble {}", filename));
        }
    }

    // Archive into libasm.a
    println!("  Creating libasm.a...");
    let o_files: Vec<_> = std::fs::read_dir(&target_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "o"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    if !o_files.is_empty() {
        let mut args = vec!["rcs".to_string(), format!("{}/target/asm/libasm.a", workdir)];
        args.extend(o_files.clone());
        
        let status = Command::new("llvm-ar")
            .args(&args)
            .status()
            .map_err(|e| format!("Failed to archive: {}", e))?;

        if !status.success() {
            return Err("Failed to create libasm.a".to_string());
        }

        // Clean up .o files
        for o_file in o_files {
            let _ = std::fs::remove_file(o_file);
        }
    }

    Ok(())
}

/// Build assembly files via container
pub fn build_asm_in_container(workdir: &Path, working_dir: &Path) -> Result<(), String> {
    println!("Assembling .asm files...");
    
    let target_dir = workdir.join("target/asm");
    
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target/asm: {}", e))?;

    let rel_workdir = workdir.strip_prefix(working_dir).unwrap_or(workdir);
    let workspace_dir = format!("/workspace/{}", rel_workdir.to_string_lossy());

    // Find and assemble all .asm files recursively in workdir
    let asm_files = find_asm_files(workdir)?;
    
    for path in asm_files {
        let filename = path.file_stem().unwrap().to_string_lossy();
        println!("  Assembling {}...", filename);
        
        let rel_path = path.strip_prefix(working_dir).unwrap_or(&path);
        
        podman_exec("/workspace", &[
            "llvm-mc",
            "--filetype=obj",
            "-triple=mos",
            "-mcpu=mosw65c02",
            &format!("/workspace/{}", rel_path.to_string_lossy()),
            "-o",
            &format!("{}/target/asm/{}.o", workspace_dir, filename),
        ])?;
    }

    // Archive into libasm.a
    let o_files: Vec<_> = std::fs::read_dir(&target_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "o"))
        .map(|e| format!("{}/target/asm/{}", workspace_dir, e.path().file_name().unwrap().to_string_lossy()))
        .collect();

    if !o_files.is_empty() {
        println!("  Creating libasm.a...");
        let mut args = vec![
            "llvm-ar".to_string(),
            "rcs".to_string(),
            format!("{}/target/asm/libasm.a", workspace_dir),
        ];
        args.extend(o_files);
        
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        podman_exec("/workspace", &args_ref)?;

        // Clean up .o files
        for entry in std::fs::read_dir(&target_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().map_or(false, |ext| ext == "o") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }

    Ok(())
}
