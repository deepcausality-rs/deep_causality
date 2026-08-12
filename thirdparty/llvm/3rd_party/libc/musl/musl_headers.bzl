load("@bazel_skylib//rules:expand_template.bzl", "expand_template")
load("@bazel_skylib//rules:write_file.bzl", "write_file")

_TYPEDEFS = [
    ("_Addr intptr_t", "intptr_t"),
    ("_Addr ptrdiff_t", "ptrdiff_t"),
    ("_Addr regoff_t", "regoff_t"),
    ("_Addr ssize_t", "ssize_t"),
    ("_Int64 blkcnt_t", "blkcnt_t"),
    ("_Int64 off_t", "off_t"),
    ("_Int64 suseconds_t", "suseconds_t"),
    ("_Int64 time_t", "time_t"),
    ("_Reg register_t", "register_t"),
    ("__builtin_va_list __isoc_va_list", "__isoc_va_list"),
    ("__builtin_va_list va_list", "va_list"),
    ("double double_t", "double_t"),
    ("double float_t", "float_t"),
    ("float float_t", "float_t"),
    ("int blksize_t", "blksize_t"),
    ("int clockid_t", "clockid_t"),
    ("int key_t", "key_t"),
    ("int pid_t", "pid_t"),
    ("int pthread_once_t", "pthread_once_t"),
    ("int pthread_spinlock_t", "pthread_spinlock_t"),
    ("int wchar_t", "wchar_t"),
    ("long blksize_t", "blksize_t"),
    ("long clock_t", "clock_t"),
    ("long double double_t", "double_t"),
    ("long double float_t", "float_t"),
    ("signed _Int64   int64_t", "int64_t"),
    ("signed _Int64   intmax_t", "intmax_t"),
    ("signed char     int8_t", "int8_t"),
    ("signed int      int32_t", "int32_t"),
    ("signed short    int16_t", "int16_t"),
    ("struct _IO_FILE FILE", "FILE"),
    ("struct __locale_struct * locale_t", "locale_t"),
    ("struct __mbstate_t { unsigned __opaque1, __opaque2; } mbstate_t", "mbstate_t"),
    ("struct __pthread * pthread_t", "pthread_t"),
    ("struct __sigset_t { unsigned long __bits[128/sizeof(long)]; } sigset_t", "sigset_t"),
    ("struct { long long __ll; long double __ld; } max_align_t", "max_align_t"),
    ("struct { union { int __i[12]; volatile int __vi[12]; void *__p[12*sizeof(int)/sizeof(void*)]; } __u; } cnd_t", "cnd_t"),
    ("struct { union { int __i[12]; volatile int __vi[12]; void *__p[12*sizeof(int)/sizeof(void*)]; } __u; } pthread_cond_t", "pthread_cond_t"),
    ("struct { union { int __i[sizeof(long)==8?10:6]; volatile int __vi[sizeof(long)==8?10:6]; volatile void *volatile __p[sizeof(long)==8?5:6]; } __u; } mtx_t", "mtx_t"),
    ("struct { union { int __i[sizeof(long)==8?10:6]; volatile int __vi[sizeof(long)==8?10:6]; volatile void *volatile __p[sizeof(long)==8?5:6]; } __u; } pthread_mutex_t", "pthread_mutex_t"),
    ("struct { union { int __i[sizeof(long)==8?14:8]; volatile int __vi[sizeof(long)==8?14:8]; void *__p[sizeof(long)==8?7:8]; } __u; } pthread_rwlock_t", "pthread_rwlock_t"),
    ("struct { union { int __i[sizeof(long)==8?14:9]; volatile int __vi[sizeof(long)==8?14:9]; unsigned long __s[sizeof(long)==8?7:9]; } __u; } pthread_attr_t", "pthread_attr_t"),
    ("struct { union { int __i[sizeof(long)==8?8:5]; volatile int __vi[sizeof(long)==8?8:5]; void *__p[sizeof(long)==8?4:5]; } __u; } pthread_barrier_t", "pthread_barrier_t"),
    ("struct { unsigned __attr; } pthread_barrierattr_t", "pthread_barrierattr_t"),
    ("struct { unsigned __attr; } pthread_condattr_t", "pthread_condattr_t"),
    ("struct { unsigned __attr; } pthread_mutexattr_t", "pthread_mutexattr_t"),
    ("struct { unsigned __attr[2]; } pthread_rwlockattr_t", "pthread_rwlockattr_t"),
    ("unsigned _Addr size_t", "size_t"),
    ("unsigned _Addr uintptr_t", "uintptr_t"),
    ("unsigned _Int64 dev_t", "dev_t"),
    ("unsigned _Int64 fsblkcnt_t", "fsblkcnt_t"),
    ("unsigned _Int64 fsfilcnt_t", "fsfilcnt_t"),
    ("unsigned _Int64 ino_t", "ino_t"),
    ("unsigned _Int64 u_int64_t", "u_int64_t"),
    ("unsigned _Int64 uint64_t", "uint64_t"),
    ("unsigned _Int64 uintmax_t", "uintmax_t"),
    ("unsigned _Reg nlink_t", "nlink_t"),
    ("unsigned char   uint8_t", "uint8_t"),
    ("unsigned gid_t", "gid_t"),
    ("unsigned id_t", "id_t"),
    ("unsigned int    uint32_t", "uint32_t"),
    ("unsigned int nlink_t", "nlink_t"),
    ("unsigned long pthread_t", "pthread_t"),
    ("unsigned long wctype_t", "wctype_t"),
    ("unsigned mode_t", "mode_t"),
    ("unsigned pthread_key_t", "pthread_key_t"),
    ("unsigned short  uint16_t", "uint16_t"),
    ("unsigned short sa_family_t", "sa_family_t"),
    ("unsigned socklen_t", "socklen_t"),
    ("unsigned uid_t", "uid_t"),
    ("unsigned useconds_t", "useconds_t"),
    ("unsigned wchar_t", "wchar_t"),
    ("unsigned wint_t", "wint_t"),
    ("void * timer_t", "timer_t"),
]

