# bootloader-linker

A quick and easy program that links your executables created using the [bootloader_api](https://crates.io/crates/bootloader_api) crate with the actual [bootloader](https://crates.io/crates/bootloader) crate into a disk image. Also capable of running disk images using qemu.

```sh
bootloader_linker -V
```
```console
bootloader_linker 0.1.0 using bootloader version 0.11.4
```

## Installation

Install with cargo:
```sh
cargo install bootloader_linker
```

## Usage

```sh
bootloader_linker -h
```
```console
A quick and easy program that links your executables created using bootloader_api with the actual bootloader.

Usage: bootloader_linker.exe [OPTIONS] <COMMAND> <FILE> [EXTRA_ARGS]...

Arguments:
  <COMMAND>        [possible values: build, run, build-run]
  <FILE>           The binary/.img file to operate on. Can be created relatively easy using the bootloader_api crate
  [EXTRA_ARGS]...  Extra args to pass to qemu

Options:
  -u, --uefi                   Sets the loader to use uefi instead of bios
  -o, --out-dir <OUT_DIR>      The directory to put output files in. Ignored if not building a disk image [default: ./]
  -q, --qemu-path <QEMU_PATH>  The name of the qemu executable. Ignored if not running a disk image [default: qemu-system-x86_64]
  -v, --verbosity...           Configures the amount of logging. If this flag appears more times more things will be logged
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

Please note that in order to run the disk image you need to have [qemu](https://www.qemu.org/) installed. If the executable is not in PATH, you can specify it with --qemu-path

## Examples

```sh
> bootloader_linker build-run test_binary -o ./target -- -serial stdio
```
```console
[bootloader-linker] INFO - Building disk image
[bootloader-linker] INFO - Running disk image
/// Bootloader booting info...
Hello world!
```

The test binary used to create this output is in the repo and was built from source code that looks like this for the x86_64-unknown-none target:
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::entry_point;

entry_point!(main);

fn main(_info: &'static mut bootloader_api::BootInfo) -> ! {
    qemu_print::qemu_print!("Hello world!");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

[qemu_print](https://crates.io/crates/qemu_print) crate

There are also short-hands for the three subcommands:
- `build` can be shortened to `b`
- `run` can be shortened to `r`
- `build-run` can be shortened to `br`

## Advanced use

bootloader-linker is suitable for use with cargo run. Add these lines to your .cargo/config.toml:
```toml
[target.'cfg(target_os = "none")']
runner = ["bootloader_linker", "br", "-o", "./target"]
```

Now, `cargo run` will invoke bootloader_linker instead of trying to run the executable directly.

If you want to pass extra args to qemu, you cannot simply add to .cargo/config.toml, you have to pass them in the run call:

```sh
cargo run --release -- -- -serial stdio
```

Note the the two double dashes are required because the first one tells cargo to pass the remaining arguments to bootloader-linker and the second one tells bootloader-linker to pass the remaining arguments to qemu.

Running that command every time can be kind of clunky, so you might want to add an alias in Cargo.toml:
```toml
[alias]
rq = "run -- -- -serial stdio"
rqr = "run --release -- -- -serial stdio"
```
Now, you just have to do `cargo rq` or `cargo rqr` to build and run the disk image in qemu in release or debug mode respectively.