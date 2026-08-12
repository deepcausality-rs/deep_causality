# Bootstrap toolchain

This package holds the LLVM bootstrap binaries, FDO profile generation rules,
and source-built C++ toolchains. The bootstrap stages are:

1. `stage0_prebuilt_seed` compiles the stage1 LLVM binaries from source.
2. `stage1_from_source` compiles the stage2 LLVM binaries with ThinLTO and FDO
   instrumentation.
3. `stage2_lto_and_fdo_instrumented` runs workloads for every
   `SUPPORTED_TARGETS` target. `stage1_from_source` compiles the stage3 LLVM
   binaries with the merged profile for the target CPU.
4. `stage3_lto_and_fdo_applied` uses the stage3 LLVM binaries as the C++
   toolchain.

`//toolchain/bootstrap/stage1:<tool>` builds the stage1 variant.
`//toolchain/bootstrap/stage2:<tool>` builds the stage2 variant.
`//toolchain/bootstrap/stage3:<tool>` builds the stage3 variant.
`//prebuilt/llvm:all` packages `//toolchain/bootstrap/stage3:llvm`.
