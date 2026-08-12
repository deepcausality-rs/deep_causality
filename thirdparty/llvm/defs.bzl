def exec_test(*, rule, name, tags = [], args = [], env = {}, data = [], tools = [], **kwargs):
    rule(
        name = name + "_",
        tags = tags + (["manual"] if "manual" not in tags else []),
        data = data,
        **kwargs
    )

    _exec_test(
        name = name,
        inner = name + "_",
        tags = tags,
        args = args,
        env = env,
        data = data,
        tools = tools,
        target_compatible_with = kwargs.get("target_compatible_with", []),
    )

def _exec_test_impl(ctx):
    inner = ctx.attr.inner[DefaultInfo]
    out = ctx.outputs.executable

    ctx.actions.symlink(
        target_file = inner.files_to_run.executable,
        output = out,
    )

    runfiles = ctx.runfiles(ctx.files.data + ctx.files.tools)

    data = ctx.attr.data + ctx.attr.tools

    return [
        DefaultInfo(
            files = depset([out]),
            executable = out,
            runfiles = runfiles.merge(inner.default_runfiles),
        ),
        RunEnvironmentInfo(
            environment = {
                k: ctx.expand_location(v, data)
                for k, v in ctx.attr.env.items()
            },
        ),
    ]

_exec_test = rule(
    implementation = _exec_test_impl,
    attrs = {
        "inner": attr.label(
            executable = True,
            cfg = "exec",
            mandatory = True,
        ),
        "data": attr.label_list(
            doc = "The service manager will merge these variables into the environment when spawning the underlying binary.",
            allow_files = True,
        ),
        "tools": attr.label_list(
            doc = "The service manager will merge these variables into the environment when spawning the underlying binary.",
            cfg = "exec",
            allow_files = True,
        ),
        "env": attr.string_dict(
            doc = "The service manager will merge these variables into the environment when spawning the underlying binary.",
        ),
    },
    test = True,
)
