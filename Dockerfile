FROM ubuntu:bionic

RUN apt update
RUN apt install -y \
    git curl autoconf automake autotools-dev \
    curl libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex \
    texinfo gperf libtool patchutils \
    bc zlib1g-dev libexpat-dev git \
    libpixman-1-dev libglib2.0-dev

RUN git clone https://github.com/qemu/qemu /tmp/qemu
WORKDIR /tmp/qemu
RUN git checkout v5.0.0
RUN ./configure --target-list=riscv64-softmmu
RUN make -j $(nproc)
RUN make install

RUN apt purge -y \
    git curl autoconf automake autotools-dev \
    curl libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex \
    texinfo gperf libtool patchutils \
    bc zlib1g-dev libexpat-dev git \
    libpixman-1-dev libglib2.0-dev

ENTRYPOINT ["qemu-system-riscv64"]
