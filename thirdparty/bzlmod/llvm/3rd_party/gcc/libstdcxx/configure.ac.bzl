# Active libstdc++ config.h composition ported from GCC's
# libstdc++-v3/configure.ac. This file should read like the supported Linux GNU
# configure flow; unsupported branches stay documented as inactive notes below.

load("//3rd_party/gcc/libstdcxx/autoconf:checks.bzl", "ac_check_headers")
load(
    ":acinclude.m4.bzl",
    "glibcxx_abi_policies",
    "glibcxx_check_c99_tr1",
    "glibcxx_check_compiler_features",
    "glibcxx_check_debugging",
    "glibcxx_check_filesystem_deps",
    "glibcxx_check_gettimeofday",
    "glibcxx_check_gthreads",
    "glibcxx_check_hardware_concurrency",
    "glibcxx_check_int64_t",
    "glibcxx_check_lfs",
    "glibcxx_check_math11_proto",
    "glibcxx_check_networking_deps",
    "glibcxx_check_pthread_clock_apis",
    "glibcxx_check_stdio_locking",
    "glibcxx_check_stdio_proto",
    "glibcxx_check_system_error",
    "glibcxx_check_text_encoding",
    "glibcxx_check_tmpnam",
    "glibcxx_check_uchar_h",
    "glibcxx_compute_stdio_integer_constants",
    "glibcxx_enable_allocator",
    "glibcxx_enable_atomic_builtins",
    "glibcxx_enable_c99",
    "glibcxx_enable_clocale",
    "glibcxx_enable_cstdio",
    "glibcxx_enable_decimal_float",
    "glibcxx_enable_hosted",
    "glibcxx_enable_int128_float128",
    "glibcxx_enable_libstdcxx_time",
    "glibcxx_enable_lock_policy",
    "glibcxx_enable_long_long",
    "glibcxx_enable_pch",
    "glibcxx_enable_verbose",
    "glibcxx_enable_wchar_t",
    "glibcxx_misc_compile_checks",
    "glibcxx_misc_link_checks",
    "glibcxx_random_policy",
    "glibcxx_resource_limits_policy",
    "glibcxx_zoneinfo_policy",
)
load(
    ":gcc_config_checks.bzl",
    "am_iconv",
    "gcc_check_tls",
    "gcc_check_unwind_getipinfo",
    "gcc_linux_futex",
)
load(
    ":linkage.m4.bzl",
    "gcc_check_math_support",
    "gcc_check_stdlib_support",
)

_HEADER_CHECKS = [
    "arpa/inet.h",
    "debugapi.h",
    "dirent.h",
    "dlfcn.h",
    "endian.h",
    "execinfo.h",
    "fcntl.h",
    "float.h",
    "fp.h",
    "ieeefp.h",
    "inttypes.h",
    "libintl.h",
    "link.h",
    "linux/random.h",
    "linux/types.h",
    "locale.h",
    "machine/endian.h",
    "machine/param.h",
    "memory.h",
    "nan.h",
    "netdb.h",
    "netinet/in.h",
    "netinet/tcp.h",
    "poll.h",
    "stdalign.h",
    "stdbool.h",
    "stdint.h",
    "stdlib.h",
    "string.h",
    "strings.h",
    "sys/ipc.h",
    "sys/isa_defs.h",
    "sys/machine.h",
    "sys/mman.h",
    "sys/param.h",
    "sys/ptrace.h",
    "sys/resource.h",
    "sys/sdt.h",
    "sys/sem.h",
    "sys/socket.h",
    "sys/stat.h",
    "sys/statvfs.h",
    "sys/time.h",
    "sys/types.h",
    "tgmath.h",
    "tlhelp32.h",
    "uchar.h",
    "unistd.h",
    "utime.h",
    "wchar.h",
    "wctype.h",
    "windows.h",
    "xlocale.h",
]

def config_entries(gcc_version):
    # This function mirrors libstdc++-v3/configure.ac order for the supported
    # hosted Linux GNU configuration. Unsupported configure branches are kept
    # below as comments so future target work has an obvious source anchor.
    return (
        glibcxx_enable_hosted() +
        glibcxx_enable_verbose() +
        glibcxx_enable_pch() +
        glibcxx_enable_atomic_builtins(gcc_version) +
        glibcxx_enable_lock_policy() +
        glibcxx_enable_decimal_float() +
        glibcxx_enable_int128_float128(gcc_version) +
        glibcxx_check_compiler_features() +
        glibcxx_enable_cstdio() +
        glibcxx_enable_clocale() +
        glibcxx_enable_allocator() +
        glibcxx_enable_long_long() +
        glibcxx_enable_wchar_t() +
        glibcxx_enable_c99(gcc_version) +
        glibcxx_check_c99_tr1() +
        glibcxx_check_stdio_proto() +
        glibcxx_check_math11_proto() +
        glibcxx_check_uchar_h(gcc_version) +
        glibcxx_check_int64_t(gcc_version) +
        glibcxx_check_lfs(gcc_version) +
        glibcxx_check_gettimeofday() +
        ac_check_headers(["sys/ioctl.h", "sys/filio.h"]) +
        glibcxx_misc_compile_checks(gcc_version) +
        ac_check_headers(["sys/uio.h", "fenv.h", "complex.h"]) +
        glibcxx_compute_stdio_integer_constants() +
        glibcxx_check_tmpnam() +
        glibcxx_check_pthread_clock_apis(gcc_version) +
        ac_check_headers(["sys/sysinfo.h", "unistd.h"]) +
        glibcxx_check_hardware_concurrency() +
        ac_check_headers(_HEADER_CHECKS) +
        gcc_check_math_support() +
        gcc_check_stdlib_support(gcc_version) +
        glibcxx_check_system_error(gcc_version) +
        glibcxx_random_policy(gcc_version) +
        gcc_check_tls() +
        glibcxx_misc_link_checks(gcc_version) +
        am_iconv() +
        gcc_check_unwind_getipinfo() +
        gcc_linux_futex() +
        glibcxx_check_gthreads(gcc_version) +
        glibcxx_abi_policies() +
        glibcxx_check_filesystem_deps(gcc_version) +
        glibcxx_check_networking_deps(gcc_version) +
        glibcxx_enable_libstdcxx_time(gcc_version) +
        glibcxx_zoneinfo_policy(gcc_version) +
        glibcxx_check_text_encoding(gcc_version) +
        glibcxx_check_debugging(gcc_version) +
        glibcxx_check_stdio_locking(gcc_version) +
        glibcxx_resource_limits_policy()
    )

# GCC 12's GCC_HEADER_STDINT(include/gstdint.h) generates a compatibility
# header for src/c++11/compatibility-atomic-c++0x.cc. That hook has no config.h
# output; the file-generation part is modeled in the GCC BUILD overlay for
# GCC <13.

# configure.ac branches intentionally inactive for the current port:
#
# Cross without Linux GNU support would call GLIBCXX_CROSSCONFIG from
# crossconfig.m4. That is documented in crossconfig.m4.bzl but not active.
#
# Newlib and picolibc branches define C library shortcuts directly in
# configure.ac. They remain inactive because this libstdc++ runtime currently
# supports Linux GNU only.
#
# Darwin, Solaris, AIX, BSD, MinGW, VxWorks, RTEMS, and other target branches
# either select different OS directories or rely on target libraries not
# provided by this port.
#
# Build-only docs/testsuite/install probes are intentionally omitted from the
# active config.h model.
