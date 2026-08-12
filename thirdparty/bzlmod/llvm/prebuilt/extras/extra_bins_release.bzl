load("@tar.bzl//tar:tar.bzl", "tar")
load("//prebuilt:mtree.bzl", "mtree")

def extra_bins_release(name):
    files = {
        "@glibc-stubs-generator//:glibc-stubs-generator": "bin/glibc-stubs-generator",
        "@libstdcxx-stubs-generator//:libstdc++-stubs-generator": "bin/libstdcxx-stubs-generator",
        "@xpkgutil//:pkgutil": "bin/pkgutil",
    }

    mtree(
        name = name + "_mtree",
        files = files,
    )

    tar(
        name = name,
        srcs = files.keys(),
        args = [
            "--options",
            "zstd:compression-level=22",
        ],
        compress = "zstd",
        mtree = name + "_mtree",
    )
