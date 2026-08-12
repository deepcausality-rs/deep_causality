# Configure-check model ported from GCC's libstdc++-v3/linkage.m4. Keep
# helper names and grouping close to the GLIBCXX_* linkage macros so GCC
# updates can be reviewed by comparing linkage.m4 against this module.
#
# The linkage.m4 helper macros GLIBCXX_CHECK_MATH_DECL_1,
# GLIBCXX_CHECK_MATH_DECL_2, GLIBCXX_CHECK_MATH_DECL_3,
# GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_1,
# GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_2,
# GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_3,
# GLIBCXX_CHECK_MATH_DECLS_AND_LINKAGES_1,
# GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_1, and
# GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_2 are represented by
# gcc_check_math_support() and gcc_check_stdlib_support().

load(
    "//3rd_party/gcc:version.bzl",
    "libstdcxx_has_stdlib_secure_getenv_check",
    "libstdcxx_has_stdlib_timespec_get_check",
)
load(
    "//3rd_party/gcc/libstdcxx/autoconf:checks.bzl",
    "function_link_check",
    "link_check",
)

MATH_LINK_FLAGS = ["-lm"]

_MATH_FUNCTIONS = [
    ("HAVE_ACOSF", "acosf(0.0f)"),
    ("HAVE_ACOSL", "acosl(0.0L)"),
    ("HAVE_ASINF", "asinf(0.0f)"),
    ("HAVE_ASINL", "asinl(0.0L)"),
    ("HAVE_ATAN2F", "atan2f(0.0f, 1.0f)"),
    ("HAVE_ATAN2L", "atan2l(0.0L, 1.0L)"),
    ("HAVE_ATANF", "atanf(0.0f)"),
    ("HAVE_ATANL", "atanl(0.0L)"),
    ("HAVE_CEILF", "ceilf(0.0f)"),
    ("HAVE_CEILL", "ceill(0.0L)"),
    ("HAVE_COSF", "cosf(0.0f)"),
    ("HAVE_COSHF", "coshf(0.0f)"),
    ("HAVE_COSHL", "coshl(0.0L)"),
    ("HAVE_COSL", "cosl(0.0L)"),
    ("HAVE_EXPF", "expf(0.0f)"),
    ("HAVE_EXPL", "expl(0.0L)"),
    ("HAVE_FABSF", "fabsf(0.0f)"),
    ("HAVE_FABSL", "fabsl(0.0L)"),
    ("HAVE_FINITE", "finite(0.0)"),
    ("HAVE_FINITEF", "finitef(0.0f)"),
    ("HAVE_FINITEL", "finitel(0.0L)"),
    ("HAVE_FPCLASS", "fpclass(0.0)"),
    ("HAVE_FLOORF", "floorf(0.0f)"),
    ("HAVE_FLOORL", "floorl(0.0L)"),
    ("HAVE_FMODF", "fmodf(1.0f, 1.0f)"),
    ("HAVE_FMODL", "fmodl(1.0L, 1.0L)"),
    ("HAVE_FREXPF", "frexpf(1.0f, &i)"),
    ("HAVE_FREXPL", "frexpl(1.0L, &i)"),
    ("HAVE_HYPOT", "hypot(1.0, 1.0)"),
    ("HAVE_HYPOTF", "hypotf(1.0f, 1.0f)"),
    ("HAVE_HYPOTL", "hypotl(1.0L, 1.0L)"),
    ("HAVE_ISINF", "isinf(0.0)"),
    ("HAVE_ISINFF", "isinff(0.0f)"),
    ("HAVE_ISINFL", "isinfl(0.0L)"),
    ("HAVE_ISNAN", "isnan(0.0)"),
    ("HAVE_ISNANF", "isnanf(0.0f)"),
    ("HAVE_ISNANL", "isnanl(0.0L)"),
    ("HAVE_LDEXPF", "ldexpf(1.0f, 1)"),
    ("HAVE_LDEXPL", "ldexpl(1.0L, 1)"),
    ("HAVE_LOG10F", "log10f(1.0f)"),
    ("HAVE_LOG10L", "log10l(1.0L)"),
    ("HAVE_LOGF", "logf(1.0f)"),
    ("HAVE_LOGL", "logl(1.0L)"),
    ("HAVE_MODF", "modf(1.0, &d)"),
    ("HAVE_MODFF", "modff(1.0f, &f)"),
    ("HAVE_MODFL", "modfl(1.0L, &ld)"),
    ("HAVE_POWF", "powf(1.0f, 1.0f)"),
    ("HAVE_POWL", "powl(1.0L, 1.0L)"),
    ("HAVE_QFPCLASS", "qfpclass(0.0)"),
    ("HAVE_SINCOS", "sincos(1.0, &sd, &cd)"),
    ("HAVE_SINCOSF", "sincosf(1.0f, &sf, &cf)"),
    ("HAVE_SINCOSL", "sincosl(1.0L, &sld, &cld)"),
    ("HAVE_SINF", "sinf(0.0f)"),
    ("HAVE_SINHF", "sinhf(0.0f)"),
    ("HAVE_SINHL", "sinhl(0.0L)"),
    ("HAVE_SINL", "sinl(0.0L)"),
    ("HAVE_SQRTF", "sqrtf(1.0f)"),
    ("HAVE_SQRTL", "sqrtl(1.0L)"),
    ("HAVE_TANF", "tanf(0.0f)"),
    ("HAVE_TANHF", "tanhf(0.0f)"),
    ("HAVE_TANHL", "tanhl(0.0L)"),
    ("HAVE_TANL", "tanl(0.0L)"),
]

