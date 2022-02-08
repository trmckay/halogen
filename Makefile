BUILD_DIR = build

CARGO_PROJ  = kernel
CARGO_FLAGS = --color=always
RUST_SOURCE = $(shell find -type f -regex '.*\.rs$$')
LDS = kernel/virt.lds

ifndef $(CROSS_COMPILE)
CROSS_COMPILE = riscv64-unknown-elf-
endif

SBI = opensbi
PLATFORM = generic

KERNEL_BUILD = kernel/target/riscv64gc-unknown-none-elf/debug/halogen

KERNEL_ELF = $(BUILD_DIR)/$(shell basename $(KERNEL_BUILD)).elf
KERNEL_BIN = $(BUILD_DIR)/$(shell basename $(KERNEL_BUILD)).bin
KERNEL_ELF_PHYS = $(BUILD_DIR)/$(shell basename $(KERNEL_BUILD))-phys.elf
KERNEL_DUMP = $(BUILD_DIR)/halogen.dump

KERNEL_TEST_ELF = $(patsubst %.elf,%-test.elf,$(KERNEL_ELF))
KERNEL_TEST_BIN = $(patsubst %.bin,%-test.bin,$(KERNEL_BIN))
KERNEL_TEST_DUMP = $(patsubst %.dump,%-test.dump,$(KERNEL_DUMP))

FIRMWARE_BUILD = opensbi/build/platform/generic/firmware/fw_jump.bin
FIRMWARE_ELF = opensbi/build/platform/generic/firmware/fw_jump.elf
FIRMWARE = $(BUILD_DIR)/opensbi.bin

LDS_VIRT = lds/virt-virt.lds
LDS_PHYS = lds/virt-phys.lds

LDFLAGS_VIRT = "-Clink-arg=-T$(LDS_VIRT)"
LDFLAGS_PHYS = "-Clink-arg=-T$(LDS_PHYS)"


.PHONY: all
all: $(KERNEL_BIN) $(FIRMWARE) $(KERNEL_DUMP)

%.dump: %.elf
	$(CROSS_COMPILE)objdump -S $^ > $@

%.bin: %.elf
	mkdir -p $(BUILD_DIR)
	$(CROSS_COMPILE)objcopy -O binary $< $@

$(KERNEL_ELF): $(RUST_SOURCE)
	cd $(CARGO_PROJ) && CARGO_BUILD_RUSTFLAGS=$(LDFLAGS_VIRT) cargo $(CARGO_FLAGS) build
	mkdir -p $(BUILD_DIR)
	cp $(KERNEL_BUILD) $@

$(KERNEL_ELF_PHYS): $(RUST_SOURCE)
	cd $(CARGO_PROJ) && CARGO_BUILD_RUSTFLAGS=$(LDFLAGS_PHYS) cargo $(CARGO_FLAGS) build
	mkdir -p $(BUILD_DIR)
	cp $(KERNEL_BUILD) $@

$(KERNEL_TEST_ELF): $(RUST_SOURCE)
	cp $$( \
		cd $(CARGO_PROJ) && \
		CARGO_BUILD_RUSTFLAGS=$(LDFLAGS_VIRT) cargo test --no-run --message-format=json | \
		jq 'select(.reason=="compiler-artifact")' | \
		jq 'select(.executable!=null)' | \
		jq -r '.executable' \
	) $@

$(SBI)/Makefile:
	git submodule update --init --recursive --remote

$(FIRMWARE_BUILD): $(SBI)/Makefile $(KERNEL_BIN)
	$(MAKE) \
		CROSS_COMPILE=$(CROSS_COMPILE) \
		PLATFORM=$(PLATFORM) \
		FW_PIC=no \
		FW_KERNEL_BIN_PATH=../$(KERNEL_BIN) \
		-C $(SBI)

$(FIRMWARE): $(FIRMWARE_BUILD)
	mkdir -p $(BUILD_DIR)
	cp $(FIRMWARE_BUILD) $(FIRMWARE)

$(KERNEL_BIN): $(KERNEL_ELF)
$(KERNEL_TEST_BIN): $(KERNEL_TEST_ELF)
$(KERNEL_DUMP): $(KERNEL_ELF)
$(KERNEL_TEST_DUMP): $(KERNEL_TEST_ELF)

.PHONY: run
run: $(KERNEL_BIN) $(FIRMWARE)
	bin/run.sh $<

.PHONY: run-debug
run-debug: $(KERNEL_BIN) $(FIRMWARE)
	bin/run.sh -g

.PHONY: run-attach
run-attach: $(KERNEL_ELF)
	bin/attach.sh $<

.PHONY: test
test: $(KERNEL_TEST_BIN) $(FIRMWARE)
	bin/run.sh $<

.PHONY: test-debug
test-debug: $(KERNEL_TEST_BIN) $(FIRMWARE)
	bin/run.sh -g $<

.PHONY: test-attach
test-attach: $(KERNEL_TEST_ELF)
	bin/attach.sh $<

.PHONY: doc
doc:
	cd $(CARGO_PROJ) && cargo doc

doc-view:
	cd $(CARGO_PROJ) && cargo doc --open

.PHONY: clean
clean:
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) clean
	rm -rf $(BUILD_DIR)/*
