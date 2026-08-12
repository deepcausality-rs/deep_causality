def _libstdcxx_gthr_headers_impl(ctx):
    gthr_h = ctx.actions.declare_file(ctx.attr.name + "/bits/gthr.h")
    gthr_single_h = ctx.actions.declare_file(ctx.attr.name + "/bits/gthr-single.h")
    gthr_posix_h = ctx.actions.declare_file(ctx.attr.name + "/bits/gthr-posix.h")
    gthr_default_h = ctx.actions.declare_file(ctx.attr.name + "/bits/gthr-default.h")
    ctx.actions.run_shell(
        inputs = [
            ctx.file.gthr_h,
            ctx.file.gthr_single_h,
            ctx.file.gthr_posix_h,
            ctx.file.gthr_default_h,
        ],
        outputs = [
            gthr_h,
            gthr_single_h,
            gthr_posix_h,
            gthr_default_h,
        ],
        arguments = [
            ctx.file.gthr_h.path,
            gthr_h.path,
            ctx.file.gthr_single_h.path,
            gthr_single_h.path,
            ctx.file.gthr_posix_h.path,
            gthr_posix_h.path,
            ctx.file.gthr_default_h.path,
            gthr_default_h.path,
        ],
        command = """set -eu
gthr_in="$1"
gthr_out="$2"
gthr_single_in="$3"
gthr_single_out="$4"
gthr_posix_in="$5"
gthr_posix_out="$6"
gthr_default_in="$7"
gthr_default_out="$8"
uppercase='[ABCDEFGHIJKLMNOPQRSTUVWXYZ_]'
sed -e '/^#pragma/b' \
    -e '/^#/s/\\('"$uppercase$uppercase"'*\\)/_GLIBCXX_\\1/g' \
    -e 's/_GLIBCXX_SUPPORTS_WEAK/__GXX_WEAK__/g' \
    -e 's/_GLIBCXX___MINGW32_GLIBCXX___/__MINGW32__/g' \
    -e 's,^#include "\\(.*\\)",#include <bits/\\1>,g' \
    < "$gthr_in" > "$gthr_out"

sed -e 's/\\(UNUSED\\)/_GLIBCXX_\\1/g' \
    -e 's/\\(GCC'"$uppercase"'*_H\\)/_GLIBCXX_\\1/g' \
    < "$gthr_single_in" > "$gthr_single_out"

sed -e 's/\\(UNUSED\\)/_GLIBCXX_\\1/g' \
    -e 's/\\(GCC'"$uppercase"'*_H\\)/_GLIBCXX_\\1/g' \
    -e 's/SUPPORTS_WEAK/__GXX_WEAK__/g' \
    -e 's/\\('"$uppercase"'*USE_WEAK\\)/_GLIBCXX_\\1/g' \
    < "$gthr_posix_in" > "$gthr_posix_out"

sed -e 's/\\(UNUSED\\)/_GLIBCXX_\\1/g' \
    -e 's/\\(GCC'"$uppercase"'*_H\\)/_GLIBCXX_\\1/g' \
    -e 's/SUPPORTS_WEAK/__GXX_WEAK__/g' \
    -e 's/\\('"$uppercase"'*USE_WEAK\\)/_GLIBCXX_\\1/g' \
    -e 's,^#include "\\(.*\\)",#include <bits/\\1>,g' \
    < "$gthr_default_in" > "$gthr_default_out"
""",
        mnemonic = "LibstdcxxGthrHeaders",
    )

    return DefaultInfo(files = depset([
        gthr_h,
        gthr_single_h,
        gthr_posix_h,
        gthr_default_h,
    ]))

# Mirrors the gthr header transformations performed by
# libstdc++-v3/include/Makefile.am for libgcc/gthr*.h.
libstdcxx_gthr_headers = rule(
    implementation = _libstdcxx_gthr_headers_impl,
    attrs = {
        "gthr_default_h": attr.label(allow_single_file = True, mandatory = True),
        "gthr_h": attr.label(allow_single_file = True, mandatory = True),
        "gthr_posix_h": attr.label(allow_single_file = True, mandatory = True),
        "gthr_single_h": attr.label(allow_single_file = True, mandatory = True),
    },
)
