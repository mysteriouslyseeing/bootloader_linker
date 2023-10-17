use clap::{Parser, ValueEnum, ArgAction};

use std::path::PathBuf;


#[derive(Parser, Debug, Clone)]
#[command(author, version = concat!(std::env!("CARGO_PKG_VERSION"), " using bootloader 0.11.4"), about, long_about = None)]
pub struct Config {
    #[arg(value_enum)]
    pub command: SubCommand,
    #[doc = "The binary/.img file to operate on. Can be created relatively easy using the bootloader_api crate."]
    pub file: PathBuf,
    /// Sets the loader to use uefi instead of bios.
    #[arg(short, long)]
    pub uefi: bool,
    /// The directory to put output files in. Ignored if not building a disk image.
    #[arg(short, long, default_value = "./")]
    pub out_dir: PathBuf,
    /// The name of the qemu executable. Ignored if not running a disk image.
    #[arg(short, long, default_value = "qemu-system-x86_64")]
    pub qemu_path: String,
    /// Configures the amount of logging. If this flag appears more times more things will be logged.
    #[arg(short, long, action = ArgAction::Count)]
    pub verbosity: u8,
    
    /// Extra args to pass to qemu.
    pub extra_args: Vec<String>,

}

#[derive(ValueEnum, Debug, Clone)]
pub enum SubCommand {
    #[value(alias = "b")]
    /// Builds a disk image from a .elf x86_64 binary
    Build,
    /// Runs a disk image in qemu
    #[value(alias = "r")]
    Run,
    /// Builds and runs a disk image from a x86_64 binary in qemu
    #[value(alias = "br")]
    BuildRun,
}

use env_logger::Builder;

pub fn init_logger(verbosity: u8) {
    let mut builder = Builder::default();
    builder.format_timestamp(None).format_target(false).format_module_path(false);
    match verbosity {
        0 => builder.filter_level(log::LevelFilter::Error),
        1 => builder.filter_level(log::LevelFilter::Warn),
        2 => builder.filter_level(log::LevelFilter::Info),
        3 => builder.filter_level(log::LevelFilter::Debug),
        4 => builder.filter_level(log::LevelFilter::Trace),
        _ => builder.filter_level(log::LevelFilter::Off),
    };
    builder.parse_default_env();

    builder.init();
}