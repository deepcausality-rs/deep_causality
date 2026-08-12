load("@llvm-project//:vars.bzl", "LLVM_VERSION_MAJOR")
load("@tar.bzl", "mtree_mutate", "mtree_spec", "tar")
load("//prebuilt:mtree.bzl", "mtree")
load("//tools:defs.bzl", "TOOLCHAIN_BINARIES")

def llvm_release(name, bin_suffix = ""):
    mtree_spec(
        name = name + "_builtin_headers_mtree_",
        srcs = [
            "@llvm-project//clang:builtin_headers_files",
        ],
        tags = ["manual"],
    )

    mtree_mutate(
        name = name + "_builtin_headers_mtree",
        mtree = name + "_builtin_headers_mtree_",
        strip_prefix = "clang/lib/Headers",
        package_dir = "lib/clang/{}/include".format(LLVM_VERSION_MAJOR),
        tags = ["manual"],
    )

    bin_files = {
        "//toolchain/bootstrap/stage3:llvm": "bin/llvm" + bin_suffix,
        "@llvm-project//compiler-rt:asan_ignorelist": "lib/clang/{llvm_major}/share/asan_ignorelist.txt",
        "@llvm-project//compiler-rt:msan_ignorelist": "lib/clang/{llvm_major}/share/msan_ignorelist.txt",
    }

    mtree(
        name = name + "_bins_mtree",
        files = bin_files,
        symlinks = {
            "bin/" + binary + bin_suffix: "llvm" + bin_suffix
            for binary in ["clang-{llvm_major}"] + TOOLCHAIN_BINARIES
        },
        format = {
            "llvm_major": LLVM_VERSION_MAJOR,
        },
        tags = ["manual"],
    )

    native.genrule(
        name = name + "_mtree",
        srcs = [
            name + "_bins_mtree",
            name + "_builtin_headers_mtree",
        ],
        cmd = """\
            cat $(SRCS) > $(@)
        """,
        outs = [
            name + "_mtree_spec.mtree",
        ],
        tags = ["manual"],
    )

    tar(
        name = name,
        srcs = bin_files.keys() + [
            "@llvm-project//clang:builtin_headers_files",
        ],
        args = [
            "--options",
            "zstd:compression-level=22",
        ],
        compress = "zstd",
        mtree = name + "_mtree",
        tags = ["manual"],
    )
