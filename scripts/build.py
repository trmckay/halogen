#!/usr/bin/env python3

import json
import subprocess
import os
import sys
from typing import List, Optional, Tuple
import multiprocessing

JOB_COUNT = multiprocessing.cpu_count()


def step(in_dir: str, cmd: List[str], capture=False) -> Optional[Tuple[str, str]]:
    cwd = os.getcwd()
    os.chdir(in_dir)
    if capture:
        (stdout, stderr) = subprocess.Popen(cmd, stdout=subprocess.PIPE).communicate()
        os.chdir(cwd)
        return (stdout, stderr)
    else:
        subprocess.run(cmd, check=True)
        os.chdir(cwd)


TASKS = {}
DEFAULT_TASK = None


def task(t):
    global TASKS
    TASKS[t.__name__] = t
    return t


def default_task(t):
    global DEFAULT_TASK
    DEFAULT_TASK = t
    task(t)
    return t


def main():
    if (
        (len(sys.argv) == 1 and default_task is None)
        or "-h" in sys.argv
        or "--help" in sys.argv
    ):
        print("build.py <task> <task> ...", file=sys.stderr)
        print("Tasks: {}".format(", ".join(TASKS.keys())), file=sys.stderr)

    elif len(sys.argv[1:]) == 0:
        DEFAULT_TASK()

    else:
        for arg in sys.argv[1:]:
            task = TASKS.get(arg)
            if task:
                task()
            else:
                print(f"Task '{arg}' undefined.")
                sys.exit(1)


######## TASK DEFINITIONS ########

KERNEL = "halogen/kernel"
PROC_MACRO = "halogen/proc-macro"
COMMON = "halogen/common"
OPENSBI = "opensbi"
CRATES = [KERNEL, PROC_MACRO, COMMON]

RUSTC_TARGET = "riscv64gc-unknown-none-elf"
CROSS_COMPILE = "riscv64-unknown-elf-"
QEMU = "qemu-system-riscv64"

BUILD_DIR = "build"
KERNEL_ELF = f"{BUILD_DIR}/halogen.elf"
KERNEL_BIN = f"{BUILD_DIR}/halogen.bin"
KERNEL_TEST_ELF = f"{BUILD_DIR}/halogen-test.elf"
KERNEL_TEST_BIN = f"{BUILD_DIR}/halogen-test.bin"
SBI_BIN = f"{OPENSBI}/build/platform/generic/firmware/fw_jump.bin"


def strip(src: str, dest: str):
    step(".", [f"{CROSS_COMPILE}objcopy", "-O", "binary", src, dest])


def qemu(bios: str, kernel: str, debug: bool = False):
    args = [
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
        "--bios",
        bios,
        "--kernel",
        kernel,
    ]

    if debug:
        args = ["-S", "-s"] + args

    step(".", [QEMU] + args)


@task
def build():
    step(".", ["mkdir", "-p", BUILD_DIR])
    step(KERNEL, ["cargo", "build"])
    step(".", ["cp", f"halogen/kernel/target/{RUSTC_TARGET}/debug/halogen", KERNEL_ELF])
    strip(KERNEL_ELF, KERNEL_BIN)


@task
def opensbi():
    step(
        OPENSBI,
        [
            "make",
            f"-j{JOB_COUNT}",
            f"CROSS_COMPILE={CROSS_COMPILE}",
            "FW_PIC=no",
            "PLATFORM=generic",
        ],
    )


@task
def test(debug: bool = False):
    """
    Compile the cargo-test configuration and parse out the executable path from
    the output. See https://github.com/rust-lang/cargo/issues/1924 for the reason
    this hacky Python build system exists.
    """
    step(".", ["mkdir", "-p", BUILD_DIR])
    (output, _) = step(
        KERNEL,
        ["cargo", "test", "--no-run", "--message-format=json"],
        capture=True,
    )
    executable = [
        x["executable"]
        for x in [json.loads(line) for line in output.splitlines()]
        if x.get("executable") is not None
    ][0]
    step(".", ["cp", executable, KERNEL_TEST_ELF])
    strip(KERNEL_TEST_ELF, KERNEL_TEST_BIN)
    opensbi()
    qemu(SBI_BIN, KERNEL_TEST_BIN, debug)


@task
def debug_server():
    test(True)


@task
def clean():
    for crate in CRATES:
        step(crate, ["cargo", "clean"])

    step(OPENSBI, ["make", "clean"])


@task
def fmt():
    for crate in CRATES:
        step(crate, ["cargo", "fmt"])


@task
def fmt_check():
    for crate in CRATES:
        step(crate, ["cargo", "fmt", "--check"])


@task
def clippy():
    for crate in CRATES:
        step(crate, ["cargo", "clippy"])


@default_task
def check():
    fmt_check()
    build()
    test()


##################################

if __name__ == "__main__":
    main()
