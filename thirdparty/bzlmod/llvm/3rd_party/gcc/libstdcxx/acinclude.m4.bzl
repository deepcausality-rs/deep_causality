# Configure-check model ported from GCC's libstdc++-v3/acinclude.m4. Keep
# function names and grouping close to the GLIBCXX_* macros in that file so GCC
# updates can be reviewed by comparing acinclude.m4 against this module.

load(
    "//3rd_party/gcc:version.bzl",
    "libstdcxx_has_alignas_init_priority_checks",
    "libstdcxx_has_arc4random_getentropy_checks",
    "libstdcxx_has_atomic_builtins_define",
    "libstdcxx_has_c99_cxx11_detail_checks",
    "libstdcxx_has_cxx11_no_sleep_define",
    "libstdcxx_has_debugging_checks",
    "libstdcxx_has_decl_strnlen_check",
    "libstdcxx_has_dev_random_policy",
    "libstdcxx_has_filesystem_chdir_chmod_getcwd_mkdir_checks",
    "libstdcxx_has_filesystem_copy_file_range_check",
    "libstdcxx_has_filesystem_dirfd_checks",
    "libstdcxx_has_filesystem_extra_posix_checks",
    "libstdcxx_has_filesystem_openat_check",
    "libstdcxx_has_fseeko_ftello_check",
    "libstdcxx_has_int128_float128_checks",
    "libstdcxx_has_int64_t_checks",
    "libstdcxx_has_networking_o_nonblock_check",
    "libstdcxx_has_no_sleep_policy",
    "libstdcxx_has_posix_semaphore_check",
    "libstdcxx_has_pthread_clock_checks",
    "libstdcxx_has_sockatmark_wfopen_checks",
    "libstdcxx_has_stdio_locking_checks",
    "libstdcxx_has_struct_tm_tm_zone_check",
    "libstdcxx_has_system_error_check",
    "libstdcxx_has_text_encoding_checks",
    "libstdcxx_has_uchar_char8_checks",
    "libstdcxx_has_uselocale_check",
    "libstdcxx_has_x86_rdseed_check",
    "libstdcxx_has_zoneinfo_policy",
)
load(
    "//3rd_party/gcc/libstdcxx/autoconf:checks.bzl",
    "compile_check",
    "function_link_check",
    "link_check",
    "policy_define",
    "policy_string_define",
    "policy_undef",
)
load(
    ":gcc_config_checks.bzl",
    "CXX_NO_EXCEPTIONS_FLAGS",
    "PTHREAD_LINK_FLAGS",
)
load(":linkage.m4.bzl", "MATH_LINK_FLAGS")

CXX_FILESYSTEM_FLAGS = ["-fno-exceptions"]

# Upstream macro coverage anchors for audit. The active Linux GNU port groups
# some acinclude.m4 macros when they share one Bazel probe/policy site:
#
# GLIBCXX_CHECK_MATH_SUPPORT and GLIBCXX_CHECK_STDLIB_SUPPORT are delegated to
# reusable linkage checks in linkage.m4.bzl.
# GLIBCXX_CHECK_MATH_DECL and GLIBCXX_CHECK_MATH_DECLS are currently represented
# by glibcxx_enable_c99() and glibcxx_check_math11_proto().
# GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_3 is currently represented by
# glibcxx_enable_c99() and glibcxx_check_c99_tr1().
# GLIBCXX_CHECK_GET_NPROCS, GLIBCXX_CHECK_SC_NPROCESSORS_ONLN,
# GLIBCXX_CHECK_SC_NPROC_ONLN, and GLIBCXX_CHECK_PTHREADS_NUM_PROCESSORS_NP are
# represented by glibcxx_check_hardware_concurrency().
# GLIBCXX_CHECK_PTHREAD_COND_CLOCKWAIT, GLIBCXX_CHECK_PTHREAD_MUTEX_CLOCKLOCK,
# and GLIBCXX_CHECK_PTHREAD_RWLOCK_CLOCKLOCK are represented by
# glibcxx_check_pthread_clock_apis().
# GLIBCXX_CHECK_SYSTEM_ERROR, GLIBCXX_CHECK_X86_RDRAND, GLIBCXX_CHECK_X86_RDSEED,
# GLIBCXX_CHECK_ALIGNAS_CACHELINE, GLIBCXX_CHECK_INIT_PRIORITY,
# GLIBCXX_STRUCT_TM_TM_ZONE, GLIBCXX_CHECK_POLL, GLIBCXX_CHECK_ARC4RANDOM,
# GLIBCXX_CHECK_GETENTROPY, GLIBCXX_CHECK_DEV_RANDOM, GCC 8
# GLIBCXX_CHECK_RANDOM_TR1, GLIBCXX_CHECK_WRITEV, GLIBCXX_CHECK_S_ISREG_OR_S_IFREG, GLIBCXX_CHECK_SDT_H,
# GLIBCXX_CHECK_LINKER_FEATURES and GLIBCXX_CHECK_EXCEPTION_PTR_SYMVER are
# represented by grouped checks below. GLIBCXX_CHECK_SIZE_T_MANGLING plus the
# compatibility size_t/ptrdiff_t checks from GLIBCXX_ENABLE_SYMVERS are
# target-derived in target_config.bzl.
# GCC 11 and older GLIBCXX_CHECK_INT64_T is represented by
# glibcxx_check_int64_t(); GCC 11 and older GLIBCXX_ENABLE_INT128_FLOAT128 is
# represented by glibcxx_enable_int128_float128() for int128 while float128
# remains a deferred build setting.
# GLIBCXX_ENABLE_EXTERN_TEMPLATE, GLIBCXX_ENABLE_FILESYSTEM_TS,
# GLIBCXX_ENABLE_LIBSTDCXX_DUAL_ABI, GLIBCXX_ENABLE_LIBSTDCXX_VISIBILITY, and
# GLIBCXX_DEFAULT_ABI are represented by glibcxx_abi_policies().

def glibcxx_check_compiler_features():
    return []

def glibcxx_enable_hosted():
    return [policy_define("_GLIBCXX_HOSTED", "__STDC_HOSTED__")]

def glibcxx_enable_verbose():
    return [policy_define("_GLIBCXX_VERBOSE")]

def glibcxx_enable_pch():
    return []

def glibcxx_enable_atomic_builtins(gcc_version):
    if libstdcxx_has_atomic_builtins_define(gcc_version):
        return [policy_define("_GLIBCXX_ATOMIC_BUILTINS")]
    return [policy_define("_GLIBCXX_ATOMIC_WORD_BUILTINS")]

def glibcxx_enable_lock_policy():
    # HAVE_ATOMIC_LOCK_POLICY is target-derived because GCC keeps RISC-V on the
    # mutex ABI even when compare-and-swap builtins are available.
    return []

def glibcxx_enable_decimal_float():
    return [
        compile_check(
            name = "_GLIBCXX_USE_DECIMAL_FLOAT",
            source = """
int main() {
    _Decimal32 d1;
    _Decimal64 d2;
    _Decimal128 d3;
    return 0;
}
""",
        ),
    ]

