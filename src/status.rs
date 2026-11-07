use colored::Colorize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const OFFICIAL_XLATE_PATH: &str = "/Library/GPUBundles/NVMTLDriver.bundle/Contents/MacOS/libnvmtl_translate.dylib";
pub const BACKUP_XLATE_PATH: &str = "/Library/GPUBundles/NVMTLDriver.bundle/Contents/MacOS/libnvmtl_translate.dylib.orig";
pub const CEMU_APP_PATH: &str = "/Applications/Cemu.app";

pub fn sha256_file<P: AsRef<Path>>(path: P) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(hex::encode(hasher.finalize()))
}

pub fn get_companion_dist_dir() -> PathBuf {
    // Look relative to executable first, or workspace default
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("dist");
            if candidate.exists() {
                return candidate;
            }
            if let Some(grandparent) = parent.parent() {
                let candidate = grandparent.join("dist");
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from("/Users/bloodtek/CemuPatchNVMTL/dist")
}

pub fn get_patched_xlate_path() -> PathBuf {
    get_companion_dist_dir().join("libnvmtl_translate.dylib")
}

pub fn get_shim_path() -> PathBuf {
    get_companion_dist_dir().join("libcemunvmtl_shim.dylib")
}

pub fn print_status() {
    println!("{}", "=== NullMoth NVMTL / Cemu Companion Status ===".bold().cyan());

    // 1. Check Cemu.app
    let cemu_exists = Path::new(CEMU_APP_PATH).exists();
    if cemu_exists {
        println!("{} Cemu: Found at {}", "✔".green(), CEMU_APP_PATH);
    } else {
        println!("{} Cemu: Not found at {}", "✖".red(), CEMU_APP_PATH);
    }

    // Check patched app
    let patched_app = Path::new("/Applications/Cemu (Patched).app");
    if patched_app.exists() {
        println!("{} Cemu (Patched): Installed at {}", "✔".green(), patched_app.display());
    } else {
        println!("{} Cemu (Patched): Not yet installed (run 'install-app' to create)", "ℹ".yellow());
    }

    // 2. Check System Driver
    println!("\n{}", "--- System Driver Files ---".bold());
    let sys_xlate = Path::new(OFFICIAL_XLATE_PATH);
    let backup_xlate = Path::new(BACKUP_XLATE_PATH);

    if sys_xlate.exists() {
        let hash = sha256_file(sys_xlate).unwrap_or_default();
        let short_hash = &hash[..8.min(hash.len())];

        let patched_hash = sha256_file(get_patched_xlate_path()).unwrap_or_default();
        let is_patched = !patched_hash.is_empty() && hash == patched_hash;

        if is_patched {
            println!(
                "{} System Translator: {} (SHA: {})",
                "✔".green(),
                "PATCHED (Vector bswap fix active globally)".green().bold(),
                short_hash
            );
        } else {
            println!(
                "{} System Translator: {} (SHA: {})",
                "ℹ".yellow(),
                "Official / Unmodified NullMoth driver".yellow(),
                short_hash
            );
        }
    } else {
        println!("{} System Translator: Missing ({})", "✖".red(), OFFICIAL_XLATE_PATH);
    }

    if backup_xlate.exists() {
        let hash = sha256_file(backup_xlate).unwrap_or_default();
        let short_hash = &hash[..8.min(hash.len())];
        println!("{} Official Backup: Present ({}, SHA: {})", "✔".green(), BACKUP_XLATE_PATH, short_hash);
    } else {
        println!("{} Official Backup: None (will be created automatically on 'patch-driver')", "ℹ".blue());
    }

    // 3. Companion files
    println!("\n{}", "--- Companion Tool Assets ---".bold());
    let patched_path = get_patched_xlate_path();
    if patched_path.exists() {
        let sz = fs::metadata(&patched_path).map(|m| m.len() / (1024 * 1024)).unwrap_or(0);
        let hash = sha256_file(&patched_path).unwrap_or_default();
        println!(
            "{} Patched Translator: {} ({} MB, SHA: {})",
            "✔".green(),
            patched_path.display(),
            sz,
            &hash[..8.min(hash.len())]
        );
    } else {
        println!("{} Patched Translator: Missing at {}", "✖".red(), patched_path.display());
    }

    let shim_path = get_shim_path();
    if shim_path.exists() {
        println!("{} Interposer Shim: Ready at {}", "✔".green(), shim_path.display());
    } else {
        println!("{} Interposer Shim: Missing at {}", "✖".red(), shim_path.display());
    }

    // 4. Check GPU
    println!("\n{}", "--- Hardware & GPU Environment ---".bold());
    if let Ok(output) = Command::new("system_profiler").arg("SPDisplaysDataType").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(15) {
            let trimmed = line.trim();
            if trimmed.starts_with("Chipset Model:") || trimmed.starts_with("VRAM") || trimmed.starts_with("Metal Support:") {
                println!("  {}", trimmed);
            }
        }
    }

    // 5. Check recent Cemu log for errors
    if let Ok(home) = std::env::var("HOME") {
        let log_file = PathBuf::from(home).join("Library/Application Support/Cemu/log.txt");
        if log_file.exists() {
            println!("\n{}", "--- Cemu Log Diagnostics ---".bold());
            if let Ok(content) = fs::read_to_string(&log_file) {
                let error_count = content.lines().filter(|l| l.contains("Failed to create graphics pipeline")).count();
                if error_count > 0 {
                    println!(
                        "{} Historical pipeline errors found in Cemu log: {} occurrences",
                        "⚠".yellow(),
                        error_count
                    );
                    println!("  Launch Cemu via 'cemu-patch-nvmtl run' or 'Cemu (Patched).app' to resolve.");
                } else {
                    println!("{} No graphics pipeline errors found in recent log!", "✔".green());
                }
            }
        }
    }

    println!("\n{}", "Usage tips:".bold());
    println!("  • To run Cemu without modifying the driver:  cemu-patch-nvmtl run");
    println!("  • To create a 1-click Mac app in /Applications: cemu-patch-nvmtl install-app");
    println!("  • To patch the system driver globally:       sudo cemu-patch-nvmtl patch-driver");
    println!("  • To restore official driver:                sudo cemu-patch-nvmtl restore-driver");
}
