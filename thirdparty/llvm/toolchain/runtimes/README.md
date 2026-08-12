# Runtime only toolchain

This package holds the toolchain args for compiling runtimes.

This is not a dedicated toolchain, but targets of the cc_args family used by
`cc_toolchain` when `//toolchain:runtime_stage=stage*`.