def glibcxx_enable_int128_float128(gcc_version):
    if not libstdcxx_has_int128_float128_checks(gcc_version):
        return []
    return [
        compile_check(
            name = "_GLIBCXX_USE_INT128",
            language = "c++",
            source = """
template<typename T1, typename T2> struct same { typedef T1 type; };
template<typename T> struct same<T, T> { };
int main() {
    typename same<long, __int128>::type i1 = 0;
    typename same<long long, __int128>::type i2 = 0;
    unsigned __int128 u = 0;
    return (int)(i1 + i2 + u);
}
""",
        ),
    ]

def glibcxx_enable_cstdio():
    return []

def glibcxx_enable_clocale():
    return [
        function_link_check("HAVE_STRERROR_L", "string.h", "char *s = strerror_l(0, (locale_t)0)"),
        function_link_check("HAVE_STRERROR_R", "string.h", "char buf[64]; strerror_r(0, buf, sizeof(buf))"),
        function_link_check("HAVE_STRXFRM_L", "string.h", 'char dst[64]; strxfrm_l(dst, "", sizeof(dst), (locale_t)0)'),
    ]

def glibcxx_enable_allocator():
    return []

def glibcxx_enable_long_long():
    return [policy_define("_GLIBCXX_USE_LONG_LONG")]

def glibcxx_enable_wchar_t():
    return [
        compile_check(
            name = "HAVE_MBSTATE_T",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <wchar.h>
mbstate_t state;
int main() { return sizeof(state); }
""",
        ),
        compile_check(
            name = "_GLIBCXX_USE_WCHAR_T",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <stddef.h>
#include <wchar.h>
#include <wctype.h>
wint_t i;
long l = WEOF;
long j = WCHAR_MIN;
long k = WCHAR_MAX;
namespace test {
    using ::btowc;
    using ::fgetwc;
    using ::fgetws;
    using ::fputwc;
    using ::fputws;
    using ::fwide;
    using ::fwprintf;
    using ::fwscanf;
    using ::getwc;
    using ::getwchar;
    using ::mbrlen;
    using ::mbrtowc;
    using ::mbsinit;
    using ::mbsrtowcs;
    using ::putwc;
    using ::putwchar;
    using ::swprintf;
    using ::swscanf;
    using ::ungetwc;
    using ::vfwprintf;
    using ::vswprintf;
    using ::vwprintf;
    using ::wcrtomb;
    using ::wcscat;
    using ::wcschr;
    using ::wcscmp;
    using ::wcscoll;
    using ::wcscpy;
    using ::wcscspn;
    using ::wcsftime;
    using ::wcslen;
    using ::wcsncat;
    using ::wcsncmp;
    using ::wcsncpy;
    using ::wcspbrk;
    using ::wcsrchr;
    using ::wcsrtombs;
    using ::wcsspn;
    using ::wcsstr;
    using ::wcstod;
    using ::wcstok;
    using ::wcstol;
    using ::wcstoul;
    using ::wcsxfrm;
    using ::wctob;
    using ::wmemchr;
    using ::wmemcmp;
    using ::wmemcpy;
    using ::wmemmove;
    using ::wmemset;
    using ::wprintf;
    using ::wscanf;
}
int main() { return 0; }
""",
        ),
    ]

_C99_MATH_GENERIC_BODY = """
volatile double d1, d2;
volatile int i;
i = fpclassify(d1);
i = isfinite(d1);
i = isinf(d1);
i = isnan(d1);
i = isnormal(d1);
i = signbit(d1);
i = isgreater(d1, d2);
i = isgreaterequal(d1, d2);
i = isless(d1, d2);
i = islessequal(d1, d2);
i = islessgreater(d1, d2);
i = islessgreater(d1, d2);
i = isunordered(d1, d2);
"""

_C99_COMPLEX_BODY = """
typedef __complex__ float float_type;
typedef __complex__ double double_type;
typedef __complex__ long double ld_type;
volatile float_type tmpf;
volatile double_type tmpd;
volatile ld_type tmpld;
volatile float f;
volatile double d;
volatile long double ld;
f = cabsf(tmpf);
f = cargf(tmpf);
tmpf = ccosf(tmpf);
tmpf = ccoshf(tmpf);
tmpf = cexpf(tmpf);
tmpf = clogf(tmpf);
tmpf = csinf(tmpf);
tmpf = csinhf(tmpf);
tmpf = csqrtf(tmpf);
tmpf = ctanf(tmpf);
tmpf = ctanhf(tmpf);
tmpf = cpowf(tmpf, tmpf);
tmpf = cprojf(tmpf);
d = cabs(tmpd);
d = carg(tmpd);
tmpd = ccos(tmpd);
tmpd = ccosh(tmpd);
tmpd = cexp(tmpd);
tmpd = clog(tmpd);
tmpd = csin(tmpd);
tmpd = csinh(tmpd);
tmpd = csqrt(tmpd);
tmpd = ctan(tmpd);
tmpd = ctanh(tmpd);
tmpd = cpow(tmpd, tmpd);
tmpd = cproj(tmpd);
ld = cabsl(tmpld);
ld = cargl(tmpld);
tmpld = ccosl(tmpld);
tmpld = ccoshl(tmpld);
tmpld = cexpl(tmpld);
tmpld = clogl(tmpld);
tmpld = csinl(tmpld);
tmpld = csinhl(tmpld);
tmpld = csqrtl(tmpld);
tmpld = ctanl(tmpld);
tmpld = ctanhl(tmpld);
tmpld = cpowl(tmpld, tmpld);
tmpld = cprojl(tmpld);
"""

_C99_COMPLEX_ARC_BODY = """
typedef __complex__ float float_type;
float_type tmpf;
cacosf(tmpf);
casinf(tmpf);
catanf(tmpf);
cacoshf(tmpf);
casinhf(tmpf);
catanhf(tmpf);
typedef __complex__ double double_type;
double_type tmpd;
cacos(tmpd);
casin(tmpd);
catan(tmpd);
cacosh(tmpd);
casinh(tmpd);
catanh(tmpd);
typedef __complex__ long double ld_type;
ld_type tmpld;
cacosl(tmpld);
casinl(tmpld);
catanl(tmpld);
cacoshl(tmpld);
casinhl(tmpld);
catanhl(tmpld);
"""

_C99_STDIO_BODY = """
va_list args;
va_start(args, fmt);
vfscanf(stderr, "%i", args);
vscanf("%i", args);
vsnprintf(fmt, 0, "%i", args);
vsscanf(fmt, "%i", args);
snprintf(fmt, 0, "%i", 1);
va_end(args);
"""

_C99_STDLIB_BODY = """
volatile float f;
volatile long double ld;
volatile unsigned long long ll;
lldiv_t mydivt;
char *tmp;
f = strtof("gnu", &tmp);
ld = strtold("gnu", &tmp);
ll = strtoll("gnu", &tmp, 10);
ll = strtoull("gnu", &tmp, 10);
ll = llabs(10);
mydivt = lldiv(10, 1);
ll = mydivt.quot;
ll = mydivt.rem;
ll = atoll("10");
_Exit(0);
"""

_C99_WCHAR_BODY = """
wchar_t *endptr;
long double ld = wcstold(L"gnu", &endptr);
long long ll = wcstoll(L"10", &endptr, 10);
unsigned long long ull = wcstoull(L"10", &endptr, 10);
"""

