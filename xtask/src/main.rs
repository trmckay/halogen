use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use clap::{Arg, Command as ClapCommand};
use serde_json as json;

fn main() {
    let debug_subcmd = ClapCommand::new("debug").about("Run in QEMU and start a GDB server");
    let dump_subcmd = ClapCommand::new("dump").about("Show the object dump of the kernel image");
    let attach_subcmd = ClapCommand::new("attach")
        .about("Attach to the GDB server")
        .arg(
            Arg::new("gdb-args")
                .takes_value(true)
                .required(false)
                .multiple_values(true)
                .last(true)
                .help("Pass extra arguments to the GDB invocation"),
        );

    let args = ClapCommand::new("Halogen cargo-xtask build system")
        .subcommand_required(true)
        .subcommand(ClapCommand::new("build").about("Build the kernel in `build`"))
        .subcommand(ClapCommand::new("clean").about("Clean up compiler artifacts"))
        .subcommand(ClapCommand::new("show-dump").about("View an object dump of the kernel"))
        .subcommand(ClapCommand::new("docs").about("Open crate documentation in the browser"))
        .subcommand(
            ClapCommand::new("test")
                .about("Run unit tests in QEMU")
                .subcommand(dump_subcmd.clone())
                .subcommand(debug_subcmd.clone())
                .subcommand(attach_subcmd.clone()),
        )
        .subcommand(
            ClapCommand::new("run")
                .about("Run the kernel in QEMU")
                .subcommand(dump_subcmd)
                .subcommand(debug_subcmd)
                .subcommand(attach_subcmd),
        )
        .subcommand(ClapCommand::new("fmt").about("Check format with `rustfmt`"))
        .subcommand(ClapCommand::new("check").about("Check that project compiles and is formatted"))
        .get_matches();

    if let Err(why) = match args.subcommand() {
        Some(("test", test_args)) => {
            match test_args.subcommand() {
                Some(("dump", _)) => show_dump(&format!("{}/{}", BUILD_DIR, KERNEL_TEST_ELF_DEST)),
                Some(("debug", _)) => test(true),
                Some(("attach", attach_args)) => {
                    attach(
                        format!("{}/{}", BUILD_DIR, KERNEL_TEST_ELF_DEST),
                        &match attach_args.values_of("gdb-args") {
                            Some(gdb_args) => gdb_args.collect::<Vec<&str>>(),
                            None => Vec::default(),
                        },
                    )
                }
                _ => test(false),
            }
        }
        Some(("build", _)) => build(),
        Some(("clean", _)) => clean(),
        Some(("show-dump", _)) => show_dump(&format!("{}/{}", BUILD_DIR, KERNEL_TEST_ELF_DEST)),
        Some(("docs", _)) => open_docs(),
        Some(("fmt", _)) => fmt(false),
        Some(("check", _)) => check(),
        Some(("run", run_args)) => {
            match run_args.subcommand() {
                Some(("dump", _)) => show_dump(&format!("{}/{}", BUILD_DIR, KERNEL_ELF_DEST)),
                Some(("debug", _)) => run(true),
                Some(("attach", attach_args)) => {
                    attach(
                        format!("{}/{}", BUILD_DIR, KERNEL_ELF_DEST),
                        &match attach_args.values_of("gdb-args") {
                            Some(gdb_args) => gdb_args.collect::<Vec<&str>>(),
                            None => Vec::default(),
                        },
                    )
                }
                _ => run(false),
            }
        }
        _ => unreachable!(),
    } {
        println!("Error: {}", why);
        std::process::exit(1);
    }
}

macro_rules! cmd {
    ($cmd:expr, $($arg:expr),+ $(,)*) => {{
        let args = &[$($arg),+];
        Command::new($cmd).args(args)
    }};
}

macro_rules! check_exit {
    ($cmd:expr, $msg:expr) => {
        if $cmd.status()?.success() {
            Ok(())
        } else {
            Err(anyhow!($msg))
        }
    };
}

const CROSS_COMPILE: &str = "riscv64-unknown-elf-";
const RUSTC_TARGET: &str = "riscv64gc-unknown-none-elf";

