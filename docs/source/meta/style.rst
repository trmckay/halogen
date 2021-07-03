Style Guide
===========

Most of the preferred style is enforced by ``rustfmt``. This includes the
basics such as:

* Proper whitespace
* Camel-case structures (``UartDriver``)
* Snake-case functions (``read_byte``)

There a few preferences that cannot be enforced by ``rustfmt``.

* No unnecessary abbreviations (use ``create_process()`` instead of ``cproc()``).
* Comment all non-trivial functions
* Prefer small modules and structures
* One structure per file with the exception of trivial submodules.
* Wrap functionality that requires assembly in safe Rust.