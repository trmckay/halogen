include .env

SHELL            = /bin/bash
NAME             = lab_os

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

# === BAREMETAL KERNEL RULES ===

init:
	cd $(CARGO_PROJ) && \
	rustup override set nightly && \
	rustup target add riscv64gc-unknown-none-elf
	eval `pwd`/scripts/install_hooks.sh

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


# === DOCKER KERNEL RULES ===

build-docker: $(DOCKERFILE)
	sudo docker build -t $(DOCKER_IMG) $(DOCKER_DIR)

run-docker: build-docker build
	sudo docker run --rm -it \
		-v `pwd`/$(BINARY):/binary:Z \
		$(DOCKER_IMG) $(QEMU_FLAGS) /binary
