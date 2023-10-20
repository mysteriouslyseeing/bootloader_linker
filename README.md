# bootloader-linker

A quick and easy program that links your executables created using the [bootloader_api](https://crates.io/crates/bootloader_api) crate with the actual [bootloader](https://crates.io/crates/bootloader) crate into a disk image. Also capable of running disk images using qemu.

```sh
bootloader_linker -V
```
```console
bootloader_linker 0.1.4 using bootloader version 0.11.4
```

## Installation

Installation directly via cargo does not currently work, as there is an issue with the bootloader crate.
However, there is a [fork](https://github.com/mysteriouslyseeing/bootloader/) of bootloader that does. As you cannot
upload crates with Github dependencies to crates.io, you need to `cargo install` using the github repo of this crate.
Additionally, the git dependency means you need bindeps, and that means you need nightly. Therefore, the command is:
```sh
cargo +nightly install --git https://github.com/mysteriouslyseeing/bootloader_linker.git -Zbindeps
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
  -V, --version                Print version            Print version
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
runner = ["bootloader_linker", "br", "-o", "./target", "-u"]
```

Now, ```sh cargo run``` will invoke bootloader_linker instead of trying to run the executable directly.

If you want to pass extra args to qemu, you cannot use the normal notation with --,
as cargo will place the extra arguments before the binary. So, for example,
```toml
runner = ["bootloader_linker", "br", "--", "-serial", "stdio"]
```
would result in the command
```sh
bootloader_linker br -- -serial stdio [BINARY]
```
which does not work.

If you want to pass in extra args, you will have to use the -a argument to pass them in one at a time:
```sh
bootloader_linker br -a"-serial" -a"stdio" [BINARY]
```
So therefore, your runner field should look like this:
```toml
runner = ["bootloader_linker", "br", "-o", "./target", "-u", "-a'-serial'", "-a'stdio'"]
```
You can probably remove the quotes too if your extra argument does not contain spaces:
```toml
runner = ["bootloader_linker", "br", "-o", "./target", "-u", "-a-serial", "-astdio"]
```