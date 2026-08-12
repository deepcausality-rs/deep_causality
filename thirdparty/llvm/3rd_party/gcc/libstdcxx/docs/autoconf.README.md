# libstdc++ Autoconf Glossary

This file is the human glossary for the Bazel port of GCC libstdc++ configure
logic. The check definition inventory is in `autoconf.checks.md`; the
configure usage inventory is in `autoconf.usage.md`; the mechanical status
inputs are `config_macro_status.txt` and `config_define_status.txt`.
The raw output from `autoconf_inventory.sh inventory` is a review queue;
it is not itself evidence that a check has been implemented.

The current active support scope is Linux with GNU libc across the supported GCC
versions declared in `3rd_party/gcc/version.bzl`. Other target branches are
documented so GCC updates are reviewable, but they do not imply support.
`autoconf.checks.md` also contains reviewed concrete entries for implemented
header, declaration, type, function, computed-value, and configure-option
checks. Those entries are narrower than the raw inventory output: they are
added only after mapping the upstream check to a Bazel probe or explicit
classification.

## Source Map

- `libstdc++-v3/configure.ac` maps to `configure.ac.bzl`.
- `libstdc++-v3/acinclude.m4` maps to `acinclude.m4.bzl`.
- `libstdc++-v3/linkage.m4` maps to math and stdlib helpers in
  `linkage.m4.bzl`.
- `libstdc++-v3/crossconfig.m4` maps to `crossconfig.m4.bzl`.
- GCC top-level `config/*.m4` files map to `gcc_config_checks.bzl`.
- `libstdc++-v3/configure.host` maps to target-derived policy in
  `target_config.bzl`, generated header selection in the
  `libstdcxx_*_header.bzl` rule files, and Bazel targets in `BUILD.bazel`.
- GCC source repositories are materialized per version by
  `3rd_party/gcc/extension/gcc.bzl`. `3rd_party/gcc/gcc.BUILD.bazel` is loaded
  inside each concrete GCC repository, while the stable `@gcc` repository is a
  reproducible selected facade over those concrete targets.
- Generic autoconf mechanics live in the `autoconf/` subpackage:
  `autoconf/checks.bzl`, `autoconf/autoconf_config.bzl`,
  `autoconf/autoconf_hdr.bzl`, `autoconf/providers.bzl`, and
  `autoconf/cc_configure_probe.bzl`. Keep these files free of libstdc++
  source-policy decisions; source-counterpart files should only declare checks
  through that local API.

## Version Matrix

The audit and smoke targets are generated from `GCC_VERSIONS`. Configure status
may be version-gated, for example `GCC 16+`, but the docs must describe whether
the older branch is inactive, absent upstream, or modeled differently. When a
new GCC version is introduced, the version commit must include the source-list,
header, config, and audit-status changes needed by that version, and the
all-version smoke target should continue to build every supported version at or
above that commit.

## Status Glossary

- `modeled`: the check or macro has an active Bazel equivalent.
- `probe-modeled`: a config define is produced by a Bazel compile, link,
  header, declaration, or related probe.
- `policy-modeled`: a config define is produced by target or fixed Bazel
  policy, not by compiling a snippet.
- `target-derived`: the answer comes from Bazel target/platform policy.
- `build-setting-later`: currently fixed, but should become a private build
  setting before claiming knob parity with GCC configure.
- `not-needed`: configure/build/install/testsuite plumbing replaced by Bazel.
- `unsupported-target`: target family branch not supported by this libstdc++
  port today.
- `unsupported-feature`: optional libstdc++ feature not built by this port
  today.

## Active Linux GNU Configure Flow

`GLIBCXX_CONFIGURE` initializes the libstdc++ configure script, computes source
and build paths, and sets make subdirectories. Bazel replaces that build-system
plumbing with explicit labels and source lists, so the macro is `not-needed`.

`GLIBCXX_CHECK_HOST` sources `configure.host` and chooses OS, CPU, ABI,
atomicity, thread, and header directories from the host triple. Bazel models
the active Linux GNU result as `target-derived` policy in `target_config.bzl`, the
generated-header rule files, and `BUILD.bazel`.

GCC 12 calls `GCC_HEADER_STDINT(include/gstdint.h)` to generate a private
`gstdint.h` header for `src/c++11/compatibility-atomic-c++0x.cc`. That hook has
no `config.h` output; Bazel models it with a generated compatibility header in
the GCC overlay for GCC <13.

