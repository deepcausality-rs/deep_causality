"""Rules for generating an LLVM bootstrap FDO profile."""

load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc:find_cc_toolchain.bzl", "CC_TOOLCHAIN_TYPE", "find_cc_toolchain", "use_cc_toolchain")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/private/rules_impl/fdo:fdo_profile.bzl", "FdoProfileInfo")  # buildifier: disable=bzl-visibility
load(":transition_settings.bzl", "LLVM_TOOLS", "SANITIZER_FLAGS", "disable_sanitizers")

LLVMFDOProfileRawInfo = provider(
    doc = "Raw LLVM profiles produced for one target platform.",
    fields = {
        "profraws": "Depset of raw LLVM profile files.",
    },
)

_COMMON_COMPILE_FLAGS = [
    "-x",
    "c",
    "-O3",
    "-fomit-frame-pointer",
    "-ffunction-sections",
    "-fdata-sections",
]

_HOSTED_COMPILE_FLAGS = [
    "-flto=thin",
    "-pthread",
    "-DZSTD_DISABLE_ASM",
    "-DZSTD_MULTITHREAD",
    "-DZSTD_NOBENCH",
    "-DZSTD_NODICT",
    "-DZSTD_NODECOMPRESS",
    "-DZSTD_NOTRACE",
    "-UZSTD_LEGACY_SUPPORT",
    "-DZSTD_LEGACY_SUPPORT=0",
]

_HOSTED_LINK_FLAGS = [
    "-O3",
    "-flto=thin",
    "-pthread",
]

_FREESTANDING_COMPILE_FLAGS = [
    "-ffreestanding",
    "-fno-builtin",
    "-nostdinc",
]

def _profile_generation_transition_impl(_settings, attr):
    transition_settings = {
        "//command_line_option:fdo_profile": None,
        "//command_line_option:platforms": str(attr.target_platform),
        "//toolchain:runtime_stage": "complete",
        "//toolchain:bootstrap_stage": "stage2_lto_and_fdo_instrumented",
        "@llvm-project//llvm:driver-tools": LLVM_TOOLS,
    }

    disable_sanitizers(transition_settings)

    return transition_settings

_profile_generation_transition = transition(
    implementation = _profile_generation_transition_impl,
    inputs = [],
    outputs = [
        "//command_line_option:fdo_profile",
        "//command_line_option:platforms",
        "//toolchain:runtime_stage",
        "//toolchain:bootstrap_stage",
        "@llvm-project//llvm:driver-tools",
    ] + SANITIZER_FLAGS,
)

def _profile_merge_transition_impl(_settings, _attr):
    return {
        "//command_line_option:fdo_profile": None,
        "//toolchain:bootstrap_stage": "stage1_from_source",
    }

_profile_merge_transition = transition(
    implementation = _profile_merge_transition_impl,
    inputs = [],
    outputs = [
        "//command_line_option:fdo_profile",
        "//toolchain:bootstrap_stage",
    ],
)

def _c_sources(files):
    return [file for file in files if file.extension == "c"]

def _profile_environment(feature_configuration, action_name, variables, profraw):
    env = dict(cc_common.get_environment_variables(
        feature_configuration = feature_configuration,
        action_name = action_name,
        variables = variables,
    ))
    env["LLVM_PROFILE_FILE"] = profraw.path
    return env

