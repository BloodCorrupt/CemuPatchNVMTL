use colored::Colorize;
use std::path::Path;
use std::process::Command;
use crate::status::{
    get_patched_xlate_path, sha256_file, BACKUP_XLATE_PATH, OFFICIAL_XLATE_PATH,
};

fn run_as_root_or_sudo(args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let is_root = unsafe { libc::geteuid() == 0 };
    let mut cmd = if is_root {
        Command::new(args[0])
    } else {
        let mut c = Command::new("sudo");
        c.arg(args[0]);
        c
    };

    for arg in &args[1..] {
        cmd.arg(arg);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Command failed with status: {}", status).into());
    }
    Ok(())
}

pub fn patch_driver(backup_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🛡️ System Driver Patching Manager".cyan().bold());

    let sys_path = Path::new(OFFICIAL_XLATE_PATH);
    let backup_path = Path::new(BACKUP_XLATE_PATH);
    let patched_path = get_patched_xlate_path();

    if !sys_path.exists() {
        return Err(format!("Target driver translator not found at {}", OFFICIAL_XLATE_PATH).into());
    }
    if !patched_path.exists() {
        return Err(format!("Patched translator binary not found at {}", patched_path.display()).into());
    }

    // 1. Create backup if it does not already exist
    if !backup_path.exists() {
        println!("Creating official driver backup at {}...", BACKUP_XLATE_PATH);
        run_as_root_or_sudo(&["cp", "-p", OFFICIAL_XLATE_PATH, BACKUP_XLATE_PATH])?;
        println!("  {} Official backup created successfully", "✔".green());
    } else {
        println!("  {} Existing backup verified at {}", "✔".green(), BACKUP_XLATE_PATH);
    }

    if backup_only {
        println!("{}", "Backup only requested; driver was not modified.".yellow());
        return Ok(());
    }

    // Check if already patched
    let current_hash = sha256_file(sys_path).unwrap_or_default();
    let new_hash = sha256_file(&patched_path).unwrap_or_default();

    if current_hash == new_hash {
        println!("{}", "System driver is already patched with the latest fix!".green());
        return Ok(());
    }

    println!("Installing patched translator into NVMTLDriver.bundle...");
    run_as_root_or_sudo(&["cp", patched_path.to_str().unwrap(), OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["chmod", "755", OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["chown", "root:wheel", OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["codesign", "-f", "-s", "-", OFFICIAL_XLATE_PATH])?;

    println!("\n{}", "🎉 System driver successfully patched!".green().bold());
    println!("Vector byte swaps (i16 & i32) are now supported globally across all Metal applications.");
    println!("To revert to the official driver at any time, run: sudo cemu-patch-nvmtl restore-driver");

    Ok(())
}

pub fn restore_driver() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🔄 Restoring Official NullMoth Driver".cyan().bold());

    let backup_path = Path::new(BACKUP_XLATE_PATH);
    if !backup_path.exists() {
        return Err(format!("No backup file found at {}. Cannot restore.", BACKUP_XLATE_PATH).into());
    }

    println!("Restoring {} from backup...", OFFICIAL_XLATE_PATH);
    run_as_root_or_sudo(&["cp", "-p", BACKUP_XLATE_PATH, OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["chmod", "755", OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["chown", "root:wheel", OFFICIAL_XLATE_PATH])?;
    run_as_root_or_sudo(&["codesign", "-f", "-s", "-", OFFICIAL_XLATE_PATH])?;

    let hash = sha256_file(OFFICIAL_XLATE_PATH).unwrap_or_default();
    println!("\n{}", "🎉 Official NullMoth driver restored successfully!".green().bold());
    println!("Driver SHA-256: {}", &hash[..16.min(hash.len())]);

    Ok(())
}
