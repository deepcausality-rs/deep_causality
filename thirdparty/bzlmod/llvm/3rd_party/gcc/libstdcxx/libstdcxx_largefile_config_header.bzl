def _libstdcxx_largefile_config_header_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + "/bits/largefile-config.h")
    ctx.actions.run_shell(
        inputs = [ctx.file.config_h],
        outputs = [out],
        arguments = [out.path, ctx.file.config_h.path],
        command = """set -eu
out="$1"
config_h="$2"
{
    grep 'define _DARWIN_USE_64_BIT_INODE' "$config_h" || true
    grep 'define _FILE_OFFSET_BITS' "$config_h" || true
    grep 'define _LARGE_FILES' "$config_h" || true
} > "$out"
""",
        mnemonic = "LibstdcxxLargefileConfigHeader",
    )

    return DefaultInfo(files = depset([out]))

# Mirrors libstdc++'s largefile-config.h generation from
# libstdc++-v3/include/Makefile.am.
libstdcxx_largefile_config_header = rule(
    implementation = _libstdcxx_largefile_config_header_impl,
    attrs = {
        "config_h": attr.label(allow_single_file = True, mandatory = True),
    },
)
