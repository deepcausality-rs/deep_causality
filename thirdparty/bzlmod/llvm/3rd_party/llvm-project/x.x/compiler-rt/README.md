# FLAGS APPLE
set(CFLAGS -fPIC -O3 -fvisibility=hidden -DVISIBILITY_HIDDEN -Wall -fomit-frame-pointer)

OSX EXCLUDED SYMBOLS:
apple_versioning
addtf3
divtf3
multf3
powitf2
subtf3
trampoline_setup


# FLAGS NOT APPLE                       
-fPIC                                       # implied
-fPIE                                       # implied
-fno-builtin                                # always
-std=c11                                    # always
-fvisibility=hidden                         # always
-fomit-frame-pointer                        # if not debug
-ffreestanding                              # if amdgcn/nvptx
-Wno-pedantic                               # LLVM CRT
-nogpulib                                   # if amdgcn/nvptx
-flto                                       # if amdgcn/nvptx
-fconvergent-functions                      # if amdgcn/nvptx
"-Xclang -mcode-object-version=none"        # if amdgcn
-Wbuiltin-declaration-mismatch              # always


if(APPLE)
  set(ARM64 arm64 arm64e)
  set(ARM32 armv7 armv7k armv7s)
  set(X86_64 x86_64 x86_64h)
endif()


set(ALL_BUILTIN_SUPPORTED_ARCH
  ${X86} ${X86_64} ${AMDGPU} ${ARM32} ${ARM64} ${AVR}
  ${HEXAGON} ${MIPS32} ${MIPS64} ${NVPTX} ${PPC32} ${PPC64}
  ${RISCV32} ${RISCV64} ${SPARC} ${SPARCV9}
  ${WASM32} ${WASM64} ${VE} ${LOONGARCH64})

# SOURCES

if(APPLE)
  set(GENERIC_SOURCES
    ${GENERIC_SOURCES}
    atomic_flag_clear.c
    atomic_flag_clear_explicit.c
    atomic_flag_test_and_set.c
    atomic_flag_test_and_set_explicit.c
    atomic_signal_fence.c
    atomic_thread_fence.c
  )
endif()