_C99_STDINT_BODY = """
typedef int8_t my_int8_t;
my_int8_t i8 = INT8_MIN;
i8 = INT8_MAX;
typedef int16_t my_int16_t;
my_int16_t i16 = INT16_MIN;
i16 = INT16_MAX;
typedef int32_t my_int32_t;
my_int32_t i32 = INT32_MIN;
i32 = INT32_MAX;
typedef int64_t my_int64_t;
my_int64_t i64 = INT64_MIN;
i64 = INT64_MAX;
typedef int_fast8_t my_int_fast8_t;
my_int_fast8_t if8 = INT_FAST8_MIN;
if8 = INT_FAST8_MAX;
typedef int_fast16_t my_int_fast16_t;
my_int_fast16_t if16 = INT_FAST16_MIN;
if16 = INT_FAST16_MAX;
typedef int_fast32_t my_int_fast32_t;
my_int_fast32_t if32 = INT_FAST32_MIN;
if32 = INT_FAST32_MAX;
typedef int_fast64_t my_int_fast64_t;
my_int_fast64_t if64 = INT_FAST64_MIN;
if64 = INT_FAST64_MAX;
typedef int_least8_t my_int_least8_t;
my_int_least8_t il8 = INT_LEAST8_MIN;
il8 = INT_LEAST8_MAX;
typedef int_least16_t my_int_least16_t;
my_int_least16_t il16 = INT_LEAST16_MIN;
il16 = INT_LEAST16_MAX;
typedef int_least32_t my_int_least32_t;
my_int_least32_t il32 = INT_LEAST32_MIN;
il32 = INT_LEAST32_MAX;
typedef int_least64_t my_int_least64_t;
my_int_least64_t il64 = INT_LEAST64_MIN;
il64 = INT_LEAST64_MAX;
typedef intmax_t my_intmax_t;
my_intmax_t im = INTMAX_MAX;
im = INTMAX_MIN;
typedef intptr_t my_intptr_t;
my_intptr_t ip = INTPTR_MAX;
ip = INTPTR_MIN;
typedef uint8_t my_uint8_t;
my_uint8_t ui8 = UINT8_MAX;
ui8 = UINT8_MAX;
typedef uint16_t my_uint16_t;
my_uint16_t ui16 = UINT16_MAX;
ui16 = UINT16_MAX;
typedef uint32_t my_uint32_t;
my_uint32_t ui32 = UINT32_MAX;
ui32 = UINT32_MAX;
typedef uint64_t my_uint64_t;
my_uint64_t ui64 = UINT64_MAX;
ui64 = UINT64_MAX;
typedef uint_fast8_t my_uint_fast8_t;
my_uint_fast8_t uif8 = UINT_FAST8_MAX;
uif8 = UINT_FAST8_MAX;
typedef uint_fast16_t my_uint_fast16_t;
my_uint_fast16_t uif16 = UINT_FAST16_MAX;
uif16 = UINT_FAST16_MAX;
typedef uint_fast32_t my_uint_fast32_t;
my_uint_fast32_t uif32 = UINT_FAST32_MAX;
uif32 = UINT_FAST32_MAX;
typedef uint_fast64_t my_uint_fast64_t;
my_uint_fast64_t uif64 = UINT_FAST64_MAX;
uif64 = UINT_FAST64_MAX;
typedef uint_least8_t my_uint_least8_t;
my_uint_least8_t uil8 = UINT_LEAST8_MAX;
uil8 = UINT_LEAST8_MAX;
typedef uint_least16_t my_uint_least16_t;
my_uint_least16_t uil16 = UINT_LEAST16_MAX;
uil16 = UINT_LEAST16_MAX;
typedef uint_least32_t my_uint_least32_t;
my_uint_least32_t uil32 = UINT_LEAST32_MAX;
uil32 = UINT_LEAST32_MAX;
typedef uint_least64_t my_uint_least64_t;
my_uint_least64_t uil64 = UINT_LEAST64_MAX;
uil64 = UINT_LEAST64_MAX;
typedef uintmax_t my_uintmax_t;
my_uintmax_t uim = UINTMAX_MAX;
uim = UINTMAX_MAX;
typedef uintptr_t my_uintptr_t;
my_uintptr_t uip = UINTPTR_MAX;
uip = UINTPTR_MAX;
"""

_C99_MATH_FUNCS_BODY = """
acosh(0.0);
acoshf(0.0f);
acoshl(0.0l);
asinh(0.0);
asinhf(0.0f);
asinhl(0.0l);
atanh(0.0);
atanhf(0.0f);
atanhl(0.0l);
exp2(0.0);
exp2f(0.0f);
exp2l(0.0l);
expm1(0.0);
expm1f(0.0f);
expm1l(0.0l);
ilogb(0.0);
ilogbf(0.0f);
ilogbl(0.0l);
log1p(0.0);
log1pf(0.0f);
log1pl(0.0l);
log2(0.0);
log2f(0.0f);
log2l(0.0l);
logb(0.0);
logbf(0.0f);
logbl(0.0l);
scalbln(0.0, 0l);
scalblnf(0.0f, 0l);
scalblnl(0.0l, 0l);
scalbn(0.0, 0);
scalbnf(0.0f, 0);
scalbnl(0.0l, 0);
cbrt(0.0);
cbrtf(0.0f);
cbrtl(0.0l);
hypot(0.0, 0.0);
hypotf(0.0f, 0.0f);
hypotl(0.0l, 0.0l);
erf(0.0);
erff(0.0f);
erfl(0.0l);
erfc(0.0);
erfcf(0.0f);
erfcl(0.0l);
lgamma(0.0);
lgammaf(0.0f);
lgammal(0.0l);
tgamma(0.0);
tgammaf(0.0f);
tgammal(0.0l);
nearbyint(0.0);
nearbyintf(0.0f);
nearbyintl(0.0l);
rint(0.0);
rintf(0.0f);
rintl(0.0l);
round(0.0);
roundf(0.0f);
roundl(0.0l);
lrint(0.0);
lrintf(0.0f);
lrintl(0.0l);
lround(0.0);
lroundf(0.0f);
lroundl(0.0l);
llrint(0.0);
llrintf(0.0f);
llrintl(0.0l);
llround(0.0);
llroundf(0.0f);
llroundl(0.0l);
trunc(0.0);
truncf(0.0f);
truncl(0.0l);
remainder(0.0, 0.0);
remainderf(0.0f, 0.0f);
remainderl(0.0l, 0.0l);
remquo(0.0, 0.0, 0);
remquof(0.0f, 0.0f, 0);
remquol(0.0l, 0.0l, 0);
copysign(0.0, 0.0);
copysignf(0.0f, 0.0f);
copysignl(0.0l, 0.0l);
nan("");
nanf("");
nanl("");
nextafter(0.0, 0.0);
nextafterf(0.0f, 0.0f);
nextafterl(0.0l, 0.0l);
nexttoward(0.0, 0.0);
nexttowardf(0.0f, 0.0f);
nexttowardl(0.0l, 0.0l);
fdim(0.0, 0.0);
fdimf(0.0f, 0.0f);
fdiml(0.0l, 0.0l);
fmax(0.0, 0.0);
fmaxf(0.0f, 0.0f);
fmaxl(0.0l, 0.0l);
fmin(0.0, 0.0);
fminf(0.0f, 0.0f);
fminl(0.0l, 0.0l);
fma(0.0, 0.0, 0.0);
fmaf(0.0f, 0.0f, 0.0f);
fmal(0.0l, 0.0l, 0.0l);
"""

