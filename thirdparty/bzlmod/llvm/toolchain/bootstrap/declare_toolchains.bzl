load("@bazel_features//:features.bzl", "bazel_features")
load("@llvm-project//:vars.bzl", "LLVM_VERSION_MAJOR")
load("@rules_cc//cc/toolchains:tool.bzl", "cc_tool")
load("@rules_cc//cc/toolchains:tool_map.bzl", "cc_tool_map")
load("//platforms:common.bzl", "SUPPORTED_TARGETS")
load("//toolchain:cc_toolchain.bzl", "cc_toolchain")
load(":bootstrap_binary.bzl", "bootstrap_binary", "bootstrap_directory")

def _validate_static_library_tool(prefix):
    if not bazel_features.cc.supports_starlarkified_toolchains:
        return {}

    return {
        "@rules_cc//cc/toolchains/actions:validate_static_library": prefix + "/static-library-validator",
    }

def _exec_prefix(exec_os, exec_cpu):
    return "%s_%s" % (exec_os, exec_cpu)

def _exec_platform_name(exec_os, exec_cpu):
    return _exec_prefix(exec_os, exec_cpu) + "_platform"

def _declare_exec_platform(exec_os, exec_cpu):
    native.platform(
        name = _exec_platform_name(exec_os, exec_cpu),
        constraint_values = [
            "@platforms//cpu:" + exec_cpu,
            "@platforms//os:" + exec_os,
        ],
    )

def _bootstrap_cc_tool(prefix, tool, bootstrap_binary_kwargs, *, capabilities = [], data = [], symlink = True):
    binary = prefix + "/bin/" + tool
    bootstrap_binary(
        name = binary,
        actual = "@llvm-project//llvm:llvm.stripped",
        symlink = symlink,
        **bootstrap_binary_kwargs
    )
    cc_tool(
        name = prefix + "/" + tool,
        src = binary,
        capabilities = capabilities,
        data = data,
    )

