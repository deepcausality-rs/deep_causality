load("@bazel_skylib//lib:structs.bzl", "structs")
load("//private:llvm_project_archive.bzl", "llvm_project_archive")
load("//private:llvm_project_from_git.bzl", "llvm_project_from_git")
load("//private:llvm_project_from_path.bzl", "llvm_project_from_path")

_DEFAULT_LLVM_VERSIONS_INDEX_FILE = "//:llvm_versions.json"

_DEFAULT_SOURCE_PATCHES = [
    "//3rd_party/llvm-project/x.x/patches:clang-prepend-arg-reexec.patch",
    "//3rd_party/llvm-project/x.x/patches:no_frontend_builtin_headers.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-driver-best-tool-match.patch",
    "//3rd_party/llvm-project/x.x/patches:compiler-rt-symbolizer_skip_cxa_atexit.patch",
    "//3rd_party/llvm-project/x.x/patches:lit_test_stub.patch",
    "//3rd_party/llvm-project/x.x/patches:lld-macho-thinlto-obj-path.patch",
]

_BEFORE_23_SOURCE_PATCHES = [
    "//3rd_party/llvm-project/before23.x/patches:llvm-extra.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-bzl-library.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-cov-multicall.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-readtapi-multicall.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-install-name-tool-output.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-driver-tool-order.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-dsymutil-corefoundation.patch",
    "//3rd_party/llvm-project/before23.x/patches:pfm-rules-cc-load.patch",
    "//3rd_party/llvm-project/before23.x/patches:clang-hardlink-filenames.patch",
    "//3rd_party/llvm-project/before23.x/patches:thinlto-roundtrip-before-codegen.patch",
    "//3rd_party/llvm-project/before23.x/patches:llvm-abi-breaking-checks.patch",
]

_LLVM_21_SOURCE_PATCHES = [
    "//3rd_party/llvm-project/21.x/patches:lld-coff-thinlto-lazy-index.patch",
    "//3rd_party/llvm-project/21.x/patches:llvm-link-multicall.patch",
    "//3rd_party/llvm-project/21.x/patches:llvm-bazel9.patch",
    "//3rd_party/llvm-project/21.x/patches:windows_link_and_genrule.patch",
    "//3rd_party/llvm-project/21.x/patches:bundle_resources_no_python.patch",
    "//3rd_party/llvm-project/21.x/patches:no_zlib_genrule.patch",
    "//3rd_party/llvm-project/21.x/patches:no_rules_python.patch",
    "//3rd_party/llvm-project/21.x/patches:llvm-windows-stack-size.patch",
    "//3rd_party/llvm-project/21.x/patches:libcxx-lgamma_r.patch",
    "//3rd_party/llvm-project/21.x/patches:llvm-bazel-blake3-windows-gnu.patch",
    "//3rd_party/llvm-project/21.x/patches:llvm-compression-defines.patch",
] + _BEFORE_23_SOURCE_PATCHES + _DEFAULT_SOURCE_PATCHES

_LLVM_22_SOURCE_PATCHES = [
    "//3rd_party/llvm-project/22.x/patches:lld-coff-thinlto-lazy-index.patch",
    "//3rd_party/llvm-project/22.x/patches:llvm-link-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:llvm-profdata-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:clang-format-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:clang-tidy-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:clangd-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:windows_link_and_genrule.patch",
    "//3rd_party/llvm-project/22.x/patches:bundle_resources_no_python.patch",
    "//3rd_party/llvm-project/22.x/patches:no_rules_python.patch",
    "//3rd_party/llvm-project/22.x/patches:llvm-windows-stack-size.patch",
    "//3rd_party/llvm-project/22.x/patches:libcxx-lgamma_r.patch",
    "//3rd_party/llvm-project/22.x/patches:llvm-bazel-blake3-windows-gnu.patch",
] + _BEFORE_23_SOURCE_PATCHES + _DEFAULT_SOURCE_PATCHES

