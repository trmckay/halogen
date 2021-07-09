include .env

SHELL             = /bin/bash
NAME              = lab_os

REQUIREMENTS      = rustup cargo git docker rustfmt

RUST_FILES        = $(shell find . -type f -name '*.rs')

TARGET            = riscv64gc-unknown-none-elf
PLATFORMS         = sifive_u virt

DOCKER_DIR        = docker
DOCKERFILE        = $(DOCKER_DIR)/Dockerfile
DOCKER_IMG        = qemu-system-riscv64

CARGO_PROJ        = $(NAME)
CARGO_FLAGS       = --verbose \
                   --target=$(TARGET)

LINKER_FLAG       = -Clink-arg=-Tld/${PLATFORM}.ld
ENV_PLATFORM_FLAG = --cfg "platform=\"${PLATFORM}\""

BINARY            = $(CARGO_PROJ)/target/$(TARGET)/debug/$(NAME)

QEMU              = qemu-system-riscv64
QEMU_FLAGS        = -machine ${PLATFORM} \
                    -cpu rv64 -m ${MEM}  \
                    -smp ${SMP}          \
                    -nographic           \
                    -serial mon:stdio    \
                    -bios none           \
                    -kernel


default: build


.env:
	# Default environment
	echo -e 'PLATFORM=virt\nSMP=1\nMEM=32M' > .env


init:
	# Check for requirements
	for req in $(REQUIREMENTS); \
	    do command -v $$req > /dev/null || echo "Missing requirement '$$req'";\
	done
	# Set-up RISC-V rv64gc toolchain
	cd $(CARGO_PROJ) && \
	rustup override set nightly && \
	rustup target add riscv64gc-unknown-none-elf
	# Install pre-commit hooks
	echo -e \
	'#!/bin/bash\n\ncd $$(git rev-parse --show-toplevel) && make fmt' \
	> .git/pre-commit
	chmod +x .git/pre-commit


fmt:
	rustfmt $(RUST_FILES)


check:
	rustfmt --check $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	$(foreach \
	    platform, \
            $(PLATFORMS), \
	    CARGO_BUILD_RUSTFLAGS="--cfg platform=\"$(platform)\"" \
	    cargo check $(CARGO_FLAGS); \
	)


build: .env
	cd $(CARGO_PROJ) && \
	CARGO_BUILD_RUSTFLAGS="$(LINKER_FLAG) $(ENV_PLATFORM_FLAG)" \
	cargo build $(CARGO_FLAGS)


clean:
	cd $(CARGO_PROJ) && \
	cargo clean


run: build .env
	sudo docker image ls | \
	grep $(DOCKER_IMG) \
	    || sudo docker build -t $(DOCKER_IMG) $(DOCKER_DIR)
	sudo docker run --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    $(DOCKER_IMG) $(QEMU_FLAGS) /binary
