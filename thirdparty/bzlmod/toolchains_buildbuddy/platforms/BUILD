package(default_visibility = ["//visibility:public"])

# Host compatible platform is automatically selected based on the
# host machine that runs Bazel. This platform should has the same
# constraint values as @local_config_platform//:host but also has
# the relevant exec_properties needed for Remote Execution.
#
# This is intended for local development and testing purposes of
# BuildBuddy server and executor (aka. --config=local).
alias(
    name = "host_compatible",
    actual = "@buildbuddy_toolchain//:platform",
)

alias(
    name = "linux_x86_64",
    actual = "@buildbuddy_toolchain//:platform_linux_x86_64",
)

alias(
    name = "linux_arm64",
    actual = "@buildbuddy_toolchain//:platform_linux_arm64",
)

alias(
    name = "darwin_x86_64",
    actual = "@buildbuddy_toolchain//:platform_darwin",
)

alias(
    name = "darwin_arm64",
    actual = "@buildbuddy_toolchain//:platform_darwin_arm64",
)

alias(
    name = "windows_x86_64",
    actual = "@buildbuddy_toolchain//:platform_windows",
)

filegroup(
    name = "all_files",
    srcs = glob(["*"]),
    visibility = ["//:__subpackages__"],
)
