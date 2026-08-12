package(default_visibility = ["//visibility:public"])

alias(
    name = "ubuntu_gcc_x86_64",
    actual = "@buildbuddy_toolchain//:ubuntu_cc_toolchain",
)

alias(
    name = "ubuntu_gcc_arm64",
    actual = "@buildbuddy_toolchain//:ubuntu_cc_toolchain_arm64",
)

alias(
    name = "windows_msvc_x86_64",
    actual = "@buildbuddy_toolchain//:windows_msvc_cc_toolchain",
)

filegroup(
    name = "all_files",
    srcs = glob(["*"]),
    visibility = ["//:__subpackages__"],
)
