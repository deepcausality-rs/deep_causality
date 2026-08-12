load("@bazel_lib//lib:repo_utils.bzl", "repo_utils")
load("@bazel_skylib//lib:paths.bzl", "paths")
load(
    "@bazel_tools//tools/build_defs/repo:cache.bzl",
    "DEFAULT_CANONICAL_ID_ENV",
    "get_default_canonical_id",
)
load(
    "@bazel_tools//tools/build_defs/repo:utils.bzl",
    "get_auth",
    "patch",
    "update_attrs",
)
load("//:http_bsdtar_archive.bzl", "download_remote_files")
load(":llvm_project_common.bzl", "LLVM_PROJECT_OVERLAY", "write_llvm_project_files")

_DEFAULT_OVERLAY_FILES = {
    "compiler-rt/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/compiler-rt:compiler-rt.BUILD.bazel"),
    "libc/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/libc:libc.BUILD.bazel"),
    "libcxx/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/libcxx:libcxx.BUILD.bazel"),
    "libcxxabi/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/libcxxabi:libcxxabi.BUILD.bazel"),
    "libunwind/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/libunwind:libunwind.BUILD.bazel"),
    "openmp/BUILD.bazel": Label("//3rd_party/llvm-project/x.x/openmp:openmp.BUILD.bazel"),
}

def _default_excludes():
    excludes = [
        "flang-rt",
        "flang",
        "polly",
        "orc-rt",
        "libclc",
        "offload",
        "libc/docs",
        "libc/utils/gn",
        "llvm/utils/mlgo-utils/*",
    ]

    test_docs_subprojects = [
        "bolt",
        "clang-tools-extra",
        "clang",
        "compiler-rt",
        "libcxx",
        "libcxxabi",
        "libunwind",
        "lld",
        "lldb",
        "llvm",
        "mlir",
    ]

    for subproject in test_docs_subprojects:
        if subproject != "mlir":
            excludes.append("{}/test/*".format(subproject))
        excludes.append("{}/docs/*".format(subproject))

    return excludes

def _source_urls(rctx):
    if not rctx.attr.url and not rctx.attr.urls:
        fail("At least one of url and urls must be provided")

    if rctx.attr.url:
        return [rctx.attr.url] + rctx.attr.urls
    return rctx.attr.urls

def _downloaded_archive_name(rctx, source_urls):
    if rctx.attr.type:
        return ".downloaded.archive." + rctx.attr.type

    basename = source_urls[0].split("?", 1)[0].split("/")[-1]
    if basename:
        return ".downloaded." + basename
    return ".downloaded.archive"

def _host_bsdtar_label(rctx):
    platform = repo_utils.platform(rctx)
    binary = "tar.exe" if platform.startswith("windows_") else "tar"
    return Label("@bsd_tar_toolchains_{}//:{}".format(platform, binary))

def _extract(rctx, bsdtar, archive, includes, excludes, strip_components):
    args = []
    for include in includes:
        args.extend(["--include", include])
    for exclude in excludes:
        args.extend(["--exclude", exclude])
    if strip_components:
        args.extend(["--strip-components", str(strip_components)])
    args.extend([
        "--no-xattrs",
        "--no-fflags",
        "--no-mac-metadata",
        "--no-same-permissions",
        "--no-acls",
        "-m",
    ])
    args.extend(["-xf", archive])

    result = rctx.execute([rctx.path(bsdtar)] + args)
    if result.return_code != 0:
        fail("Failed to extract archive: {}\n{}".format(result.stderr, result.stdout))

def _extract_source_and_overlay(rctx, archive):
    strip_prefix = rctx.attr.strip_prefix.strip("/")
    source_includes = [paths.join(strip_prefix, include) for include in rctx.attr.includes]
    if strip_prefix and not source_includes:
        source_includes = [paths.join(strip_prefix, "*")]

    source_excludes = [paths.join(strip_prefix, exclude) for exclude in rctx.attr.excludes]
    source_excludes.append(paths.join(strip_prefix, LLVM_PROJECT_OVERLAY))

    bsdtar = _host_bsdtar_label(rctx)
    _extract(
        rctx,
        bsdtar,
        archive,
        source_includes,
        source_excludes,
        strip_prefix.count("/") + 1 if strip_prefix else 0,
    )

    overlay_prefix = paths.join(strip_prefix, LLVM_PROJECT_OVERLAY)

    # openmp/BUILD.bazel treats openmp/runtime as part of the openmp package.
    overlay_excludes = [paths.join(overlay_prefix, "openmp/runtime")]
    _extract(
        rctx,
        bsdtar,
        archive,
        [paths.join(overlay_prefix, "*")],
        overlay_excludes,
        overlay_prefix.count("/") + 1,
    )

