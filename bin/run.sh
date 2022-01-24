#!/bin/bash

firmware="build/fw.bin"
qemu="qemu-system-riscv64"
debug_flags=""

if [[ $# -gt 2 ]]; then
    echo "Usage: $0 [-g] /path/to/kernel"
    exit 1
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        "-g")
            debug_flags="-s -S"
            shift 1
            ;;
        *)
            kernel="$1"
            shift 1
            ;;
    esac
done

if [[ ! -f $kernel ]]; then
    "Kernel '$kernel' does not exist"
    exit 1
fi
if [[ ! -f $firmware ]]; then
    "Firmware '$firmware' does not exist"
    exit 1
fi

echo -e "\nFirmware: $firmware"
echo "Kernel: $kernel"

if [[ "$debug_flags" != "" ]]; then
    echo "Debug: True"
else
    echo "Debug: False"
fi

cmd="$qemu -machine virt -cpu rv64 -m 512M -smp 1 -nographic -serial mon:stdio -bios $firmware $debug_flags -kernel $kernel"
echo -e "\n$cmd\n"
exec $cmd
