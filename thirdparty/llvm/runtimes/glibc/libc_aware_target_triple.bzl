load("//constraints/libc:libc_versions.bzl", "DEFAULT_LIBC", "LIBCS")
load("//platforms:common.bzl", "LIBC_SUPPORTED_TARGETS")

# Per-cpu remaps for zig target triples. Zig uses generic arch names with an
# ABI-bearing libc suffix (e.g. arm/gnueabihf), not bazel's (armv7/gnu).
_ZIG_CPU_OVERRIDES = {
    "armv7": "arm",
}

_ZIG_LIBC_FAMILY_OVERRIDES = {
    ("armv7", "gnu"): "gnueabihf",
    ("armv7", "musl"): "musleabihf",
}

def _zig_triple(target_os, target_cpu, target_libc_suffix):
    zig_cpu = _ZIG_CPU_OVERRIDES.get(target_cpu, target_cpu)
    libc_family, _, libc_version = target_libc_suffix.partition(".")
    zig_libc_family = _ZIG_LIBC_FAMILY_OVERRIDES.get((target_cpu, libc_family), libc_family)
    zig_libc_suffix = zig_libc_family + ("." + libc_version if libc_version else "")
    return "{}-{}-{}".format(zig_cpu, target_os, zig_libc_suffix)

# For use with zig tools that consume parse zig targets triples
# Zig target triples only, not LLVM
def libc_aware_target_triple():
    target = {}
    for (target_os, target_cpu) in LIBC_SUPPORTED_TARGETS:
        for libc_version in LIBCS + ["unconstrained"]:
            target_libc_suffix = libc_version if libc_version != "unconstrained" else DEFAULT_LIBC
            target["//platforms/config:{}_{}_{}".format(target_os, target_cpu, libc_version)] = _zig_triple(target_os, target_cpu, target_libc_suffix)

    return select(target)
