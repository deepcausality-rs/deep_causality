"""Module extension that declares LLVM minimal prebuilt toolchain repositories."""

load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@llvm_version//:version.bzl", "LLVM_VERSION")

DEFAULT_LLVM_TOOLCHAIN_MINIMAL_INDEX_FILE = "//extensions:llvm_toolchain_minimal_index.json"

_TARGETS = [
    "darwin-amd64",
    "darwin-arm64",
    "linux-amd64-musl",
    "linux-arm64-musl",
    "windows-amd64",
    "windows-arm64",
]

def _repo_target(target):
    return target.replace("-musl", "")

def _repo_name(target):
    return "llvm-toolchain-minimal-{target}".format(
        target = _repo_target(target),
    )

def _build_file(target):
    if target.startswith("windows-"):
        return Label("//toolchain/llvm:llvm_release_windows.BUILD.bazel")
    return Label("//toolchain/llvm:llvm_release.BUILD.bazel")

def _get_index(module_ctx):
    index_file = Label(DEFAULT_LLVM_TOOLCHAIN_MINIMAL_INDEX_FILE)
    dependency_index_file = None

    for module in module_ctx.modules:
        index_files = [index.file for index in module.tags.index]
        if len(index_files) > 1:
            fail("Only 1 llvm_toolchain_minimal.index(...) tag is allowed per module")

        if not index_files:
            continue

        if module.is_root:
            index_file = index_files[0]
            break

        dependency_index_file = index_files[0]

    if dependency_index_file != None and index_file == Label(DEFAULT_LLVM_TOOLCHAIN_MINIMAL_INDEX_FILE):
        index_file = dependency_index_file

    return json.decode(module_ctx.read(module_ctx.path(index_file)), default = None)

def _release_key(index):
    latest_by_llvm_version = index.get("latest_by_llvm_version", {})
    release_key = latest_by_llvm_version.get(LLVM_VERSION)
    if release_key != None:
        return release_key

    # Keep default_prebuilt_llvm_version as the bootstrap seed when LLVM_VERSION
    # does not have an indexed prebuilt release.
    default_prebuilt_llvm_version = index.get("default_prebuilt_llvm_version")
    if default_prebuilt_llvm_version == None:
        fail("LLVM {} has no minimal prebuilt release and the index has no default_prebuilt_llvm_version".format(LLVM_VERSION))

    # buildifier: disable=print
    print("WARNING: LLVM {} has no indexed minimal prebuilt release; using default_prebuilt_llvm_version {} as the bootstrap seed".format(
        LLVM_VERSION,
        default_prebuilt_llvm_version,
    ))

    release_key = latest_by_llvm_version.get(default_prebuilt_llvm_version)
    if release_key == None:
        fail("Default prebuilt LLVM version {} has no minimal prebuilt release".format(default_prebuilt_llvm_version))

    return release_key

def _release_repo_specs(index):
    release_key = _release_key(index)
    archives = index["releases"][release_key]
    return {
        _repo_name(target): struct(
            build_file = _build_file(target),
            sha256 = archives[target]["sha256"],
            urls = [archives[target]["url"]],
        )
        for target in _TARGETS
    }

def _llvm_toolchain_minimal_impl(module_ctx):
    index = _get_index(module_ctx)
    repo_specs = _release_repo_specs(index)

    for repo_name, spec in repo_specs.items():
        http_archive(
            name = repo_name,
            build_file = spec.build_file,
            sha256 = spec.sha256,
            urls = spec.urls,
        )

    metadata_kwargs = {}
    if bazel_features.external_deps.extension_metadata_has_reproducible:
        metadata_kwargs["reproducible"] = True

    root_uses_extension = any([module.is_root for module in module_ctx.modules])
    root_direct_deps = sorted(repo_specs.keys()) if root_uses_extension else []
    root_direct_dev_deps = []
    if not module_ctx.root_module_has_non_dev_dependency:
        root_direct_dev_deps = root_direct_deps
        root_direct_deps = []

    return module_ctx.extension_metadata(
        root_module_direct_deps = root_direct_deps,
        root_module_direct_dev_deps = root_direct_dev_deps,
        **metadata_kwargs
    )

_index_tag = tag_class(
    attrs = {
        "file": attr.label(
            allow_single_file = True,
            default = Label(DEFAULT_LLVM_TOOLCHAIN_MINIMAL_INDEX_FILE),
        ),
    },
)

llvm_toolchain_minimal = module_extension(
    implementation = _llvm_toolchain_minimal_impl,
    doc = "Declares version-neutral minimal prebuilt compiler repositories for the LLVM version selected by llvm.",
    tag_classes = {
        "index": _index_tag,
    },
)
