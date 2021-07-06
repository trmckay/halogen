FROM ubuntu:bionic as builder

# deps
RUN apt update
RUN apt install -y \
    git curl autoconf automake autotools-dev \
    curl libmpc-dev libmpfr-dev libgmp-dev \
    gawk build-essential bison flex \
    texinfo gperf libtool patchutils \
    bc zlib1g-dev libexpat-dev git \
    libpixman-1-dev libglib2.0-dev

# build qemu
RUN git clone https://github.com/qemu/qemu /tmp/qemu
WORKDIR /tmp/qemu
RUN git checkout v5.0.0
RUN ./configure --target-list=riscv64-softmmu
RUN make -j $(nproc)
RUN make install

# rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh
RUN chmod +x /tmp/rustup.sh
RUN /tmp/rustup.sh -y
ENV PATH=/root/.cargo/bin:/usr/bin:/usr/local/bin:/bin
RUN rustup default nightly
RUN rustup target add riscv64gc-unknown-none-elf

# cargo project dir
RUN mkdir -p /cargo
WORKDIR /cargo

ENTRYPOINT ["cargo"]
CMD ["run"]