const SBI_DIR: &str = "opensbi";
const KERNEL_DIR: &str = "halogen/kernel";
const COMMON_LIB_DIR: &str = "halogen/common";
const PROGRAMS_DIR: &str = "halogen/userspace/programs";
const XTASK_DIR: &str = "xtask";
const INCLUDE_PROGRAMS_DIR: &str = "halogen/userspace/include-programs";
const BUILD_DIR: &str = "build";

const CRATES: &[&str] = &[KERNEL_DIR, COMMON_LIB_DIR, INCLUDE_PROGRAMS_DIR, XTASK_DIR];

const KERNEL_ELF_DEST: &str = "halogen.elf";
const KERNEL_BIN_DEST: &str = "halogen.bin";
const KERNEL_TEST_ELF_DEST: &str = "halogen-test.elf";
const KERNEL_TEST_BIN_DEST: &str = "halogen-test.bin";

const SBI_BIN_DEST: &str = "opensbi.bin";
const SBI_PIC: &str = "no";
const SBI_PLATFORM: &str = "generic";
const SBI_BIN: &str = "build/platform/generic/firmware/fw_jump.bin";

fn build() -> Result<()> {
    check_exit!(
        cmd!("cargo", "build").current_dir(KERNEL_DIR),
        "failed to build kernel"
    )?;

    check_exit!(
        cmd!(
            "make",
            format!("-j{}", num_cpus::get()),
            format!("CROSS_COMPILE={}", CROSS_COMPILE),
            format!("FW_PIC={}", SBI_PIC),
            format!("PLATFORM={}", SBI_PLATFORM),
        )
        .current_dir(SBI_DIR),
        "failed to build firmware"
    )?;

    fs::create_dir_all(BUILD_DIR)?;

    let kernel_elf = format!("{}/{}", BUILD_DIR, KERNEL_ELF_DEST);
    let kernel_bin = format!("{}/{}", BUILD_DIR, KERNEL_BIN_DEST);

    fs::copy(
        format!("{}/target/{}/debug/halogen", KERNEL_DIR, RUSTC_TARGET),
        &kernel_elf,
    )?;

    fs::copy(
        format!("{}/{}", SBI_DIR, SBI_BIN),
        format!("{}/{}", BUILD_DIR, SBI_BIN_DEST),
    )?;

    check_exit!(
        cmd!(
            format!("{}objcopy", CROSS_COMPILE),
            "-O",
            "binary",
            &kernel_elf,
            &kernel_bin,
        ),
        "failed to flatten kernel binary"
    )?;

    let cargo_kernel_test_elf: String = String::from_utf8_lossy(
        &cmd!("cargo", "test", "--no-run", "--message-format=json")
            .current_dir(KERNEL_DIR)
            .stderr(Stdio::inherit())
            .output()?
            .stdout,
    )
    .strip_suffix('\n')
    .expect("failed to parse cargo output")
    .split('\n')
    .find_map(|json_str| {
        Some(
            json::from_str::<json::Value>(json_str)
                .ok()?
                .get("executable")?
                .as_str()?
                .to_string(),
        )
    })
    .expect("could not parse test executable from cargo ouput");

    let kernel_test_elf = format!("{}/{}", BUILD_DIR, KERNEL_TEST_ELF_DEST);
    let kernel_test_bin = format!("{}/{}", BUILD_DIR, KERNEL_TEST_BIN_DEST);

    fs::copy(&cargo_kernel_test_elf, &kernel_test_elf)?;

    check_exit!(
        cmd!(
            format!("{}objcopy", CROSS_COMPILE),
            "-O",
            "binary",
            &kernel_test_elf,
            &kernel_test_bin,
        ),
        "failed to flatten kernel test binary"
    )?;

    Ok(())
}

fn fmt(check: bool) -> Result<()> {
    for dir in CRATES {
        check_exit!(
            cmd!("cargo", "clippy").current_dir(dir),
            "failed clippy check"
        )?;
        if check {
            check_exit!(
                cmd!("cargo", "fmt", "--check").current_dir(dir),
                "failed format check"
            )?;
        } else {
            check_exit!(cmd!("cargo", "fmt").current_dir(dir), "failed to format")?;
        }
    }
    Ok(())
}

