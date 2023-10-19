use bootloader_linker::{Config, SubCommand, init_logger};
use clap::Parser;
use log::{info, error, trace};

use bootloader::{UefiBoot, BiosBoot};

fn main() {
    let config: Config = Config::parse();

    let (command, mut input_file, uefi, out_dir, qemu_path, verbosity, extra_args)
        = (config.command, config.file, config.uefi, config.out_dir, config.qemu_path, config.verbosity, config.extra_args);

    
    init_logger(verbosity);

    let (build, run);
    match command {
        SubCommand::Build => { build = true; run = false; },
        SubCommand::Run => { build = false; run = true; },
        SubCommand::BuildRun => { build = true; run = true; },
    }

    if build {
        trace!("Building disk image");
        if uefi {
            let uefi_path = if out_dir.is_dir() {
                out_dir.join("uefi.img")
            } else if out_dir.to_str().map(|s| s.as_bytes().last().map(|&b| b == b'/' || b == b'\\')).unwrap_or(Some(false)).unwrap_or(false) {
                if let Ok(_) = std::fs::create_dir_all(&out_dir) {
                    out_dir.join("uefi.img")
                } else {
                    out_dir
                }
            } else {
                out_dir
            };

            if let Err(e) = UefiBoot::new(&input_file).create_disk_image(&uefi_path) {
                error!("Fatal error encountered while building disk image: {e}");
                e.chain().skip(1).for_each(|cause| info!("Caused by: {cause}"));
                std::process::exit(1);
            }

            input_file = uefi_path;
        } else {
            let bios_path = if out_dir.is_dir() {
                out_dir.join("bios.img")
            } else if out_dir.to_str().map(|s| s.as_bytes().last().map(|&b| b == b'/' || b == b'\\')).unwrap_or(Some(false)).unwrap_or(false) {
                if let Ok(_) = std::fs::create_dir_all(&out_dir) {
                    out_dir.join("bios.img")
                } else {
                    out_dir
                }
            } else {
                out_dir
            };

            if let Err(e) = BiosBoot::new(&input_file).create_disk_image(&bios_path) {
                error!("Fatal error encountered while building disk image: {e}");
                e.chain().skip(1).for_each(|cause| info!("Caused by: {cause}"));
                std::process::exit(1);
            }

            input_file = bios_path;
        }
    }
    if run {
        trace!("Running disk image");
        let mut cmd = std::process::Command::new(qemu_path);
        if uefi {
            cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        }
        cmd.arg("-drive").arg(format!("format=raw,file={}", input_file.to_str().unwrap()));

        // Add in all the excess args
        cmd.args(extra_args);

        let child = cmd.spawn();
        match child {
            Err(e) => {
                error!("Fatal error spawning child process: {e}");
                std::process::exit(1);
            },
            Ok(mut child) => { if let Err(e) = child.wait() {
                error!("Fatal error encountered while awaiting child process: {e}");
                std::process::exit(1);
            } }
        }
    }
}
