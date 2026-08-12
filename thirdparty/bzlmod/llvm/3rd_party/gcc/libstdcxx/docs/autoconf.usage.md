# libstdc++ Autoconf Usage Checklist

This checklist tracks configure checks as they are used by
`libstdc++-v3/configure.ac` and by the macros it reaches. It is ordered by the
supported Linux GNU flow first, then inactive branches and build-only plumbing.

Status meanings are defined in `AGENTS.md`.

## Supported Linux GNU Flow

- [x] `GLIBCXX_CONFIGURE` - configure initialization and source subdirectory setup; `not-needed`.
- [x] `GLIBCXX_CHECK_HOST` - target-derived host directories and policies; `target-derived`.
- [x] `GCC_NO_EXECUTABLES` - cross/no-link configure control; `not-needed`.
- [x] `GLIBCXX_ENABLE_HOSTED` - hosted mode is modeled for the supported runtime; `modeled`.
- [x] `GLIBCXX_ENABLE_LONG_LONG` - long long policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_WCHAR_T` - wide-character probes are modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_C99` - C++98/C++11 C99 probe groups are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_C99_TR1` - TR1 C99 probe groups are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_LFS` - full large-file probe group is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_GETTIMEOFDAY` - `gettimeofday` probe is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_TIME` - Linux time probes are modeled; the
  Win32 `Sleep` fallback is inactive for supported Linux GNU targets and is
  version-scoped as GCC 12 `HAVE_WIN32_SLEEP` or GCC 13+
  `_GLIBCXX_USE_WIN32_SLEEP`; `modeled`.