fn clean() -> Result<()> {
    let _ = fs::remove_dir_all(BUILD_DIR);
    for dir in CRATES {
        check_exit!(
            cmd!("cargo", "clean").current_dir(dir),
            "failed to cargo clean"
        )?;
    }
    check_exit!(
        cmd!("make", "clean").current_dir(SBI_DIR),
        "failed to clean OpenSBI"
    )?;
    check_exit!(
        cmd!(
            "sh",
            "-c",
            &format!(
                "find {} -type f -regex '.*\\.\\(o\\|elf\\|bin\\)$' -print0 | xargs -0 rm -f",
                PROGRAMS_DIR
            ),
        ),
        "failed to find and delete compiler artifacts"
    )?;
    Ok(())
}

const QEMU: &str = "qemu-system-riscv64";
const QEMU_ARGS: &[&str] = &[
    "-machine",
    "virt",
    "-cpu",
    "rv64",
    "-m",
    "512M",
    "-smp",
    "1",
    "-nographic",
    "-serial",
    "mon:stdio",
];

fn qemu<T>(kernel: T, debug: bool) -> Result<()>
where
    T: AsRef<Path>, {
    let mut args = QEMU_ARGS.to_vec();
    let bios = format!("{}/{}", BUILD_DIR, SBI_BIN_DEST);
    let kernel = kernel.as_ref().to_string_lossy();

    args.extend_from_slice(&["--bios", &bios, "--kernel", &kernel]);

    if debug {
        args.extend_from_slice(&["-s", "-S"]);
    }

    let err = exec::Command::new(QEMU).args(&args).exec();
    Err(anyhow!("failed to exec QEMU: {}", err))
}

fn run(debug: bool) -> Result<()> {
    build()?;
    qemu(format!("{}/{}", BUILD_DIR, KERNEL_BIN_DEST), debug)
}

fn test(debug: bool) -> Result<()> {
    check_exit!(
        cmd!("cargo", "test", "--color=always").current_dir(COMMON_LIB_DIR),
        "failed unit tests"
    )?;
    build()?;
    qemu(format!("{}/{}", BUILD_DIR, KERNEL_TEST_BIN_DEST), debug)
}

const GDBINIT: &[&str] = &[
    "target remote :1234",
    "set architecture riscv:rv64",
    "set disassemble-next-line auto",
    "set riscv use-compressed-breakpoints yes",
];

fn attach<T>(elf: T, gdb_args: &[&str]) -> Result<()>
where
    T: AsRef<Path>, {
    let gdb = env::var("RUST_GDB").unwrap_or(format!("{}gdb", CROSS_COMPILE));
    let rust_gdb_env = format!("RUST_GDB={}", gdb);
    let mut args: Vec<&str> = vec![&rust_gdb_env, "rust-gdb", "-nh"];

    GDBINIT
        .iter()
        .flat_map(|&cmd| ["-ex", cmd])
        .for_each(|arg| args.push(arg));

    let symbol_file_cmd = format!("symbol-file '{}'", elf.as_ref().to_string_lossy());
    args.extend_from_slice(&["-q", "-ex", &symbol_file_cmd]);
    args.extend_from_slice(gdb_args);

    let err = exec::Command::new("env").args(&args).exec();
    Err(anyhow!("failed to exec rust-gdb: {}", err))
}

fn check() -> Result<()> {
    fmt(true)?;
    for dir in &[KERNEL_DIR, XTASK_DIR, INCLUDE_PROGRAMS_DIR] {
        check_exit!(
            cmd!("cargo", "check").current_dir(dir),
            "failed cargo check"
        )?;
        check_exit!(
            cmd!("cargo", "clippy").current_dir(dir),
            "failed cargo clippy"
        )?;
    }
    Ok(())
}

fn show_dump(elf: &str) -> Result<()> {
    build()?;

    let objdump = format!("{}objdump", CROSS_COMPILE);
    let pager = env::var("PAGER").unwrap_or_else(|_| "less".to_string());

    check_exit!(
        cmd!("sh", "-c", &format!("{} -S {} | {}", objdump, elf, pager)),
        "failed to open object dump in pager"
    )?;

    Ok(())
}

fn open_docs() -> Result<()> {
    check_exit!(
        cmd!("cargo", "doc", "--document-private-items", "--open").current_dir(KERNEL_DIR),
        "Failed to build documentation"
    )?;
    Ok(())
}
