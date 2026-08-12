# libstdc++ Autoconf Check Definitions

This checklist tracks configure checks that are implemented, deliberately
target-derived, deliberately deferred, not needed, or explicitly out of scope
in the Bazel libstdc++ port. Raw discoveries from `autoconf_inventory.sh`
are input to this file, but they must not be checked off here until the Bazel
port has an implementation or an intentional classification.

Status meanings are defined in `AGENTS.md`.

## Modeled

- [x] `GCC_CHECK_TLS` - native GCC TLS probe, modeled in `gcc_config_checks.bzl`.
- [x] `GCC_CHECK_UNWIND_GETIPINFO` - GCC unwind policy, modeled for Linux GNU.
- [x] `GCC_HEADER_STDINT` - GCC 12-only `gstdint.h` generator, modeled by a
  Bazel-generated compatibility header.
- [x] `GCC_LINUX_FUTEX` - Linux futex probe, modeled as a link probe.
- [x] `GLIBCXX_CHECK_ALIGNAS_CACHELINE` - GCC 13+ cacheline alignment compile probe.
- [x] `GLIBCXX_CHECK_ARC4RANDOM` - GCC 12+ `arc4random` function probe.
- [x] `GLIBCXX_CHECK_C99_TR1` - TR1 C99 support probes.
- [x] `GLIBCXX_CHECK_COMPILER_FEATURES` - compiler feature policy/probe group.
- [x] `GLIBCXX_CHECK_DEBUGGING` - debug support checks.
- [x] `GLIBCXX_CHECK_DEV_RANDOM` - Linux random-device policy.
- [x] `GLIBCXX_CHECK_EXCEPTION_PTR_SYMVER` - exception pointer symbol-version policy.
- [x] `GLIBCXX_CHECK_FILESYSTEM_DEPS` - filesystem dependency probes.
- [x] `GLIBCXX_CHECK_GETENTROPY` - GCC 12+ `getentropy` function probe.
- [x] `GLIBCXX_CHECK_GETTIMEOFDAY` - `gettimeofday` probe.
- [x] `GLIBCXX_CHECK_GET_NPROCS` - GNU `get_nprocs` probe.
- [x] `GLIBCXX_CHECK_GTHREADS` - gthreads capability checks.
- [x] `GLIBCXX_CHECK_INT64_T` - legacy `int64_t` availability and
  typedef-shape probes for `streamoff`.
