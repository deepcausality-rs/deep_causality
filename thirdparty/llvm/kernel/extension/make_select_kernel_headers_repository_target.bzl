load(
    "@kernel_headers//:linux_kernel_version_map.bzl",
    "LINUX_KERNEL_ARCHS_BY_VERSION",
    "LINUX_KERNEL_VERSION_MAP",
)
load("//constraints/kernel/linux:linux_kernel_versions.bzl", "LINUX_KERNEL_VERSIONS")
load("//constraints/libc:libc_versions.bzl", "LIBCS")
load("//platforms:common.bzl", "LIBC_SUPPORTED_TARGETS")
load(":kernel_helpers.bzl", "arch_to_kernel_arch")
load(":libc_kernel_versions.bzl", "LIBC_KERNEL_VERSIONS")

def _kernel_headers_repository_target(target_arch, kernel_version, bazel_target):
    return "@linux_kernel_headers_{}.{}//:{}".format(arch_to_kernel_arch(target_arch), kernel_version, bazel_target)

def _kernel_headers_available(target_arch, kernel_version):
    return arch_to_kernel_arch(target_arch) in LINUX_KERNEL_ARCHS_BY_VERSION.get(kernel_version, [])

def _version_alias_name(kernel_version, bazel_target):
    return "{}_{}".format(kernel_version, bazel_target)

def _fallback_alias_name(bazel_target):
    return "libc_mapped_{}".format(bazel_target)

def _linux_kernel_headers_version(kernel_version):
    if kernel_version not in LINUX_KERNEL_VERSION_MAP:
        fail("Missing Linux kernel version map entry for {}".format(kernel_version))

    return LINUX_KERNEL_VERSION_MAP[kernel_version]

def _make_select_kernel_headers_repository_target_for_linux_kernel(kernel_version, bazel_target):
    """Select the right kernel headers repository based on the target architecture."""
    selection = {}
    full_kernel_version = _linux_kernel_headers_version(kernel_version)
    for (target_os, target_arch) in LIBC_SUPPORTED_TARGETS:
        if not _kernel_headers_available(target_arch, full_kernel_version):
            continue
        apparent_target = _kernel_headers_repository_target(target_arch, full_kernel_version, bazel_target)
        selection["@llvm//platforms/config:{}_{}".format(target_os, target_arch)] = apparent_target

    return select(selection)

def _make_select_kernel_headers_repository_target_from_libc(bazel_target):
    """Select the right kernel headers repository based on the target architecture and libc version."""
    selection = {}
    for (target_os, target_arch) in LIBC_SUPPORTED_TARGETS:
        for libc_version in LIBCS + ["unconstrained"]:
            kernel_version = LIBC_KERNEL_VERSIONS[libc_version]
            if not _kernel_headers_available(target_arch, kernel_version):
                continue
            apparent_target = _kernel_headers_repository_target(target_arch, kernel_version, bazel_target)
            selection["@llvm//platforms/config:{}_{}_{}".format(target_os, target_arch, libc_version)] = apparent_target

    return select(selection)

def _make_select_kernel_headers_repository_target_from_linux_kernel(bazel_target):
    """Select explicit Linux kernel constraints, falling back to the libc-derived default."""
    selection = {
        "@llvm//constraints/kernel/linux:{}".format(kernel_version): _version_alias_name(kernel_version, bazel_target)
        for kernel_version in LINUX_KERNEL_VERSIONS
    }
    selection["@llvm//constraints/kernel/linux:unconstrained"] = _fallback_alias_name(bazel_target)

    return select(selection)

def declare_kernel_headers_repository_target(name, bazel_target = None, **kwargs):
    """Declare a kernel headers alias that honors explicit Linux kernel constraints.

    Args:
        name: The public alias name to declare.
        bazel_target: The target within the kernel headers repository to select. Defaults to name.
        **kwargs: Extra keyword arguments to forward to the public alias.
    """
    if bazel_target == None:
        bazel_target = name

    native.alias(
        name = _fallback_alias_name(name),
        actual = _make_select_kernel_headers_repository_target_from_libc(bazel_target),
    )

    for kernel_version in LINUX_KERNEL_VERSIONS:
        native.alias(
            name = _version_alias_name(kernel_version, name),
            actual = _make_select_kernel_headers_repository_target_for_linux_kernel(kernel_version, bazel_target),
        )

    native.alias(
        name = name,
        actual = _make_select_kernel_headers_repository_target_from_linux_kernel(name),
        **kwargs
    )