_LLVM_23_SOURCE_PATCHES = [
    "//3rd_party/llvm-project/22.x/patches:lld-coff-thinlto-lazy-index.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-http-windows-gnu.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-link-multicall.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-profdata-multicall.patch",
    "//3rd_party/llvm-project/x.x/patches:clang-format-multicall.patch",
    "//3rd_party/llvm-project/x.x/patches:clang-tidy-multicall.patch",
    "//3rd_party/llvm-project/x.x/patches:clangd-multicall.patch",
    "//3rd_party/llvm-project/22.x/patches:bundle_resources_no_python.patch",
    "//3rd_party/llvm-project/x.x/patches:no_rules_python.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-extra.patch",
    "//3rd_party/llvm-project/x.x/patches:llvm-cov-multicall.patch",
    "//3rd_party/llvm-project/x.x/patches:thinlto-roundtrip-before-codegen.patch",
] + _DEFAULT_SOURCE_PATCHES

_LLVM_PATCHES_BY_MAJOR = {
    21: _LLVM_21_SOURCE_PATCHES,
    22: _LLVM_22_SOURCE_PATCHES,
    23: _LLVM_23_SOURCE_PATCHES,
    # So that anyone can test with the next LLVM major easily.
    24: _LLVM_23_SOURCE_PATCHES,
}

def _create_llvm_project_repository(mctx, llvm_version, llvm_version_index, targets):
    had_override = False

    for module in mctx.modules:
        for tag in module.tags.from_path:
            if had_override:
                fail("Only one LLVM source override is allowed")
            had_override = True
            llvm_project_from_path(name = "llvm-project", path = tag.path, targets = targets)

        for tag in module.tags.from_git:
            if had_override:
                fail("Only one LLVM source override is allowed")
            had_override = True
            llvm_project_from_git(name = "llvm-project", targets = targets, **structs.to_dict(tag))

        for tag in module.tags.from_archive:
            if had_override:
                fail("Only one LLVM source override is allowed")
            had_override = True
            llvm_project_archive(name = "llvm-project", targets = targets, **structs.to_dict(tag))

    if not had_override:
        source_archive = _source_archive_for_version(llvm_version, llvm_version_index)
        llvm_project_archive(name = "llvm-project", targets = targets, **structs.to_dict(source_archive))

def _parse_llvm_major(llvm_version):
    major = llvm_version.partition(".")[0]
    if not major.isdigit():
        fail("Invalid LLVM version '{}': expected numeric major version prefix".format(llvm_version))

    return int(major)

def _source_archive_for_version(llvm_version, llvm_version_index):
    major = _parse_llvm_major(llvm_version)
    source_info = llvm_version_index.get(llvm_version)
    if source_info == None:
        fail("LLVM version '{}' is missing from llvm version index.".format(llvm_version))

    if type(source_info) != "dict":
        fail("Invalid llvm version index entry for '{}': expected dict, got {}".format(llvm_version, type(source_info)))

    if source_info.get("url") == None or source_info.get("sha256") == None:
        fail("Invalid llvm version index entry for '{}': expected keys 'url' and 'sha256'".format(llvm_version))

    return struct(
        strip_prefix = source_info.get("strip_prefix", "llvm-project-{}.src".format(llvm_version)),
        url = source_info["url"],
        sha256 = source_info["sha256"],
        patch_args = ["-p1"],
        patches = _LLVM_PATCHES_BY_MAJOR.get(major, []),
    )

def _llvm_version_repository_impl(rctx):
    rctx.file("BUILD.bazel", """\
load("@bazel_lib//:bzl_library.bzl", "bzl_library")

bzl_library(
    name = "version",
    srcs = ["version.bzl"],
    visibility = ["//visibility:public"],
)
""")
    rctx.file(
        "version.bzl",
        "LLVM_VERSION = {}\n".format(repr(rctx.attr.llvm_version)),
    )
    return rctx.repo_metadata(reproducible = True)

_llvm_version_repository = repository_rule(
    implementation = _llvm_version_repository_impl,
    attrs = {
        "llvm_version": attr.string(mandatory = True),
    },
)

def _root_direct_deps(mctx):
    for module in mctx.modules:
        if module.is_root and module.name == "llvm":
            return ["llvm-project", "llvm_version"]
    return ["llvm-project"]

