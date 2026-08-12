load("//platforms:common.bzl", "SUPPORTED_EXECS", "SUPPORTED_TARGETS")

_BUILD_TEMPLATE = """\
load("@llvm//toolchain:declare_toolchains.bzl", "declare_toolchains")
load("@llvm//toolchain/bootstrap:declare_toolchains.bzl", declare_bootstrap_toolchains = "declare_toolchains")

_EXECS = [
    {execs}
]

_TARGETS = [
    {targets}
]

declare_toolchains(execs = _EXECS, targets = _TARGETS)
declare_bootstrap_toolchains(execs = _EXECS, targets = _TARGETS)
"""

def _toolchains_repository_impl(rctx):
    rctx.file("BUILD.bazel", rctx.attr.build_file_content)
    return rctx.repo_metadata(reproducible = True)

_toolchains_repository = repository_rule(
    implementation = _toolchains_repository_impl,
    attrs = {
        "build_file_content": attr.string(mandatory = True),
    },
)

def _format_platform_list(platforms):
    return ",\n    ".join([repr(platform) for platform in platforms])

def _validate_platform_pair(kind, platform, supported):
    if platform not in supported:
        fail("Unsupported {} platform {}".format(kind, platform))

def _toolchain_impl(mctx):
    execs = []
    targets = []

    for module in mctx.modules:
        for exec in module.tags.exec:
            platform = (exec.os, exec.arch)
            _validate_platform_pair("exec", platform, SUPPORTED_EXECS)
            execs.append(platform)
        for target in module.tags.target:
            platform = (target.os, target.arch)
            _validate_platform_pair("target", platform, SUPPORTED_TARGETS)
            targets.append(platform)

    if not execs:
        execs = SUPPORTED_EXECS

    if not targets:
        targets = SUPPORTED_TARGETS

    _toolchains_repository(
        name = "llvm_toolchains",
        build_file_content = _BUILD_TEMPLATE.format(
            execs = _format_platform_list(execs),
            targets = _format_platform_list(targets),
        ),
    )

    return mctx.extension_metadata(
        reproducible = True,
        root_module_direct_deps = ["llvm_toolchains"],
        root_module_direct_dev_deps = [],
    )

_exec_platform_tag = tag_class(
    attrs = {
        "os": attr.string(
            mandatory = True,
            values = ["linux", "macos", "windows"],
        ),
        "arch": attr.string(
            mandatory = True,
            values = ["x86_64", "aarch64", "riscv64", "s390x", "armv7"],
        ),
    },
)

_target_platform_tag = tag_class(
    attrs = {
        "os": attr.string(
            mandatory = True,
            values = ["linux", "macos", "windows", "none"],
        ),
        "arch": attr.string(
            mandatory = True,
            values = [
                "x86_64",
                "aarch64",
                "riscv64",
                "s390x",
                "armv7",
                "bpfeb",
                "bpfel",
                "wasm32",
                "wasm64",
            ],
        ),
    },
)

toolchain = module_extension(
    implementation = _toolchain_impl,
    doc = "Generates LLVM toolchains for the requested target/exec platform pairs.",
    tag_classes = {
        "target": _target_platform_tag,
        "exec": _exec_platform_tag,
    },
)
