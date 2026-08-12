load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
load("//:http_bsdtar_archive.bzl", "http_bsdtar_archive")
load(
    "//3rd_party/gcc:version.bzl",
    "GCC_RELEASES",
    "GCC_VERSIONS",
    "gcc_has_config_toolexeclibdir_m4",
    "gcc_patches",
    "gcc_repo_name",
)

# Sparse archive roots required by 3rd_party/gcc/gcc.BUILD.bazel. Before
# trimming or extending this list, check the BUILD file against GCC's
# libstdc++-v3/include/Makefile.am, libsupc++/Makefile.am, and
# src/*/Makefile.am inputs.
_GCC_ARCHIVE_INCLUDES = [
    "gcc/BASE-VER",
    "gcc/DATESTAMP",
    "gcc/ginclude/unwind-arm-common.h",
    "config/acx.m4",
    "config/cet.m4",
    "config/futex.m4",
    "config/gc++filt.m4",
    "config/gthr.m4",
    "config/hwcaps.m4",
    "config/iconv.m4",
    "config/lthostflags.m4",
    "config/multi.m4",
    "config/no-executables.m4",
    "config/tls.m4",
    "config/unwind_ipinfo.m4",
    "include/ansidecl.h",
    "include/demangle.h",
    "include/dyn-string.h",
    "include/getopt.h",
    "include/libiberty.h",
    "libgcc/gthr-posix.h",
    "libgcc/gthr-single.h",
    "libgcc/gthr.h",
    "libgcc/config/arm/unwind-arm.h",
    "libgcc/unwind-generic.h",
    "libgcc/unwind-pe.h",
    "libiberty/cp-demangle.c",
    "libiberty/cp-demangle.h",
    "libstdc++-v3/acinclude.m4",
    "libstdc++-v3/config/**",
    "libstdc++-v3/configure.ac",
    "libstdc++-v3/configure.host",
    "libstdc++-v3/crossconfig.m4",
    "libstdc++-v3/linkage.m4",
    "libstdc++-v3/include/**",
    "libstdc++-v3/libsupc++/**",
    "libstdc++-v3/src/**",
]

_GCC_10_ARCHIVE_INCLUDES = [
    "config/toolexeclibdir.m4",
]

_from_path = tag_class(
    attrs = {
        "path": attr.string(
            doc = "Local GCC source tree. The tree must include version.bzl defining GCC_VERSION.",
            mandatory = True,
        ),
    },
)

def _gcc_impl(module_ctx):
    root_path = None
    dependency_path = None
    for mod in module_ctx.modules:
        for tag in mod.tags.from_path:
            if mod.is_root:
                if root_path != None:
                    fail("Only one root GCC source path override is allowed.")
                root_path = tag.path
            else:
                if dependency_path != None and dependency_path != tag.path:
                    fail("Only one dependency GCC source path override is allowed.")
                dependency_path = tag.path

    path = root_path or dependency_path

    metadata_kwargs = {}

    for version in GCC_VERSIONS:
        repo_name = gcc_repo_name(version)

        if path != None:
            new_local_repository(
                name = repo_name,
                build_file = Label("//3rd_party/gcc:gcc.BUILD.bazel"),
                path = path,
            )
        else:
            release = GCC_RELEASES[version]
            http_bsdtar_archive(
                name = repo_name,
                build_file = Label("//3rd_party/gcc:gcc.BUILD.bazel"),
                generated_files = {
                    "version.bzl": "GCC_VERSION = \"{}\"\n".format(version),
                },
                includes = _GCC_ARCHIVE_INCLUDES + (_GCC_10_ARCHIVE_INCLUDES if gcc_has_config_toolexeclibdir_m4(version) else []),
                patch_args = ["-p1"],
                patches = [Label(patch) for patch in gcc_patches(version)],
                sha256 = release["sha256"],
                strip_prefix = "gcc-{}".format(release["commit"]),
                urls = ["https://github.com/gcc-mirror/gcc/archive/{}.tar.gz".format(release["commit"])],
            )

    _gcc_trampoline_repository(name = "gcc")

    if bazel_features.external_deps.extension_metadata_has_reproducible:
        metadata_kwargs["reproducible"] = True

    return module_ctx.extension_metadata(
        root_module_direct_deps = ["gcc"],
        root_module_direct_dev_deps = [],
        **metadata_kwargs
    )

gcc = module_extension(
    implementation = _gcc_impl,
    tag_classes = {
        "from_path": _from_path,
    },
)

def _gcc_trampoline_repository_impl(repository_ctx):
    repository_ctx.template("BUILD.bazel", repository_ctx.attr._build_file)
    return repository_ctx.repo_metadata(reproducible = True)

_gcc_trampoline_repository = repository_rule(
    implementation = _gcc_trampoline_repository_impl,
    attrs = {
        "_build_file": attr.label(
            allow_single_file = True,
            default = ":trampoline.BUILD.bazel",
        ),
    },
)
