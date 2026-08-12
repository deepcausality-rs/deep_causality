load("//constraints/libc:libc_versions.bzl", "DEFAULT_LIBC", "GLIBCS")
load("//platforms:common.bzl", "LIBC_SUPPORTED_TARGETS")
load(":glibc.bzl", "glibc_triple")

def make_select_glibc_repository_target(bazel_repository, bazel_target):
    selection = {}
    for (target_os, target_arch) in LIBC_SUPPORTED_TARGETS:
        triple = glibc_triple(target_os, target_arch)
        for libc_version in GLIBCS + ["unconstrained"]:
            apparent_libc_value = libc_version if libc_version != "unconstrained" else DEFAULT_LIBC

            # libc constraint values are "gnu.X.Y"; the glibc repos are keyed
            # by the numeric version only.
            glibc_version = apparent_libc_value.split(".", 1)[1]
            apparent_target = "{}_{}.{}//:{}".format(bazel_repository, triple, glibc_version, bazel_target)
            selection["@llvm//platforms/config:{}_{}_{}".format(target_os, target_arch, libc_version)] = apparent_target

    return select(selection)