_C99_FENV_BODY = """
int except, mode;
fexcept_t *pflag;
fenv_t *penv;
int ret;
ret = feclearexcept(except);
ret = fegetexceptflag(pflag, except);
ret = feraiseexcept(except);
ret = fesetexceptflag(pflag, except);
ret = fetestexcept(except);
ret = fegetround();
ret = fesetround(mode);
ret = fegetenv(penv);
ret = feholdexcept(penv);
ret = fesetenv(penv);
ret = feupdateenv(penv);
"""

_C99_INTTYPES_BODY = """
intmax_t i, numer, denom, base;
const char *s;
char **endptr;
intmax_t ret = imaxabs(i);
imaxdiv_t dret = imaxdiv(numer, denom);
ret = strtoimax(s, endptr, base);
uintmax_t uret = strtoumax(s, endptr, base);
"""

_C99_INTTYPES_WCHAR_BODY = """
intmax_t base;
const wchar_t *s;
wchar_t **endptr;
intmax_t ret = wcstoimax(s, endptr, base);
uintmax_t uret = wcstoumax(s, endptr, base);
"""

def _scoped(body):
    return "{\n" + body + "\n}\n"

def _compile_body_check(name, header, body, standard):
    return compile_check(
        name = name,
        language = "c++",
        flags = ["-std=" + standard, "-nostdinc++"],
        source = """
#include <{header}>
int main() {{
{body}
    return 0;
}}
""".format(header = header, body = body),
    )

def _stdint_compile_check(name, standard):
    return compile_check(
        name = name,
        language = "c++",
        flags = ["-std=" + standard, "-nostdinc++"],
        source = """
#define __STDC_LIMIT_MACROS
#define __STDC_CONSTANT_MACROS
#include <stdint.h>
int main() {{
{body}
    return 0;
}}
""".format(body = _C99_STDINT_BODY),
    )

def _link_body_check(name, headers, body, standard, link_flags = []):
    includes = "\n".join(["#include <{}>".format(header) for header in headers])
    return link_check(
        name = name,
        language = "c++",
        compile_flags = ["-std=" + standard, "-fno-exceptions", "-nostdinc++"],
        link_flags = link_flags,
        source = """
{includes}
void test(char *fmt, ...) {{
{body}
}}
int main() {{
    char fmt[16] = {{0}};
    test(fmt);
    return 0;
}}
""".format(includes = includes, body = body),
    )

