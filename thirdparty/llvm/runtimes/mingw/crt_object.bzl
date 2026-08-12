load("@bazel_skylib//rules:copy_file.bzl", "copy_file")
load("@llvm//toolchain/runtimes:cc_runtime_library.bzl", "cc_runtime_stage0_library")

def crt_object(name, out, visibility, **kwargs):
    cc_runtime_stage0_library(
        name = name + "_lib",
        **kwargs
    )

    native.filegroup(
        name = name + "_file",
        srcs = [name + "_lib"],
        output_group = "compilation_outputs",
    )

    copy_file(
        name = name,
        src = name + "_file",
        out = out,
        allow_symlink = True,
        visibility = visibility,
    )
