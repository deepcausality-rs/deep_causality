# Local autoconf-style check executor.
#
# This rule consumes JSON check declarations from checks.bzl and emits ordered
# result metadata for header renderers. It is intentionally generic so the
# libstdc++ source-counterpart files only declare GCC configure checks.

load("@llvm//toolchain/runtimes:with_cfg_runtimes_common.bzl", "configure_builder_for_runtimes")
load("@with_cfg.bzl", "with_cfg")
load(
    ":cc_configure_probe.bzl",
    "cc_configure_probe_context",
    "cc_configure_probe_toolchains",
    "declare_compile_probe",
    "declare_link_probe",
    "policy_result",
)
load(":providers.bzl", "AutoconfConfigInfo")

def _as_struct(value):
    return struct(**value)

def _probe_context_include_root(files, marker, context_name):
    marker_dir = "/".join(marker.split("/")[:-1])
    for file in files:
        if not file.short_path.endswith(marker) and not file.path.endswith(marker):
            continue

        if not marker_dir:
            return file.dirname

        suffix = "/" + marker_dir
        if file.dirname.endswith(suffix):
            return file.dirname[:-len(suffix)]
        if file.dirname == marker_dir:
            return "."

        fail("Probe context '{}' marker '{}' matched file '{}', but its directory '{}' does not end with '{}'.".format(
            context_name,
            marker,
            file.path,
            file.dirname,
            marker_dir,
        ))

    fail("Probe context '{}' did not contain marker file '{}'.".format(context_name, marker))

def _probe_extra_inputs_and_flags(ctx, check):
    names = getattr(check, "probe_contexts", [])
    if not names:
        return ([], [])

    inputs = []
    include_roots = []
    for name in names:
        if name not in ctx.attr.probe_contexts:
            fail("Check '{}' references unknown probe context '{}'.".format(check.name, name))

        files = ctx.attr.probe_contexts[name].files.to_list()
        if not files:
            fail("Probe context '{}' has no files.".format(name))
        inputs.extend(files)

        marker = ctx.attr.probe_context_include_root_markers.get(name)
        if marker:
            include_roots.append(_probe_context_include_root(files, marker, name))
        else:
            include_roots.extend([file.dirname for file in files])

    deduped_include_roots = []
    seen = {}
    for include_root in include_roots:
        if include_root in seen:
            continue
        seen[include_root] = True
        deduped_include_roots.append(include_root)

    return (inputs, ["-I" + include_root for include_root in deduped_include_roots])

def _autoconf_config_impl(ctx):
    probe_context = cc_configure_probe_context(
        ctx = ctx,
        source_placeholder = "__autoconf_probe_source__.c",
        object_placeholder = "__autoconf_probe_output__.o",
        binary_placeholder = "__autoconf_probe_binary__",
    )

    results = []
    result_files = []
    for check_json in ctx.attr.checks:
        check = _as_struct(json.decode(check_json))
        check_type = check.type
        extra_inputs, extra_flags = _probe_extra_inputs_and_flags(ctx, check)
        if check_type == "compile":
            result = declare_compile_probe(
                ctx = ctx,
                probe_context = probe_context,
                check = check,
                extra_inputs = extra_inputs,
                extra_flags = extra_flags,
            )
        elif check_type == "link":
            result = declare_link_probe(
                ctx = ctx,
                probe_context = probe_context,
                check = check,
                extra_inputs = extra_inputs,
                compile_extra_flags = extra_flags,
            )
        elif check_type in ["define", "string_define", "undef"]:
            result = policy_result(check)
        else:
            fail("Check '{}' has unsupported autoconf check type '{}'.".format(check.name, check_type))

        results.append(result)
        if result.result:
            result_files.append(result.result)

    return [
        AutoconfConfigInfo(results = results),
        OutputGroupInfo(autoconf_results = depset(result_files)),
    ]

_autoconf_config = rule(
    implementation = _autoconf_config_impl,
    attrs = {
        "checks": attr.string_list(
            doc = "JSON-encoded checks from //3rd_party/gcc/libstdcxx/autoconf:checks.bzl.",
            default = [],
        ),
        "probe_context_include_root_markers": attr.string_dict(
            doc = "Marker files used to derive include roots for named probe contexts.",
            default = {},
        ),
        "probe_contexts": attr.string_keyed_label_dict(
            doc = "Named file sets that individual checks may opt into.",
            allow_files = True,
        ),
    },
    fragments = ["cpp"],
    toolchains = cc_configure_probe_toolchains,
)

_autoconf_config_builder = with_cfg(
    _autoconf_config,
    extra_providers = [AutoconfConfigInfo],
)

autoconf_config, _autoconf_config_internal = configure_builder_for_runtimes(
    _autoconf_config_builder,
    "stage1_hosted",
).build()