- [x] `GLIBCXX_CHECK_INIT_PRIORITY` - GCC 13+ init-priority attribute probe.
- [x] `GLIBCXX_CHECK_LFS` - large-file support probe group.
- [x] `GLIBCXX_CHECK_LINKER_FEATURES` - linker/symver capability policy.
- [x] `GLIBCXX_CHECK_MATH_DECL` - math declaration helper, represented by math support probes.
- [x] `GLIBCXX_CHECK_MATH_DECLS` - math declaration helper, represented by math support probes.
- [x] `GLIBCXX_CHECK_MATH_DECL_1` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECL_2` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECL_3` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECLS_AND_LINKAGES_1` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_1` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_2` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_3` - `linkage.m4` math helper.
- [x] `GLIBCXX_CHECK_MATH11_PROTO` - Linux obsolete math probe path modeled; Solaris branch inactive.
- [x] `GLIBCXX_CHECK_MATH_SUPPORT` - native math support probe group.
- [x] `GLIBCXX_CHECK_POLL` - `poll` function probe.
- [x] `GLIBCXX_CHECK_PTHREADS_NUM_PROCESSORS_NP` - pthread processor-count probe.
- [x] `GLIBCXX_CHECK_PTHREAD_COND_CLOCKWAIT` - pthread cond clockwait probe.
- [x] `GLIBCXX_CHECK_PTHREAD_MUTEX_CLOCKLOCK` - pthread mutex clocklock probe.
- [x] `GLIBCXX_CHECK_PTHREAD_RWLOCK_CLOCKLOCK` - pthread rwlock clocklock probe.
- [x] `GLIBCXX_CHECK_RANDOM_TR1` - GCC 8 TR1 random-device policy.
- [x] `GLIBCXX_CHECK_SC_NPROCESSORS_ONLN` - `sysconf` processor-count probe.
- [x] `GLIBCXX_CHECK_SC_NPROC_ONLN` - `sysconf` processor-count probe.
- [x] `GLIBCXX_CHECK_SDT_H` - systemtap header probe.
- [x] `GLIBCXX_CHECK_SIZE_T_MANGLING` - `size_t` ABI mangling policy.
- [x] `GLIBCXX_CHECK_STDIO_LOCKING` - GCC 16+ stdio locking probe.
- [x] `GLIBCXX_CHECK_STDIO_PROTO` - `gets` declaration probe.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_1` - `linkage.m4` stdlib helper.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_2` - `linkage.m4` stdlib helper.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_3` - stdlib helper represented by stdlib support probes.
- [x] `GLIBCXX_CHECK_STDLIB_SUPPORT` - native stdlib support probe group.
- [x] `GLIBCXX_CHECK_SYSTEM_ERROR` - GCC 8 Linux errno availability policy.
- [x] `GLIBCXX_CHECK_S_ISREG_OR_S_IFREG` - `S_ISREG`/`S_IFREG` probe.
- [x] `GLIBCXX_CHECK_TEXT_ENCODING` - GCC 14+ text encoding support probes.
- [x] `GLIBCXX_CHECK_TMPNAM` - `tmpnam` probe.
- [x] `GLIBCXX_CHECK_UCHAR_H` - `uchar.h` support probes; GCC 12+ adds
  the C8 `c8rtomb`/`mbrtoc8` probes.
- [x] `GLIBCXX_CHECK_WRITEV` - `writev` probe.
- [x] `GLIBCXX_CHECK_X86_RDRAND` - x86 RDRAND assembler/builtin probe.
- [x] `GLIBCXX_CHECK_X86_RDSEED` - x86 RDSEED assembler/builtin probe.
- [x] `GLIBCXX_COMPUTE_STDIO_INTEGER_CONSTANTS` - stdio constants policy for Linux GNU.
- [x] `GLIBCXX_DEFAULT_ABI` - default ABI policy.
- [x] `GLIBCXX_ENABLE_ALLOCATOR` - allocator policy.
- [x] `GLIBCXX_ENABLE_ATOMIC_BUILTINS` - atomic builtin policy.
- [x] `GLIBCXX_ENABLE_C99` - C99 support probe groups.
- [x] `GLIBCXX_ENABLE_CLOCALE` - C locale policy/probes.
- [x] `GLIBCXX_ENABLE_CSTDIO` - C stdio policy/probes.
- [x] `GLIBCXX_ENABLE_DECIMAL_FLOAT` - decimal float compile probe.
- [x] `GLIBCXX_ENABLE_EXTERN_TEMPLATE` - extern template policy.
- [x] `GLIBCXX_ENABLE_FILESYSTEM_TS` - filesystem source policy.
- [x] `GLIBCXX_ENABLE_HOSTED` - hosted policy.
- [x] `GLIBCXX_ENABLE_INT128_FLOAT128` - legacy int128 probe; the
  float128 half stays deferred with the existing float128 policy.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_DUAL_ABI` - dual ABI policy.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_TIME` - time support probes. The Win32
  `Sleep` fallback is classified as inactive for supported Linux GNU targets;
  GCC 12 names that fallback `HAVE_WIN32_SLEEP`, while GCC 13+ uses
  `_GLIBCXX_USE_WIN32_SLEEP`.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_VISIBILITY` - visibility policy.
- [x] `GLIBCXX_ENABLE_LONG_LONG` - long long policy.
- [x] `GLIBCXX_ENABLE_SYMVERS` - symbol version policy.
- [x] `GLIBCXX_ENABLE_WCHAR_T` - wide character probes.
- [x] `GLIBCXX_STRUCT_TM_TM_ZONE` - GCC 15+ `struct tm::tm_zone` probe.
- [x] `GLIBCXX_ZONEINFO_DIR` - GCC 13+ zoneinfo policy.

## Concrete Header Checks

These are reviewed `AC_CHECK_HEADERS` arguments from the inventory that are
implemented as `ac_check_headers(...)` entries in `configure.ac.bzl`.

