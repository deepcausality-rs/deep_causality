load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_skylib//lib:paths.bzl", "paths")
load("@bazel_skylib//rules/directory:providers.bzl", "DirectoryInfo")
load("//:directory.bzl", "SourceDirectoryInfo")

IncludePathInfo = provider(
    "IncludePathInfo",
    fields = {
        "submodule_directories": "A depset of File objects representing directories to be included as umbrella submodules.",
        "textual_headers": "A depset of File objects representing headers to be included as textual headers.",
    },
)

def _umbrella_submodule(directory):
    path = paths.normalize(directory.path).replace("//", "/")

    return """
  module "{path}" {{
    umbrella "{path}"
  }}""".format(path = path)

def _module_map_impl(ctx):
    module_map = ctx.actions.declare_file(ctx.attr.name + ".modulemap")

    include_path_info = ctx.attr.include_path[IncludePathInfo]

    module_map_args = ctx.actions.args()
    module_map_args.set_param_file_format("multiline")
    module_map_args.add('module "crosstool" [system] {')

    module_map_args.add_joined(
        include_path_info.submodule_directories,
        join_with = "\n",
        map_each = _umbrella_submodule,
        expand_directories = False,
    )

    module_map_args.add_joined(
        include_path_info.textual_headers,
        join_with = "\n",
        format_each = "  textual header \"%s\"",
        expand_directories = False,
    )

    module_map_args.add("}")

    write_kwargs = {}
    if bazel_features.rules.write_action_has_mnemonic:
        write_kwargs["mnemonic"] = "CppModuleMap"

    ctx.actions.write(
        output = module_map,
        content = module_map_args,
        **write_kwargs
    )
    return DefaultInfo(files = depset([module_map]))

module_map = rule(
    doc = """Generates a Clang module map for the toolchain and system headers.

    Source and output directories are included as umbrella submodules.
    Individual header files (typically `run_binary` outputs like in mingw) are included as textual headers.""",
    implementation = _module_map_impl,
    attrs = {
        "include_path": attr.label(
            providers = [IncludePathInfo],
            mandatory = True,
        ),
    },
)

def _include_path_impl(ctx):
    submodule_directories = []
    textual_headers_depsets = []

    for src in ctx.attr.srcs:
        if SourceDirectoryInfo in src or DirectoryInfo not in src:
            # We're either a source directory or an output directory (Tree Artifact).
            submodule_directories.append(src[DefaultInfo].files)
        else:
            textual_headers_depsets.append(src[DirectoryInfo].transitive_files)

    return [
        IncludePathInfo(
            submodule_directories = depset([], transitive = submodule_directories),
            textual_headers = depset([], transitive = textual_headers_depsets),
        ),
    ]

include_path = rule(
    implementation = _include_path_impl,
    attrs = {
        "srcs": attr.label_list(),
    },
)
