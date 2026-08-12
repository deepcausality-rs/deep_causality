def _mtree_impl(ctx):
    lines = []

    for tgt, dst in ctx.attr.files.items():
        lines.append(
            "%s uid=0 gid=0 time=1672560000 mode=0755 type=file content=$(location %s)" %
            (dst, str(tgt.label)),
        )

    for link_path, target in ctx.attr.symlinks.items():
        lines.append(
            "%s uid=0 gid=0 time=1672560000 mode=0755 type=link link=%s" %
            (link_path, target),
        )

    content = "\n".join(lines)
    content = ctx.expand_location(content, targets = ctx.attr.files.keys())

    if ctx.attr.format:
        content = content.format(**ctx.attr.format)

    out = ctx.actions.declare_file(ctx.attr.name)

    ctx.actions.write(
        output = out,
        content = content + "\n",
    )

    return [
        DefaultInfo(files = depset([out])),
    ]

mtree = rule(
    implementation = _mtree_impl,
    attrs = {
        "files": attr.label_keyed_string_dict(allow_files = True),
        "format": attr.string_dict(),
        "symlinks": attr.string_dict(),
    },
)
