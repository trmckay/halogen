PROJECT = rvr-kernel
BUILD = target/riscv32i-unknown-none-elf/debug
DEST = target/otter

RISCV_PREFIX = riscv32-unknown-elf-

OBJCOPY = $(RISCV_PREFIX)objcopy
OBJDUMP = $(RISCV_PREFIX)objdump

all: dirs $(DEST)/mem.txt $(DEST)/$(PROJECT).dump

# link with gcc
$(BUILD)/$(PROJECT):
	cargo build

$(DEST)/$(PROJECT).dump: $(BUILD)/$(PROJECT)
	$(OBJDUMP) -S -s $< > $@

$(DEST)/mem.bin: $(BUILD)/$(PROJECT)
	$(OBJCOPY) -O binary --only-section=.data* --only-section=.text* $< $@

$(DEST)/mem.txt: $(DEST)/mem.bin
	hexdump -v -e '"%08x\n"' $< > $@

dirs:
	mkdir -p $(DEST)

clean:
	rm -rf $(TEST)
	cargo clean
