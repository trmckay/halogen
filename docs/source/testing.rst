Testing
=======

Unit-testing
------------

Testing an operating system from the outside with a unit-testing
framework is very difficult, if not impossible. Therefore, a simple
script (``test/run_tests.py``) is provided to test algorithms outside
the rest of the system.

The script simply looks for any source files containing a test module.
If it finds any, it will bring them into a virtual Rust environment
and run the declared tests using Cargo.

The unit tests run on the host machine, because Rust's testing framework
relies on a functioning operating system. This leads to some limitations.

For the tests to work, the file *must* compile on its own. It cannot
rely on any imports. A file that has many imports cannot easily be tested
in isolation and in this context, likely relies on modules that interact
with the RISC-V hardware in some way. This effectively limits the functionality
to algorithms only.

It is still, however, possible and encouraged to use assertions throughout
the code wherever it may apply.


Test-suite
----------

Once the operating system has reach minimum functionality, it is the plan
to have a test-suite that can be run with the OS to test functionality.