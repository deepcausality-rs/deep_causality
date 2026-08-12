load(
    ":llvm_project_common.bzl",
    "LLVM_PROJECT_OVERLAY",
    "symlink_source_and_overlay",
    "write_llvm_project_files",
)

def _materialize_third_party_build(rctx, source_root):
    # symlink_source_and_overlay creates utils as a symlink to source_root/utils.
    # Replace utils, utils/bazel, and utils/bazel/third_party_build with directories
    # of symlinks before overwriting BUILD.bazel, so source_root remains unchanged.
    for directory in ["utils", "utils/bazel", "utils/bazel/third_party_build"]:
        rctx.delete(directory)
        target = rctx.path(directory)
        for entry in source_root.get_child(directory).readdir():
            rctx.symlink(entry, target.get_child(entry.basename))

def _llvm_project_from_path_impl(rctx):
    source_root = rctx.workspace_root.get_child(rctx.attr.path)
    overlay_root = source_root.get_child(LLVM_PROJECT_OVERLAY)
    if not overlay_root.exists:
        fail("LLVM source path {} does not contain {}".format(source_root, LLVM_PROJECT_OVERLAY))

    symlink_source_and_overlay(rctx, source_root)
    _materialize_third_party_build(rctx, source_root)
    write_llvm_project_files(rctx)
    return rctx.repo_metadata(reproducible = False)

llvm_project_from_path = repository_rule(
    implementation = _llvm_project_from_path_impl,
    attrs = {
        "path": attr.string(mandatory = True),
        "targets": attr.string_list(mandatory = True),
    },
    local = True,
)
