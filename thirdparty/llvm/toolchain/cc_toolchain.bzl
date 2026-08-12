load("@rules_cc//cc/toolchains:feature_set.bzl", "cc_feature_set")
load("@rules_cc//cc/toolchains:toolchain.bzl", _cc_toolchain = "cc_toolchain")

def cc_toolchain(name, tool_map, module_map = None, extra_args = []):
    cc_feature_set(
        name = name + "_known_features",
        all_of = [
            "@rules_cc//cc/toolchains/args/layering_check:layering_check",
            "@rules_cc//cc/toolchains/args/layering_check:use_module_maps",
            "@llvm//toolchain/features:static_link_cpp_runtimes",
            "@llvm//toolchain/features/runtime_library_search_directories:feature",
            "@llvm//toolchain/features:parse_headers",
            "@llvm//toolchain/features:external_include_paths",
            "@llvm//toolchain/features:generate_pdb_file",
            "@llvm//toolchain/features:fdo_optimize",
            "@rules_cc//cc/toolchains/args/thin_lto:feature",
        ] + select({
            "@platforms//os:linux": [
                "@llvm//toolchain/features/interface_libraries:feature",
            ],
            "@platforms//os:macos": [
                "@llvm//toolchain/features/interface_libraries:feature",
            ],
            "//conditions:default": [],
        }) + select({
            "@llvm//toolchain:macos_complete": [
                "@llvm//toolchain/features:generate_dsym_file",
            ],
            "//conditions:default": [],
        }) + [
            # Those features are enabled internally by --compilation_mode flags family.
            # We add them to the list of known_features but not in the list of enabled_features.
            "@llvm//toolchain/features:all_non_legacy_builtin_features",
            "@llvm//toolchain/features/legacy:all_legacy_builtin_features",
            # Always last (contains user_compile_flags and user_link_flags who should apply last).
            "@llvm//toolchain/features/legacy:experimental_replace_legacy_action_config_features",
        ],
    )

    cc_feature_set(
        name = name + "_enabled_features",
        all_of = select({
            "@platforms//os:linux": [
                "@llvm//toolchain/features/interface_libraries:feature",
                "@llvm//toolchain/features:static_link_cpp_runtimes",
                "@llvm//toolchain/features/runtime_library_search_directories:feature",
            ],
            "@platforms//os:macos": [
                "@llvm//toolchain/features/interface_libraries:feature",
                # macOS links libc++ from the SDK, so it doesn't statically link
                # the C++ runtimes. But it does need dynamic runtime libs (e.g.
                # sanitizer dylibs) placed in runfiles with an @loader_path rpath,
                # which these two features provide. static_link_cpp_runtimes is
                # required for the toolchain to consult dynamic_runtime_lib; it is
                # a no-op for the (empty) macOS C++ runtime libs.
                "@llvm//toolchain/features:static_link_cpp_runtimes",
                "@llvm//toolchain/features/runtime_library_search_directories:feature",
            ],
            "@platforms//os:windows": [
                "@llvm//toolchain/features:static_link_cpp_runtimes",
                "@llvm//toolchain/features/runtime_library_search_directories:feature",
                "@rules_cc//cc/toolchains/args/def_file:def_file",
                "@llvm//toolchain/features:targets_windows",
            ],
            "@platforms//os:none": [],
        }) + [
            "@llvm//toolchain/features:prefer_pic_for_opt_binaries",
            "@llvm//toolchain/features:sanitize_pwd",
            "@rules_cc//cc/toolchains/args/layering_check:module_maps",
            "@llvm//toolchain/features:module_map_home_cwd",
            # These are "enabled" but they only _actually_ get enabled when the underlying compilation mode is set.
            # This lets us properly order them before user_compile_flags and user_link_flags below.
            "@llvm//toolchain/features:opt",
            "@llvm//toolchain/features:dbg",
            "@llvm//toolchain/features:archive_param_file",
            "@llvm//toolchain/features:parse_headers_wrapper",
            "@llvm//toolchain/features/legacy:all_legacy_builtin_features",
            # Always last (contains user_compile_flags and user_link_flags who should apply last).
            "@llvm//toolchain/features/legacy:experimental_replace_legacy_action_config_features",
        ],
    )

    cc_feature_set(
        name = name + "_runtimes_only_enabled_features",
        all_of = [
            "@llvm//toolchain/features:prefer_pic_for_opt_binaries",
            "@llvm//toolchain/features:sanitize_pwd",
            "@rules_cc//cc/toolchains/args/layering_check:module_maps",
            "@llvm//toolchain/features:module_map_home_cwd",
            "@llvm//toolchain/features:archive_param_file",
            # Always last (contains user_compile_flags and user_link_flags who should apply last).
            "@llvm//toolchain/features/legacy:experimental_replace_legacy_action_config_features",
        ],
    )

    _cc_toolchain(
        name = name,
        args = select({
            "@llvm//toolchain:runtimes_none": ["@llvm//toolchain/runtimes:toolchain_args"],
            "@llvm//toolchain:runtimes_stage1": ["@llvm//toolchain/runtimes:toolchain_args"],
            "@llvm//toolchain:runtimes_stage1_hosted": ["@llvm//toolchain/runtimes:toolchain_args"],
            "//conditions:default": ["@llvm//toolchain:toolchain_args"],
        }) + [
            # TODO: rules_cc passes extra args to these actions, ideally these would be fixed in rules_cc.
            "@llvm//toolchain/args:ignore_unused_command_line_argument",
        ] + extra_args,
        supports_header_parsing = True,
        supports_param_files = True,
        artifact_name_patterns = select({
            "@platforms//os:macos": [
                "@llvm//toolchain:macos_dynamic_library_pattern",
                "@llvm//toolchain:macos_interface_library_pattern",
            ],
            "@platforms//os:windows": [
                "@llvm//toolchain:windows_executable_pattern",
            ],
            "//conditions:default": [],
        }),
        known_features = select({
            "@llvm//toolchain:runtimes_none": [
                "@llvm//toolchain/features:external_include_paths",
                "@llvm//toolchain/features:fdo_optimize",
            ],
            "@llvm//toolchain:runtimes_stage1": [
                "@llvm//toolchain/features:external_include_paths",
                "@llvm//toolchain/features:fdo_optimize",
            ],
            "@llvm//toolchain:runtimes_stage1_hosted": [
                "@llvm//toolchain/features:external_include_paths",
                "@llvm//toolchain/features:fdo_optimize",
            ],
            "//conditions:default": [name + "_known_features"],
        }),
        enabled_features = select({
            "@llvm//toolchain:runtimes_none": [name + "_runtimes_only_enabled_features"],
            "@llvm//toolchain:runtimes_stage1": [name + "_runtimes_only_enabled_features"],
            "@llvm//toolchain:runtimes_stage1_hosted": [name + "_runtimes_only_enabled_features"],
            "//conditions:default": [name + "_enabled_features"],
        }),
        tool_map = tool_map,
        module_map = module_map,
        static_runtime_lib = select({
            "@llvm//toolchain:runtimes_none": "@llvm//runtimes:none",
            "@llvm//toolchain:runtimes_stage1": "@llvm//runtimes:none",
            "@llvm//toolchain:runtimes_stage1_hosted": "@llvm//runtimes:none",
            "//conditions:default": "@llvm//runtimes:static_runtime_lib",
        }),
        dynamic_runtime_lib = select({
            "@llvm//toolchain:runtimes_none": "@llvm//runtimes:none",
            "@llvm//toolchain:runtimes_stage1": "@llvm//runtimes:none",
            "@llvm//toolchain:runtimes_stage1_hosted": "@llvm//runtimes:none",
            "//conditions:default": "@llvm//runtimes:dynamic_runtime_lib",
        }),
        compiler = "clang",
    )
