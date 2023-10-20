use clap::{Parser, ValueEnum};

use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version = concat!(std::env!("CARGO_PKG_VERSION"), " using bootloader 0.11.4"), about, long_about = None)]
pub struct Config {
    #[arg(value_enum)]
    pub command: SubCommand,
    #[doc = "The binary/.img file to operate on. Can be created relatively easy using the bootloader_api crate."]
    pub input_file: PathBuf,
    /// Sets the loader to use uefi instead of bios.
    #[arg(short, long)]
    pub uefi: bool,
    /// The directory to put output files in. Ignored if not building a disk image.
    #[arg(short, long, default_value = "./")]
    pub out_dir: PathBuf,
    /// The name of the qemu executable. Ignored if not running a disk image.
    #[arg(short, long, default_value = "qemu-system-x86_64")]
    pub qemu_path: String,

    /// Specifies the minimum frame buffer height desired.
    /// If it is not possible, the bootloader will fall back to a smaller format.
    #[arg(short = 'H', long = "min_height")]
    pub minimum_framebuffer_height: Option<u64>,
    /// Specifies the minimum frame buffer width desired.
    /// If it is not possible, the bootloader will fall back to a smaller format.
    #[arg(short = 'W', long = "min_width")]
    pub minimum_framebuffer_width: Option<u64>,
    /// The minimum level of logging to still display.
    #[arg(value_enum, short, long, default_value_t = LevelFilter::Trace)]
    pub log_level: LevelFilter,
    /// Whether the bootloader should print log messages to the framebuffer during boot.
    #[arg(short = 'f', long = "frame_logging")]
    pub frame_buffer_logging: bool,
    /// Whether the bootloader should print log messages to the serial port during boot.
    ///
    /// If `-serial stdio` is passed to qemu, this will print to the terminal.
    #[arg(short = 's', long = "serial_logging")]
    pub serial_logging: bool,

    /// Extra args to pass to qemu. You can also put them after -- at the end of the command.
    #[arg(short, long)]
    pub args: Vec<String>,
    /// Extra args to pass to qemu.
    pub extra_args: Vec<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
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

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum LevelFilter {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LevelFilter {
    pub fn to_bootloader(self) -> bootloader_boot_config::LevelFilter {
        match self {
            LevelFilter::Off => bootloader_boot_config::LevelFilter::Off,
            LevelFilter::Trace => bootloader_boot_config::LevelFilter::Trace,
            LevelFilter::Debug => bootloader_boot_config::LevelFilter::Debug,
            LevelFilter::Info => bootloader_boot_config::LevelFilter::Info,
            LevelFilter::Warn => bootloader_boot_config::LevelFilter::Warn,
            LevelFilter::Error => bootloader_boot_config::LevelFilter::Error,
        }
    }
}

use env_logger::Builder;

pub fn init_logger() {
    let mut builder = Builder::default();
    builder
        .format_timestamp(None)
        .format_target(false)
        .format_module_path(false);

    builder.filter_level(log::LevelFilter::Info);

    builder.parse_default_env();

    builder.init();
}