def declare_tool_map(exec_os, exec_cpu, prefix = None, fdo_profile = None, fdo_instrumented = False):
    if not prefix:
        prefix = _exec_prefix(exec_os, exec_cpu)

    platform_name = _exec_platform_name(exec_os, exec_cpu)
    bootstrap_binary_kwargs = {
        "fdo_profile": fdo_profile,
        "platform": platform_name,
        "fdo_instrumented": fdo_instrumented,
        "visibility": ["//visibility:public"],
    }

    TOOLS_WITHOUT_LINKER = {
        "@rules_cc//cc/toolchains/actions:assembly_actions": prefix + "/clang",
        "@rules_cc//cc/toolchains/actions:c_compile": prefix + "/clang",
        "@rules_cc//cc/toolchains/actions:gcov": prefix + "/gcov",
        "@rules_cc//cc/toolchains/actions:llvm_cov": prefix + "/llvm-cov",
        "@rules_cc//cc/toolchains/actions:llvm_profdata": prefix + "/llvm-profdata",
        "@rules_cc//cc/toolchains/actions:objc_compile": prefix + "/clang",
        "@llvm//toolchain:cpp_compile_actions_without_header_parsing": prefix + "/clang++",
        "@rules_cc//cc/toolchains/actions:dwp": prefix + "/llvm-dwp",
        "@rules_cc//cc/toolchains/actions:objcopy_embed_data": prefix + "/llvm-objcopy",
        "@rules_cc//cc/toolchains/actions:strip": prefix + "/llvm-strip",
    }
    BASE_TOOLS = TOOLS_WITHOUT_LINKER | {
        "@rules_cc//cc/toolchains/actions:link_actions": prefix + "/lld",
    }

    COMPLETE_ONLY_TOOLS = {
        "@rules_cc//cc/toolchains/actions:cpp_header_parsing": prefix + "/header-parser",
    } | _validate_static_library_tool(prefix)

    cc_tool_map(
        name = prefix + "/default_tools",
        tools = BASE_TOOLS | COMPLETE_ONLY_TOOLS | {
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-ar",
        },
    )

    cc_tool_map(
        name = prefix + "/tools_with_interface_libraries",
        tools = TOOLS_WITHOUT_LINKER | COMPLETE_ONLY_TOOLS | {
            "@rules_cc//cc/toolchains/actions:link_executable_actions": prefix + "/lld",
            "@rules_cc//cc/toolchains/actions:dynamic_library_link_actions": prefix + "/link-wrapper",
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-ar",
        },
    )

    cc_tool_map(
        name = prefix + "/tools_with_libtool",
        tools = BASE_TOOLS | COMPLETE_ONLY_TOOLS | {
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-libtool-darwin",
        },
    )

    cc_tool_map(
        name = prefix + "/tools_with_dsym",
        tools = BASE_TOOLS | COMPLETE_ONLY_TOOLS | {
            "@rules_cc//cc/toolchains/actions:link_actions": prefix + "/link-wrapper",
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-ar",
        },
    )

    cc_tool_map(
        name = prefix + "/tools_with_dsym_and_libtool",
        tools = BASE_TOOLS | COMPLETE_ONLY_TOOLS | {
            "@rules_cc//cc/toolchains/actions:link_actions": prefix + "/link-wrapper",
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-libtool-darwin",
        },
    )

    cc_tool_map(
        name = prefix + "/staged_default_tools",
        tools = BASE_TOOLS | {
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-ar",
        },
    )

    cc_tool_map(
        name = prefix + "/staged_tools_with_libtool",
        tools = BASE_TOOLS | {
            "@rules_cc//cc/toolchains/actions:ar_actions": prefix + "/llvm-libtool-darwin",
        },
    )

    native.alias(
        name = prefix + "/default_tools_for_runtime",
        actual = select({
            "@llvm//toolchain:runtimes_all": prefix + "/default_tools",
            "//conditions:default": prefix + "/staged_default_tools",
        }),
    )

    native.alias(
        name = prefix + "/tools_with_libtool_for_runtime",
        actual = select({
            "@llvm//toolchain:runtimes_all": prefix + "/tools_with_libtool",
            "//conditions:default": prefix + "/staged_tools_with_libtool",
        }),
    )

    bootstrap_directory(
        name = prefix + "/clang_builtin_headers_include_directory",
        srcs = "@llvm-project//clang:builtin_headers_files",
        # TODO(zbarsky): Probably shouldn't force platform here.
        platform = platform_name,
        destination = prefix + "/lib/clang/{}/include".format(LLVM_VERSION_MAJOR),
        strip_prefix = "clang/lib/Headers",
    )

    _bootstrap_cc_tool(
        prefix,
        "clang",
        bootstrap_binary_kwargs,
        data = [
            prefix + "/clang_builtin_headers_include_directory",
        ],
        capabilities = ["@rules_cc//cc/toolchains/capabilities:supports_pic"],
    )

    _bootstrap_cc_tool(
        prefix,
        "clang++",
        bootstrap_binary_kwargs,
        # Copy instead of symlink so clang's InstalledDir matches the packaged tree.
        # This is crucial for properly locating the various linkers, since we don't use `-ld-path`.
        symlink = False,
        data = [
            prefix + "/clang_builtin_headers_include_directory",
        ],
        capabilities = ["@rules_cc//cc/toolchains/capabilities:supports_pic"],
    )

    bootstrap_binary(
        name = prefix + "/bin/header-parser",
        actual = "@llvm//tools/internal:header-parser",
        **bootstrap_binary_kwargs
    )

    cc_tool(
        name = prefix + "/header-parser",
        src = prefix + "/bin/header-parser",
        data = [
            prefix + "/clang_builtin_headers_include_directory",
            prefix + "/bin/clang++",
        ],
        env = {
            "LLVM_CLANGXX": "{clangxx}",
        },
        format = {
            "clangxx": prefix + "/bin/clang++",
        },
    )

    for tool in [
        "llvm",
        "llvm-ifs",
        "llvm-nm",
        "llvm-readtapi",
    ]:
        bootstrap_binary(
            name = prefix + "/bin/" + tool,
            actual = "@llvm-project//llvm:llvm.stripped",
            **bootstrap_binary_kwargs
        )

    bootstrap_binary(
        name = prefix + "/bin/c++filt",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    bootstrap_binary(
        name = prefix + "/bin/dsymutil",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    bootstrap_binary(
        name = prefix + "/bin/static-library-validator",
        actual = "@llvm//tools/internal:static-library-validator",
        **bootstrap_binary_kwargs
    )

    cc_tool(
        name = prefix + "/static-library-validator",
        src = prefix + "/bin/static-library-validator",
        data = [
            prefix + "/bin/c++filt",
            prefix + "/bin/llvm-nm",
        ],
        env = {
            "LLVM_CXXFILT": "{cxxfilt}",
            "LLVM_NM": "{llvm_nm}",
        },
        format = {
            "cxxfilt": prefix + "/bin/c++filt",
            "llvm_nm": prefix + "/bin/llvm-nm",
        },
    )

    bootstrap_binary(
        name = prefix + "/bin/ld.lld",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    bootstrap_binary(
        name = prefix + "/bin/ld64.lld",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    bootstrap_binary(
        name = prefix + "/bin/lld",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    bootstrap_binary(
        name = prefix + "/bin/wasm-ld",
        actual = "@llvm-project//llvm:llvm.stripped",
        **bootstrap_binary_kwargs
    )

    cc_tool(
        name = prefix + "/lld",
        src = prefix + "/bin/clang++",
        data = [
            prefix + "/bin/ld.lld",
            prefix + "/bin/ld64.lld",
            prefix + "/bin/lld",
            prefix + "/bin/wasm-ld",
        ],
        capabilities = ["@rules_cc//cc/toolchains/capabilities:supports_start_end_lib"],
    )

    # Preserve the `-clang++` suffix for rustc's linker-flavor inference.
    bootstrap_binary(
        name = prefix + "/bin/link-wrapper-clang++",
        actual = "@llvm//tools/internal:link-wrapper",
        **bootstrap_binary_kwargs
    )

    cc_tool(
        name = prefix + "/link-wrapper",
        src = prefix + "/bin/link-wrapper-clang++",
        data = [
            prefix + "/bin/clang++",
            prefix + "/bin/dsymutil",
            prefix + "/bin/llvm-strip",
            prefix + "/bin/ld.lld",
            prefix + "/bin/ld64.lld",
            prefix + "/bin/lld",
            prefix + "/bin/llvm-ifs",
            prefix + "/bin/llvm-nm",
            prefix + "/bin/llvm-readtapi",
            prefix + "/bin/wasm-ld",
        ],
        capabilities = [
            "@rules_cc//cc/toolchains/capabilities:has_configured_linker_path",
            "@rules_cc//cc/toolchains/capabilities:supports_interface_shared_libraries",
            "@rules_cc//cc/toolchains/capabilities:supports_start_end_lib",
        ],
        env = {
            "COMPILER_PATH": "{llvm}-",
            "LLVM_CLANGXX": "{clangxx}",
            "LLVM_DSYMUTIL": "{dsymutil}",
            "LLVM_IFS": "{llvm_ifs}",
            "LLVM_NM": "{llvm_nm}",
            "LLVM_READTAPI": "{llvm_readtapi}",
            "LLVM_STRIP": "{strip}",
        },
        format = {
            "clangxx": prefix + "/bin/clang++",
            "dsymutil": prefix + "/bin/dsymutil",
            "llvm": prefix + "/bin/llvm",
            "llvm_ifs": prefix + "/bin/llvm-ifs",
            "llvm_nm": prefix + "/bin/llvm-nm",
            "llvm_readtapi": prefix + "/bin/llvm-readtapi",
            "strip": prefix + "/bin/llvm-strip",
        },
    )

    for tool in [
        "llvm-ar",
        "llvm-libtool-darwin",
        "llvm-dwp",
        "gcov",
        "llvm-cov",
        "llvm-profdata",
        "llvm-objcopy",
    ]:
        _bootstrap_cc_tool(prefix, tool, bootstrap_binary_kwargs)

    _bootstrap_cc_tool(
        prefix,
        "llvm-strip",
        bootstrap_binary_kwargs,
        # TODO: Remove this once rules_cc includes validate_static_library in
        # all_files, or cc_static_library uses the validate action's files
        # directly. This hangs validator files off strip because strip is an
        # exec-configured tool already included in rules_cc 0.2.18's legacy
        # file groups.
        data = select({
            "@llvm//toolchain:runtimes_all": [
                prefix + "/bin/static-library-validator",
            ],
            "//conditions:default": [],
        }) + [
            prefix + "/bin/c++filt",
            prefix + "/bin/llvm-nm",
            # CcToolchainInfo.all_files only includes rules_cc's
            # LEGACY_FILE_GROUPS. Include llvm-profdata for
            # LLVMFDOProfileMerge.
            prefix + "/bin/llvm-profdata",
        ],
    )

def declare_toolchains(*, execs = None, targets = SUPPORTED_TARGETS):
    """Declares the configured LLVM toolchains.

    Args:
        execs: List of (os, arch) tuples describing exec platforms.
        targets: List of (os, arch) tuples describing target platforms.
    """
    if not execs:
        execs = [
            (arch, os)
            # Any supported target that can run a compiler is a supported exec.
            # If we can compile a compiler for that target, we can use that compiler
            # to compile for any other target.
            for (arch, os) in targets
            if arch != "none"  # wasm is no good for us.
        ]

    for (exec_os, exec_cpu) in execs:
        exec_prefix = _exec_prefix(exec_os, exec_cpu)
        stage1_prefix = "stage1_" + exec_prefix
        stage2_prefix = "stage2_" + exec_prefix
        stage3_prefix = "stage3_" + exec_prefix

        _declare_exec_platform(exec_os, exec_cpu)
        declare_tool_map(
            exec_os,
            exec_cpu,
            prefix = stage3_prefix,
            fdo_profile = "//toolchain/bootstrap/stage3:llvm_fdo_profdata",
        )
        declare_tool_map(
            exec_os,
            exec_cpu,
            prefix = stage2_prefix,
            fdo_instrumented = True,
        )
        declare_tool_map(
            exec_os,
            exec_cpu,
            prefix = stage1_prefix,
        )

        for stage_name, tool_prefix, target_setting in [
            ("stage3", stage3_prefix, "@llvm//toolchain:bootstrap_stage3_lto_and_fdo_applied"),
            ("stage2", stage2_prefix, "@llvm//toolchain:bootstrap_stage2_lto_and_fdo_instrumented"),
            ("stage1", stage1_prefix, "@llvm//toolchain:bootstrap_stage1_from_source"),
        ]:
            cc_toolchain_name = "%s_%s_%s_cc_toolchain" % (stage_name, exec_os, exec_cpu)

            # Even though `tool_map` has an exec transition, Bazel doesn't properly handle
            # binding a single `cc_toolchain` to multiple toolchains with different `exec_compatible_with`.
            # See https://github.com/bazelbuild/rules_cc/issues/299#issuecomment-2660340534
            cc_toolchain(
                name = cc_toolchain_name,
                tool_map = select({
                    "@llvm//toolchain:linux_complete": ":%s/tools_with_interface_libraries" % tool_prefix,
                    "@llvm//toolchain:macos_complete_with_libtool": ":%s/tools_with_dsym_and_libtool" % tool_prefix,
                    "@llvm//toolchain:macos_complete": ":%s/tools_with_dsym" % tool_prefix,
                    "@rules_cc//cc/toolchains/args/archiver_flags:use_libtool_on_apple_setting": ":%s/tools_with_libtool_for_runtime" % tool_prefix,
                    "//conditions:default": ":%s/default_tools_for_runtime" % tool_prefix,
                }),
            )

            for (target_os, target_cpu) in targets:
                native.toolchain(
                    name = "%s_%s_%s_to_%s_%s" % (stage_name, exec_os, exec_cpu, target_os, target_cpu),
                    exec_compatible_with = [
                        "@platforms//cpu:" + exec_cpu,
                        "@platforms//os:" + exec_os,
                    ],
                    target_compatible_with = [
                        "@platforms//cpu:" + target_cpu,
                        "@platforms//os:" + target_os,
                    ],
                    target_settings = [
                        target_setting,
                    ],
                    toolchain = cc_toolchain_name,
                    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
                    visibility = ["//visibility:public"],
                )
