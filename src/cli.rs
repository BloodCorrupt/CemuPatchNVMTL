use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "cemu-patch-nvmtl",
    about = "Harmless companion tool to fix Cemu on official NullMoth NVIDIA macOS driver",
    version = "1.0.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch Cemu with the harmless runtime fix (no system modifications)
    #[command(alias = "launch")]
    Run {
        /// Optional path to a game (.rpx or game folder) to boot directly
        #[arg(short, long)]
        game: Option<PathBuf>,

        /// Enable verbose debugging from the NVMTL shim
        #[arg(short, long)]
        debug: bool,

        /// Additional arguments to pass directly to Cemu
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        extra_args: Vec<String>,
    },

    /// Create a 1-click 'Cemu (Patched).app' bundle
    InstallApp {
        /// Optional path to source Cemu.app (e.g. ~/Downloads/Cemu_release.app)
        #[arg(short, long)]
        source: Option<PathBuf>,

        /// Target folder for the .app bundle (default: /Applications)
        #[arg(short, long, default_value = "/Applications")]
        target_dir: PathBuf,
    },

    /// Safely install the patched translator into NVMTLDriver.bundle (with auto-backup)
    PatchDriver {
        /// Only create the .orig backup if not already present
        #[arg(long)]
        backup_only: bool,
    },

    /// Restore the official untouched NullMoth libnvmtl_translate.dylib from .orig
    RestoreDriver,

    /// Check driver, translator, Cemu, and GPU status
    Status,

    /// Verify the patched translator symbols, markers, and SHA-256
    Verify,
}

