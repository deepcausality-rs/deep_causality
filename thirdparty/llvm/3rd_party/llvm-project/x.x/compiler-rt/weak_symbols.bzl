"""Darwin weak/undefined symbol link flags for compiler-rt runtimes.

On Darwin the sanitizer runtimes are linked with `-Wl,-U,<symbol>` for each
symbol listed in `compiler-rt/lib/<name>/weak_symbols.txt`, so that programs may
override them at runtime (for example `__asan_default_options`). This mirrors
`add_weak_symbols()` in `compiler-rt/cmake/Modules/SanitizerUtils.cmake`, which
emits `-Wl,-U,${symbol}` per symbol. The symbol names are kept verbatim from the
`weak_symbols.txt` files (Mach-O leading-underscore form).

`weak_symbols_in_sync_tests()` guards these lists against drift from upstream
on LLVM version bumps.
"""

load("@bazel_skylib//rules:diff_test.bzl", "diff_test")
load("@bazel_skylib//rules:write_file.bzl", "write_file")
load("@llvm-project//:vars.bzl", "LLVM_VERSION_MAJOR")

# compiler-rt/lib/sanitizer_common/weak_symbols.txt
SANITIZER_COMMON_WEAK_SYMBOLS = [
    "___sanitizer_free_hook",
    "___sanitizer_get_dtls_size",
    "___sanitizer_malloc_hook",
    "___sanitizer_report_error_summary",
    "___sanitizer_sandbox_on_notify",
    "___sanitizer_symbolize_code",
    "___sanitizer_symbolize_data",
    "___sanitizer_symbolize_frame",
    "___sanitizer_symbolize_demangle",
    "___sanitizer_symbolize_flush",
    "___sanitizer_symbolize_set_demangle",
    "___sanitizer_symbolize_set_inline_frames",
    "__dyld_get_dyld_header",
]

# compiler-rt/lib/asan/weak_symbols.txt
ASAN_WEAK_SYMBOLS = [
    "___asan_default_options",
    "___asan_default_suppressions",
    "___asan_on_error",
    "___asan_set_shadow_00",
    "___asan_set_shadow_01",
    "___asan_set_shadow_02",
    "___asan_set_shadow_03",
    "___asan_set_shadow_04",
    "___asan_set_shadow_05",
    "___asan_set_shadow_06",
    "___asan_set_shadow_07",
    "___asan_set_shadow_f1",
    "___asan_set_shadow_f2",
    "___asan_set_shadow_f3",
    "___asan_set_shadow_f4",
    "___asan_set_shadow_f5",
    "___asan_set_shadow_f6",
    "___asan_set_shadow_f7",
    "___asan_set_shadow_f8",
]

# compiler-rt/lib/lsan/weak_symbols.txt
LSAN_WEAK_SYMBOLS = [
    "___lsan_default_options",
    "___lsan_default_suppressions",
    "___lsan_is_turned_off",
]

# compiler-rt/lib/ubsan/weak_symbols.txt
UBSAN_WEAK_SYMBOLS = [
    "___ubsan_default_options",
] + (["___ubsan_default_suppressions"] if int(LLVM_VERSION_MAJOR) >= 23 else [])

# compiler-rt/lib/xray/weak_symbols.txt
XRAY_WEAK_SYMBOLS = [
    "___start_xray_fn_idx",
    "___start_xray_instr_map",
    "___stop_xray_fn_idx",
    "___stop_xray_instr_map",
    "___xray_default_options",
]

def weak_symbol_link_flags(symbol_lists):
    """Returns `-Wl,-U,<symbol>` link flags for the given lists of weak symbols.

    Args:
      symbol_lists: a list of symbol-name lists (e.g. [ASAN_WEAK_SYMBOLS, ...]).
    """
    return [
        "-Wl,-U," + symbol
        for symbols in symbol_lists
        for symbol in symbols
    ]

# Maps each runtime to (its weak-symbol list, the upstream weak_symbols.txt
# under compiler-rt/lib/). Used by weak_symbols_in_sync_tests().
_WEAK_SYMBOLS_BY_RUNTIME = {
    "asan": ASAN_WEAK_SYMBOLS,
    "lsan": LSAN_WEAK_SYMBOLS,
    "sanitizer_common": SANITIZER_COMMON_WEAK_SYMBOLS,
    "ubsan": UBSAN_WEAK_SYMBOLS,
    "xray": XRAY_WEAK_SYMBOLS,
}

def weak_symbols_in_sync_tests():
    """Asserts each *_WEAK_SYMBOLS list matches upstream lib/<name>/weak_symbols.txt.

    The lists above are a verbatim copy of compiler-rt's weak_symbols.txt files;
    these diff_tests fail if they drift (e.g. after an LLVM version bump), so the
    lists must be updated to match upstream. Must be called from the compiler-rt
    overlay package (the one that owns lib/<name>/weak_symbols.txt).
    """
    for name, symbols in _WEAK_SYMBOLS_BY_RUNTIME.items():
        write_file(
            name = "_{}_weak_symbols_expected".format(name),
            out = "_{}_weak_symbols.expected.txt".format(name),
            # Trailing "" reproduces the file's final newline so the exact
            # diff_test matches lib/<name>/weak_symbols.txt.
            content = symbols + [""],
            newline = "unix",
        )
        diff_test(
            name = "{}_weak_symbols_in_sync_test".format(name),
            failure_message = (
                "weak_symbols.bzl {}_WEAK_SYMBOLS is out of sync with ".format(name.upper()) +
                "lib/{}/weak_symbols.txt; update it to match upstream.".format(name)
            ),
            file1 = "_{}_weak_symbols.expected.txt".format(name),
            file2 = "lib/{}/weak_symbols.txt".format(name),
        )
