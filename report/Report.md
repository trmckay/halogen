---
title: "Operating system kernel for RISC-V in Rust"
author: "Trevor McKay"
date: \today
documentclass: article
papersize: letter
numbersections: true
indent: true
fontsize: 11pt
linestretch: 1.15
toc: true
geometry:
  - margin=1in
---

<!-- 1. Abstract -->
<!-- 2. Introduction -->
<!-- 3. Background -->
<!-- 4. Key Design Decisions -->
<!-- 5. Future Work -->
<!-- 6. Reflections -->
<!-- 7. Conclusion -->

\newpage

# Abstract

This project is an operating system kernel that targets RISC-V and is built with Rust. This kernel
does not aim to be POSIX-compliant (yet), or meaningfully differentiate its design from the kernel
used in CPE-454. Rather, the goals are to allow experimentation with operating system implementation
to further understand the subject, teach about the RISC-V architecture beyond the baremetal-only ISA
taught in CPE-233 and CPE-333, evaluate Rust as a language for kernel and systems programming, and
provide a base for potential future RISC-V student projects.

This report will provide an overview of the project, more detailed discussion on the more
interesting portions of the design, and a reflection on Rust, RISC-V, and the project.

# Introduction

When this project was first considered, the goal was just to create a minimally-viable operating
system for RISC-V. RISC-V was chosen as the ISA due to its open nature, its relative simplicity
compared to x86, and standardization compared to ARM. Another key decision that would drastically
alter the course of the project was the decision to use Rust.

# Background

## The RISC-V ISA

RISC-V is a relatively new instruction-set architecture introduced by UC Berkely in 2010. As the
name would imply, it is a load-store RISC ISA. A RISC-V ISA is made up of a base ISA, RV32I,
RV64I, or RV128I[^1], and some number of extensions for functionaly such as multiplication,
atomic memory access, compressed instructions, or hypervisor support.

[^1]: The "I" refers to the base integer-only ISA.

RISC-V also specifies some other parts of the platform outside the ISA, namely the paging
implementation and the interrupt controllers. For paging, there are Sv32, Sv39, and Sv47 using 32-,
39- and 47-bit virtual addresses respectively.

This kernel targets the QEMU virt platform, which has RISC-V RV64GC cores (hardware threads, or
harts) and an Sv39 MMU. The minimum extensions to support the kernel are the multiplication (_m_),
atomics (_a_), CSR (_Zicsr_), and fence (_Zifencei_) extensions.

RISC-V has three privilege modes: machine, supervisor, and user. The machine mode software is
typically a firmware, or software-binary interface (SBI) in RISC-V terms. The SBI software provides
an interface to machine-mode only faculties such as timers, hart-state management, and some CSRs.
The SBI itself is included in the RISC-V specifications. OpenSBI, the firmware used by this
project, is an open-source implementation of the RISC-V SBI specificiation with support for many
platforms, including QEMU.

## The Rust programming language

Rust is a systems programming language that, like RISC-V, debuted in 2010. Its unique features
include memory-safety, performant abstractions, and lack of runtime.

These features make Rust an interesting candidate for operating systems development. The lack of
runtime or reliance on system libraries make it possible to use in baremetal environments. And the
features it provides make it much more ergonomic than C.

Whether or not these features come at the cost of lower-level control is one of the questions this
project seeks to answer.

# Design

## Initialization overview

In order to provide context for the later discussion of design decisions, this section provides a
cursory over of the design of each of the kernel's subsystems.

When the kernel first gains control of the booting hart, the argument registers contain arguments
from the calling firmware, other the registers are zeroed, and paging is not enabled. The first few
instructions set up a Rust runtime, which is as simple as loading the stack pointer, the global
pointer, and zeroing out the BSS section. Once this has been done, the kernel can call into Rust
code.

At this point, the kernel can safely execute any Rust code that is position-independent (the kernel
image is linked for the virtual address space), and allocation-free. The elimination of these
limitiations and subsequent handoff to the thread scheduler is what this kernel refers to as the
"boot process".

To enable position-dependent code, the kernel must be mapped to the address it was linked at. The
physical page allocator is initialized and used to create page tables which are filled in with a
linear map of the kernel image, offset to the address it was linked at.

The virtual space that this stage will uses addresses are 39-bits. Since pointers are still 64-bits,
bits 63-39 must match bit 38. This divides the address space into two regions, both of which are
256 GB. To follow convention, the kernel is mapped to to the upper of these two regions.

The address space is divided into seven regions: text, read-only statics, read-write statics, unused
physical memory, stacks for kernel threads, the heap, and virtual addresses for dynamic mappings.

![Diagram of the kernel address space layout](Resources/Address space.png)

These mappings are identical for every process and address space. Since this is the case, they can
be marked as global, ensuring that their entries in the TLB will persist accross context switches.

The heap and stack regions are static in size, but not necessarily backed by physical frames. The
stack does not need to be a distinct region, strictly speaking; each kernel stack could allocate
some virtual addresses and physical frames on from the dynamic mappings region. However, a separate
region makes demand-paging simpler since addresses can be easily identified as belonging to a
stack. The same goes for the heap, however it is already necessary that this region be contiguous in
virtual space so that an allocator can manage it.

Once all of these are mapped, any registers and structures containing pointers are updated, paging
is enabled, and the kernel traps to the next stage of the boot process.

In the next stage, the kernel can use any previously unavailable features of the Rust language.
The goal for this stage will be to enable allocation and kickstart the first kernel thread.
The heap allocator is initialized and future calls to the Rust global allocator should now succeed.
The final remaining initialization code sets up logging, trap handlers, and the interrupt controller.

Finally, the kernel can handoff execution to the thread scheduler, creating and starting the main
function as a kernel thread.

## Discussion of design decisions

### Using a machine-mode firmware

The decision to use an M-mode firmware such as OpenSBI was not obvious at first. In the early stages
of the project, it was developed to run in machine-mode. Machine-mode software technically has
access to any resource that the supervisor does. Additionally, there even are some resources like
interrupt delegation, timers, and hart-state management that are required for a functioning kernel.
One might come to the reasonable, but incorrect, conclusion that it would be simpler to write a kernel
in machine-mode.

There is one caveat to machine-mode that makes this especially unfeasable. Physical memory protection
may remain enabled, however addresses are not translated even when paging is enabled; machine-mode can
only use physical addresses. Writing a kernel that does not run on top of paging is practically impossible
for many reasons, some of which include:

- The address space is limited to whatever the kernel is physically loaded at.
- User memory is difficult to access as each pointer requires the page table to be walked in
  software.
- Safety features such as user memory being unreadable by default are disabled.

It eventually became clear that the core of the kernel would need to run in supervisor-mode. However,
as mentioned before, much of the machine-mode functionality is required by the kernel.

The solution to this is to use a firmware to expose the machine-mode-only functionality. Much like
user software can make requests to the kernel with a system call API, the SBI specification defines
an interface between the kernel, and the machine-mode firmware. OpenSBI is the most ubiquitous
implementation of this interface and supports the QEMU machine and most common commercial RISC-V
platforms.

With OpenSBI as a support, the kernel could then be fully contained to supervisor-mode.

### Targeting RISC-V

### Using Rust
