load("@bazel_skylib//lib:paths.bzl", "paths")

_MUSL_SUPPORTED_ARCH = [
    "aarch64",
    "arm",
    "generic",
    "i386",
    "loongarch64",
    "m68k",
    "microblaze",
    "mips",
    "mips64",
    "mipsn32",
    "or1k",
    "powerpc",
    "powerpc64",
    "riscv32",
    "riscv64",
    "s390x",
    "sh",
    "x32",
    "x86_64",
]

def _libc_musl_srcs_filegroup_impl(ctx):
    input_files = ctx.files.srcs
    file_map = {f.path: f for f in input_files}

    arch = ctx.attr.arch

    filtered = []
    for f in input_files:
        dir_part = paths.dirname(f.path)  # e.g. "foo/bar"
        base = paths.basename(f.path)  # e.g. "baz.c"
        noext = base.rsplit(".", 1)[0]  # e.g  "baz"
        dir_base = paths.basename(dir_part) if dir_part else ""

        # 1) if it's already in an arch-specific dir:
        if dir_base in _MUSL_SUPPORTED_ARCH:
            # only keep it if it matches our target arch
            if dir_base == arch:
                filtered.append(f)

            # otherwise drop it
            continue

        # 2) otherwise look for overrides under src/.../<arch>/
        found = False
        for ext in [".s", ".S", ".c"]:
            cand = "{}/{}/{}{}".format(
                dir_part,
                arch,
                noext,
                ext,
            )
            if cand in file_map:
                filtered.append(file_map[cand])
                found = True
                break

        # 3) fallback to the generic file if no override
        if not found:
            filtered.append(f)

    malloc_files = []
    internal_files = []
    string_files = []
    other_files = []

    # Those files have different compile options
    for f in filtered:
        if "malloc/" in f.path:
            malloc_files.append(f)
        elif "internal/" in f.path:
            internal_files.append(f)
        elif "string/" in f.path:
            string_files.append(f)
        else:
            other_files.append(f)

    return [
        DefaultInfo(
            files = depset(filtered),
        ),
        OutputGroupInfo(
            malloc_files = depset(malloc_files),
            internal_files = depset(internal_files),
            string_files = depset(string_files),
            default_files = depset(other_files),
        ),
    ]

libc_musl_srcs_filegroup = rule(
    implementation = _libc_musl_srcs_filegroup_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = [".c", ".S", ".s"]),
        "arch": attr.string(mandatory = True, values = _MUSL_SUPPORTED_ARCH),
    },
)