def _llvm_fdo_profile_workload_impl(ctx):
    cc_toolchain = find_cc_toolchain(ctx)

    if ctx.attr.workload_kind == "hosted":
        compile_flags = _COMMON_COMPILE_FLAGS + _HOSTED_COMPILE_FLAGS
    else:
        compile_flags = _COMMON_COMPILE_FLAGS + _FREESTANDING_COMPILE_FLAGS

    include_dirs = {}
    for file in ctx.files.srcs:
        include_dirs[file.dirname] = None

    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
        requested_features = ctx.features,
        unsupported_features = ctx.disabled_features,
    )
    compile_executable = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = ACTION_NAMES.c_compile,
    )
    profraws = []
    objects = []
    for index, source in enumerate(_c_sources(ctx.files.srcs)):
        object_file = ctx.actions.declare_file("%s.%s.o" % (ctx.label.name, index))
        profraw = ctx.actions.declare_file("%s.%s.profraw" % (ctx.label.name, index))
        compile_variables = cc_common.create_compile_variables(
            cc_toolchain = cc_toolchain,
            feature_configuration = feature_configuration,
            include_directories = depset(sorted(include_dirs.keys())),
            output_file = object_file.path,
            source_file = source.path,
            user_compile_flags = compile_flags,
        )

        ctx.actions.run(
            executable = compile_executable,
            arguments = cc_common.get_memory_inefficient_command_line(
                feature_configuration = feature_configuration,
                action_name = ACTION_NAMES.c_compile,
                variables = compile_variables,
            ),
            env = _profile_environment(feature_configuration, ACTION_NAMES.c_compile, compile_variables, profraw),
            inputs = depset(
                [source],
                transitive = [ctx.attr.srcs[DefaultInfo].files],
            ),
            tools = cc_toolchain.all_files,
            outputs = [
                object_file,
                profraw,
            ],
            mnemonic = "LLVMFDOProfileCompile",
            progress_message = "Generating LLVM FDO profile input with %{label}",
            execution_requirements = {"supports-path-mapping": "1"},
            toolchain = CC_TOOLCHAIN_TYPE,
        )

        objects.append(object_file)
        profraws.append(profraw)

    if ctx.attr.workload_kind == "freestanding":
        profraws_depset = depset(profraws)
        return [
            DefaultInfo(files = profraws_depset),
            LLVMFDOProfileRawInfo(profraws = profraws_depset),
        ]

    binary_file = ctx.actions.declare_file(ctx.label.name + ".zstd")
    link_variables = cc_common.create_link_variables(
        cc_toolchain = cc_toolchain,
        feature_configuration = feature_configuration,
        output_file = binary_file.path,
        user_link_flags = _HOSTED_LINK_FLAGS,
    )
    link_args = ctx.actions.args()
    link_args.add_all(cc_common.get_memory_inefficient_command_line(
        feature_configuration = feature_configuration,
        action_name = ACTION_NAMES.cpp_link_executable,
        variables = link_variables,
    ))
    link_args.add_all(objects)

    link_profraw = ctx.actions.declare_file(ctx.label.name + ".link.profraw")
    link_executable = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = ACTION_NAMES.cpp_link_executable,
    )
    ctx.actions.run(
        executable = link_executable,
        arguments = [link_args],
        env = _profile_environment(feature_configuration, ACTION_NAMES.cpp_link_executable, link_variables, link_profraw),
        inputs = objects,
        tools = cc_toolchain.all_files,
        outputs = [
            binary_file,
            link_profraw,
        ],
        mnemonic = "LLVMFDOProfileLink",
        progress_message = "Linking LLVM FDO training binary with %{label}",
        execution_requirements = {"supports-path-mapping": "1"},
        toolchain = CC_TOOLCHAIN_TYPE,
    )
    profraws.append(link_profraw)

    profraws_depset = depset(profraws)
    return [
        DefaultInfo(files = profraws_depset),
        LLVMFDOProfileRawInfo(profraws = profraws_depset),
    ]

llvm_fdo_profile_workload = rule(
    implementation = _llvm_fdo_profile_workload_impl,
    attrs = {
        "srcs": attr.label(
            allow_files = [".c", ".h"],
            mandatory = True,
            doc = "Training sources compiled for target_platform.",
        ),
        "target_platform": attr.label(
            mandatory = True,
            doc = "Target platform compiled by this training workload.",
        ),
        "workload_kind": attr.string(
            mandatory = True,
            values = [
                "freestanding",
                "hosted",
            ],
            doc = "Whether this workload compiles only or compiles and links.",
        ),
    },
    cfg = _profile_generation_transition,
    fragments = ["cpp"],
    toolchains = use_cc_toolchain(),
)

def _llvm_fdo_profile_data_impl(ctx):
    cc_toolchain = find_cc_toolchain(ctx)
    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
        requested_features = ctx.features,
        unsupported_features = ctx.disabled_features,
    )
    llvm_profdata = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = ACTION_NAMES.llvm_profdata,
    )
    profdata = ctx.actions.declare_file(ctx.label.name + ".profdata")
    profraws = depset(transitive = [
        profile[LLVMFDOProfileRawInfo].profraws
        for profile in ctx.attr.profiles
    ])

    merge_args = ctx.actions.args()
    merge_args.add("merge")
    merge_args.add("--output")
    merge_args.add(profdata)
    merge_args.add_all(profraws)

    ctx.actions.run(
        executable = llvm_profdata,
        arguments = [merge_args],
        inputs = profraws,
        tools = cc_toolchain.all_files,
        outputs = [profdata],
        mnemonic = "LLVMFDOProfileMerge",
        progress_message = "Merging LLVM FDO profiles for %{label}",
        execution_requirements = {"supports-path-mapping": "1"},
        toolchain = CC_TOOLCHAIN_TYPE,
    )

    return [
        DefaultInfo(files = depset([profdata])),
        FdoProfileInfo(
            artifact = profdata,
            proto_profile_artifact = None,
            memprof_artifact = None,
        ),
    ]

llvm_fdo_profile_data = rule(
    implementation = _llvm_fdo_profile_data_impl,
    attrs = {
        "profiles": attr.label_list(
            mandatory = True,
            providers = [LLVMFDOProfileRawInfo],
            doc = "Target-platform training workloads merged into this profile.",
        ),
    },
    cfg = _profile_merge_transition,
    fragments = ["cpp"],
    toolchains = use_cc_toolchain(),
)