def _get_llvm_version(mctx):
    module_selected_version = None

    for mod in mctx.modules:
        module_versions = [tag.llvm_version for tag in mod.tags.version]
        if len(module_versions) > 1:
            fail("Only one llvm.version(...) tag is allowed per module")

        if not module_versions:
            continue

        if getattr(mod, "is_root", False):
            return module_versions[0]

        module_selected_version = module_versions[0]

    if module_selected_version != None:
        return module_selected_version

    fail("Missing llvm.version(...): set llvm.version(llvm_version = \"<major>.<minor>.<patch>\") in your MODULE.bazel")

def _get_llvm_version_index(mctx):
    decoded = json.decode(mctx.read(Label(_DEFAULT_LLVM_VERSIONS_INDEX_FILE)))
    if type(decoded) != "dict":
        fail("Invalid llvm version index in '{}': expected top-level dict".format(_DEFAULT_LLVM_VERSIONS_INDEX_FILE))
    return decoded

def _get_llvm_targets(mctx):
    targets = {}
    for module in mctx.modules:
        for configure in module.tags.configure:
            for target in configure.targets:
                targets[target] = None
    return targets.keys()

def _llvm_impl(mctx):
    llvm_version = _get_llvm_version(mctx)
    llvm_version_index = _get_llvm_version_index(mctx)

    _llvm_version_repository(
        name = "llvm_version",
        llvm_version = llvm_version,
    )
    _create_llvm_project_repository(mctx, llvm_version, llvm_version_index, _get_llvm_targets(mctx))

    return mctx.extension_metadata(
        reproducible = True,
        root_module_direct_deps = _root_direct_deps(mctx),
        root_module_direct_dev_deps = [],
    )

_version_tag = tag_class(
    attrs = {
        "llvm_version": attr.string(mandatory = True),
    },
)

_configure_tag = tag_class(
    attrs = {
        "targets": attr.string_list(mandatory = True),
    },
)

_from_path_tag = tag_class(
    attrs = {
        "path": attr.string(mandatory = True),
    },
)

_from_git_tag = tag_class(
    attrs = {
        "remote": attr.string(mandatory = True),
        "commit": attr.string(default = ""),
        "tag": attr.string(default = ""),
        "branch": attr.string(default = ""),
        "shallow_since": attr.string(default = ""),
        "init_submodules": attr.bool(default = False),
        "recursive_init_submodules": attr.bool(default = False),
        "strip_prefix": attr.string(default = ""),
        "patches": attr.label_list(default = []),
        "patch_args": attr.string_list(default = ["-p0"]),
        "patch_cmds": attr.string_list(default = []),
        "patch_cmds_win": attr.string_list(default = []),
        "patch_tool": attr.string(default = ""),
        "verbose": attr.bool(default = False),
    },
)

_from_archive_tag = tag_class(
    attrs = {
        "url": attr.string(default = ""),
        "urls": attr.string_list(default = []),
        "sha256": attr.string(default = ""),
        "integrity": attr.string(default = ""),
        "netrc": attr.string(default = ""),
        "auth_patterns": attr.string_dict(default = {}),
        "strip_prefix": attr.string(default = ""),
        "files": attr.string_keyed_label_dict(default = {}),
        "type": attr.string(default = ""),
        "patches": attr.label_list(default = []),
        "patch_strip": attr.int(default = 0),
        "patch_args": attr.string_list(default = ["-p0"]),
        "patch_cmds": attr.string_list(default = []),
        "patch_cmds_win": attr.string_list(default = []),
        "patch_tool": attr.string(default = ""),
        "canonical_id": attr.string(default = ""),
        "remote_file_urls": attr.string_list_dict(default = {}),
        "remote_file_integrity": attr.string_dict(default = {}),
        "remote_module_file_urls": attr.string_list(default = []),
        "remote_module_file_integrity": attr.string(default = ""),
        "remote_patches": attr.string_dict(default = {}),
        "remote_patch_strip": attr.int(default = 0),
        "includes": attr.string_list(default = []),
        "excludes": attr.string_list(default = []),
    },
)

llvm = module_extension(
    implementation = _llvm_impl,
    tag_classes = {
        "configure": _configure_tag,
        "version": _version_tag,
        "from_path": _from_path_tag,
        "from_git": _from_git_tag,
        "from_archive": _from_archive_tag,
    },
)
