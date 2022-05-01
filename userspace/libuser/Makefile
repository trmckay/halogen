RISCV = riscv64-unknown-elf-
CC = $(RISCV)gcc
AR = $(RISCV)ar

SRC = $(wildcard *.c)
OBJ = $(patsubst %.c,%.o,$(SRC))

LIB = lib.o

.PHONY: all
all: $(LIB)

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

.PHONY: clean
clean:
	rm -f $(OBJ) $(ELF)
