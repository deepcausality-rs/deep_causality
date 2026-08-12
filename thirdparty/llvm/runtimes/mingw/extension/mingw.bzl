load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _mingw_extension_impl(module_ctx):
    """Implementation of the mingw module extension."""

    http_archive(
        name = "mingw",
        # Technically sourceforge is the project's recommended primary download location, but they serve
        # an HTML page to Go-based downloader so they're incompatible with Buildbuddy's remote downloader...
        # https://sourceforge.net/projects/mingw-w64/files/mingw-w64/mingw-w64-release/mingw-w64-v14.0.0.tar.bz2
        urls = ["https://github.com/mingw-w64/mingw-w64/archive/refs/tags/v14.0.0.tar.gz"],
        integrity = "sha256-1xzGRM1aN8M38nGfPgx52J6NjV+54pUqYtP6I2I9wTc=",
        strip_prefix = "mingw-w64-14.0.0",
        build_file = "//runtimes/mingw:mingw.BUILD.bazel",
    )

    metadata_kwargs = {}
    if bazel_features.external_deps.extension_metadata_has_reproducible:
        metadata_kwargs["reproducible"] = True

    return module_ctx.extension_metadata(
        root_module_direct_deps = ["mingw"],
        root_module_direct_dev_deps = [],
        **metadata_kwargs
    )

mingw = module_extension(
    implementation = _mingw_extension_impl,
    doc = "Extension for downloading and configuring mingw",
)
