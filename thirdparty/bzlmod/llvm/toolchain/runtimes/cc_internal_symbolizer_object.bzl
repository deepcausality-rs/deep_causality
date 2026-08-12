load("@rules_cc//cc:action_names.bzl", "ACTION_NAMES")
load("@rules_cc//cc:find_cc_toolchain.bzl", "CC_TOOLCHAIN_TYPE", "find_cc_toolchain", "use_cc_toolchain")
load("@rules_cc//cc/common:cc_common.bzl", "cc_common")
load("@rules_cc//cc/common:cc_info.bzl", "CcInfo")

def _internal_symbolizer_libcxx_transition_impl(settings, attr):
    return {
        "//command_line_option:copt": settings["//command_line_option:copt"] + attr.copts,
        "//command_line_option:cxxopt": settings["//command_line_option:cxxopt"] + attr.cxxopts,
        "//command_line_option:platforms": str(attr.platform),
    }

_internal_symbolizer_libcxx_transition = transition(
    implementation = _internal_symbolizer_libcxx_transition_impl,
    inputs = [
        "//command_line_option:copt",
        "//command_line_option:cxxopt",
        "//command_line_option:platforms",
    ],
    outputs = [
        "//command_line_option:copt",
        "//command_line_option:cxxopt",
        "//command_line_option:platforms",
    ],
)

def _link_files(targets):
    archives = {}
    objects = {}
    for target in targets:
        for linker_input in target[CcInfo].linking_context.linker_inputs.to_list():
            for library in linker_input.libraries:
                archive = library.pic_static_library or library.static_library
                if archive:
                    archives[archive.path] = archive
                else:
                    for object_file in library.pic_objects or library.objects:
                        objects[object_file.path] = object_file
    return archives.values(), objects.values()

def _cc_internal_symbolizer_object_impl(ctx):
    if len(ctx.attr.target_triple) != 1:
        fail("target_triple must contain exactly one value")

    archives, objects = _link_files(
        [ctx.attr.symbolizer] + ctx.attr.libcxx + ctx.attr.libcxxabi,
    )
    inputs = archives + objects
    if not inputs:
        fail("internal symbolizer dependency does not contain linkable files")

    cc_toolchain = find_cc_toolchain(ctx)
    feature_configuration = cc_common.configure_features(
        ctx = ctx,
        cc_toolchain = cc_toolchain,
    )
    compiler = cc_common.get_tool_for_action(
        feature_configuration = feature_configuration,
        action_name = ACTION_NAMES.cpp_link_executable,
    )

    bitcode = ctx.actions.declare_file(ctx.label.name + ".bc")
    link_args = ctx.actions.args()
    link_args.add_all([
        "-target",
        ctx.attr.target_triple[0],
        "-fuse-ld=lld",
        "-flto",
        "-nostdlib",
        "-shared",
        "-Wl,--lto-emit-llvm",
        "-Wl,--lto-newpm-passes=internalize",
        "-Xlinker",
        "--plugin-opt=-internalize-public-api-list=" + ",".join(ctx.attr.global_symbols),
    ])
    link_args.add_all(
        ctx.attr.global_symbols,
        format_each = "-Wl,--export-dynamic-symbol=%s",
    )
    link_args.add("-Wl,--whole-archive")
    link_args.add_all(archives)
    link_args.add("-Wl,--no-whole-archive")
    link_args.add_all(objects)
    link_args.add_all(["-o", bitcode])
    ctx.actions.run(
        executable = compiler,
        arguments = [link_args],
        inputs = inputs,
        outputs = [bitcode],
        execution_requirements = {"supports-path-mapping": "1"},
        mnemonic = "InternalizeSymbolizerBitcode",
        tools = cc_toolchain.all_files,
        toolchain = CC_TOOLCHAIN_TYPE,
    )

    compile_args = ctx.actions.args()
    compile_args.add_all([
        "-target",
        ctx.attr.target_triple[0],
        "-x",
        "ir",
        "-c",
        "-fno-lto",
        "-Oz",
        "-g0",
        "-fPIC",
        bitcode,
        "-o",
        ctx.outputs.out,
    ])
    ctx.actions.run(
        executable = compiler,
        arguments = [compile_args],
        inputs = [bitcode],
        outputs = [ctx.outputs.out],
        execution_requirements = {"supports-path-mapping": "1"},
        mnemonic = "CompileInternalSymbolizerObject",
        tools = cc_toolchain.all_files,
        toolchain = CC_TOOLCHAIN_TYPE,
    )

    return [DefaultInfo(files = depset([ctx.outputs.out]))]

cc_internal_symbolizer_object = rule(
    implementation = _cc_internal_symbolizer_object_impl,
    attrs = {
        "copts": attr.string_list(),
        "cxxopts": attr.string_list(),
        "global_symbols": attr.string_list(mandatory = True),
        "libcxx": attr.label(
            cfg = _internal_symbolizer_libcxx_transition,
            mandatory = True,
            providers = [CcInfo],
        ),
        "libcxxabi": attr.label(
            cfg = _internal_symbolizer_libcxx_transition,
            mandatory = True,
            providers = [CcInfo],
        ),
        "out": attr.output(mandatory = True),
        "platform": attr.label(mandatory = True),
        "symbolizer": attr.label(mandatory = True, providers = [CcInfo]),
        "target_triple": attr.string_list(mandatory = True),
    },
    fragments = ["cpp"],
    toolchains = use_cc_toolchain(),
)
