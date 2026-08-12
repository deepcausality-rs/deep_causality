"""
Utilities used by compiler-rt BUILD rules.

`filter_builtin_sources` is a rule similar to filegroup, but it also ensures
that only the last occurrence of each basename is kept. This makes it possible
to list architecture specific builtins after the generic ones without pulling
both into the build. The rule also filters out the entries listed in
`lib/builtins/Darwin-excludes.txt` when building for macOS so that we match the
upstream Darwin build.
"""

# TODO: Generate this list from lib/builtins/Darwin-excludes.txt.
_MACOS_EXCLUDE_LIST = [
    "apple_versioning",
    "addtf3",
    "divtf3",
    "multf3",
    "powitf2",
    "subtf3",
    "trampoline_setup",
]

def _filter_builtin_sources_impl(ctx):
    is_macos = ctx.target_platform_has_constraint(
        ctx.attr._macos_constraint[platform_common.ConstraintValueInfo],
    )

    basename_to_file = {}
    for f in ctx.files.srcs:
        excluded = False

        if is_macos:
            for e in _MACOS_EXCLUDE_LIST:
                if e in f.path:
                    excluded = True
                    break

        if not excluded:
            basename = f.basename[:-(len(f.extension) + 1)]
            basename_to_file[basename] = f

    # Only keep the last occurrence for each basename (without extension)
    unique_files = list(basename_to_file.values())

    return [
        DefaultInfo(
            files = depset(unique_files),
        ),
    ]

filter_builtin_sources = rule(
    implementation = _filter_builtin_sources_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "_macos_constraint": attr.label(
            default = Label("@platforms//os:macos"),
            providers = [platform_common.ConstraintValueInfo],
        ),
    },
    doc = "Like filegroup, but filters OSX sources and removes duplicate basenames, keeping only the last one.",
)
