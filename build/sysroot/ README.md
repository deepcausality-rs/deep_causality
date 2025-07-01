# LLVM only cross compilation setup

This is an alternative configuration without MUSL that cross compiles with LLVM only.

Use this setup if your repo does not build with MUSL.

## Considerations

Pros:
* No MUSL
* Single LLVM toolchain

Cons:
* LLVM compiles about 5x to 10x slower than MUSL
* LLVM requires custom sysroot to compile system dependencies


## LLVM only cross compilation setup

1) Copy the LLVM toolchain section from the sample MODULE in this folder to your actual MODULE file.
2) Double check the .bazelrc file in this folder if you need to add any custom settings
3) Remove MUSL
4) Build
5) Run tests

For configuring a custom sysroot, see:

https://steven.casagrande.io/posts/2024/sysroot-generation-toolchains-llvm/