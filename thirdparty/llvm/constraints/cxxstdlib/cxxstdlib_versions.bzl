load("//3rd_party/gcc:version.bzl", "GCC_VERSIONS", "libstdcxx_constraint_value")

LIBSTDCXXS = [
    libstdcxx_constraint_value(version)
    for version in GCC_VERSIONS
]

CXXSTDLIBS = [
    "libcxx",
] + LIBSTDCXXS

DEFAULT_CXXSTDLIB = "libcxx"