- [x] `arpa/inet.h`
- [x] `complex.h`
- [x] `debugapi.h`
- [x] `dirent.h`
- [x] `dlfcn.h`
- [x] `endian.h`
- [x] `execinfo.h`
- [x] `fcntl.h`
- [x] `fenv.h`
- [x] `float.h`
- [x] `fp.h`
- [x] `ieeefp.h`
- [x] `inttypes.h`
- [x] `libintl.h`
- [x] `link.h`
- [x] `linux/random.h`
- [x] `linux/types.h`
- [x] `locale.h`
- [x] `machine/endian.h`
- [x] `machine/param.h`
- [x] `memory.h`
- [x] `nan.h`
- [x] `netdb.h`
- [x] `netinet/in.h`
- [x] `netinet/tcp.h`
- [x] `poll.h`
- [x] `stdalign.h`
- [x] `stdbool.h`
- [x] `stdint.h`
- [x] `stdlib.h`
- [x] `string.h`
- [x] `strings.h`
- [x] `sys/filio.h`
- [x] `sys/ioctl.h`
- [x] `sys/ipc.h`
- [x] `sys/isa_defs.h`
- [x] `sys/machine.h`
- [x] `sys/mman.h`
- [x] `sys/param.h`
- [x] `sys/ptrace.h`
- [x] `sys/resource.h`
- [x] `sys/sdt.h`
- [x] `sys/sem.h`
- [x] `sys/socket.h`
- [x] `sys/stat.h`
- [x] `sys/statvfs.h`
- [x] `sys/sysinfo.h`
- [x] `sys/time.h`
- [x] `sys/types.h`
- [x] `sys/uio.h`
- [x] `tgmath.h`
- [x] `tlhelp32.h`
- [x] `uchar.h`
- [x] `unistd.h`
- [x] `utime.h`
- [x] `wchar.h`
- [x] `wctype.h`
- [x] `windows.h`
- [x] `xlocale.h`

## Concrete Declaration And Type Checks

- [x] `F_GETFL`, `F_SETFL`, and `O_NONBLOCK` - modeled as the combined
  `HAVE_O_NONBLOCK` networking declaration probe used by
  `libstdc++-v3/configure.ac`.
- [x] `strnlen` - GCC 12+; modeled as `HAVE_DECL_STRNLEN`.
- [x] `pthread_rwlock_t` - modeled as `_GLIBCXX_USE_PTHREAD_RWLOCK_T`.

## Concrete Function Checks

These are reviewed `AC_CHECK_FUNCS` arguments and equivalent function checks
that are implemented as compile/link probes.

- [x] `__cxa_thread_atexit`
- [x] `__cxa_thread_atexit_impl`
- [x] `_aligned_malloc`
- [x] `_wfopen` - GCC 9+.
- [x] `aligned_alloc`
- [x] `arc4random`
- [x] `at_quick_exit`
- [x] `getentropy`
- [x] `gettimeofday`
- [x] `memalign`
- [x] `poll`
- [x] `posix_memalign`
- [x] `quick_exit`
- [x] `secure_getenv` - GCC 11.4+.
- [x] `setenv`
- [x] `sleep`
- [x] `sockatmark` - GCC 9+.
- [x] `strtof`
- [x] `strtold`
- [x] `timespec_get` - GCC 9+.
- [x] `tmpnam`
- [x] `uselocale`
- [x] `usleep`
- [x] `writev`

## Concrete Computed Values

- [x] `glibcxx_cv_stdio_eof` - modeled as `_GLIBCXX_STDIO_EOF`.
- [x] `glibcxx_cv_stdio_seek_cur` - modeled as `_GLIBCXX_STDIO_SEEK_CUR`.
- [x] `glibcxx_cv_stdio_seek_end` - modeled as `_GLIBCXX_STDIO_SEEK_END`.
- [x] `glibcxx_cv_at_least_32bit` - classified through zoneinfo policy;
  static tzdata remains fixed off for the supported Linux GNU runtime.

## Concrete Configure Options

These reviewed configure options have active Bazel equivalents or an explicit
classification. They are not generic raw inventory entries.

- [x] `hosted-libstdcxx` / `libstdcxx-hosted` - modeled by hosted runtime
  policy.
- [x] `default-libstdcxx-abi` - modeled by dual-ABI policy.
- [x] `libstdcxx-lock-policy` - target-derived lock policy.
- [x] `libstdcxx-zoneinfo` - modeled by fixed zoneinfo policy.
- [x] `system-libunwind` - modeled by the supported Linux GNU unwind policy.
- [x] `libstdcxx-static-eh-pool` and `libstdcxx-eh-pool-obj-count` -
  `build-setting-later`.
- [x] `libstdcxx-verbose` - `build-setting-later`.
- [x] `nls` - `build-setting-later`.
- [x] `newlib` and `picolibc` - `unsupported-target`.
- [x] `cross-host`, `target-subdir`, `build-libsubdir`, `toolexeclibdir`,
  `gxx-include-dir`, `python-dir`, `multilib`,
  `version-specific-runtime-libs`, `bugurl`, `pkgversion`, and
  `gcc-major-version-only` - configure/install/build plumbing replaced by
  Bazel labels and toolchain metadata.

## Target Derived

