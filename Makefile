SHELL       = /bin/bash
NAME        = lab-os

RUST_FILES  = $(shell find . -type f -name '*.rs')

TARGET      = riscv64gc-unknown-none-elf

DOCKER_DIR  = docker
DOCKERFILE  = $(DOCKER_DIR)/Dockerfile
DOCKER_IMG  = trmckay/riscv-rv64gc-dev
DOCKER_NET  = lab-os-gdb

CARGO_PROJ  = $(NAME)
CARGO_TOML  = $(CARGO_PROJ)/Cargo.toml
CARGO_FLAGS = --verbose

LINKER_FLAG = -Clink-arg=-Tld/virt.ld

BINARY      = $(CARGO_PROJ)/target/$(TARGET)/debug/$(NAME)
GDB         = riscv64-unknown-linux-gnu-gdb

OBJDUMP     = riscv64-linux-gnu-objdump

QEMU        = qemu-system-riscv64
QEMU_FLAGS  = -machine virt        \
              -cpu rv64 -m 32M     \
              -smp 2               \
              -nographic           \
              -serial mon:stdio    \
              -bios none           \
              -kernel

default: build

init:
	bin/init.sh

format: $(RUST_FILES)
	rustfmt -q $(RUST_FILES)

build: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && \
	cargo build $(CARGO_FLAGS)

check: build
	rustfmt --check $(RUST_FILES)

image:
	docker image ls | grep -oq $(DOCKER_IMG) || \
	    docker pull trmckay/riscv-rv64gc-dev

gdb-server: build
	docker network ls | grep -oq lab-os-gdb || \
	    docker network create lab-os-gdb
	docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    --network $(DOCKER_NET) \
	    --name lab-os-gdb-server \
	    $(DOCKER_IMG) $(QEMU) -s -S $(QEMU_FLAGS) /binary

gdb-attach: image build
	docker network ls | grep -oq lab-os-gdb || \
	    docker network create lab-os-gdb
	docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    -v `pwd`/$(NAME)/src:/root/src:ro \
	    -v `pwd`/docker/gdbinit:/root/.gdbinit:ro \
	    --network $(DOCKER_NET) \
	    --name lab-os-gdb-frontend \
	    $(DOCKER_IMG) $(GDB) -q /binary

run: image build
	docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    $(DOCKER_IMG) $(QEMU) $(QEMU_FLAGS) /binary

dump: image build
	docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
            $(DOCKER_IMAGE) $(OBJDUMP) -S $(BINARY) | less

clean:
	cd $(CARGO_PROJ) && \
	cargo clean
