# Generic CC compile/link probe execution for autoconf-style checks.
# Keep this file free of runtime policy, generated header, and config.h
# emission details; callers own those decisions.

load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc:find_cc_toolchain.bzl", "CC_TOOLCHAIN_TYPE", "find_cc_toolchain", "use_cc_toolchain")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")

cc_configure_probe_toolchains = use_cc_toolchain()

def _compile_template(cc_toolchain, feature_configuration, action_name, source_file, output_file, user_compile_flags):
    compile_variables = cc_common.create_compile_variables(
        feature_configuration = feature_configuration,
        cc_toolchain = cc_toolchain,
        user_compile_flags = user_compile_flags,
        source_file = source_file,
        output_file = output_file,
    )
    return struct(
        command_line = cc_common.get_memory_inefficient_command_line(
            feature_configuration = feature_configuration,
            action_name = action_name,
            variables = compile_variables,
        ),
        env = cc_common.get_environment_variables(
            feature_configuration = feature_configuration,
            action_name = action_name,
            variables = compile_variables,
        ),
        tool = cc_common.get_tool_for_action(
            feature_configuration = feature_configuration,
            action_name = action_name,
        ),
    )

def _link_template(cc_toolchain, feature_configuration, output_file):
    link_variables = cc_common.create_link_variables(
        feature_configuration = feature_configuration,
        cc_toolchain = cc_toolchain,
        output_file = output_file,
        is_using_linker = True,
        is_linking_dynamic_library = False,
    )
    return struct(
        command_line = cc_common.get_memory_inefficient_command_line(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.cpp_link_executable,
            variables = link_variables,
        ),
        env = cc_common.get_environment_variables(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.cpp_link_executable,
            variables = link_variables,
        ),
        tool = cc_common.get_tool_for_action(
            feature_configuration = feature_configuration,
            action_name = ACTION_NAMES.cpp_link_executable,
        ),
    )

def cc_configure_probe_context(ctx, source_placeholder, object_placeholder, binary_placeholder):
    cc_toolchain = find_cc_toolchain(ctx)
    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
    )
    return struct(
        binary_placeholder = binary_placeholder,
        cc_toolchain = cc_toolchain,
        compile_templates = {
            "c": _compile_template(
                cc_toolchain = cc_toolchain,
                feature_configuration = feature_configuration,
                action_name = ACTION_NAMES.c_compile,
                source_file = source_placeholder,
                output_file = object_placeholder,
                user_compile_flags = ctx.fragments.cpp.copts + ctx.fragments.cpp.conlyopts + [
                    "-Werror=implicit-function-declaration",
                ],
            ),
            "c++": _compile_template(
                cc_toolchain = cc_toolchain,
                feature_configuration = feature_configuration,
                action_name = ACTION_NAMES.cpp_compile,
                source_file = source_placeholder,
                output_file = object_placeholder,
                user_compile_flags = ctx.fragments.cpp.copts + ctx.fragments.cpp.cxxopts + [
                    "-nostdinc++",
                ],
            ),
        },
        link_template = _link_template(
            cc_toolchain = cc_toolchain,
            feature_configuration = feature_configuration,
            output_file = binary_placeholder,
        ),
        object_placeholder = object_placeholder,
        source_placeholder = source_placeholder,
    )

def _declare_source(ctx, check):
    stem = ctx.attr.name + "_" + check.name.lower()
    extension = ".cc" if check.language == "c++" else ".c"
    source = ctx.actions.declare_file(stem + extension)
    ctx.actions.write(
        output = source,
        content = check.source,
    )
    return source

