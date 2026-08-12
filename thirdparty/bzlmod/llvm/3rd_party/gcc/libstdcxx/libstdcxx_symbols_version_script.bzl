# Generates libstdc++'s linker version script from the same inputs and shell
# recipe used by libstdc++-v3/src/Makefile.am. The selected base map comes from
# GLIBCXX_ENABLE_SYMVERS in libstdc++-v3/acinclude.m4.

load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc:find_cc_toolchain.bzl", "CC_TOOLCHAIN_TYPE", "find_cc_toolchain", "use_cc_toolchain")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")

def _libstdcxx_symbols_version_script_impl(ctx):
    cc_toolchain = find_cc_toolchain(ctx)
    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
    )
    source_placeholder = "__libstdcxx_symbols_source__.ver"
    output_placeholder = "__libstdcxx_symbols_output__.ver"
    config_h_placeholder = "__libstdcxx_symbols_config_h__.h"
    filtered = ctx.actions.declare_file(ctx.attr.name + ".filtered.ver")
    output = ctx.actions.declare_file(ctx.attr.name + ".ver")
    preprocess_action = ACTION_NAMES.preprocess_assemble
    variables = cc_common.create_compile_variables(
        feature_configuration = feature_configuration,
        cc_toolchain = cc_toolchain,
        source_file = source_placeholder,
        output_file = output_placeholder,
        user_compile_flags = [
            "-x",
            "c",
            "-E",
            "-P",
            "-include",
            config_h_placeholder,
        ],
    )
    raw_command_line = cc_common.get_memory_inefficient_command_line(
        feature_configuration = feature_configuration,
        action_name = preprocess_action,
        variables = variables,
    )
    command_line = [
        arg
        for arg in raw_command_line
        # The cc action template is compile-shaped. This action only
        # preprocesses GCC's version script, so keep the cc_common-derived
        # target and include flags but drop the compile-only marker.
        if arg != "-c"
    ]
    env = cc_common.get_environment_variables(
        feature_configuration = feature_configuration,
        action_name = preprocess_action,
        variables = variables,
    )
    compiler = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = preprocess_action,
    )

    # Mirrors the libstdc++-v3/src/Makefile.am libstdc++-symbols.ver recipe:
    # copy the selected SYMVER_FILE, append or splice port-specific fragments,
    # then strip comments before the compiler preprocessor pass below.
    ctx.actions.run_shell(
        inputs = [ctx.file.base_version_script] + ctx.files.port_version_scripts,
        outputs = [filtered],
        arguments = [
            filtered.path,
            ctx.file.base_version_script.path,
        ] + [port.path for port in ctx.files.port_version_scripts],
        command = """set -eu
out="$1"
base="$2"
shift 2
tmp="${out}.tmp"
top="${out}.top"
bottom="${out}.bottom"
trap 'rm -f "$tmp" "$top" "$bottom"' EXIT

cp "$base" "$tmp"
chmod +w "$tmp"
if [ "$#" -gt 0 ]; then
    if grep '^# Appended to version file.' "$@" > /dev/null 2>&1; then
        cat "$@" >> "$tmp"
    else
        sed -n '1,/DO NOT DELETE/p' "$tmp" > "$top"
        sed -n '/DO NOT DELETE/,$p' "$tmp" > "$bottom"
        cat "$top" "$@" "$bottom" > "$tmp"
    fi
fi

grep -E -v '^[	 ]*#(#| |$)' "$tmp" > "$out"
""",
        mnemonic = "LibstdcxxSymbolsVersionScriptAssemble",
    )

    preprocessor_args = ctx.actions.args()
    for arg in command_line:
        if arg == source_placeholder:
            preprocessor_args.add(filtered)
        elif arg == output_placeholder:
            preprocessor_args.add(output)
        elif arg == config_h_placeholder:
            preprocessor_args.add(ctx.file.config_h)
        else:
            preprocessor_args.add(arg)

    ctx.actions.run(
        executable = compiler,
        inputs = depset(
            direct = [filtered, ctx.file.config_h],
            transitive = [cc_toolchain.all_files],
        ),
        outputs = [output],
        arguments = [preprocessor_args],
        env = env,
        mnemonic = "LibstdcxxSymbolsVersionScriptPreprocess",
        toolchain = CC_TOOLCHAIN_TYPE,
    )

    return [DefaultInfo(files = depset([output]))]

# Adapts GCC's libstdc++ symbol version map flow. Compare with
# libstdc++-v3/src/Makefile.am, acinclude.m4's GLIBCXX_ENABLE_SYMVERS, and
# libstdc++-v3/config/abi/pre/*.ver before changing the inputs.
libstdcxx_symbols_version_script = rule(
    implementation = _libstdcxx_symbols_version_script_impl,
    attrs = {
        "base_version_script": attr.label(allow_single_file = True, mandatory = True),
        "config_h": attr.label(allow_single_file = True, mandatory = True),
        "port_version_scripts": attr.label_list(allow_files = True),
    },
    fragments = ["cpp"],
    toolchains = use_cc_toolchain(),
)
