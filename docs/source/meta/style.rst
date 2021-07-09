Style Guide
===========

Most of the preferred style is enforced by ``rustfmt``. This includes the
basics such as:

* Proper whitespace
* Camel-case structures (``UartDriver``)
* Snake-case functions (``read_byte``)

There a few preferences that cannot be enforced by ``rustfmt``.

* Project should be simple to clone and begin work within seconds.
* Prioritize good documentation and comment as much as reasonable.
* No unnecessary abbreviations or unhelpful names
  (``create`` instead of ``creat``, ``swap_fd`` instead of ``dup2``).
* Prefer small modules and structures
* One structure per file with the exception of trivial sub-modules.
* Wrap functionality that requires assembly in safe Rust.
