use std::path::PathBuf;

use bootloader_linker::{init_logger, Config, SubCommand};
use clap::Parser;
use log::{error, info, trace};

use bootloader::{BiosBoot, BootConfig, UefiBoot};

fn get_output_path(out_dir: PathBuf, default_file_name: &str) -> PathBuf {
    if out_dir.is_dir() {
        out_dir.join(default_file_name)
    } else if out_dir
        .to_str()
        .map(|s| s.as_bytes().last().map(|&b| b == b'/' || b == b'\\').unwrap_or(false))
        .unwrap_or(false)
    {
        if let Ok(_) = std::fs::create_dir_all(&out_dir) {
            out_dir.join(default_file_name)
        } else {
            out_dir
        }
    } else {
        out_dir
    }
}

fn report_error_and_exit(e: anyhow::Error) -> ! {
    error!("Fatal error encountered while building disk image: {e}");
    e.chain()
        .skip(1)
        .for_each(|cause| info!("Caused by: {cause}"));
    std::process::exit(1)
}

fn main() {
    let config: Config = Config::parse();

    let Config {
        command,
        mut input_file,
        uefi,
        out_dir,
        qemu_path,
        minimum_framebuffer_height,
        minimum_framebuffer_width,
        log_level,
        frame_buffer_logging,
        serial_logging,
        args,
        extra_args,
    } = config;

    init_logger();

    let (build, run);
    match command {
        SubCommand::Build => {
            build = true;
            run = false;
        }
        SubCommand::Run => {
            build = false;
            run = true;
        }
        SubCommand::BuildRun => {
            build = true;
            run = true;
        }
    }

    if build {
        trace!("Building disk image");
        let mut boot_config = BootConfig::default();
        boot_config.frame_buffer.minimum_framebuffer_height = minimum_framebuffer_height;
        boot_config.frame_buffer.minimum_framebuffer_width = minimum_framebuffer_width;
        boot_config.log_level = log_level.to_bootloader();
        boot_config.frame_buffer_logging = frame_buffer_logging;
        boot_config.serial_logging = serial_logging;

        if uefi {
            let uefi_path = get_output_path(out_dir, "uefi.img");

            if let Err(e) = UefiBoot::new(&input_file)
                .set_boot_config(&boot_config)
                .create_disk_image(&uefi_path)
            {
                report_error_and_exit(e)
            }

            input_file = uefi_path;
        } else {
            let bios_path = get_output_path(out_dir, "bios.img");

            if let Err(e) = BiosBoot::new(&input_file)
                .set_boot_config(&boot_config)
                .create_disk_image(&bios_path)
            {
                report_error_and_exit(e)
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
        cmd.arg("-drive")
            .arg(format!("format=raw,file={}", input_file.to_str().unwrap()));

        // Add in all the excess args
        cmd.args(args);
        cmd.args(extra_args);

        let child = cmd.spawn();
        match child {
            Err(e) => {
                error!("Fatal error spawning child process: {e}");
                std::process::exit(1);
            }
            Ok(mut child) => {
                if let Err(e) = child.wait() {
                    error!("Fatal error encountered while awaiting child process: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}