def glibcxx_enable_c99(gcc_version):
    checks = [
        _link_body_check("_GLIBCXX98_USE_C99_MATH", ["math.h"], _C99_MATH_GENERIC_BODY, "c++98", MATH_LINK_FLAGS),
        _link_body_check("_GLIBCXX98_USE_C99_COMPLEX", ["complex.h"], _C99_COMPLEX_BODY, "c++98", MATH_LINK_FLAGS),
        _link_body_check("_GLIBCXX98_USE_C99_STDIO", ["stdarg.h", "stdio.h"], _C99_STDIO_BODY, "c++98"),
        _link_body_check("_GLIBCXX98_USE_C99_STDLIB", ["stdlib.h"], _C99_STDLIB_BODY, "c++98"),
        _compile_body_check("_GLIBCXX98_USE_C99_WCHAR", "wchar.h", _C99_WCHAR_BODY, "c++98"),
        _link_body_check("_GLIBCXX_USE_C99", ["complex.h", "math.h", "stdarg.h", "stdio.h", "stdlib.h", "wchar.h", "wctype.h"], _scoped(_C99_MATH_GENERIC_BODY) + _scoped(_C99_COMPLEX_BODY) + _scoped(_C99_STDIO_BODY) + _scoped(_C99_STDLIB_BODY) + _scoped(_C99_WCHAR_BODY), "c++98", MATH_LINK_FLAGS),
    ]
    if libstdcxx_has_c99_cxx11_detail_checks(gcc_version):
        checks.extend([
            _stdint_compile_check("_GLIBCXX_USE_C99_STDINT", "c++11"),
            _compile_body_check("_GLIBCXX_USE_C99_INTTYPES", "inttypes.h", _C99_INTTYPES_BODY, "c++11"),
            _compile_body_check("_GLIBCXX_USE_C99_INTTYPES_WCHAR_T", "inttypes.h", _C99_INTTYPES_WCHAR_BODY, "c++11"),
        ])
    checks.append(_link_body_check("_GLIBCXX11_USE_C99_MATH", ["math.h"], _C99_MATH_GENERIC_BODY, "c++11", MATH_LINK_FLAGS))
    if libstdcxx_has_c99_cxx11_detail_checks(gcc_version):
        checks.extend([
            compile_check(
                name = "HAVE_C99_FLT_EVAL_TYPES",
                language = "c++",
                flags = ["-std=c++11", "-nostdinc++"],
                source = """
#include <math.h>
float_t f;
double_t d;
int main() { return sizeof(f) == sizeof(d); }
""",
            ),
            _compile_body_check("_GLIBCXX_USE_C99_MATH_FUNCS", "math.h", _C99_MATH_FUNCS_BODY, "c++11"),
        ])
    checks.extend([
        _link_body_check("_GLIBCXX11_USE_C99_COMPLEX", ["complex.h"], _C99_COMPLEX_BODY, "c++11", MATH_LINK_FLAGS),
    ])
    if libstdcxx_has_c99_cxx11_detail_checks(gcc_version):
        checks.append(_compile_body_check("_GLIBCXX_USE_C99_COMPLEX_ARC", "complex.h", _C99_COMPLEX_ARC_BODY, "c++11"))
    checks.extend([
        _link_body_check("_GLIBCXX11_USE_C99_STDIO", ["stdarg.h", "stdio.h"], _C99_STDIO_BODY, "c++11"),
        _link_body_check("_GLIBCXX11_USE_C99_STDLIB", ["stdlib.h"], _C99_STDLIB_BODY, "c++11"),
        _compile_body_check("_GLIBCXX11_USE_C99_WCHAR", "wchar.h", _C99_WCHAR_BODY, "c++11"),
        compile_check(
            name = "HAVE_ISWBLANK",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <wctype.h>
int main() { return iswblank(L' '); }
""",
        ),
        link_check(
            name = "HAVE_VFWSCANF",
            source = """
#include <stdarg.h>
#include <stdio.h>
#include <wchar.h>
int test(const wchar_t *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int result = vfwscanf((FILE *)0, fmt, ap);
    va_end(ap);
    return result;
}
int main() { return 0; }
""",
        ),
        link_check(
            name = "HAVE_VSWSCANF",
            source = """
#include <stdarg.h>
#include <wchar.h>
int test(const wchar_t *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int result = vswscanf(L"", fmt, ap);
    va_end(ap);
    return result;
}
int main() { return 0; }
""",
        ),
        link_check(
            name = "HAVE_VWSCANF",
            source = """
#include <stdarg.h>
#include <wchar.h>
int test(const wchar_t *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    int result = vwscanf(fmt, ap);
    va_end(ap);
    return result;
}
int main() { return 0; }
""",
        ),
        function_link_check("HAVE_WCSTOF", "wchar.h", "float f = wcstof(L\"1\", (wchar_t **)0)"),
        policy_undef("_GLIBCXX_NO_C99_ROUNDING_FUNCS"),
    ])
    if libstdcxx_has_c99_cxx11_detail_checks(gcc_version):
        checks.extend([
            _compile_body_check("_GLIBCXX_USE_C99_CTYPE", "ctype.h", "int ch;\nint ret;\nret = isblank(ch);", "c++11"),
            _compile_body_check("_GLIBCXX_USE_C99_FENV", "fenv.h", _C99_FENV_BODY, "c++11"),
        ])
    return checks

def glibcxx_check_c99_tr1():
    return [
        _compile_body_check("_GLIBCXX_USE_C99_COMPLEX_TR1", "complex.h", _C99_COMPLEX_ARC_BODY, "c++98"),
        _compile_body_check("_GLIBCXX_USE_C99_CTYPE_TR1", "ctype.h", "int ch;\nint ret;\nret = isblank(ch);", "c++98"),
        _compile_body_check("_GLIBCXX_USE_C99_FENV_TR1", "fenv.h", _C99_FENV_BODY, "c++98"),
        _stdint_compile_check("_GLIBCXX_USE_C99_STDINT_TR1", "c++98"),
        _compile_body_check("_GLIBCXX_USE_C99_MATH_TR1", "math.h", "typedef double_t my_double_t;\ntypedef float_t my_float_t;\n" + _C99_MATH_FUNCS_BODY, "c++98"),
        _compile_body_check("_GLIBCXX_USE_C99_INTTYPES_TR1", "inttypes.h", _C99_INTTYPES_BODY, "c++98"),
        _compile_body_check("_GLIBCXX_USE_C99_INTTYPES_WCHAR_T_TR1", "inttypes.h", _C99_INTTYPES_WCHAR_BODY, "c++98"),
    ]

def glibcxx_check_uchar_h(gcc_version):
    checks = [
        compile_check(
            name = "_GLIBCXX_USE_C11_UCHAR_CXX11",
            language = "c++",
            flags = ["-std=c++11"],
            source = """
#include <uchar.h>
#ifdef __STDC_UTF_16__
long i = __STDC_UTF_16__;
#endif
#ifdef __STDC_UTF_32__
long j = __STDC_UTF_32__;
#endif
namespace test {
    using ::c16rtomb;
    using ::c32rtomb;
    using ::mbrtoc16;
    using ::mbrtoc32;
}
int main() { return 0; }
""",
        ),
    ]
    if libstdcxx_has_uchar_char8_checks(gcc_version):
        checks.extend([
            compile_check(
                name = "_GLIBCXX_USE_UCHAR_C8RTOMB_MBRTOC8_FCHAR8_T",
                language = "c++",
                flags = ["-std=c++11", "-fchar8_t"],
                source = """
#include <uchar.h>
namespace test {
    using ::c8rtomb;
    using ::mbrtoc8;
}
int main() { return 0; }
""",
            ),
            compile_check(
                name = "_GLIBCXX_USE_UCHAR_C8RTOMB_MBRTOC8_CXX20",
                language = "c++",
                flags = ["-std=c++20"],
                source = """
#include <uchar.h>
namespace test {
    using ::c8rtomb;
    using ::mbrtoc8;
}
int main() { return 0; }
""",
            ),
        ])
    return checks

def glibcxx_check_int64_t(gcc_version):
    if not libstdcxx_has_int64_t_checks(gcc_version):
        return []
    return [
        compile_check(
            name = "HAVE_INT64_T",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <stdint.h>
int main() {
    int64_t value = 0;
    return (int)value;
}
""",
        ),
        compile_check(
            name = "HAVE_INT64_T_LONG",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <stdint.h>
template<typename T1, typename T2> struct same { enum { value = -1 }; };
template<typename T> struct same<T, T> { enum { value = 1 }; };
int check[same<int64_t, long>::value];
int main() { return check[0]; }
""",
        ),
        compile_check(
            name = "HAVE_INT64_T_LONG_LONG",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <stdint.h>
template<typename T1, typename T2> struct same { enum { value = -1 }; };
template<typename T> struct same<T, T> { enum { value = 1 }; };
int check[same<int64_t, long long>::value];
int main() { return check[0]; }
""",
        ),
    ]

def glibcxx_check_lfs(gcc_version):
    checks = [
        link_check(
            name = "_GLIBCXX_USE_LFS",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#define _LARGEFILE64_SOURCE 1
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
int main() {
    FILE *f = fopen64("", "r");
    off64_t off = 0;
    off = fseeko64(f, off, SEEK_SET);
    off += ftello64(f);
    off += lseek64(0, off, SEEK_SET);
    struct stat64 st;
    return stat64("", &st) + fstat64(0, &st) + off;
}
""",
        ),
    ]
    if libstdcxx_has_fseeko_ftello_check(gcc_version):
        checks.append(function_link_check("_GLIBCXX_USE_FSEEKO_FTELLO", "stdio.h", "fseeko((FILE *)0, 0, SEEK_SET); ftello((FILE *)0)"))
    return checks

def glibcxx_check_gettimeofday():
    return [
        link_check(
            name = "_GLIBCXX_USE_GETTIMEOFDAY",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <sys/time.h>
int main() {
    timeval tv;
    gettimeofday(&tv, 0);
    return 0;
}
""",
        ),
    ]

def glibcxx_enable_libstdcxx_time(gcc_version):
    checks = [
        link_check(
            name = "_GLIBCXX_USE_CLOCK_MONOTONIC",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <time.h>
int main() {
    timespec tp;
    clock_gettime(CLOCK_MONOTONIC, &tp);
    return 0;
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_CLOCK_REALTIME",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <time.h>
int main() {
    timespec tp;
    clock_gettime(CLOCK_REALTIME, &tp);
    return 0;
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_NANOSLEEP",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <time.h>
int main() {
    timespec tp;
    nanosleep(&tp, 0);
    return 0;
}
""",
        ),
        function_link_check("_GLIBCXX_USE_SCHED_YIELD", "sched.h", "sched_yield()", compile_flags = CXX_NO_EXCEPTIONS_FLAGS),
        policy_undef("_GLIBCXX_USE_CLOCK_GETTIME_SYSCALL"),
        policy_undef("_GLIBCXX_USE_WIN32_SLEEP"),
    ]
    if libstdcxx_has_no_sleep_policy(gcc_version):
        checks.append(policy_undef("_GLIBCXX_NO_SLEEP" if libstdcxx_has_cxx11_no_sleep_define(gcc_version) else "NO_SLEEP"))
    return checks

def glibcxx_check_stdio_proto():
    return [function_link_check("HAVE_GETS", "stdio.h", "char buf[8]; gets(buf)")]

def glibcxx_check_math11_proto():
    return [
        compile_check(
            name = "HAVE_OBSOLETE_ISINF",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <math.h>
#undef isinf
namespace std {
    using ::isinf;
    bool isinf(float);
    bool isinf(long double);
}
using std::isinf;
bool b = isinf(0.0);
int main() { return b; }
""",
        ),
        compile_check(
            name = "HAVE_OBSOLETE_ISNAN",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <math.h>
#undef isnan
namespace std {
    using ::isnan;
    bool isnan(float);
    bool isnan(long double);
}
using std::isnan;
bool b = isnan(0.0);
int main() { return b; }
""",
        ),
    ]

def glibcxx_compute_stdio_integer_constants():
    return [
        policy_define("_GLIBCXX_STDIO_EOF", "-1"),
        policy_define("_GLIBCXX_STDIO_SEEK_CUR", "1"),
        policy_define("_GLIBCXX_STDIO_SEEK_END", "2"),
    ]

def glibcxx_check_tmpnam():
    return [function_link_check("_GLIBCXX_USE_TMPNAM", "stdio.h", "char buf[L_tmpnam]; tmpnam(buf)")]

def glibcxx_check_pthread_clock_apis(gcc_version):
    if not libstdcxx_has_pthread_clock_checks(gcc_version):
        return []
    return [
        link_check(
            name = "_GLIBCXX_USE_PTHREAD_COND_CLOCKWAIT",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            link_flags = PTHREAD_LINK_FLAGS,
            source = """
#include <pthread.h>
#include <time.h>
int main() {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    timespec ts;
    return pthread_cond_clockwait(&cond, &mutex, CLOCK_REALTIME, &ts);
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_PTHREAD_MUTEX_CLOCKLOCK",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            link_flags = PTHREAD_LINK_FLAGS,
            source = """
#include <pthread.h>
#include <time.h>
int main() {
    pthread_mutex_t mutex;
    timespec ts;
    return pthread_mutex_clocklock(&mutex, CLOCK_REALTIME, &ts);
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_PTHREAD_RWLOCK_CLOCKLOCK",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            link_flags = PTHREAD_LINK_FLAGS,
            source = """
#include <pthread.h>
#include <time.h>
int main() {
    pthread_rwlock_t rwl;
    timespec ts;
    int n = pthread_rwlock_clockrdlock(&rwl, CLOCK_REALTIME, &ts);
    int m = pthread_rwlock_clockwrlock(&rwl, CLOCK_REALTIME, &ts);
    return n + m;
}
""",
        ),
    ]

def glibcxx_check_hardware_concurrency():
    return [
        function_link_check("_GLIBCXX_USE_GET_NPROCS", "sys/sysinfo.h", "int n = get_nprocs()", compile_flags = CXX_NO_EXCEPTIONS_FLAGS),
        function_link_check("_GLIBCXX_USE_SC_NPROCESSORS_ONLN", "unistd.h", "int n = sysconf(_SC_NPROCESSORS_ONLN)", compile_flags = CXX_NO_EXCEPTIONS_FLAGS),
        function_link_check("_GLIBCXX_USE_SC_NPROC_ONLN", "unistd.h", "int n = sysconf(_SC_NPROC_ONLN)", compile_flags = CXX_NO_EXCEPTIONS_FLAGS),
        function_link_check("_GLIBCXX_USE_PTHREADS_NUM_PROCESSORS_NP", "pthread.h", "int n = pthread_num_processors_np()", compile_flags = CXX_NO_EXCEPTIONS_FLAGS, link_flags = PTHREAD_LINK_FLAGS),
    ]

def glibcxx_check_gthreads(gcc_version):
    checks = [
        compile_check(
            name = "_GTHREAD_USE_MUTEX_TIMEDLOCK",
            language = "c++",
            flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#define _PTHREADS 1
#include <unistd.h>
int main() {
#if defined(_PTHREADS) && (!defined(_POSIX_TIMEOUTS) || _POSIX_TIMEOUTS <= 0)
#error POSIX mutex timedlock cannot be assumed
#endif
    return 0;
}
""",
        ),
        compile_check(
            name = "_GLIBCXX_HAS_GTHREADS",
            language = "c++",
            flags = CXX_NO_EXCEPTIONS_FLAGS,
            probe_contexts = ["gthreads"],
            source = """
#define _PTHREADS 1
#include <bits/gthr.h>
int main() {
#ifndef __GTHREADS_CXX0X
#error gthreads are missing C++11 support
#endif
    return 0;
}
""",
        ),
        compile_check(
            name = "_GLIBCXX_USE_PTHREAD_RWLOCK_T",
            language = "c++",
            flags = CXX_NO_EXCEPTIONS_FLAGS,
            probe_contexts = ["gthreads"],
            source = """
#define _PTHREADS 1
#include <bits/gthr.h>
int main() {
    pthread_rwlock_t rwl;
    return sizeof(rwl) == 0;
}
""",
        ),
    ]
    if libstdcxx_has_posix_semaphore_check(gcc_version):
        checks.append(
            compile_check(
                name = "HAVE_POSIX_SEMAPHORE",
                language = "c++",
                flags = CXX_NO_EXCEPTIONS_FLAGS,
                source = """
#include <limits.h>
#include <semaphore.h>
#include <unistd.h>
int main() {
#if !defined _POSIX_TIMEOUTS || _POSIX_TIMEOUTS <= 0
#error POSIX timeouts option not supported
#elif !defined _POSIX_SEMAPHORES || _POSIX_SEMAPHORES <= 0
#error POSIX semaphores option not supported
#else
#if defined SEM_VALUE_MAX
    constexpr int sem_value_max = SEM_VALUE_MAX;
#elif defined _POSIX_SEM_VALUE_MAX
    constexpr int sem_value_max = _POSIX_SEM_VALUE_MAX;
#else
#error SEM_VALUE_MAX not available
#endif
    sem_t sem;
    sem_init(&sem, 0, sem_value_max);
    struct timespec ts = {0};
    sem_timedwait(&sem, &ts);
#endif
    return 0;
}
""",
            ),
        )
    return checks

def glibcxx_check_filesystem_deps(gcc_version):
    checks = [
        compile_check(
            name = "HAVE_STRUCT_DIRENT_D_TYPE",
            language = "c++",
            flags = CXX_FILESYSTEM_FLAGS,
            source = """
#include <dirent.h>
int test(dirent *entry) { return entry->d_type; }
int main(void) { return 0; }
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_REALPATH",
            compile_flags = CXX_FILESYSTEM_FLAGS,
            source = """
#include <limits.h>
#include <stdlib.h>
#include <unistd.h>
int main() {
#if _XOPEN_VERSION < 500
#error _XOPEN_VERSION is too old
#elif _XOPEN_VERSION >= 700 || defined(PATH_MAX)
    char *tmp = realpath((const char *)0, (char *)0);
    return tmp != 0;
#else
#error realpath needs PATH_MAX before XSI 700
#endif
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_UTIMENSAT",
            compile_flags = CXX_FILESYSTEM_FLAGS,
            source = """
#include <fcntl.h>
#include <sys/stat.h>
int main() {
    timespec ts[2] = {{0, UTIME_OMIT}, {1, 1}};
    int i = utimensat(AT_FDCWD, "path", ts, 0);
    return i;
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_ST_MTIM",
            compile_flags = CXX_FILESYSTEM_FLAGS,
            source = """
#include <sys/stat.h>
int main() {
    struct stat st;
    return st.st_mtim.tv_nsec;
}
""",
        ),
        function_link_check("_GLIBCXX_USE_FCHMOD", "sys/stat.h", "fchmod(1, S_IWUSR)", compile_flags = CXX_FILESYSTEM_FLAGS),
        link_check(
            name = "_GLIBCXX_USE_FCHMODAT",
            compile_flags = CXX_FILESYSTEM_FLAGS,
            source = """
#include <fcntl.h>
#include <sys/stat.h>
int main() { fchmodat(AT_FDCWD, "", 0, AT_SYMLINK_NOFOLLOW); return 0; }
""",
        ),
        function_link_check("_GLIBCXX_USE_SENDFILE", "sys/sendfile.h", "sendfile(1, 2, (off_t *)0, sizeof 1)", compile_flags = CXX_FILESYSTEM_FLAGS),
    ]
    if libstdcxx_has_filesystem_extra_posix_checks(gcc_version):
        checks.extend([
            link_check(
                name = "_GLIBCXX_USE_UTIME",
                compile_flags = CXX_FILESYSTEM_FLAGS,
                source = """
#include <utime.h>
int main() {
    utimbuf t = {1, 1};
    int i = utime("path", &t);
    return i;
}
""",
            ),
            link_check(
                name = "_GLIBCXX_USE_LSTAT",
                compile_flags = CXX_FILESYSTEM_FLAGS,
                source = """
#include <sys/stat.h>
int main() {
    struct stat st;
    int i = lstat("path", &st);
    return i;
}
""",
            ),
            function_link_check("HAVE_LINK", "unistd.h", 'link("", "")', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("HAVE_READLINK", "unistd.h", 'char buf[32]; readlink("", buf, sizeof(buf))', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("HAVE_SYMLINK", "unistd.h", 'symlink("", "")', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("HAVE_TRUNCATE", "unistd.h", 'truncate("", 99)', compile_flags = CXX_FILESYSTEM_FLAGS),
        ])
    if libstdcxx_has_filesystem_dirfd_checks(gcc_version):
        checks.extend([
            function_link_check("HAVE_FDOPENDIR", "dirent.h", "DIR *dir = fdopendir(1)", compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("HAVE_DIRFD", "dirent.h", "int fd = dirfd((DIR *)0)", compile_flags = CXX_FILESYSTEM_FLAGS),
            link_check(
                name = "HAVE_UNLINKAT",
                compile_flags = CXX_FILESYSTEM_FLAGS,
                source = """
#include <fcntl.h>
#include <unistd.h>
int main() { unlinkat(AT_FDCWD, "", AT_REMOVEDIR); return 0; }
""",
            ),
        ])
    if libstdcxx_has_filesystem_openat_check(gcc_version):
        checks.append(function_link_check("HAVE_OPENAT", "fcntl.h", 'int fd = openat(AT_FDCWD, "", 0)', compile_flags = CXX_FILESYSTEM_FLAGS))
    if libstdcxx_has_filesystem_chdir_chmod_getcwd_mkdir_checks(gcc_version):
        checks.extend([
            function_link_check("_GLIBCXX_USE_CHMOD", "sys/stat.h", 'int i = chmod("", S_IRUSR)', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("_GLIBCXX_USE_MKDIR", "sys/stat.h", 'int i = mkdir("", S_IRUSR)', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("_GLIBCXX_USE_CHDIR", "unistd.h", 'int i = chdir("")', compile_flags = CXX_FILESYSTEM_FLAGS),
            function_link_check("_GLIBCXX_USE_GETCWD", "unistd.h", "char *s = getcwd((char *)0, 1)", compile_flags = CXX_FILESYSTEM_FLAGS),
        ])
    if libstdcxx_has_filesystem_copy_file_range_check(gcc_version):
        checks.extend([
            function_link_check("HAVE_LSEEK", "unistd.h", "lseek(1, 0, SEEK_SET)", compile_flags = CXX_FILESYSTEM_FLAGS),
            link_check(
                name = "_GLIBCXX_USE_COPY_FILE_RANGE",
                compile_flags = CXX_FILESYSTEM_FLAGS,
                source = """
#define _GNU_SOURCE 1
#include <sys/types.h>
#include <unistd.h>
int main() {
    copy_file_range(1, (loff_t *)0, 2, (loff_t *)0, 1, 0);
    return 0;
}
""",
            ),
        ])
    return checks

def glibcxx_check_networking_deps(gcc_version):
    if not libstdcxx_has_networking_o_nonblock_check(gcc_version):
        return []
    return [
        compile_check(
            name = "HAVE_O_NONBLOCK",
            source = """
#include <fcntl.h>
#ifndef O_NONBLOCK
#error O_NONBLOCK is not defined
#endif
int main(void) { return O_NONBLOCK == 0; }
""",
        ),
    ]

def glibcxx_check_text_encoding(gcc_version):
    if not libstdcxx_has_text_encoding_checks(gcc_version):
        return []
    return [
        link_check(
            name = "_GLIBCXX_USE_NL_LANGINFO_L",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <langinfo.h>
#include <locale.h>
int main() {
    locale_t loc = newlocale(LC_ALL_MASK, "", (locale_t)0);
    const char *enc = nl_langinfo_l(CODESET, loc);
    freelocale(loc);
    return enc == 0;
}
""",
        ),
    ]

def glibcxx_check_debugging(gcc_version):
    if not libstdcxx_has_debugging_checks(gcc_version):
        return []
    return [
        link_check(
            name = "_GLIBCXX_USE_PTRACE",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <sys/ptrace.h>
#include <sys/types.h>
int main() { return ptrace(PTRACE_TRACEME, (pid_t)0, 1, 0); }
""",
        ),
        policy_define("_GLIBCXX_USE_PROC_SELF_STATUS"),
    ]

def glibcxx_check_stdio_locking(gcc_version):
    if not libstdcxx_has_stdio_locking_checks(gcc_version):
        return []
    return [
        function_link_check("HAVE_FWRITE_UNLOCKED", "stdio.h", 'fwrite_unlocked("", 1, 1, stdout)'),
        link_check(
            name = "_GLIBCXX_USE_STDIO_LOCKING",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <stdio.h>
int main() {
    FILE *f = fopen("", "");
    flockfile(f);
    putc_unlocked(' ', f);
    funlockfile(f);
    fclose(f);
    return 0;
}
""",
        ),
        link_check(
            name = "_GLIBCXX_USE_GLIBC_STDIO_EXT",
            compile_flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
#include <stdio.h>
#include <stdio_ext.h>
extern "C" {
using f1_type = int (*)(FILE *) noexcept;
using f2_type = size_t (*)(FILE *) noexcept;
}
int main() {
    f1_type twritable = &::__fwritable;
    f1_type tblk = &::__flbf;
    f2_type pbufsize = &::__fbufsize;
    FILE *f = fopen("", "");
    int i = __overflow(f, EOF);
    bool writeable = __fwritable(f);
    bool line_buffered = __flbf(f);
    size_t bufsz = __fbufsize(f);
    char *&pptr = f->_IO_write_ptr;
    char *&epptr = f->_IO_buf_end;
    fflush_unlocked(f);
    fclose(f);
    return i + writeable + line_buffered + bufsz + (pptr == epptr) + (twritable == tblk) + (pbufsize == 0);
}
""",
        ),
    ]

def glibcxx_misc_compile_checks(gcc_version):
    checks = [
        compile_check(
            name = "HAVE_S_IFREG",
            source = """
#include <sys/stat.h>
#ifndef S_IFREG
#error S_IFREG is not defined
#endif
int main(void) { return S_IFREG == 0; }
""",
        ),
        compile_check(
            name = "HAVE_S_ISREG",
            source = """
#include <sys/stat.h>
#ifndef S_ISREG
#error S_ISREG is not defined
#endif
int main(void) { return S_ISREG(0); }
""",
        ),
        compile_check(
            name = "_GLIBCXX_X86_RDRAND",
            language = "c++",
            flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
int main() { unsigned int v; asm("rdrand %eax"); return __builtin_ia32_rdrand32_step(&v); }
""",
        ),
    ]
    if libstdcxx_has_decl_strnlen_check(gcc_version):
        checks.append(compile_check(
            name = "HAVE_DECL_STRNLEN",
            language = "c++",
            flags = ["-nostdinc++"],
            source = """
#include <string.h>
int main() { return strnlen("", 1); }
""",
        ))
    if libstdcxx_has_x86_rdseed_check(gcc_version):
        checks.append(compile_check(
            name = "_GLIBCXX_X86_RDSEED",
            language = "c++",
            flags = CXX_NO_EXCEPTIONS_FLAGS,
            source = """
int main() { unsigned int v; asm("rdseed %eax"); return __builtin_ia32_rdseed_si_step(&v); }
""",
        ))
    if libstdcxx_has_alignas_init_priority_checks(gcc_version):
        checks.extend([
            compile_check(
                name = "_GLIBCXX_CAN_ALIGNAS_DESTRUCTIVE_SIZE",
                language = "c++",
                flags = CXX_NO_EXCEPTIONS_FLAGS,
                source = """
struct alignas(__GCC_DESTRUCTIVE_SIZE) Aligned {};
alignas(Aligned) static char buf[sizeof(Aligned) * 16];
int main() { return sizeof(buf) == 0; }
""",
            ),
            compile_check(
                name = "_GLIBCXX_USE_INIT_PRIORITY_ATTRIBUTE",
                language = "c++",
                flags = CXX_NO_EXCEPTIONS_FLAGS,
                source = """
#if !__has_attribute(init_priority)
#error init_priority not supported
#endif
int main() { return 0; }
""",
            ),
        ])
    if libstdcxx_has_struct_tm_tm_zone_check(gcc_version):
        checks.append(
            compile_check(
                name = "_GLIBCXX_USE_STRUCT_TM_TM_ZONE",
                language = "c++",
                flags = ["-std=c++20"],
                source = """
#include <time.h>
int main() { struct tm t{}; t.tm_zone = (char *)0; return 0; }
""",
            ),
        )
    return checks

def glibcxx_misc_link_checks(gcc_version):
    checks = [
        function_link_check("HAVE_POLL", "poll.h", "struct pollfd pfd; poll(&pfd, 1, 0)"),
        function_link_check("HAVE_LC_MESSAGES", "locale.h", "int i = LC_MESSAGES"),
        link_check(
            name = "HAVE___CXA_THREAD_ATEXIT",
            source = """
extern "C" int __cxa_thread_atexit(void (*)(void *), void *, void *);
int main() { return __cxa_thread_atexit((void (*)(void *))0, (void *)0, (void *)0); }
""",
        ),
        link_check(
            name = "HAVE___CXA_THREAD_ATEXIT_IMPL",
            source = """
extern "C" int __cxa_thread_atexit_impl(void (*)(void *), void *, void *);
int main() { return __cxa_thread_atexit_impl((void (*)(void *))0, (void *)0, (void *)0); }
""",
        ),
        function_link_check("HAVE_SLEEP", "unistd.h", "sleep(0)"),
        function_link_check("HAVE_USLEEP", "unistd.h", "usleep(0)"),
        function_link_check("HAVE_WRITEV", "sys/uio.h", "struct iovec iov; writev(1, &iov, 1)"),
    ]
    if libstdcxx_has_uselocale_check(gcc_version):
        checks.append(function_link_check("HAVE_USELOCALE", "locale.h", "locale_t loc = uselocale((locale_t)0)"))
    if libstdcxx_has_arc4random_getentropy_checks(gcc_version):
        checks.extend([
            function_link_check("HAVE_ARC4RANDOM", "stdlib.h", "unsigned x = arc4random()"),
            function_link_check("HAVE_GETENTROPY", "unistd.h", "char buf[8]; getentropy(buf, sizeof(buf))"),
        ])
    if libstdcxx_has_sockatmark_wfopen_checks(gcc_version):
        checks.append(function_link_check("HAVE_SOCKATMARK", "sys/socket.h", "int i = sockatmark(0)"))
    if libstdcxx_has_sockatmark_wfopen_checks(gcc_version):
        checks.append(function_link_check("HAVE__WFOPEN", "wchar.h", 'FILE *f = _wfopen(L"", L"r")'))
    return checks

def glibcxx_abi_policies():
    return [
        policy_define("_GLIBCXX_USE_DUAL_ABI"),
        policy_define("_GLIBCXX_USE_CXX11_ABI"),
        policy_define("_GLIBCXX_FULLY_DYNAMIC_STRING", "0"),
        policy_undef("_GLIBCXX_CONCEPT_CHECKS"),
    ]

def glibcxx_check_system_error(gcc_version):
    if not libstdcxx_has_system_error_check(gcc_version):
        return []
    return [
        policy_define("HAVE_EBADMSG"),
        policy_define("HAVE_ECANCELED"),
        policy_define("HAVE_ECHILD"),
        policy_define("HAVE_EIDRM"),
        policy_define("HAVE_ENODATA"),
        policy_define("HAVE_ENOLINK"),
        policy_define("HAVE_ENOSPC"),
        policy_define("HAVE_ENOSR"),
        policy_define("HAVE_ENOSTR"),
        policy_define("HAVE_ENOTRECOVERABLE"),
        policy_define("HAVE_ENOTSUP"),
        policy_define("HAVE_EOVERFLOW"),
        policy_define("HAVE_EOWNERDEAD"),
        policy_define("HAVE_EPERM"),
        policy_define("HAVE_EPROTO"),
        policy_define("HAVE_ETIME"),
        policy_define("HAVE_ETIMEDOUT"),
        policy_define("HAVE_ETXTBSY"),
        policy_define("HAVE_EWOULDBLOCK"),
    ]

def glibcxx_random_policy(gcc_version):
    if not libstdcxx_has_dev_random_policy(gcc_version):
        return [
            policy_define("_GLIBCXX_USE_RANDOM_TR1"),
        ]
    return [
        # GLIBCXX_CHECK_DEV_RANDOM uses one host/filesystem decision to define
        # both the C++11 and TR1 random_device feature macros.
        policy_define(
            "_GLIBCXX_USE_DEV_RANDOM",
            defines_on_success = [
                "_GLIBCXX_USE_DEV_RANDOM",
                "_GLIBCXX_USE_RANDOM_TR1",
            ],
        ),
    ]

def glibcxx_zoneinfo_policy(gcc_version):
    if not libstdcxx_has_zoneinfo_policy(gcc_version):
        return []
    return [
        policy_string_define("_GLIBCXX_ZONEINFO_DIR", "/usr/share/zoneinfo"),
        policy_undef("_GLIBCXX_STATIC_TZDATA"),
    ]

def glibcxx_resource_limits_policy():
    return [policy_undef("_GLIBCXX_RES_LIMITS")]
