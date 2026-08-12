load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _musl_extension_impl(module_ctx):
    """Implementation of the musl module extension."""

    http_archive(
        name = "musl_libc",
        # When bumping musl, also update version_h in 3rd_party/libc/musl/musl.BUILD.bazel.
        urls = ["https://musl.libc.org/releases/musl-1.2.6.tar.gz"],
        strip_prefix = "musl-1.2.6",
        integrity = "sha256-1YX9O2E8ZhUfwySejtRPdwIMtebB5jWmFtP5+CRgUSo=",
        build_file = "//3rd_party/libc/musl:musl.BUILD.bazel",
    )

    metadata_kwargs = {}
    if bazel_features.external_deps.extension_metadata_has_reproducible:
        metadata_kwargs["reproducible"] = True

    return module_ctx.extension_metadata(
        root_module_direct_deps = ["musl_libc"],
        root_module_direct_dev_deps = [],
        **metadata_kwargs
    )

musl = module_extension(
    implementation = _musl_extension_impl,
    doc = "Extension for downloading and configuring musl libc",
)
