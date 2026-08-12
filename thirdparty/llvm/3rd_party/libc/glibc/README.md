# GNU C Library ("glibc") Edits

This package contains patches to a few files from the GNU C Library project.

## `csu/abi-note-2.31.S`

`csu/abi-note-2.31.S` is a copy of `csu/abi-note.S` from the glibc
`release/2.31/master` branch.

Starting in version 2.32, this file was rewritten in C (`abi-note.c`) and began
including many headers that are difficult to handle in Bazel.

Since the behavior of the new C version is identical to the original assembly
file, we chose to keep using the 2.31 version for simplicity.

## `csu/elf-init-2.31.c` 

`csu/elf-init-2.31.c` is a copy of `csu/elf-init.c` from the glibc 
`release/2.31/master` branch.

In glibc versions 2.32 and 2.33, support for `_init` 
and `_fini` became mandatory and could no longer be disabled with compile-time 
flags.

Before 2.32, these functions could be excluded by defining `-DNO_INITFINI`.
Starting with 2.32, this was replaced with an internal `#define ELF_INITFINI`
in a header file, which could not be overridden.

In version 2.34, `_init` and `_fini` were removed entirely.

We use the version from 2.31 because it still allows opting out of `_init` and `_fini`.

## `nptl/pthread_atfork.c`, `stdlib/at_exit.c` and `stdlib/at_quick_exit.c`

Those 2 files are a copy from the `ziglang/zig` glibc support package
which themselves are edited versions of the same files from the glibc project.

Similar to `csu/abi-note.c`, They were including many headers that are too
difficult to handle in Bazel.

Headers inclusions were replaced with inlined functions definitions for 
simplicity.

# Notes

Those files will be monitored everytime a new version of the glibc is supported
to ensure that they can be used for all versions, otherwise we will branch as
needed.
