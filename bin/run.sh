#!/bin/bash

usage="Usage: $0 [-g] [-t] [/path/to/kernel]"

firmware="build/opensbi.bin"
qemu="qemu-system-riscv64"
debug_flags=""
kernel="build/halogen.bin"

if [[ $# -gt 3 ]]; then
    echo "$usage"
    exit 1
fi


while [[ $# -gt 0 ]]; do
    case $1 in
        -*)
            if [[ "$1" =~ "h" ]]; then
                echo "$usage"
                exit 0
            fi
            if [[ "$1" =~ "t" ]]; then
                kernel="build/halogen-test.bin"
            fi
            if [[ "$1" =~ "g" ]]; then
                debug_flags="-s -S"
            fi
            shift 1
            ;;
        *)
            kernel="$1"
            shift 1
            ;;
    esac
done

if [[ ! -f $kernel ]]; then
    echo "Kernel '$kernel' does not exist"
    exit 1
fi
if [[ ! -f $firmware ]]; then
    echo "Firmware '$firmware' does not exist"
    exit 1
fi

echo -e "Firmware: $firmware"
echo "Kernel: $kernel"

if [[ "$debug_flags" != "" ]]; then
    echo "Debug: True"
else
    echo "Debug: False"
fi

cmd="$qemu -machine virt -cpu rv64 -m 512M -smp 1 -nographic -serial mon:stdio -bios $firmware $debug_flags -kernel $kernel"
echo -e "\n$cmd\n"
exec $cmd
