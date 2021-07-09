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

Running the OS aims to be a simple process by using Docker to create a predictable
environment in which the built binary can be executed. After installing Docker for
your system, ``make run`` will build the project, build the Docker image, then run
the OS on QEMU within the container.