- [x] `GCC_AC_THREAD_HEADER` - selected by target thread model.
- [x] `GCC_AC_THREAD_MODEL` - selected by target thread model.
- [x] `GLIBCXX_CHECK_HOST` - represented by `target_config.bzl` target policy.
- [x] `GLIBCXX_ENABLE_CHEADERS` - represented by configured header selection.
- [x] `GLIBCXX_ENABLE_LOCK_POLICY` - represented by target lock policy.
- [x] `GLIBCXX_ENABLE_THREADS` - represented by target thread policy.

## Build Setting Later

- [x] `GLIBCXX_EMERGENCY_EH_ALLOC` - GCC 13+ setting; needs private EH pool knobs.
- [x] `GLIBCXX_ENABLE_CONCEPT_CHECKS` - needs a private feature knob if exposed.
- [x] `GLIBCXX_ENABLE_FLOAT128` - GCC 12+ policy/probe knob and version-map
  work is deferred.
- [x] `GLIBCXX_ENABLE_FULLY_DYNAMIC_STRING` - needs ABI-affecting private knob.
- [x] `GLIBCXX_ENABLE_VERBOSE` - needs private verbose-mode knob.
- [x] `nls` / `_GLIBCXX_USE_NLS` - needs private NLS enablement if message catalogs are built.
- [x] `stdio_pure` / `_GLIBCXX_USE_STDIO_PURE` - needs a private C stdio variant knob if exposed.
- [x] `malloc` allocator mode - needs an allocator variant knob if exposed.

## Not Needed

- [x] `GCC_BASE_VER` - install path/version plumbing.
- [x] `GCC_NO_EXECUTABLES` - replaced by Bazel probe execution policy.
- [x] `GCC_TRY_COMPILE_OR_LINK` - represented by explicit probe kinds.
- [x] `GCC_WITH_TOOLEXECLIBDIR` - install path plumbing.
- [x] `GLIBCXX_CHECK_SETRLIMIT` - testsuite/build plumbing.
- [x] `GLIBCXX_CONDITIONAL` - automake conditional plumbing.
- [x] `GLIBCXX_CONFIGURE` - configure path/subdir plumbing.
- [x] `GLIBCXX_CONFIGURE_DOCBOOK` - documentation build plumbing.
- [x] `GLIBCXX_CONFIGURE_TESTSUITE` - testsuite build plumbing.
- [x] `GLIBCXX_ENABLE` - generic enable-option plumbing.
- [x] `GLIBCXX_ENABLE_CXX_FLAGS` - user flags are handled by Bazel/toolchain options.
- [x] `GLIBCXX_ENABLE_PCH` - GCC make PCH plumbing not modeled.
- [x] `GLIBCXX_ENABLE_PYTHON` - install-only pretty-printer path.
- [x] `GLIBCXX_ENABLE_WERROR` - configure build policy not modeled.
- [x] `GLIBCXX_EVALUATE_CONDITIONALS` - automake conditional plumbing.
- [x] `GLIBCXX_EXPORT_FLAGS` - makefile export plumbing.
- [x] `GLIBCXX_EXPORT_INCLUDES` - makefile export plumbing.
- [x] `GLIBCXX_EXPORT_INSTALL_INFO` - install metadata plumbing.

## Unsupported Target

- [x] `GCC_CHECK_ASSEMBLER_HWCAP` - Solaris assembler HWCAP path.
- [x] `GCC_PROG_GNU_CXXFILT` - Sun/Solaris symbol-version support path.
- [x] `GLIBCXX_MAYBE_UNDERSCORED_FUNCS` - fallback for targets with underscored C symbols.
- [x] `GLIBCXX_CHECK_FILEBUF_NATIVE_HANDLES` - GCC 14+ Windows native file handle path.
- [x] `GLIBCXX_CHECK_SYSCTL_HW_NCPU` - BSD/macOS CPU-count path.
- [x] `GLIBCXX_CROSSCONFIG` - non-current cross branches are inactive.

## Unsupported Feature

- [x] `GCC_CET_FLAGS` - CET library flag policy is not modeled yet.
- [x] `GLIBCXX_ENABLE_BACKTRACE` - GCC 12+ libbacktrace and `<stacktrace>`
  are not built.
- [x] `GLIBCXX_ENABLE_DEBUG` - debug library variant is not built.
- [x] `GLIBCXX_ENABLE_DEBUG_FLAGS` - debug library flags are not exposed.
- [x] `GLIBCXX_ENABLE_PARALLEL` - parallel mode/libgomp integration is not built.
- [x] `GLIBCXX_ENABLE_VTABLE_VERIFY` - vtable verification runtime is not built.