`GLIBCXX_ENABLE_HOSTED`, `GLIBCXX_ENABLE_LONG_LONG`,
`GLIBCXX_ENABLE_WCHAR_T`, `GLIBCXX_ENABLE_C99`, `GLIBCXX_CHECK_C99_TR1`,
`GLIBCXX_CHECK_LFS`, `GLIBCXX_CHECK_GETTIMEOFDAY`, and
`GLIBCXX_ENABLE_LIBSTDCXX_TIME` define the core hosted C/C++ library surface.
The active Linux GNU path is modeled with Bazel policies and probes. Wide
character, C99, TR1, LFS, and time support now follow upstream-style compile or
link probe groups. The Linux `SYS_clock_gettime` fallback from
`GLIBCXX_ENABLE_LIBSTDCXX_TIME` is intentionally left undefined for the current
hosted Linux GNU matrix, where libc `clock_gettime` is expected. Supporting the
fallback still requires decision-tree support in the probe model so one
conditional result can decide which later probe runs. GCC 12 spells the Win32
fallback define as `HAVE_WIN32_SLEEP`; GCC 13+ spells it as
`_GLIBCXX_USE_WIN32_SLEEP`.

TODO: Model autoconf-style fallback decision trees separately from multi-output
checks. A compile or link probe can emit multiple success defines, matching a
single `AC_COMPILE_IFELSE` or `AC_LINK_IFELSE` branch, but the clock fallback
still needs an ordered decision where one result controls which later probe runs.

`GLIBCXX_CHECK_STDIO_PROTO`, `GLIBCXX_CHECK_MATH11_PROTO`,
`GLIBCXX_CHECK_POLL`, `GLIBCXX_CHECK_S_ISREG_OR_S_IFREG`,
`GLIBCXX_CHECK_WRITEV`, `GLIBCXX_CHECK_UCHAR_H` with GCC 12+ C8 probes,
GCC 11 and older `GLIBCXX_CHECK_INT64_T`,
`GLIBCXX_COMPUTE_STDIO_INTEGER_CONSTANTS`, `GLIBCXX_CHECK_TMPNAM`,
`GLIBCXX_CHECK_PTHREAD_COND_CLOCKWAIT`,
`GLIBCXX_CHECK_PTHREAD_MUTEX_CLOCKLOCK`,
`GLIBCXX_CHECK_PTHREAD_RWLOCK_CLOCKLOCK`, `GLIBCXX_CHECK_GET_NPROCS`,
`GLIBCXX_CHECK_SC_NPROCESSORS_ONLN`, `GLIBCXX_CHECK_SC_NPROC_ONLN`,
`GLIBCXX_CHECK_PTHREADS_NUM_PROCESSORS_NP`, and `GLIBCXX_CHECK_SDT_H`
cover targeted libc, pthread, stdio, system, and header capabilities used by
the supported runtime. Their active Linux GNU behavior is modeled. The stdio
integer constants are policy-modeled to glibc values for the current Linux GNU
matrix; they should become computed constants if non-GNU libc support is added.

`GCC_CHECK_TLS`, `GCC_CHECK_UNWIND_GETIPINFO`, and `GCC_LINUX_FUTEX` are GCC
top-level checks used by libstdc++. TLS and futex are modeled with target
compiler/linker probes; unwind IP info is modeled as GCC target policy for the
supported Linux GNU configuration.

`GLIBCXX_CHECK_COMPILER_FEATURES`, `GLIBCXX_ENABLE_ATOMIC_BUILTINS`,
`GLIBCXX_ENABLE_DECIMAL_FLOAT`, GCC 11 and older
`GLIBCXX_ENABLE_INT128_FLOAT128`, and `GLIBCXX_CHECK_GTHREADS` cover compiler
section flags, lock-free atomic word support, decimal floating-point support,
int128 support, and gthreads support. The active Linux GNU behavior is modeled,
with some answers represented as target policy where GCC's result is
target-derived. Decimal floating point and GCC 11 int128 support are compile
probes. Float128 remains deferred. The gthreads availability and pthread
read/write lock checks compile against the generated `bits/gthr.h` overlay so
they follow the staged header context used by this Bazel port.

`GLIBCXX_CHECK_LINKER_FEATURES`, `GLIBCXX_ENABLE_SYMVERS`,
`GLIBCXX_CHECK_EXCEPTION_PTR_SYMVER`, `GLIBCXX_DEFAULT_ABI`,
`GLIBCXX_ENABLE_LIBSTDCXX_DUAL_ABI`, and
`GLIBCXX_ENABLE_LIBSTDCXX_VISIBILITY` cover ABI and shared-library policy.
The Bazel port models the Linux GNU dynamic libstdc++ path.

