load("@bazel_lib//lib:run_binary.bzl", "run_binary")

_SH_TOOLCHAIN_TYPE = Label("@rules_shell//shell:toolchain_type")

def _sh_script_impl(ctx):
    shell_path = ctx.toolchains[_SH_TOOLCHAIN_TYPE].path
    if not shell_path:
        fail("No sh_toolchain.path is set for the execution platform")

    ctx.actions.write(
        output = ctx.outputs.out,
        content = "\n".join([
            "#!{}".format(shell_path),
            "set -euo pipefail",
            ctx.attr.cmd,
            "",
        ]),
        is_executable = True,
    )

    return DefaultInfo(
        executable = ctx.outputs.out,
        files = depset([ctx.outputs.out]),
    )

_sh_script = rule(
    implementation = _sh_script_impl,
    attrs = {
        "cmd": attr.string(mandatory = True),
        "out": attr.output(mandatory = True),
    },
    toolchains = [_SH_TOOLCHAIN_TYPE],
)

def sh_script(name, cmd, env = {}, **kwargs):
    _sh_script(
        name = name + "_bin",
        out = name + ".sh",
        cmd = cmd,
    )

    run_binary(
        name = name,
        tool = name + "_bin",
        env = {
            # Some default paths that are likely to be needed for genrule-type scripts.
            # We must specify this because otherwise we leak the host's PATH, which may
            # not be compatible with the execution platform...
            # See https://github.com/bazelbuild/bazel/issues/28065
            "PATH": "/usr/bin:/bin:/usr/sbin:/sbin",
        } | env,
        **kwargs
    )
