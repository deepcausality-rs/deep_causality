load("@rules_cc//cc:defs.bzl", "cc_binary")

cc_binary(
    name = "main",
    srcs = ["main.cc"],
)

platform(
    name = "my_linux_platform",
    exec_properties = {
        "some_property": "some_value",
    },
    parents = ["@toolchains_buildbuddy//platforms:linux_x86_64"],
)
