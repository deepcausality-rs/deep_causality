"""Shared settings for LLVM bootstrap transitions."""

load("@llvm-project//:vars.bzl", "LLVM_VERSION_MAJOR")

# Enable the same set of tools we provide with prebuilts.
LLVM_TOOLS = ([
    "clang-format",
    "clang-tidy",
    "clangd",
] if int(LLVM_VERSION_MAJOR) >= 22 else []) + [
    "clang",
    "clang-scan-deps",
    "dsymutil",
    "lld",
    "llvm-ar",
    "llvm-cgdata",
    "llvm-cov",
    "llvm-cxxfilt",
    "llvm-debuginfod-find",
    "llvm-dwp",
    "llvm-gsymutil",
    "llvm-ifs",
    "llvm-libtool-darwin",
    "llvm-link",
    "llvm-lipo",
    "llvm-ml",
    "llvm-mt",
    "llvm-nm",
    "llvm-objcopy",
    "llvm-objdump",
    "llvm-profdata",
    "llvm-rc",
    "llvm-readobj",
    "llvm-readtapi",
    "llvm-size",
    "llvm-symbolizer",
    "sancov",
]

SANITIZER_FLAGS = [
    "//config:ubsan",
    "//config:cfi",
    "//config:msan",
    "//config:dfsan",
    "//config:nsan",
    "//config:safestack",
    "//config:rtsan",
    "//config:tysan",
    "//config:tsan",
    "//config:asan",
    "//config:lsan",
    "//config:xray",
    "//config:fuzzer",
    "//config:profile",
    "//config:host_ubsan",
    "//config:host_cfi",
    "//config:host_msan",
    "//config:host_dfsan",
    "//config:host_nsan",
    "//config:host_safestack",
    "//config:host_rtsan",
    "//config:host_tysan",
    "//config:host_tsan",
    "//config:host_asan",
    "//config:host_lsan",
    "//config:host_xray",
    "//config:host_fuzzer",
    "//config:host_profile",
]

def disable_sanitizers(settings):
    for flag in SANITIZER_FLAGS:
        settings[flag] = False