def declare_compile_probe(ctx, probe_context, check, extra_inputs = [], extra_flags = []):
    source = _declare_source(ctx, check)
    stem = ctx.attr.name + "_" + check.name.lower()
    result = ctx.actions.declare_file(stem + ".result")
    log = ctx.actions.declare_file(stem + ".log")
    template = probe_context.compile_templates[check.language]

    # TODO(corentin): replace this shell runner with a portable probe helper
    # binary once the command-line substitution contract is stable.
    ctx.actions.run_shell(
        inputs = depset(
            direct = [source] + extra_inputs,
        ),
        outputs = [result, log],
        tools = probe_context.cc_toolchain.all_files,
        arguments = [
            template.tool,
            source.path,
            result.path,
            log.path,
            probe_context.source_placeholder,
            probe_context.object_placeholder,
        ] + check.flags + extra_flags + ["--"] + template.command_line,
        env = template.env,
        command = """set -eu
tool="$1"
source="$2"
result="$3"
log="$4"
source_placeholder="$5"
output_placeholder="$6"
shift 6
extra_flags=()
while [ "$#" -gt 0 ]; do
    if [ "$1" = "--" ]; then
        shift
        break
    fi
    extra_flags+=("$1")
    shift
done

tmp="${TMPDIR:-/tmp}/cc-configure-probe-$$"
mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT
object="$tmp/probe.o"

cmd=("$tool")
for arg in "$@"; do
    case "$arg" in
        "$source_placeholder")
            cmd+=("${extra_flags[@]}")
            cmd+=("$source")
            ;;
        "$output_placeholder")
            cmd+=("$object")
            ;;
        *)
            cmd+=("$arg")
            ;;
    esac
done

if "${cmd[@]}" >"$log" 2>&1; then
    echo true > "$result"
else
    echo false > "$result"
fi
""",
        mnemonic = "CcConfigureCompileProbe",
        toolchain = CC_TOOLCHAIN_TYPE,
    )
    return struct(
        check = check,
        kind = "compile",
        result = result,
    )

def declare_link_probe(ctx, probe_context, check, extra_inputs = [], compile_extra_flags = []):
    source = _declare_source(ctx, check)
    stem = ctx.attr.name + "_" + check.name.lower()
    result = ctx.actions.declare_file(stem + ".result")
    log = ctx.actions.declare_file(stem + ".log")
    compile_template = probe_context.compile_templates[check.language]
    link_template = probe_context.link_template

    # TODO(corentin): replace this shell runner with a portable probe helper
    # binary once the command-line substitution contract is stable.
    ctx.actions.run_shell(
        inputs = depset(
            direct = [source] + extra_inputs,
        ),
        outputs = [result, log],
        tools = probe_context.cc_toolchain.all_files,
        arguments = [
            compile_template.tool,
            link_template.tool,
            source.path,
            result.path,
            log.path,
            probe_context.source_placeholder,
            probe_context.object_placeholder,
            probe_context.binary_placeholder,
        ] + check.compile_flags + compile_extra_flags + ["--"] + compile_template.command_line + ["--"] + check.link_flags + ["--"] + link_template.command_line,
        env = compile_template.env | link_template.env,
        command = """set -eu
compile_tool="$1"
link_tool="$2"
source="$3"
result="$4"
log="$5"
source_placeholder="$6"
object_placeholder="$7"
binary_placeholder="$8"
shift 8
compile_extra_flags=()
while [ "$#" -gt 0 ]; do
    if [ "$1" = "--" ]; then
        shift
        break
    fi
    compile_extra_flags+=("$1")
    shift
done
compile_args=()
while [ "$#" -gt 0 ]; do
    if [ "$1" = "--" ]; then
        shift
        break
    fi
    compile_args+=("$1")
    shift
done
link_extra_flags=()
while [ "$#" -gt 0 ]; do
    if [ "$1" = "--" ]; then
        shift
        break
    fi
    link_extra_flags+=("$1")
    shift
done

tmp="${TMPDIR:-/tmp}/cc-configure-link-probe-$$"
mkdir -p "$tmp"
trap 'rm -rf "$tmp"' EXIT
object="$tmp/probe.o"
binary="$tmp/probe.exe"

compile_cmd=("$compile_tool")
for arg in "${compile_args[@]}"; do
    case "$arg" in
        "$source_placeholder")
            compile_cmd+=("${compile_extra_flags[@]}")
            compile_cmd+=("$source")
            ;;
        "$object_placeholder")
            compile_cmd+=("$object")
            ;;
        *)
            compile_cmd+=("$arg")
            ;;
    esac
done

link_cmd=("$link_tool")
for arg in "$@"; do
    case "$arg" in
        "$binary_placeholder")
            link_cmd+=("$binary")
            ;;
        *)
            link_cmd+=("$arg")
            ;;
    esac
done
link_cmd+=("$object")
link_cmd+=("${link_extra_flags[@]}")

if "${compile_cmd[@]}" >"$log" 2>&1 && "${link_cmd[@]}" >>"$log" 2>&1; then
    echo true > "$result"
else
    echo false > "$result"
fi
""",
        mnemonic = "CcConfigureLinkProbe",
        toolchain = CC_TOOLCHAIN_TYPE,
    )
    return struct(
        check = check,
        kind = "link",
        result = result,
    )

def policy_result(policy):
    return struct(
        check = policy,
        kind = policy.type,
        result = None,
    )
