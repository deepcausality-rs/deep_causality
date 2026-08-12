load("@bazel_lib//lib:run_binary.bzl", "run_binary")

def stub_library(name, out = None, visibility = None):
    out = out or "lib%s.a" % name
    run_binary(
        name = name,
        outs = [out],
        tool = "//tools:llvm-ar",
        args = ["rc", "$@"] + select({
            # By default, llvm-ar's behavior differs based on the exec platform. Force some sane defaults
            # so we get the same output regardless of exec platform and avoid invalidating downstream actions.
            # The gnu-style empty libraries are consumable by the toolchain downstream, but force
            # darwin-style ones for maximal compatibility when we make exportable sysroots/toolchains for non-Bazel.
            "@platforms//os:macos": ["--format=darwin"],
            "//conditions:default": ["--format=gnu"],
        }),
        visibility = visibility,
    )
