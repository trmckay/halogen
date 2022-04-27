BUILD_DIR = build

CARGO_PROJ  = kernel
CARGO_FLAGS = --color=always
RUST_SOURCE = $(shell find -type f -regex '.*\.rs$$')

ifndef $(CROSS_COMPILE)
CROSS_COMPILE = riscv64-unknown-elf-
endif

SBI = opensbi
PLATFORM = generic

KERNEL_BUILD = target/riscv64gc-unknown-none-elf/debug/halogen

KERNEL_ELF = $(BUILD_DIR)/halogen.elf
KERNEL_BIN = $(BUILD_DIR)/halogen.bin
KERNEL_DUMP = $(BUILD_DIR)/halogen.dump

KERNEL_TEST_ELF = $(patsubst %.elf,%-test.elf,$(KERNEL_ELF))
KERNEL_TEST_BIN = $(patsubst %.bin,%-test.bin,$(KERNEL_BIN))
KERNEL_TEST_DUMP = $(patsubst %.dump,%-test.dump,$(KERNEL_DUMP))

FIRMWARE_BUILD = opensbi/build/platform/generic/firmware/fw_jump.bin
FIRMWARE_ELF = opensbi/build/platform/generic/firmware/fw_jump.elf
FIRMWARE = $(BUILD_DIR)/opensbi.bin

.PHONY: all
all: $(KERNEL_BIN) $(FIRMWARE) $(KERNEL_DUMP) $(KERNEL_TEST_BIN) $(KERNEL_TEST_DUMP)

%.dump: %.elf
	$(CROSS_COMPILE)objdump -S $^ > $@

%.bin: %.elf
	mkdir -p $(BUILD_DIR)
	$(CROSS_COMPILE)objcopy -O binary $< $@

$(KERNEL_ELF): $(RUST_SOURCE)
	cd $(CARGO_PROJ) && cargo build
	mkdir -p $(BUILD_DIR)
	cp $(KERNEL_BUILD) $@

$(KERNEL_TEST_ELF): $(RUST_SOURCE)
	cp $$(cd $(CARGO_PROJ) && cargo test --no-run --message-format=json | jq 'select(.reason=="compiler-artifact")' | jq 'select(.executable!=null)' | jq -r '.executable') $@ 2> /dev/null || (cd $(CARGO_PROJ) && cargo test --no-run )


$(SBI)/Makefile:
	git submodule update --init --recursive --remote

$(FIRMWARE_BUILD): $(SBI)/Makefile $(KERNEL_BIN)
	$(MAKE) CROSS_COMPILE=$(CROSS_COMPILE) PLATFORM=$(PLATFORM) FW_PIC=no FW_KERNEL_BIN_PATH=../$(KERNEL_BIN) -C $(SBI)

$(FIRMWARE): $(FIRMWARE_BUILD)
	mkdir -p $(BUILD_DIR)
	cp $(FIRMWARE_BUILD) $(FIRMWARE)

$(KERNEL_BIN): $(KERNEL_ELF)
$(KERNEL_TEST_BIN): $(KERNEL_TEST_ELF)
$(KERNEL_DUMP): $(KERNEL_ELF)
$(KERNEL_TEST_DUMP): $(KERNEL_TEST_ELF)

.PHONY: run
run: $(FIRMWARE) $(KERNEL_BIN) $(KERNEL_ELF)
	bin/run.sh $(FIRMWARE) $(KERNEL_BIN) $(KERNEL_ELF)

.PHONY: test
test: $(FIRMWARE) $(KERNEL_TEST_BIN) $(KERNEL_TEST_ELF)
	bin/run.sh $(FIRMWARE) $(KERNEL_TEST_BIN) $(KERNEL_TEST_ELF)

.PHONY: doc
doc:
	cargo doc

.PHONY: doc-open
doc-open:
	cargo doc --open

.PHONY: clean
clean:
	cargo clean
	rm -rf $(BUILD_DIR)/*
	make -C $(SBI) clean

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: check
check:
	cargo check