_MATH_SOURCE_PREFIX = """
#define _GNU_SOURCE 1
#include <math.h>
int i;
double d;
float f;
long double ld;
double sd;
double cd;
float sf;
float cf;
long double sld;
long double cld;
"""

def gcc_check_math_support():
    return [
        link_check(
            name = name,
            link_flags = MATH_LINK_FLAGS,
            source = _MATH_SOURCE_PREFIX + """
int main() {{
    {expression};
    return 0;
}}
""".format(expression = expression),
        )
        for name, expression in _MATH_FUNCTIONS
    ]

def gcc_check_stdlib_support(gcc_version):
    checks = [
        function_link_check("HAVE_POSIX_MEMALIGN", "stdlib.h", "void *p = 0; posix_memalign(&p, 16, 16)"),
        function_link_check("HAVE_MEMALIGN", "malloc.h", "void *p = memalign(16, 16)"),
        function_link_check("HAVE__ALIGNED_MALLOC", "malloc.h", "void *p = _aligned_malloc(16, 16)"),
        function_link_check("HAVE_ALIGNED_ALLOC", "stdlib.h", "void *p = aligned_alloc(16, 16)"),
        function_link_check("HAVE_AT_QUICK_EXIT", "stdlib.h", "at_quick_exit((void (*)(void))0)"),
        function_link_check("HAVE_QUICK_EXIT", "stdlib.h", "quick_exit(0)"),
        function_link_check("HAVE_SETENV", "stdlib.h", 'setenv("A", "B", 1)'),
        function_link_check("HAVE_STRTOF", "stdlib.h", 'float f = strtof("1", (char **)0)'),
        function_link_check("HAVE_STRTOLD", "stdlib.h", 'long double ld = strtold("1", (char **)0)'),
    ]
    if libstdcxx_has_stdlib_timespec_get_check(gcc_version):
        checks.append(function_link_check("HAVE_TIMESPEC_GET", "time.h", "struct timespec ts; timespec_get(&ts, TIME_UTC)"))
    if libstdcxx_has_stdlib_secure_getenv_check(gcc_version):
        checks.append(function_link_check("HAVE_SECURE_GETENV", "stdlib.h", 'char *p = secure_getenv("PATH")'))
    return checks
