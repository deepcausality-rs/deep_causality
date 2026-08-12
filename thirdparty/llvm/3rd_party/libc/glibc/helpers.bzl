# A good starting point for this is https://codeberg.org/ziglang/zig/src/branch/master/src/libs/glibc.zig add_include_dirs function
def glibc_includes(cpu):
    os_abi_variant = []
    abi_variant = []
    arch_parent = []
    wordsize = "wordsize-64"

    if cpu == "x86_64":
        os_abi_variant = [
            "sysdeps/unix/sysv/linux/x86_64/64",
        ]

        # x86_64 inherits from x86 in glibc's sysdeps Implies hierarchy.
        arch_parent = [
            "sysdeps/unix/sysv/linux/x86",
            "sysdeps/x86",
        ]
    elif cpu == "riscv64":
        os_abi_variant = [
            "sysdeps/unix/sysv/linux/riscv/rv64",
        ]
        cpu = "riscv"
    elif cpu == "s390x":
        # s390x uses s390/s390-64 subdirectories in glibc's sysdeps hierarchy.
        os_abi_variant = [
            "sysdeps/unix/sysv/linux/s390/s390-64",
        ]
        abi_variant = [
            "sysdeps/s390/s390-64",
        ]
        cpu = "s390"
    elif cpu == "armv7":
        # 32-bit ARM; glibc's sysdeps hierarchy uses "arm" with wordsize-32.
        wordsize = "wordsize-32"
        cpu = "arm"

    return [
        "include",
    ] + os_abi_variant + abi_variant + [
        "sysdeps/unix/sysv/linux/{}".format(cpu),
    ] + arch_parent + [
        "sysdeps/{}/nptl".format(cpu),
        "sysdeps/unix/sysv/linux/generic",
        "sysdeps/unix/sysv/linux/include",
        "sysdeps/unix/sysv/linux",
        "sysdeps/nptl",
        "sysdeps/pthread",
        "sysdeps/unix/sysv",
        "sysdeps/unix/{}".format(cpu),
        "sysdeps/unix",
        "sysdeps/{}".format(cpu),
        "sysdeps/{}".format(wordsize),
        "sysdeps/generic",
        ".",
    ]

# hdrs = glob([
#     "lib/libc/glibc/**/*.h",
# ], exclude = [
#     "lib/libc/glibc/sysdeps/**",
#     "lib/libc/glibc/include/**",
# ]) + glob([
#     "lib/libc/glibc/include/*.h",
#     "lib/libc/glibc/include/*.h",
# ])
# + glob(
#     [
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/x86_64/**",
#         "lib/libc/glibc/sysdeps/x86_64/**",
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/generic/**",
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/include/**",
#     ],
#     allow_empty = True
# ) + glob(
#     [
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/*",
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/bits/**",
#         "lib/libc/glibc/sysdeps/unix/sysv/linux/sys/**",
#     ],
#     allow_empty = True
# )
# + glob([
#         # "lib/libc/glibc/sysdeps/nptl/**",
#         "lib/libc/glibc/sysdeps/pthread/**",
#         "lib/libc/glibc/sysdeps/unix/x86_64/**",
#         # "lib/libc/glibc/sysdeps/x86_64/**",
#         "lib/libc/glibc/sysdeps/generic/**",
#     ],
#     allow_empty = True
# ),