`GLIBCXX_CHECK_MATH_SUPPORT`, `GLIBCXX_CHECK_STDLIB_SUPPORT`,
`GLIBCXX_CHECK_MATH_DECL`, `GLIBCXX_CHECK_MATH_DECLS`,
`GLIBCXX_CHECK_MATH_DECL_1`, `GLIBCXX_CHECK_MATH_DECL_2`,
`GLIBCXX_CHECK_MATH_DECL_3`, `GLIBCXX_CHECK_MATH_DECLS_AND_LINKAGES_1`,
`GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_1`,
`GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_2`,
`GLIBCXX_CHECK_MATH_DECL_AND_LINKAGE_3`,
`GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_1`,
`GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_2`, and
`GLIBCXX_CHECK_STDLIB_DECL_AND_LINKAGE_3` are the math and stdlib support
groups from `acinclude.m4` and `linkage.m4`. The Bazel port represents these
as grouped link probes in `linkage.m4.bzl`.

GCC 8 `GLIBCXX_CHECK_RANDOM_TR1`, GCC 9+ `GLIBCXX_CHECK_DEV_RANDOM`, GCC 12+
`GLIBCXX_CHECK_ARC4RANDOM`, GCC 12+ `GLIBCXX_CHECK_GETENTROPY`,
`GLIBCXX_CHECK_FILESYSTEM_DEPS`, GCC 14+ `GLIBCXX_CHECK_TEXT_ENCODING`, GCC 16+
`GLIBCXX_CHECK_DEBUGGING`, GCC 16+ `GLIBCXX_CHECK_STDIO_LOCKING`, GCC 15+
`GLIBCXX_STRUCT_TM_TM_ZONE`, GCC 13+ `GLIBCXX_ZONEINFO_DIR`, GCC 13+
`GLIBCXX_CHECK_ALIGNAS_CACHELINE`, GCC 13+ `GLIBCXX_CHECK_INIT_PRIORITY`,
GCC 8 `GLIBCXX_CHECK_SYSTEM_ERROR`, `GLIBCXX_CHECK_X86_RDRAND`,
`GLIBCXX_CHECK_X86_RDSEED`, and `GLIBCXX_CHECK_SIZE_T_MANGLING` cover runtime
library details after the core libc checks. The active Linux GNU behavior is
modeled as probes or policy. The random-device checks are policy-modeled
rather than probed because GCC's native check reads the execution host's
`/dev/random` and `/dev/urandom`, while the supported Bazel target decision is
Linux GNU and GCC's cross configuration hardcodes that answer for Linux-family
targets.

`GLIBCXX_ENABLE_ALLOCATOR`, `GLIBCXX_ENABLE_CLOCALE`,
`GLIBCXX_ENABLE_CSTDIO`, `GLIBCXX_ENABLE_CHEADERS`,
`GLIBCXX_ENABLE_THREADS`, `GCC_AC_THREAD_MODEL`, `GCC_AC_THREAD_HEADER`,
`GLIBCXX_ENABLE_LOCK_POLICY`, `GLIBCXX_ENABLE_EXTERN_TEMPLATE`, and
`GLIBCXX_ENABLE_FILESYSTEM_TS` select headers, source families, allocator,
locale, stdio, thread, and source graph policy. These are modeled or
target-derived. `GLIBCXX_ENABLE_LOCK_POLICY` is target-derived rather than a
plain probe because GCC intentionally keeps RISC-V on the mutex shared_ptr
reference-count ABI even when compare-and-swap builtins exist.

## Deferred Knobs

`GLIBCXX_ENABLE_VERBOSE`, `GLIBCXX_ENABLE_CONCEPT_CHECKS`, GCC 12+
`GLIBCXX_ENABLE_FLOAT128`, `GLIBCXX_ENABLE_FULLY_DYNAMIC_STRING`,
`GLIBCXX_ENABLE_CSTDIO`'s `stdio_pure` mode, `GLIBCXX_ENABLE_ALLOCATOR`'s
`malloc` mode, NLS, and GCC 13+ `GLIBCXX_EMERGENCY_EH_ALLOC` are currently
fixed or disabled policies. They should become explicit private Bazel settings
only if the port exposes the corresponding GCC variant.
`GLIBCXX_ENABLE_DECIMAL_FLOAT` is already represented by a compile probe and is
not a deferred knob. Float128 remains fixed disabled until the probe result and
`float128.ver` version-script input can be modeled together.

## Audited Policy And Defaults

The current Linux GNU policy/default set was reviewed against the active
support scope. `ICONV_CONST` is intentionally left undefined because the
supported glibc iconv declaration uses a non-const input pointer. `USE_EMUTLS`
is intentionally left undefined because the supported compiler and target path
uses real TLS through `GCC_CHECK_TLS`. The long-double compatibility defines
are intentionally left undefined because the supported CPU/version-script
policy does not enable GCC's long-double compatibility port files. The
`SYS_clock_gettime` and Win32 sleep fallback defines are left undefined for the
supported Linux GNU path, where libc `clock_gettime`, `nanosleep`, and
`sched_yield` probes are expected to decide the active time support. GCC 12
spells the Win32 fallback define as `HAVE_WIN32_SLEEP`; GCC 13+ spells it as
`_GLIBCXX_USE_WIN32_SLEEP`. The stdio integer constants are policy-modeled to
glibc values; this remains acceptable only while non-GNU libc support is out of
scope.

