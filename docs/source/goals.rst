Goals
==========

Scope
-----

To start, the goal of this project is explicitly not to create a platform
for desktop computing, nor is it to replace Linux or any other Unix-like
OS as a platform for IoT or embedded applications. The project aims to
be a platform for learning, rather than for any serious computing. It aims
to showcase fundamental operating systems concepts in an environment that
is easy to experiment with. This includes:

* Processes, scheduling, and LDE
* Memory virtualization
* Device drivers
* Timers and interrupts
* Filesystems

There will not be a display-server, web browser, or anything else that
makes operating systems useful from a user's perspective.


Priorities
----------

This project is primarily intended as a learning experience for myself
and any who chose to work on it. With that, the project should prioritize
choices that lead to maximal learning outcomes.

Some of the priorities that I believe can help with this are:

* **Modularity**: it should be possible for different implementations
  of key operating system functionality to be drop-in replaced. This
  encourages experimentation.

* **Simplicity**: Minimizing the amount of code that must be understood
  before one can work on the project should be a priority. The goal is to
  learn operating systems concepts, not how this specific code-base
  implements them.

* **Safety**: Memory-safety and security are important in there own rights.
  But this can also enable less painful work by minimizing hard-to-debug
  errors.


Rust
^^^^

Each of these principles lead to Rust as the natural language choice. While
the borrow-checker and other unique features of Rust can lead to a slightly
higher beginner learning-curve, writing good systems programs in Rust is much
easier than in C.

Furthermore, Rust is rapidly growing in popularity for systems programmers.
This is a fantastic opportunity for myself and others to get comfortable with
the language.


RISC-V
^^^^^^

A kernel ties together the hardware and a user's software. So, experimenting
with the software is only part of the fun. As an open ISA, RISC-V enables
experimentation much more readily than other ISAs like x86 or ARM. Also, the
assembler and ISA is relatively simple. Like Rust, RISC-V is becoming
increasingly prevalent.