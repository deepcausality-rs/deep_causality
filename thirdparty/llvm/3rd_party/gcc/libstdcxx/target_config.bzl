# Mirrors the host-derived directory choices from GCC's
# libstdc++-v3/configure.host. When adding a target or changing one of these
# fields, compare against configure.host plus configure.ac's *_SRCDIR
# substitutions for the generated include/source paths.

load("//3rd_party/gcc:version.bzl", "libstdcxx_has_atomic_lock_policy_define")
load("//3rd_party/gcc/libstdcxx/autoconf:checks.bzl", "policy_define", "policy_undef")

_SUPPORTED_TARGETS = {
    "//platforms/config:linux_x86_64_gnu": {
        "abi_baseline_pair": "x86_64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/i486",
        "error_constants_dir": "os/generic",
        "host_triple": "x86_64-linux-gnu",
        "locale_dir": "locale/gnu",
        "os_include_dir": "os/gnu-linux",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_aarch64_gnu": {
        "abi_baseline_pair": "aarch64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/aarch64",
        "error_constants_dir": "os/generic",
        "host_triple": "aarch64-linux-gnu",
        "locale_dir": "locale/gnu",
        "os_include_dir": "os/gnu-linux",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_armv7_gnu": {
        "abi_baseline_pair": "armv7-linux-gnueabihf",
        "abi_tweaks_dir": "cpu/arm",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/arm",
        "cpu_opt_dir": "cpu/generic/opt",
        "error_constants_dir": "os/generic",
        "host_triple": "armv7-linux-gnueabihf",
        "locale_dir": "locale/gnu",
        "os_include_dir": "os/gnu-linux",
        "port_symver_files": ["os/gnu-linux/arm-eabi-extra.ver"],
        "ptrdiff_t_is_int": True,
        "size_t_is_uint": True,
        "size_t_mangling": "j",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_riscv64_gnu": {
        "abi_baseline_pair": "riscv64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": False,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/generic",
        "error_constants_dir": "os/generic",
        "host_triple": "riscv64-linux-gnu",
        "locale_dir": "locale/gnu",
        "os_include_dir": "os/gnu-linux",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_s390x_gnu": {
        "abi_baseline_pair": "s390x-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/generic",
        "error_constants_dir": "os/generic",
        "host_triple": "s390x-linux-gnu",
        "locale_dir": "locale/gnu",
        "os_include_dir": "os/gnu-linux",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_x86_64_musl": {
        "abi_baseline_pair": "x86_64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/i486",
        "error_constants_dir": "os/generic",
        "host_triple": "x86_64-linux-musl",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/generic",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_aarch64_musl": {
        "abi_baseline_pair": "aarch64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/aarch64",
        "error_constants_dir": "os/generic",
        "host_triple": "aarch64-linux-musl",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/generic",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_armv7_musl": {
        "abi_baseline_pair": "armv7-linux-musleabihf",
        "abi_tweaks_dir": "cpu/arm",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/arm",
        "cpu_opt_dir": "cpu/generic/opt",
        "error_constants_dir": "os/generic",
        "host_triple": "armv7-linux-musleabihf",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/generic",
        "port_symver_files": ["os/gnu-linux/arm-eabi-extra.ver"],
        "ptrdiff_t_is_int": True,
        "size_t_is_uint": True,
        "size_t_mangling": "j",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_riscv64_musl": {
        "abi_baseline_pair": "riscv64-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": False,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/generic",
        "error_constants_dir": "os/generic",
        "host_triple": "riscv64-linux-musl",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/generic",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:linux_s390x_musl": {
        "abi_baseline_pair": "s390x-linux-gnu",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/generic",
        "error_constants_dir": "os/generic",
        "host_triple": "s390x-linux-musl",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/generic",
        "symver_file": "abi/pre/gnu.ver",
        "symver_style": "gnu",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:windows_x86_64": {
        "abi_baseline_pair": "x86_64-mingw32",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/i486",
        "error_constants_dir": "os/mingw32-w64",
        "host_triple": "x86_64-w64-mingw32",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/mingw32-w64",
        "thread_header": "gthr-posix.h",
    },
    "//platforms/config:windows_aarch64": {
        "abi_baseline_pair": "aarch64-mingw32",
        "abi_tweaks_dir": "cpu/generic",
        "atomic_lock_policy": True,
        "atomic_word_dir": "cpu/generic",
        "atomicity_dir": "cpu/generic/atomicity_builtins",
        "cpu_defines_dir": "cpu/generic",
        "cpu_include_dir": "cpu/aarch64",
        "error_constants_dir": "os/mingw32-w64",
        "host_triple": "aarch64-w64-mingw32",
        "locale_dir": "locale/generic",
        "os_include_dir": "os/mingw32-w64",
        "thread_header": "gthr-posix.h",
    },
}

_DEFAULT_POLICY_FIELDS = {
    "enable_float128": False,
    "extern_template": 1,
    "have_attribute_visibility": 1,
    "inline_version": 0,
    "port_symver_files": [],
    "ptrdiff_t_is_int": False,
    "size_t_is_uint": False,
    "size_t_mangling": "m",
    "symver_file": "abi/pre/none.ver",
    "symver_style": "none",
    "use_allocator_new": 1,
    "use_cxx11_abi": 1,
    "use_dual_abi": 1,
}

_NO_MATCH_ERROR = "Unsupported libstdc++ target platform"

def _field_value(values, field):
    if field in values:
        return values[field]
    if field == "cpu_opt_dir":
        return "{}/opt".format(values["cpu_include_dir"])
    if field in _DEFAULT_POLICY_FIELDS:
        return _DEFAULT_POLICY_FIELDS[field]
    fail("Unknown libstdc++ target field: {}".format(field))

def _select_field(field):
    return select({
        Label(config): _field_value(values, field)
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def libstdcxx_host_triple():
    return _select_field("host_triple")

def libstdcxx_cpu_include_dir():
    return _select_field("cpu_include_dir")

def libstdcxx_os_include_dir():
    return _select_field("os_include_dir")

def libstdcxx_abi_baseline_pair():
    return _select_field("abi_baseline_pair")

def libstdcxx_abi_tweaks_dir():
    return _select_field("abi_tweaks_dir")

def libstdcxx_atomicity_dir():
    return _select_field("atomicity_dir")

def libstdcxx_atomic_word_dir():
    return _select_field("atomic_word_dir")

def libstdcxx_atomic_lock_policy():
    return _select_field("atomic_lock_policy")

def libstdcxx_cpu_defines_dir():
    return _select_field("cpu_defines_dir")

def libstdcxx_error_constants_dir():
    return _select_field("error_constants_dir")

def libstdcxx_locale_dir():
    return _select_field("locale_dir")

def libstdcxx_thread_header():
    return _select_field("thread_header")

def libstdcxx_enable_float128():
    return _select_field("enable_float128")

def libstdcxx_extern_template():
    return _select_field("extern_template")

def libstdcxx_have_attribute_visibility():
    return _select_field("have_attribute_visibility")

def libstdcxx_inline_version():
    return _select_field("inline_version")

def libstdcxx_use_allocator_new():
    return _select_field("use_allocator_new")

def libstdcxx_use_cxx11_abi():
    return _select_field("use_cxx11_abi")

def libstdcxx_use_dual_abi():
    return _select_field("use_dual_abi")

def libstdcxx_symver_style():
    return _select_field("symver_style")

def libstdcxx_config_h_policy_defines(gcc_version):
    return select({
        Label(config): (
            ([
                policy_define("_GLIBCXX_SYMVER"),
                policy_define("_GLIBCXX_SYMVER_GNU"),
                policy_define("HAVE_AS_SYMVER_DIRECTIVE"),
                policy_define("HAVE_SYMVER_SYMBOL_RENAMING_RUNTIME_SUPPORT"),
                policy_define("HAVE_EXCEPTION_PTR_SINCE_GCC46"),
            ] if _field_value(values, "symver_style") == "gnu" else []) +
            ([policy_define("HAVE_ATOMIC_LOCK_POLICY")] if libstdcxx_has_atomic_lock_policy_define(gcc_version) and values["atomic_lock_policy"] else []) +
            [policy_define("_GLIBCXX_MANGLE_SIZE_T", _field_value(values, "size_t_mangling"))] +
            ([
                policy_define("_GLIBCXX_PTRDIFF_T_IS_INT"),
            ] if _field_value(values, "ptrdiff_t_is_int") else [
                policy_undef("_GLIBCXX_PTRDIFF_T_IS_INT"),
            ]) +
            ([
                policy_define("_GLIBCXX_SIZE_T_IS_UINT"),
            ] if _field_value(values, "size_t_is_uint") else [
                policy_undef("_GLIBCXX_SIZE_T_IS_UINT"),
            ])
        )
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

# These adapters turn configure.host-style directory fields into the exact
# config headers and sources consumed by libstdc++-v3/include/Makefile.am and
# libstdc++-v3/src/*/Makefile.am.
def _gcc_config_header_label(field, basename):
    return select({
        Label(config): "libstdc++-v3/config/{}/{}".format(_field_value(values, field), basename)
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def _gcc_config_source_label(field, basename):
    return select({
        Label(config): "libstdc++-v3/config/{}/{}".format(_field_value(values, field), basename)
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def _gcc_config_path_label(field):
    return select({
        Label(config): "libstdc++-v3/config/{}".format(_field_value(values, field))
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def _gcc_config_path_labels(field):
    return select({
        Label(config): [
            "libstdc++-v3/config/{}".format(path)
            for path in _field_value(values, field)
        ]
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def _gcc_libgcc_header_label(field):
    return select({
        Label(config): "libgcc/{}".format(values[field])
        for config, values in _SUPPORTED_TARGETS.items()
    }, no_match_error = _NO_MATCH_ERROR)

def libstdcxx_ctype_base_h():
    return _gcc_config_header_label("os_include_dir", "ctype_base.h")

def libstdcxx_ctype_inline_h():
    return _gcc_config_header_label("os_include_dir", "ctype_inline.h")

def libstdcxx_os_defines_h():
    return _gcc_config_header_label("os_include_dir", "os_defines.h")

def libstdcxx_atomic_word_h():
    return _gcc_config_header_label("atomic_word_dir", "atomic_word.h")

def libstdcxx_atomicity_h():
    return _gcc_config_header_label("atomicity_dir", "atomicity.h")

def libstdcxx_cxxabi_tweaks_h():
    return _gcc_config_header_label("abi_tweaks_dir", "cxxabi_tweaks.h")

def libstdcxx_cpu_defines_h():
    return _gcc_config_header_label("cpu_defines_dir", "cpu_defines.h")

def libstdcxx_error_constants_h():
    return _gcc_config_header_label("error_constants_dir", "error_constants.h")

def libstdcxx_bits_opt_random_h():
    return _gcc_config_header_label("cpu_opt_dir", "bits/opt_random.h")

def libstdcxx_ext_opt_random_h():
    return _gcc_config_header_label("cpu_opt_dir", "ext/opt_random.h")

def libstdcxx_c_locale_h():
    return _gcc_config_header_label("locale_dir", "c_locale.h")

def libstdcxx_c_locale_internal_h():
    return _gcc_config_header_label("locale_dir", "c++locale_internal.h")

def libstdcxx_messages_members_h():
    return _gcc_config_header_label("locale_dir", "messages_members.h")

def libstdcxx_time_members_h():
    return _gcc_config_header_label("locale_dir", "time_members.h")

def libstdcxx_ctype_configure_char_cc():
    return _gcc_config_source_label("os_include_dir", "ctype_configure_char.cc")

def libstdcxx_c_locale_cc():
    return _gcc_config_source_label("locale_dir", "c_locale.cc")

def libstdcxx_codecvt_members_cc():
    return _gcc_config_source_label("locale_dir", "codecvt_members.cc")

def libstdcxx_collate_members_cc():
    return _gcc_config_source_label("locale_dir", "collate_members.cc")

def libstdcxx_ctype_members_cc():
    return _gcc_config_source_label("locale_dir", "ctype_members.cc")

def libstdcxx_messages_members_cc():
    return _gcc_config_source_label("locale_dir", "messages_members.cc")

def libstdcxx_monetary_members_cc():
    return _gcc_config_source_label("locale_dir", "monetary_members.cc")

def libstdcxx_numeric_members_cc():
    return _gcc_config_source_label("locale_dir", "numeric_members.cc")

def libstdcxx_time_members_cc():
    return _gcc_config_source_label("locale_dir", "time_members.cc")

def libstdcxx_thread_header_h():
    return _gcc_libgcc_header_label("thread_header")

def libstdcxx_symver_file():
    return _gcc_config_path_label("symver_file")

def libstdcxx_port_symver_files():
    return _gcc_config_path_labels("port_symver_files")
