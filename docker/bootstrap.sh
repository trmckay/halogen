#!/bin/bash

set -e
savedir=$(pwd)

apt update && apt install -y $(cat build-deps.txt)

mkdir -p /src
cd /src

git clone --single-branch --branch master https://github.com/riscv-software-src/riscv-gnu-toolchain.git
cd /src/riscv-gnu-toolchain
./configure --prefix=/usr/local --with-arch=rv64gc
make linux

cd /src
rm -rf /src/*

git clone --single-branch --branch v5.2.0 https://github.com/qemu/qemu
cd /src/qemu
./configure --target-list=riscv64-softmmu && make -j $(nproc)
make install

cd /src
rm -rf /src/*


cd $savedir
apt purge -y $(cat build-deps.txt)