- [x] `GLIBCXX_CHECK_STDIO_PROTO` - `gets` declaration probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_MATH11_PROTO` - Linux obsolete `isinf`/`isnan` path is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_POLL` - `poll` probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_S_ISREG_OR_S_IFREG` - regular-file macro probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_WRITEV` - `writev` probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_UCHAR_H` - C11 uchar probes are modeled; GCC 12+
  adds C8 uchar probes; `modeled`.
- [x] `GLIBCXX_CHECK_INT64_T` - GCC 11 and older `int64_t`
  typedef-shape probes are modeled; `modeled`.
- [x] `GLIBCXX_COMPUTE_STDIO_INTEGER_CONSTANTS` - Linux GNU constants are policy-modeled; `modeled`.
- [x] `GLIBCXX_CHECK_TMPNAM` - `tmpnam` probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_PTHREAD_COND_CLOCKWAIT` - pthread probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_PTHREAD_MUTEX_CLOCKLOCK` - pthread probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_PTHREAD_RWLOCK_CLOCKLOCK` - pthread probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_GET_NPROCS` - GNU processor-count probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_SC_NPROCESSORS_ONLN` - `sysconf` processor-count probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_SC_NPROC_ONLN` - `sysconf` processor-count probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_PTHREADS_NUM_PROCESSORS_NP` - pthread processor-count probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_SDT_H` - systemtap header probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_COMPILER_FEATURES` - compiler feature checks are modeled as build policy; `modeled`.
- [x] `GLIBCXX_ENABLE_ATOMIC_BUILTINS` - atomic builtin policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_DECIMAL_FLOAT` - decimal floating-point support is probed; `modeled`.
- [x] `GLIBCXX_ENABLE_INT128_FLOAT128` - GCC 11 and older int128 support is
  probed; float128 remains a deferred policy; `modeled`.
- [x] `GLIBCXX_CHECK_GTHREADS` - gthreads checks are modeled; `modeled`.
- [x] `GCC_CHECK_TLS` - TLS probe is modeled; `modeled`.
- [x] `GCC_CHECK_UNWIND_GETIPINFO` - unwind policy is modeled; `modeled`.
- [x] `GCC_LINUX_FUTEX` - futex probe is modeled; `modeled`.
- [x] `GCC_HEADER_STDINT` - GCC 12-only `gstdint.h` generator is modeled by a
  Bazel-generated compatibility header; `modeled`.
- [x] `GLIBCXX_CHECK_LINKER_FEATURES` - linker/symver policy is modeled for GNU; `modeled`.
- [x] `GLIBCXX_ENABLE_SYMVERS` - GNU symbol version policy is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_EXCEPTION_PTR_SYMVER` - exception pointer symbol-version policy is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_SUPPORT` - native math probes are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_STDLIB_SUPPORT` - native stdlib probes are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_DEV_RANDOM` - Linux random-device policy is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_RANDOM_TR1` - GCC 8 TR1 random-device policy is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_ARC4RANDOM` - GCC 12+ `arc4random` probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_GETENTROPY` - GCC 12+ `getentropy` probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_FILESYSTEM_DEPS` - filesystem probes are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_TEXT_ENCODING` - GCC 14+ text encoding probes are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_DEBUGGING` - GCC 16+ Linux debug probes are modeled; `modeled`.
- [x] `GLIBCXX_CHECK_STDIO_LOCKING` - GCC 16+ stdio locking probe is modeled; `modeled`.
- [x] `GLIBCXX_STRUCT_TM_TM_ZONE` - GCC 15+ `tm_zone` probe is modeled; `modeled`.
- [x] `GLIBCXX_ZONEINFO_DIR` - GCC 13+ zoneinfo policy is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_ALIGNAS_CACHELINE` - GCC 13+ cacheline alignment probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_INIT_PRIORITY` - GCC 13+ init-priority probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_X86_RDRAND` - x86 RDRAND probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_X86_RDSEED` - x86 RDSEED probe is modeled; `modeled`.
- [x] `GLIBCXX_CHECK_SIZE_T_MANGLING` - size_t mangling policy is modeled; `modeled`.
- [x] `GLIBCXX_DEFAULT_ABI` - default ABI policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_DUAL_ABI` - dual ABI policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_LIBSTDCXX_VISIBILITY` - visibility policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_EXTERN_TEMPLATE` - extern-template policy is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_ALLOCATOR` - allocator selection is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_CLOCALE` - GNU locale path is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_CSTDIO` - stdio path is modeled; `modeled`.
- [x] `GLIBCXX_ENABLE_CHEADERS` - C header path is target-derived; `target-derived`.
- [x] `GLIBCXX_ENABLE_THREADS` - thread model is target-derived; `target-derived`.
- [x] `GCC_AC_THREAD_MODEL` - thread model is target-derived; `target-derived`.
- [x] `GCC_AC_THREAD_HEADER` - thread header is target-derived; `target-derived`.
- [x] `GLIBCXX_ENABLE_LOCK_POLICY` - lock policy is target-derived; `target-derived`.
- [x] `GLIBCXX_ENABLE_FILESYSTEM_TS` - filesystem source policy is modeled; `modeled`.

## Linkage Helper Usage

- [x] `GLIBCXX_CHECK_MATH_DECL` - represented by `GLIBCXX_CHECK_MATH_SUPPORT`; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECLS` - represented by `GLIBCXX_CHECK_MATH_SUPPORT`; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_1` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_2` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_3` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECLS_AND_LINKAGES_1` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_1` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_2` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_3` - represented by math support probes; `modeled`.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_1` - represented by stdlib support probes; `modeled`.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_2` - represented by stdlib support probes; `modeled`.
- [x] `GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_3` - represented by stdlib support probes; `modeled`.
- [x] `GLIBCXX_CHECK_SYSTEM_ERROR` - GCC 8 Linux errno availability is modeled; `modeled`.

## Build Setting Later

- [x] `GLIBCXX_ENABLE_VERBOSE` - fixed today; should become a private knob.
- [x] `GLIBCXX_ENABLE_CONCEPT_CHECKS` - fixed today; should become a private knob if exposed.
- [x] `GLIBCXX_ENABLE_FLOAT128` - GCC 12+ fixed policy today; needs target/probe knob and version-map work.
- [x] `GLIBCXX_ENABLE_FULLY_DYNAMIC_STRING` - fixed today; needs ABI knob work.
- [x] `GLIBCXX_EMERGENCY_EH_ALLOC` - GCC 13+ setting is fixed today; needs EH pool knobs.
- [x] `nls` / `_GLIBCXX_USE_NLS` - fixed off; needs NLS catalog build policy if exposed.
- [x] `stdio_pure` / `_GLIBCXX_USE_STDIO_PURE` - fixed off; needs a C stdio variant knob if exposed.
- [x] `malloc` allocator mode - fixed to the normal allocator; needs an allocator variant knob if exposed.

## Not Needed In Bazel

- [x] `GCC_BASE_VER` - install path versioning.
- [x] `GCC_NO_EXECUTABLES` - configure no-link control; Bazel probe policy replaces it.
- [x] `GCC_TRY_COMPILE_OR_LINK` - represented by explicit probe kinds.
- [x] `GCC_WITH_TOOLEXECLIBDIR` - install directory plumbing.
- [x] `GLIBCXX_CHECK_SETRLIMIT` - testsuite/build plumbing.
- [x] `GLIBCXX_CONDITIONAL` - automake conditional plumbing.
- [x] `GLIBCXX_CONFIGURE_DOCBOOK` - documentation build plumbing.
- [x] `GLIBCXX_CONFIGURE_TESTSUITE` - testsuite build plumbing.
- [x] `GLIBCXX_ENABLE` - generic configure option plumbing.
- [x] `GLIBCXX_ENABLE_CXX_FLAGS` - user flags are Bazel/toolchain inputs.
- [x] `GLIBCXX_ENABLE_PCH` - GCC make PCH build path.
- [x] `GLIBCXX_ENABLE_PYTHON` - install-only pretty-printer path.
- [x] `GLIBCXX_ENABLE_WERROR` - configure build policy.
- [x] `GLIBCXX_EVALUATE_CONDITIONALS` - automake conditional plumbing.
- [x] `GLIBCXX_EXPORT_FLAGS` - makefile export plumbing.
- [x] `GLIBCXX_EXPORT_INCLUDES` - makefile export plumbing.
- [x] `GLIBCXX_EXPORT_INSTALL_INFO` - install metadata plumbing.

## Inactive Target Branches

- [x] `GCC_CHECK_ASSEMBLER_HWCAP` - Solaris branch; `unsupported-target`.
- [x] `GCC_PROG_GNU_CXXFILT` - Sun/Solaris symbol-version branch; `unsupported-target`.
- [x] `GLIBCXX_CHECK_FILEBUF_NATIVE_HANDLES` - GCC 14+ Windows branch; `unsupported-target`.
- [x] `GLIBCXX_MAYBE_UNDERSCORED_FUNCS` - GCC 13 and older underscored C-symbol fallback branch; `unsupported-target`.
- [x] `GLIBCXX_CHECK_SYSCTL_HW_NCPU` - BSD/macOS branch; `unsupported-target`.
- [x] `GLIBCXX_CROSSCONFIG` - non-current cross branches; `unsupported-target`.

## Inactive Feature Branches

- [x] `GCC_CET_FLAGS` - CET flags not modeled; `unsupported-feature`.
- [x] `GLIBCXX_ENABLE_BACKTRACE` - GCC 12+ libbacktrace and `<stacktrace>` not built; `unsupported-feature`.
- [x] `GLIBCXX_ENABLE_DEBUG` - debug library variant not built; `unsupported-feature`.
- [x] `GLIBCXX_ENABLE_DEBUG_FLAGS` - debug library flags not exposed; `unsupported-feature`.
- [x] `GLIBCXX_ENABLE_PARALLEL` - parallel mode/libgomp integration not built; `unsupported-feature`.
- [x] `GLIBCXX_ENABLE_VTABLE_VERIFY` - vtable verification runtime not built; `unsupported-feature`.