def _copy_files(rctx):
    files = dict(rctx.attr.overlay_files)
    files.update(rctx.attr.files)
    for path, label in files.items():
        source = rctx.path(label)
        if not source.exists:
            fail("Input {} does not exist".format(label))
        rctx.file(path, rctx.read(source), executable = False)

def _update_integrity_attrs(rctx, archive_info, remote_files_info, remote_patches_info):
    overrides = {}
    if not rctx.attr.sha256 and not rctx.attr.integrity:
        overrides["integrity"] = archive_info.integrity

    remote_file_integrity = {path: info.integrity for path, info in remote_files_info.items()}
    if rctx.attr.remote_file_integrity != remote_file_integrity:
        overrides["remote_file_integrity"] = remote_file_integrity

    if remote_patches_info:
        remote_patch_integrity = {url: info.integrity for url, info in remote_patches_info.items()}
        if rctx.attr.remote_patches != remote_patch_integrity:
            overrides["remote_patches"] = remote_patch_integrity

    if not overrides:
        return rctx.repo_metadata(reproducible = True)
    return rctx.repo_metadata(
        attrs_for_reproducibility = update_attrs(rctx.attr, _llvm_project_archive_attrs.keys(), overrides),
    )

def _llvm_project_archive_impl(rctx):
    source_urls = _source_urls(rctx)
    archive = _downloaded_archive_name(rctx, source_urls)
    archive_info = rctx.download(
        source_urls,
        archive,
        rctx.attr.sha256,
        canonical_id = rctx.attr.canonical_id or get_default_canonical_id(rctx, source_urls),
        auth = get_auth(rctx, source_urls),
        integrity = rctx.attr.integrity,
    )

    _extract_source_and_overlay(rctx, archive)
    rctx.delete(archive)

    remote_files_info = download_remote_files(rctx)
    remote_patches_info = patch(rctx)
    _copy_files(rctx)
    write_llvm_project_files(rctx)

    return _update_integrity_attrs(rctx, archive_info, remote_files_info, remote_patches_info)

_llvm_project_archive_attrs = {
    "url": attr.string(),
    "urls": attr.string_list(),
    "sha256": attr.string(),
    "integrity": attr.string(),
    "netrc": attr.string(),
    "auth_patterns": attr.string_dict(),
    "canonical_id": attr.string(),
    "strip_prefix": attr.string(),
    "files": attr.string_keyed_label_dict(default = {}),
    "overlay_files": attr.string_keyed_label_dict(default = _DEFAULT_OVERLAY_FILES),
    "type": attr.string(),
    "patches": attr.label_list(default = []),
    "remote_file_urls": attr.string_list_dict(default = {}),
    "remote_file_integrity": attr.string_dict(default = {}),
    "remote_module_file_urls": attr.string_list(default = []),
    "remote_module_file_integrity": attr.string(),
    "remote_patches": attr.string_dict(default = {}),
    "remote_patch_strip": attr.int(default = 0),
    "patch_tool": attr.string(),
    "patch_args": attr.string_list(default = []),
    "patch_strip": attr.int(default = 0),
    "patch_cmds": attr.string_list(default = []),
    "patch_cmds_win": attr.string_list(default = []),
    "includes": attr.string_list(default = []),
    "excludes": attr.string_list(default = _default_excludes()),
    "targets": attr.string_list(mandatory = True),
}

llvm_project_archive = repository_rule(
    implementation = _llvm_project_archive_impl,
    attrs = _llvm_project_archive_attrs,
    environ = [DEFAULT_CANONICAL_ID_ENV],
)
