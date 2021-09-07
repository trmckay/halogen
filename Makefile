SHELL       = /bin/bash
NAME        = halogen
RUST_FILES  = $(shell find . -type f -name '*.rs')

TARGET      = riscv64gc-unknown-none-elf

DOCKER_DIR  = docker
ifeq ($(PULL),1)
DOCKER_IMG  = trmckay/riscv-toolchain
else
DOCKER_IMG  = riscv-toolchain
endif
DOCKER_NET  = halogen-gdb

CARGO_PROJ  = $(NAME)
CARGO_TOML  = $(CARGO_PROJ)/Cargo.toml
CARGO_FLAGS = --verbose --target $(TARGET)
RUSTC_FLAGS = --cfg platform=\"$(QEMU_MACHN)\" \
	      -g \
	      --emit=obj,link \
	      -Clink-arg=-Tlink.ld

BINARY      = $(CARGO_PROJ)/target/$(TARGET)/debug/$(NAME)

GDB         = riscv64-unknown-linux-gnu-gdb
OBJDUMP     = riscv64-linux-gnu-objdump

MEM         = 256M
NCORE       = 1
QEMU        = qemu-system-riscv64
QEMU_MACHN  = virt
QEMU_FLAGS  = -machine virt        \
              -cpu rv64 -m $(MEM)  \
              -smp $(NCORE)        \
              -nographic           \
              -serial mon:stdio    \
              -bios none           \
              -kernel

.PHONY: all
all: build

.PHONY: init
init:
	bin/init.sh

.PHONY: format
format: $(RUST_FILES)
	rustfmt $(RUST_FILES)

.PHONY: build
build: $(RUST_FILES) $(CARGO_TOML)
	@cd $(CARGO_PROJ) && \
	CARGO_BUILD_RUSTFLAGS="$(RUSTC_FLAGS)" cargo build $(CARGO_FLAGS) && \
	CARGO_BUILD_RUSTFLAGS="$(RUSTC_FLAGS)" cargo test --no-run $(CARGO_FLAGS)

.PHONY: docker
docker:
	@docker image ls | grep -oq $(DOCKER_IMG) || \
	    docker pull $(DOCKER_IMG) || \
	    docker build -t $(DOCKER_IMG) $(DOCKER_DIR)
	@docker network ls | grep -oq $(DOCKER_NET) || \
	    docker network create $(DOCKER_NET)

.PHONY: test
test: $(RUST_FILES) $(CARGO_TOML) docker
	@docker run --rm \
	    -v \
		$(shell \
	            cd $(CARGO_PROJ) && \
		    	CARGO_BUILD_RUSTFLAGS="$(RUSTC_FLAGS)" \
			cargo test $(CARGO_FLAGS) --no-run --message-format=json | \
	            jq -r --slurp '.[-2]["executable"]' \
		):/binary:ro \
	    $(DOCKER_IMG) $(QEMU) $(QEMU_FLAGS) /binary

.PHONY: gdb-run
gdb-run: build docker
	@docker run --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    --network $(DOCKER_NET) \
	    --name halogen-gdb-server \
	    $(DOCKER_IMG) $(QEMU) -s -S $(QEMU_FLAGS) /binary

.PHONY: gdb-test
gdb-test: build docker
	@docker run --rm -it \
	    -v \
		$(shell \
	            cd $(CARGO_PROJ) && \
		    	CARGO_BUILD_RUSTFLAGS="$(RUSTC_FLAGS)" \
			cargo test $(CARGO_FLAGS) --no-run --message-format=json | \
	            jq -r --slurp '.[-2]["executable"]' \
		):/binary:ro \
	    --network $(DOCKER_NET) \
	    --name halogen-gdb-server \
	    $(DOCKER_IMG) $(QEMU) -s -S $(QEMU_FLAGS) /binary

.PHONY: gdb-attach
gdb-attach: build docker
	@docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    -v `pwd`/$(NAME)/src:/root/src:ro \
	    -v `pwd`/docker/gdbinit:/root/.gdbinit:ro \
	    --network $(DOCKER_NET) \
	    --name halogen-gdb-frontend \
	    $(DOCKER_IMG) $(GDB) -q /binary

.PHONY: run
run: build docker
	@docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
	    $(DOCKER_IMG) $(QEMU) $(QEMU_FLAGS) /binary

.PHONY: build
dump: build docker
	@docker run \
	    --rm -it \
	    -v `pwd`/$(BINARY):/binary:ro \
            $(DOCKER_IMG) $(OBJDUMP) -S $(BINARY) | less

.PHONY: clean
clean:
	@cd $(CARGO_PROJ) && \
	cargo clean

.PHONY: check-format
check-format:
	rustfmt --check $(RUST_FILES)
