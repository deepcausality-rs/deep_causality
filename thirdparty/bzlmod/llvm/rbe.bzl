def _rbe_platform_repo_impl(rctx):
    arch = rctx.os.arch
    if arch in ["x86_64", "amd64"]:
        host_platform = "rbe_linux_x86_64"
    elif arch in ["aarch64", "arm64"]:
        host_platform = "rbe_linux_aarch64"
    else:
        fail("Unsupported host arch for rbe platform: {}".format(arch))

    rctx.file("BUILD.bazel", """\
platform(
    name = "rbe_linux_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
        "@llvm//constraints/libc:gnu.2.28",
    ],
    exec_properties = {{
        "container-image": "docker://ubuntu:22.04",
        "Arch": "amd64",
        "OSFamily": "Linux",
    }},
    visibility = ["//visibility:public"],
)

platform(
    name = "rbe_linux_aarch64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:linux",
        "@llvm//constraints/libc:gnu.2.28",
    ],
    exec_properties = {{
        "container-image": "docker://ubuntu:22.04",
        "Arch": "arm64",
        "OSFamily": "Linux",
    }},
    visibility = ["//visibility:public"],
)

alias(
    name = "rbe_platform",
    actual = ":{host_platform}",
    visibility = ["//visibility:public"],
)
""".format(
        host_platform = host_platform,
    ))

rbe_platform_repository = repository_rule(
    implementation = _rbe_platform_repo_impl,
    doc = "Sets up AMD64 and ARM64 Linux platforms for remote builds.",
)
