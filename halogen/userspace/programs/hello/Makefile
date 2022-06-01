RISCV = riscv64-unknown-elf-
CC = $(RISCV)gcc
OBJCOPY = $(RISCV)objcopy

SRC = $(wildcard *.c)
OBJ = $(patsubst %.c,%.o,$(SRC))
LIB = ../../libuser/lib.o
LDS = ../../libuser/link.ld

CFLAGS = -I../../libuser -ffreestanding
LDFLAGS = -nostdlib -T$(LDS)

ELF = hello.elf

.PHONY: all
all: $(ELF)

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

$(ELF): $(OBJ) $(LIB)
	$(CC) $(LDFLAGS) -o $@ $^
	@$(RISCV)objdump -S $@

.PHONY: clean
clean:
	rm -f $(OBJ) $(ELF)