_STRUCTS = [
    ("_IO_FILE", "{ char __x; }"),
    ("iovec", "{ void *iov_base; size_t iov_len; }"),
    ("timespec", "{ time_t tv_sec; int :8*(sizeof(time_t)-sizeof(long))*(__BYTE_ORDER==4321); long tv_nsec; int :8*(sizeof(time_t)-sizeof(long))*(__BYTE_ORDER!=4321); }"),
    ("timeval", "{ time_t tv_sec; suseconds_t tv_usec; }"),
    ("winsize", "{ unsigned short ws_row, ws_col, ws_xpixel, ws_ypixel; }"),
]

def _typedef_replacement(declaration, name):
    return "\n".join([
        "#if defined(__NEED_{name}) && !defined(__DEFINED_{name})".format(name = name),
        "typedef {declaration};".format(declaration = declaration),
        "#define __DEFINED_{name}".format(name = name),
        "#endif",
        "",
    ])

def _struct_replacement(name, body):
    return "\n".join([
        "#if defined(__NEED_struct_{name}) && !defined(__DEFINED_struct_{name})".format(name = name),
        "struct {name} {body};".format(name = name, body = body),
        "#define __DEFINED_struct_{name}".format(name = name),
        "#endif",
        "",
    ])

def _alltypes_substitutions():
    substitutions = {}
    for declaration, name in _TYPEDEFS:
        substitutions["TYPEDEF {declaration};".format(declaration = declaration)] = _typedef_replacement(declaration, name)
    for name, body in _STRUCTS:
        substitutions["STRUCT {name} {body};".format(name = name, body = body)] = _struct_replacement(name, body)
    return substitutions

def musl_alltypes_headers(name, arch, visibility = None):
    arch_name = name + "_arch"
    common_name = name + "_common"
    wrapper_name = name + "_wrapper"
    substitutions = _alltypes_substitutions()

    expand_template(
        name = arch_name,
        out = "obj/%s/include/bits/alltypes_arch.h" % arch,
        substitutions = substitutions,
        template = "arch/%s/bits/alltypes.h.in" % arch,
        visibility = visibility,
    )

    expand_template(
        name = common_name,
        out = "obj/%s/include/bits/alltypes_common.h" % arch,
        substitutions = substitutions,
        template = "include/alltypes.h.in",
        visibility = visibility,
    )

    write_file(
        name = wrapper_name,
        out = "obj/%s/include/bits/alltypes.h" % arch,
        content = [
            "#include <bits/alltypes_arch.h>",
            "#include <bits/alltypes_common.h>",
            "",
        ],
        visibility = visibility,
    )

    return [
        arch_name,
        common_name,
        wrapper_name,
    ]
