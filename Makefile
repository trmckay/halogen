SHELL             = /bin/bash
NAME              = lab_os

RUST_FILES        = $(shell find . -type f -name '*.rs')

TARGET            = riscv64gc-unknown-none-elf

DOCKER_DIR        = .
DOCKERFILE        = $(DOCKER_DIR)/Dockerfile
DOCKER_IMG        = qemu-system-riscv64

CARGO_PROJ        = $(NAME)
CARGO_FLAGS       = --verbose

LINKER_FLAG       = -Clink-arg=-Tld/virt.ld

BINARY            = $(CARGO_PROJ)/target/$(TARGET)/debug/$(NAME)

QEMU              = qemu-system-riscv64
QEMU_FLAGS        = -machine virt        \
                    -cpu rv64 -m 32M     \
                    -smp 1               \
                    -nographic           \
                    -serial mon:stdio    \
                    -bios none           \
                    -kernel


default: build

init:
	cd $(CARGO_PROJ) && \
	rustup override set nightly && \
	rustup target add riscv64gc-unknown-none-elf
	# Install pre-commit hooks
	echo -e \
	    '#!/bin/bash\n\ncd $$(git rev-parse --show-toplevel) && make check' \
	    > .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit

fmt: $(RUST_FILES)
	rustfmt -q $(RUST_FILES)

check: $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	cargo check $(CARGO_FLAGS);
	rustfmt -q --check $(RUST_FILES)

build: $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	cargo build $(CARGO_FLAGS)

run: $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	cargo run $(CARGO_FLAGS)

test: $(RUST_FILES)
	cd $(CARGO_PROJ) && \
	cargo test $(CARGO_FLAGS)

dump: build
	riscv64-linux-gnu-objdump -S $(BINARY) | less

run-docker: build
	docker image ls | \
	grep -oq $(DOCKER_IMG) || \
	docker build -t $(DOCKER_IMG) $(DOCKER_DIR)
	docker run --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    $(DOCKER_IMG) $(QEMU_FLAGS) /binary

clean:
	cd $(CARGO_PROJ) && \
	cargo clean
