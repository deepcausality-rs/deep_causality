def _libstdcxx_cxxconfig_header_impl(ctx):
    out = ctx.actions.declare_file(ctx.attr.name + "/bits/c++config.h")
    float128 = "define _GLIBCXX_USE_FLOAT128 1" if ctx.attr.enable_float128 else "undef _GLIBCXX_USE_FLOAT128"
    ctx.actions.run_shell(
        inputs = [
            ctx.file.config_h,
            ctx.file.cxxconfig_template,
            ctx.file.datestamp,
            ctx.file.basever,
        ],
        outputs = [out],
        arguments = [
            out.path,
            ctx.file.config_h.path,
            ctx.file.cxxconfig_template.path,
            ctx.file.datestamp.path,
            ctx.file.basever.path,
            float128,
            str(ctx.attr.inline_version),
            str(ctx.attr.have_attribute_visibility),
            str(ctx.attr.extern_template),
            str(ctx.attr.use_dual_abi),
            str(ctx.attr.use_cxx11_abi),
            str(ctx.attr.use_allocator_new),
        ],
        command = """set -eu
out="$1"
config_h="$2"
cxxconfig_template="$3"
datestamp="$4"
basever="$5"
float128="$6"
ns_version="$7"
visibility="$8"
externtemplate="$9"
dualabi="${10}"
cxx11abi="${11}"
allocatornew="${12}"

date="$(cat "$datestamp")"
release="$(sed 's/^\\([0-9]*\\).*$/\\1/' "$basever")"
ldbl_compat='s,g,g,'
ldbl_alt128_compat='s,g,g,'
verbose_assert='s,g,g,'

if grep '^[	 ]*#[	 ]*define[	 ][	 ]*_GLIBCXX_LONG_DOUBLE_COMPAT[	 ][	 ]*1[	 ]*$' "$config_h" >/dev/null 2>&1; then
    ldbl_compat='s,^#undef _GLIBCXX_LONG_DOUBLE_COMPAT$,#define _GLIBCXX_LONG_DOUBLE_COMPAT 1,'
fi
if grep '^[	 ]*#[	 ]*define[	 ][	 ]*_GLIBCXX_LONG_DOUBLE_ALT128_COMPAT[	 ][	 ]*1[	 ]*$' "$config_h" >/dev/null 2>&1; then
    ldbl_alt128_compat='s,^#undef _GLIBCXX_LONG_DOUBLE_ALT128_COMPAT$,#define _GLIBCXX_LONG_DOUBLE_ALT128_COMPAT 1,'
fi
if grep '^[	 ]*#[	 ]*define[	 ][	 ]*_GLIBCXX_HOSTED[	 ][	 ]*__STDC_HOSTED__[	 ]*$' "$config_h" >/dev/null 2>&1 \
    && grep '^[	 ]*#[	 ]*define[	 ][	 ]*_GLIBCXX_VERBOSE[	 ][	 ]*1[	 ]*$' "$config_h" >/dev/null 2>&1; then
    verbose_assert='s,^#undef _GLIBCXX_VERBOSE_ASSERT$,#define _GLIBCXX_VERBOSE_ASSERT 1,'
fi

sed -e "s,define __GLIBCXX__,define __GLIBCXX__ $date," \
    -e "s,define _GLIBCXX_RELEASE,define _GLIBCXX_RELEASE $release," \
    -e "s,define _GLIBCXX_INLINE_VERSION, define _GLIBCXX_INLINE_VERSION $ns_version," \
    -e "s,define _GLIBCXX_HAVE_ATTRIBUTE_VISIBILITY, define _GLIBCXX_HAVE_ATTRIBUTE_VISIBILITY $visibility," \
    -e "s,define _GLIBCXX_EXTERN_TEMPLATE$, define _GLIBCXX_EXTERN_TEMPLATE $externtemplate," \
    -e "s,define _GLIBCXX_USE_DUAL_ABI, define _GLIBCXX_USE_DUAL_ABI $dualabi," \
    -e "s,define _GLIBCXX_USE_CXX11_ABI, define _GLIBCXX_USE_CXX11_ABI $cxx11abi," \
    -e "s,define _GLIBCXX_USE_ALLOCATOR_NEW, define _GLIBCXX_USE_ALLOCATOR_NEW $allocatornew," \
    -e "s,define _GLIBCXX_USE_FLOAT128,$float128," \
    -e "$ldbl_compat" \
    -e "$ldbl_alt128_compat" \
    -e "$verbose_assert" \
    < "$cxxconfig_template" > "$out"

sed -e 's/HAVE_/_GLIBCXX_HAVE_/g' \
    -e '/^#ifndef LLVM_LIBSTDCXX_CONFIG_H$/d' \
    -e '/^#define LLVM_LIBSTDCXX_CONFIG_H 1$/d' \
    -e '/^#endif$/d' \
    -e '/PACKAGE/s,^,// ,' \
    -e '/PACKAGE/!s/VERSION/_GLIBCXX_VERSION/g' \
    -e 's/WORDS_/_GLIBCXX_WORDS_/g' \
    -e 's/LT_OBJDIR/_GLIBCXX_LT_OBJDIR/g' \
    -e 's,^#.*STDC_HEADERS,// &,' \
    -e 's/_DARWIN_USE_64_BIT_INODE/_GLIBCXX_DARWIN_USE_64_BIT_INODE/g' \
    -e 's/_FILE_OFFSET_BITS/_GLIBCXX_FILE_OFFSET_BITS/g' \
    -e 's/_LARGE_FILES/_GLIBCXX_LARGE_FILES/g' \
    -e 's/ICONV_CONST/_GLIBCXX_ICONV_CONST/g' \
    -e '/[	 ]_GLIBCXX_LONG_DOUBLE_COMPAT[	 ]/d' \
    -e '/[	 ]_GLIBCXX_LONG_DOUBLE_ALT128_COMPAT[	 ]/d' \
    -e '/[	 ]_GLIBCXX_USE_DUAL_ABI[	 ]/d' \
    -e '/[	 ]_GLIBCXX_USE_CXX11_ABI[	 ]/d' \
    < "$config_h" >> "$out"
{
    echo ""
    echo "#endif // _GLIBCXX_CXX_CONFIG_H"
} >> "$out"
        """,
        mnemonic = "LibstdcxxCxxConfigHeader",
    )

    return DefaultInfo(files = depset([out]))

# Generates bits/c++config.h from libstdc++-v3/include/bits/c++config.
# Keep the substitutions aligned with libstdc++-v3/include/Makefile.am and
# the config.h names produced by libstdc++-v3/configure.ac.
libstdcxx_cxxconfig_header = rule(
    implementation = _libstdcxx_cxxconfig_header_impl,
    attrs = {
        "basever": attr.label(allow_single_file = True, mandatory = True),
        "config_h": attr.label(allow_single_file = True, mandatory = True),
        "cxxconfig_template": attr.label(allow_single_file = True, mandatory = True),
        "datestamp": attr.label(allow_single_file = True, mandatory = True),
        "enable_float128": attr.bool(default = False),
        "extern_template": attr.int(default = 1),
        "have_attribute_visibility": attr.int(default = 1),
        "inline_version": attr.int(default = 0),
        "use_allocator_new": attr.int(default = 1),
        "use_cxx11_abi": attr.int(default = 1),
        "use_dual_abi": attr.int(default = 1),
    },
)
