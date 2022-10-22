set dotenv-load
set shell             := ["/usr/bin/env", "bash", "-c"]

rustc_target          := "riscv64gc-unknown-none-elf"

kernel_crate_name     := "halogen"

base_dir              := "halogen"

kernel_crate_dir      := base_dir / "kernel"
common_crate_dir      := base_dir / "common"
proc_macro_crate_dir  := base_dir / "proc-macro"

debug_cargo_build_elf := kernel_crate_dir / "target" / rustc_target / "debug" / kernel_crate_name

build_dir             := "build"

debug_build_elf       := build_dir / "halogen-debug.elf"
debug_build_bin       := build_dir / "halogen-debug.bin"

rustfmt_config        := "rustfmt.toml"

default: lint

build-dir:
    mkdir -p {{build_dir}}

elf: build-dir
    cd {{kernel_crate_dir}} && cargo build
    cp {{debug_cargo_build_elf}} {{debug_build_elf}}

bin: elf
    ${CROSS_COMPILE}objcopy -O binary {{debug_build_elf}} {{debug_build_bin}}

clippy:
    for crate in {{kernel_crate_dir}} {{common_crate_dir}} {{proc_macro_crate_dir}}; do \
        (cd "$crate" && cargo clippy) \
    done

fmt:
    find . -type f -regex '.*\.rs$' -print0 | \
        xargs -r0 rustfmt --config-path {{rustfmt_config}}

fmt-check:
    find . -type f -regex '.*\.rs$' -print0 | \
        xargs -r0 rustfmt --check --config-path {{rustfmt_config}}

lint: clippy fmt-check

clean:
    for crate in {{kernel_crate_dir}} {{common_crate_dir}} {{proc_macro_crate_dir}}; do \
        (cd "$crate" && cargo clean) \
    done
    rm -f {{debug_build_elf}}
    rm -f {{debug_build_bin}}
