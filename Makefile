include .env

SHELL            = /bin/bash
NAME             = lab_os

REQUIREMENTS     = rustup cargo git

RUST_FILES       = $(shell find . -type f -name '*.rs')
SHELL_FILES      = $(shell find . -type f -name '*.sh')

TARGET           = riscv64gc-unknown-none-elf

DOCKER_DIR       = docker
DOCKERFILE       = $(DOCKER_DIR)/Dockerfile
DOCKER_IMG       = qemu-system-riscv64

CARGO_PROJ       = $(NAME)
CARGO_FLAGS      = --verbose \
                   --target=$(TARGET)
RUST_FLAGS       = -Clink-arg=-Tld/${PLATFORM}.ld \
                   --cfg "platform=\"${PLATFORM}\""

SPHINX_DIR       = docs

BINARY           = $(CARGO_PROJ)/target/$(TARGET)/debug/$(NAME)

QEMU             = qemu-system-riscv64
QEMU_FLAGS       = -machine ${PLATFORM} \
                   -cpu rv64 -m ${MEM}  \
                   -smp ${SMP}          \
                   -nographic           \
                   -serial mon:stdio    \
                   -bios none           \
                   -kernel


default: build


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
	'#!/bin/bash\n\ncd $$(git rev-parse --show-toplevel) && make check' \
	> .git/pre-commit
	chmod +x .git/pre-commit

check:
	rustfmt --check $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	CARGO_BUILD_RUSTFLAGS="$(RUST_FLAGS)" cargo check $(CARGO_FLAGS)


build:
	cd $(CARGO_PROJ) && \
	CARGO_BUILD_RUSTFLAGS="$(RUST_FLAGS)" cargo build $(CARGO_FLAGS)


run: build
	$(QEMU_RUNNER) $(BINARY)


release:
	cd $(CARGO_PROJ) && \
	cargo build --release $(CARGO_FLAGS)


clean:
	cd $(CARGO_PROJ) && \
	cargo clean


build-docker: $(DOCKERFILE)
	sudo docker build -t $(DOCKER_IMG) $(DOCKER_DIR)


run-docker: build-docker build
	sudo docker run --rm -it \
		-v `pwd`/$(BINARY):/binary:Z \
		$(DOCKER_IMG) $(QEMU_FLAGS) /binary
