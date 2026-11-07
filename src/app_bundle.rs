use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::status::{get_patched_xlate_path, get_shim_path, CEMU_APP_PATH};

pub fn install_app_bundle(source: Option<&Path>, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source_app: PathBuf = if let Some(s) = source {
        s.to_path_buf()
    } else if Path::new(CEMU_APP_PATH).exists() {
        PathBuf::from(CEMU_APP_PATH)
    } else {
        return Err("No source Cemu.app found. Specify --source <path>.".into());
    };

    let base_name = source_app
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Cemu.app");
    let patched_name = if base_name.ends_with(".app") {
        format!("{} (Patched).app", &base_name[..base_name.len() - 4])
    } else {
        format!("{} (Patched).app", base_name)
    };

    let app_dir = target_dir.join(&patched_name);
    let contents_dir = app_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let frameworks_dir = contents_dir.join("Frameworks");

    println!("{}", format!("📦 Creating self-contained patched app bundle at: {}", app_dir.display()).cyan().bold());

    // Copy source app
    let status = Command::new("cp")
        .args(["-R", source_app.to_str().unwrap(), app_dir.to_str().unwrap()])
        .status()?;
    if !status.success() {
        return Err(format!("Failed to copy {} to {}", source_app.display(), app_dir.display()).into());
    }

    // Strip quarantine
    let _ = Command::new("xattr")
        .args(["-cr", app_dir.to_str().unwrap()])
        .output();

    fs::create_dir_all(&frameworks_dir)?;

    // Copy patched translator and shim into Contents/Frameworks/
    let shim = get_shim_path();
    let xlate = get_patched_xlate_path();
    fs::copy(&shim, frameworks_dir.join("libcemunvmtl_shim.dylib"))?;
    fs::copy(&xlate, frameworks_dir.join("libnvmtl_translate.dylib"))?;
    println!("  {} Embedded patched translator and shim into Frameworks/", "✔".green());

    // Find main binary in Contents/MacOS/
    let mut main_bin = None;
    for entry in fs::read_dir(&macos_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.ends_with(".sh") && !name_str.ends_with("_bin") {
            main_bin = Some(name_str.to_string());
            break;
        }
    }

    let bin_name = main_bin.ok_or("Could not locate Cemu executable in Contents/MacOS")?;
    let orig_bin_path = macos_dir.join(&bin_name);
    let renamed_bin_path = macos_dir.join(format!("{}_bin", bin_name));

    if orig_bin_path.exists() {
        fs::rename(&orig_bin_path, &renamed_bin_path)?;
    }

    let launcher_script = format!(
        r#"#!/bin/bash
DIR="$(cd "$(dirname "$0")/.." && pwd)"
export DYLD_INSERT_LIBRARIES="$DIR/Frameworks/libcemunvmtl_shim.dylib"
export NVMTL_TRANSLATE_OVERRIDE="$DIR/Frameworks/libnvmtl_translate.dylib"
exec "$DIR/MacOS/{}_bin" "$@"
"#,
        bin_name
    );

    fs::write(&orig_bin_path, launcher_script)?;
    fs::set_permissions(&orig_bin_path, fs::Permissions::from_mode(0o755))?;
    println!("  {} Created executable launcher wrapper for {}", "✔".green(), bin_name);

    // Deep codesign
    let _ = Command::new("codesign")
        .args(["--deep", "-f", "-s", "-", app_dir.to_str().unwrap()])
        .output();
    println!("  {} Applied ad-hoc codesign", "✔".green());

    println!("\n{}", "🎉 Self-contained app bundle ready!".green().bold());
    println!("Target: {}", app_dir.display());

    Ok(())
}
