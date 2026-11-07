use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::status::{get_patched_xlate_path, get_shim_path, CEMU_APP_PATH};

pub fn run_cemu(game: Option<PathBuf>, debug: bool, extra_args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let cemu_bin = Path::new(CEMU_APP_PATH).join("Contents/MacOS/Cemu");
    if !cemu_bin.exists() {
        return Err(format!("Cemu executable not found at {}", cemu_bin.display()).into());
    }

    let shim = get_shim_path();
    let xlate = get_patched_xlate_path();

    if !shim.exists() {
        return Err(format!("Shim library not found at {}. Build it first.", shim.display()).into());
    }
    if !xlate.exists() {
        return Err(format!("Patched translator not found at {}.", xlate.display()).into());
    }

    println!("{}", "🚀 Launching Cemu with harmless NVMTL companion shim...".green().bold());
    println!("  • Shim:       {}", shim.display());
    println!("  • Translator: {}", xlate.display());

    let mut cmd = Command::new(&cemu_bin);
    cmd.env("DYLD_INSERT_LIBRARIES", &shim);
    cmd.env("NVMTL_TRANSLATE_OVERRIDE", &xlate);

    if debug {
        cmd.env("CEMU_NVMTL_DEBUG", "1");
        println!("  • Debug:      {}", "ENABLED".yellow().bold());
    }

    if let Some(g) = game {
        println!("  • Game:       {}", g.display());
        cmd.arg("-g").arg(g);
    }

    for arg in extra_args {
        cmd.arg(arg);
    }

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let mut child = cmd.spawn()?;
    let status = child.wait()?;

    if !status.success() {
        eprintln!("{}", format!("Cemu exited with status: {}", status).yellow());
    }

    Ok(())
}

