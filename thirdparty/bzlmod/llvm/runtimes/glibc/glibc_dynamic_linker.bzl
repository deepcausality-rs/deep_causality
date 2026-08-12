# Keep aligned with clang/lib/Driver/ToolChains/Linux.cpp from LLVM source code.
_GLIBC_DYNAMIC_LINKERS = {
    "//platforms/config:linux_aarch64_gnu": "ld-linux-aarch64.so.1",
    "//platforms/config:linux_armv7_gnu": "ld-linux-armhf.so.3",
    "//platforms/config:linux_riscv64_gnu": "ld-linux-riscv64-lp64d.so.1",
    "//platforms/config:linux_s390x_gnu": "ld64.so.1",
    "//platforms/config:linux_x86_64_gnu": "ld-linux-x86-64.so.2",
}

_NO_MATCH_ERROR = "Unsupported glibc dynamic linker target. Add the target architecture to runtimes/glibc/glibc_dynamic_linker.bzl."

def glibc_dynamic_linker():
    return select(_GLIBC_DYNAMIC_LINKERS, no_match_error = _NO_MATCH_ERROR)

def glibc_dynamic_linker_as_needed():
    return select({
        config: ["AS_NEEDED({})".format(dynamic_linker)]
        for (config, dynamic_linker) in _GLIBC_DYNAMIC_LINKERS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def glibc_dynamic_linker_soname_flags():
    return select({
        config: ["-Wl,-soname,{}".format(dynamic_linker)]
        for (config, dynamic_linker) in _GLIBC_DYNAMIC_LINKERS.items()
    }, no_match_error = _NO_MATCH_ERROR)
