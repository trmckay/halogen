set shell := ["/usr/bin/env", "bash", "-c"]

set dotenv-load

rustc_target := "riscv64gc-unknown-none-elf"

kernel_crate := "halogen"

build_dir := "build"

base_dir := "halogen"
kernel_crate_dir := base_dir / "kernel"
common_crate_dir := base_dir / "common"
proc_macro_crate_dir := base_dir / "proc-macro"
build-dir := "build"

debug_cargo_build_elf := kernel_crate_dir / "target" / rustc_target / "debug" / kernel_crate

debug_build_elf := build_dir / "halogen-debug.elf"
debug_build_bin := build_dir / "halogen-debug.bin"

rustfmt_config := "rustfmt.toml"

default: lint build

build-dir:
    mkdir -p {{build_dir}}

build: build-dir
    cd {{kernel_crate_dir}} && cargo build
    cp {{debug_cargo_build_elf}} {{debug_build_elf}}

clippy:
    for crate in {{kernel_crate_dir}} {{common_crate_dir}} {{proc_macro_crate_dir}}; do \
        (cd "$crate" && cargo clippy) \
    done

fmt:
    find . -type f -regex '.*\.rs$' -print0 | \
        xargs -r -0 rustfmt --config-path {{rustfmt_config}}

lint: fmt clippy

clean:
    for crate in {{kernel_crate_dir}} {{common_crate_dir}} {{proc_macro_crate_dir}}; do \
        (cd "$crate" && cargo clean) \
    done
    rm -rf {{build_dir}}
