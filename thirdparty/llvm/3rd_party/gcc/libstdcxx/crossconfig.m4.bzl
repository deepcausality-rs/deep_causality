# Cross-configuration check model ported from GCC's
# libstdc++-v3/crossconfig.m4. Only the Linux GNU-relevant branch is active in
# this Bazel port; other target branches remain documented as unsupported.

load("//3rd_party/gcc:version.bzl", "libstdcxx_has_dev_random_policy")
load("//3rd_party/gcc/libstdcxx/autoconf:checks.bzl", "policy_define")
load(
    ":gcc_config_checks.bzl",
    "am_iconv",
    "gcc_check_tls",
    "gcc_linux_futex",
)
load(
    ":linkage.m4.bzl",
    "gcc_check_math_support",
    "gcc_check_stdlib_support",
)

def glibcxx_crossconfig_linux_gnu(gcc_version):
    # libstdc++-v3/crossconfig.m4 uses this branch for *-linux*, *-uclinux*,
    # *-gnu*, *-kfreebsd*-gnu, *-cygwin*, and *-solaris*. Only Linux GNU is
    # active in this Bazel port; cygwin/solaris and non-glibc targets are out
    # of scope.
    if libstdcxx_has_dev_random_policy(gcc_version):
        random_policy = [
            # crossconfig.m4 hardcodes the same GLIBCXX_CHECK_DEV_RANDOM
            # feature pair for Linux-family targets.
            policy_define(
                "_GLIBCXX_USE_DEV_RANDOM",
                defines_on_success = [
                    "_GLIBCXX_USE_DEV_RANDOM",
                    "_GLIBCXX_USE_RANDOM_TR1",
                ],
            ),
        ]
    else:
        random_policy = [policy_define("_GLIBCXX_USE_RANDOM_TR1")]
    return (
        gcc_check_math_support() +
        gcc_check_stdlib_support(gcc_version) +
        random_policy +
        gcc_check_tls() +
        am_iconv() +
        gcc_linux_futex()
    )

# Unsupported crossconfig.m4 branches intentionally left inactive:
#
# arm*-*-symbianelf*: freestanding target, not supported by this libstdc++ port.
#
# avr*-*-*: AVR/newlib-style target, not supported.
#
# mips*-sde-elf*: SDE C library target, not supported.
#
# *-aix*: AIX target, not supported.
#
# *-darwin*: Darwin target, not supported for libstdc++ in this repository.
#
# *djgpp: DOS/DJGPP target, not supported.
#
# *-freebsd*, *-netbsd*, *-openbsd*: BSD libc targets, not supported.
#
# *-fuchsia*: Fuchsia target, not supported.
#
# *-hpux*: HP-UX target, not supported.
#
# *-mingw32*: Windows libstdc++ target is intentionally unsupported for now.
#
# *-qnx*, *-tpf, *-*vms*, *-vxworks*: non-Linux targets, not supported.
#
# newlib and picolibc branches are handled directly in configure.ac upstream;
# this Bazel port does not support those C libraries today.