## High-Risk Probe Audit

The high-risk modeled groups were reviewed for source-counterpart alignment.
C99/TR1 and wide-character checks live in `acinclude.m4.bzl` and keep the
upstream C++98/C++11 split. Filesystem checks keep the upstream function and
member probes, with non-Linux branches inactive. Stdio locking is modeled as
three independent probes for locking, `fwrite_unlocked`, and glibc FILE
internals; this is slightly less conditional than upstream but safe for the
supported glibc path because each generated define still requires its own
successful compile/link probe. Gthreads probes compile against the generated
`bits/gthr.h` overlay so they see the same staged header context as the
runtime build. Linker, symbol-version, and ABI policy remains target-selected
for Linux GNU and tied to the generated version script and `libstdc++.so.6`
soname. The version-script assembly intentionally mirrors the shell recipe in
`libstdc++-v3/src/Makefile.am`; the final `CC -E -P -include config.h`
preprocessor pass still uses the Bazel C++ toolchain API so target flags and
tool inputs remain modeled.

## Configure Plumbing Replaced By Bazel

`GCC_BASE_VER`, `GCC_NO_EXECUTABLES`, `GCC_TRY_COMPILE_OR_LINK`,
`GCC_WITH_TOOLEXECLIBDIR`, `GLIBCXX_CHECK_SETRLIMIT`,
`GLIBCXX_CONDITIONAL`, `GLIBCXX_CONFIGURE_DOCBOOK`,
`GLIBCXX_CONFIGURE_TESTSUITE`, `GLIBCXX_ENABLE`, `GLIBCXX_ENABLE_CXX_FLAGS`,
`GLIBCXX_ENABLE_PCH`, `GLIBCXX_ENABLE_PYTHON`, `GLIBCXX_ENABLE_WERROR`,
`GLIBCXX_EVALUATE_CONDITIONALS`, `GLIBCXX_EXPORT_FLAGS`,
`GLIBCXX_EXPORT_INCLUDES`, and `GLIBCXX_EXPORT_INSTALL_INFO` are make,
install, doc, testsuite, or generic configure plumbing. Bazel replaces these
with explicit build graph structure or user/toolchain options.

## Inactive Target Branches

`GCC_CHECK_ASSEMBLER_HWCAP` is Solaris-only assembler HWCAP handling.
`GCC_PROG_GNU_CXXFILT` is needed for Sun/Solaris symbol versioning.
GCC 14+ `GLIBCXX_CHECK_FILEBUF_NATIVE_HANDLES` is the Windows `_get_osfhandle`
path.
`GLIBCXX_MAYBE_UNDERSCORED_FUNCS` is a GCC 13 and older fallback for targets
whose C library exports underscored function names; the supported Linux GNU
targets use normal names.
`GLIBCXX_CHECK_SYSCTL_HW_NCPU` is the BSD/macOS CPU-count path.
`GLIBCXX_CROSSCONFIG` covers cross and non-current target branches. These are
classified `unsupported-target`.

## Inactive Feature Branches

`GCC_CET_FLAGS` is not modeled as a target-library flag policy yet.
GCC 12+ `GLIBCXX_ENABLE_BACKTRACE` is inactive because libbacktrace and
`<stacktrace>` are not built. `GLIBCXX_ENABLE_DEBUG`, `GLIBCXX_ENABLE_DEBUG_FLAGS`,
`GLIBCXX_ENABLE_PARALLEL`, and `GLIBCXX_ENABLE_VTABLE_VERIFY` are optional
runtime-library feature families not built by this port today.

## Config Define Status

The detailed config define list is tracked in `config_define_status.txt`.
Important groups are:

- C99/TR1 and wide-character defines are `probe-modeled`.
- Math and stdlib function defines are `probe-modeled`.
- Header checks from `configure.ac` are generated through `ac_check_headers`
  and listed explicitly in `autoconf.checks.md` for auditability.
- `F_GETFL` and `F_SETFL` are configure cache gates for `HAVE_O_NONBLOCK`, not
  separate generated config defines in this port.
- ABI, symbol version, hosted, and Linux GNU fixed choices are
  `policy-modeled` or `target-derived`.
- Generic `acx.m4` defines that are not called by the active libstdc++ flow are
  `not-needed`.
- Windows, Solaris, BSD/macOS, and libbacktrace-only outputs are
  `unsupported-target` or `unsupported-feature`.

Run `bazel test --config remote //3rd_party/gcc/libstdcxx/tests:config_define_audit_test`
to verify the source inventory and status files still agree.
