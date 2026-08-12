# Shared configure-check model ported from GCC's top-level config/*.m4 files.
# Compare these helpers with config/tls.m4, config/unwind_ipinfo.m4,
# config/futex.m4, config/iconv.m4, and config/no-executables.m4 before
# changing their semantics.
#
# AC_COMPILE_IFELSE and AC_LINK_IFELSE success branches may define several
# macros. The check name stays the audit key; defines_on_success records the
# config.h fan-out for that successful probe.
#
# GCC_HEADER_STDINT has no config.h define. GCC 12's private gstdint.h output is
# modeled by the GCC overlay's generated libstdcxx_gstdint_h header.

load(
    "//3rd_party/gcc/libstdcxx/autoconf:checks.bzl",
    "compile_check",
    "link_check",
    "policy_define",
)

CXX_NO_EXCEPTIONS_FLAGS = ["-fno-exceptions"]
PTHREAD_LINK_FLAGS = ["-lpthread"]

def gcc_check_tls():
    return [
        compile_check(
            name = "HAVE_CC_TLS",
            source = "__thread int a; int b; int main(void) { return a = b; }",
        ),
        link_check(
            name = "HAVE_TLS",
            language = "c",
            source = "__thread int a; int b; int main(void) { return a = b; }",
        ),
    ]

def gcc_check_unwind_getipinfo():
    # config/unwind_ipinfo.m4 defines this by target policy for GCC's own
    # unwinder on Linux GNU targets. It is about _Unwind_GetIPInfo, not
    # networking APIs.
    return [policy_define("HAVE_GETIPINFO")]

def gcc_linux_futex():
    return [
        link_check(
            name = "HAVE_LINUX_FUTEX",
            source = """
#include <linux/futex.h>
#include <sys/syscall.h>
#include <unistd.h>
int main() { return syscall(SYS_futex, (int *)0, FUTEX_WAKE, 1, 0, 0, 0); }
""",
        ),
    ]

def am_iconv():
    return [
        link_check(
            name = "HAVE_ICONV",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <iconv.h>
int main() {
    iconv_t cd = iconv_open("", "");
    iconv(cd, (char **)0, (size_t *)0, (char **)0, (size_t *)0);
    iconv_close(cd);
    return 0;
}
""",
        ),
    ]
