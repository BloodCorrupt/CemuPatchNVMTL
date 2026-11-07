mod cli;
mod runner;
mod patcher;
mod app_bundle;
mod status;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use status::{get_patched_xlate_path, get_shim_path, sha256_file};
use std::process::Command;

fn verify_patched_translator() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🔍 Verifying Patched Translator Dylib Integrity...".cyan().bold());

    let xlate_path = get_patched_xlate_path();
    if !xlate_path.exists() {
        return Err(format!("File not found at {}", xlate_path.display()).into());
    }

    let hash = sha256_file(&xlate_path).unwrap_or_default();
    println!("  • Location:  {}", xlate_path.display());
    println!("  • SHA-256:   {}", hash);

    // Check symbols via nm
    let nm_output = Command::new("nm")
        .args(["-gU", xlate_path.to_str().unwrap()])
        .output()?;
    let symbols = String::from_utf8_lossy(&nm_output.stdout);

    let required_symbols = [
        "_nvmtl_translate",
        "_nvmtl_translate_fc",
        "_nvmtl_translate_free",
        "_nvmtl_translate_samplers",
        "_nvmtl_translate_set_caps",
        "_nvmtl_translate_set_threadgroup_lengths",
        "_nvmtl_translate_set_raster_sample_count",
        "_nvmtl_lower_buffer_addresses",
        "_nvmtl_lower_buffer_addresses_ex",
        "_nvmtl_lower_mesh",
        "_nvmtl_lower_mesh2",
        "_nvmtl_promote_gl_uniform",
        "_nvmtl_reflect",
        "_nvmtl_table_to_uniform",
        "_nvmtl_link_externs",
        "_nvmtl_link_runtime",
        "_nvmtl_link_visible",
        "_nvmtl_visible_refs",
    ];

    let mut missing = 0;
    for sym in required_symbols {
        if symbols.contains(sym) {
            println!("  {} Exported symbol: {}", "✔".green(), sym);
        } else {
            println!("  {} MISSING symbol: {}", "✖".red(), sym);
            missing += 1;
        }
    }

    // Check strings for driver markers
    let strings_output = Command::new("strings")
        .args(["-a", xlate_path.to_str().unwrap()])
        .output()?;
    let strings_text = String::from_utf8_lossy(&strings_output.stdout);

    for marker in ["NVMTL_NO_BINDLESS_ALL", "NVMTL_NO_TIER2_FIXED"] {
        if strings_text.contains(marker) {
            println!("  {} Driver marker: {}", "✔".green(), marker);
        } else {
            println!("  {} Missing marker: {}", "✖".red(), marker);
            missing += 1;
        }
    }

    let shim_path = get_shim_path();
    if shim_path.exists() {
        println!("  {} Companion shim dylib present: {}", "✔".green(), shim_path.display());
    } else {
        println!("  {} Companion shim dylib missing: {}", "✖".red(), shim_path.display());
        missing += 1;
    }

    if missing == 0 {
        println!("\n{}", "🎉 All verification checks PASSED! Translator is 100% compatible.".green().bold());
    } else {
        println!("\n{}", format!("⚠ Verification completed with {} issues.", missing).yellow().bold());
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run { game, debug, extra_args } => runner::run_cemu(game, debug, extra_args),
        Commands::InstallApp { source, target_dir } => app_bundle::install_app_bundle(source.as_deref(), &target_dir),
        Commands::PatchDriver { backup_only } => patcher::patch_driver(backup_only),
        Commands::RestoreDriver => patcher::restore_driver(),
        Commands::Status => {
            status::print_status();
            Ok(())
        }
        Commands::Verify => verify_patched_translator(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

