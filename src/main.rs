use std::path::PathBuf;

use bootloader_linker::{init_logger, Config, SubCommand};
use clap::Parser;
use log::{error, info, trace, warn};

use bootloader::{BootConfig, DiskImageBuilder};

fn get_output_path(out_dir: PathBuf, default_file_name: &str) -> PathBuf {
    if out_dir.is_dir() {
        out_dir.join(default_file_name)
    } else if out_dir
        .to_str()
        .map(|s| {
            s.as_bytes()
                .last()
                .map(|&b| b == b'/' || b == b'\\')
                .unwrap_or(false)
        })
        .unwrap_or(false)
    {
        if std::fs::create_dir_all(&out_dir).is_ok() {
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
        files_to_mount,
        no_ovmf,
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

        let output_path = get_output_path(out_dir, if uefi { "uefi.img" } else { "bios.img" });

        let mut builder = DiskImageBuilder::new(input_file);
        builder.set_boot_config(&boot_config);
        files_to_mount.into_iter().for_each(|path| {
            let name = match path.file_name() {
                Some(s) => {
                    if let Some(s) = s.to_str() {
                        s.to_owned()
                    } else {
                        warn!("File name {} not valid utf-8", s.to_string_lossy());
                        return;
                    }
                }
                None => {
                    warn!("File {} could not be resolved.", path.display());
                    return;
                }
            };
            builder.set_file(name, path);
        });

        let result = if uefi {
            builder.create_uefi_image(&output_path)
        } else {
            builder.create_bios_image(&output_path)
        };

        if let Err(e) = result {
            report_error_and_exit(e);
        }
        input_file = output_path;
    }
    if run {
        trace!("Running disk image");
        let mut cmd = std::process::Command::new(qemu_path);
        if uefi && !no_ovmf {
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
