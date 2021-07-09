Getting started
===============

Cloning the repository
----------------------

The code is hosted on GitHub at
`trmckay/lab_os <https://github.com/trmckay/lab_os/>`__.

Clone the repository::

    git clone git@github.com:trmckay/lab_os.git


Set-up the project
------------------

The primary dependency for the project is ``rustup``. The official way to install
is detailed at `rustup.rs <https://rustup.rs/>`__.

Once ``rustup`` is installed, there is a ``Makefile`` rule to perform the remaining
configuration. This includes:

* Checking that requirements are installed

* Configuring the correct Rust toolchain

* Installing pre-commit hooks


Run the OS
----------

There are two ways to run the operating system: "bare-metal" using QEMU and Docker.

For QEMU, ``qemu-system-riscv64`` must be installed. This option works best
if you are running Linux. Running is as simple as doing ``make run`` from the
project root.

Docker is the best option if you are not running Linux or your distribution does
not package QEMU for RISC-V. With the Docker daemon or Docker Desktop running,
you can build and run the image with ``make run-docker`` from the project root.
