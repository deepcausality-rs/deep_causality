load("@bazel_tools//tools/build_defs/repo:cache.bzl", "DEFAULT_CANONICAL_ID_ENV")
load("@bazel_tools//tools/build_defs/repo:git_worker.bzl", "git_repo")
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "patch")
load(
    ":llvm_project_common.bzl",
    "copy_overlay",
    "write_llvm_project_files",
)

def _llvm_project_from_git_impl(rctx):
    if ((rctx.attr.tag and rctx.attr.commit) or
        (rctx.attr.tag and rctx.attr.branch) or
        (rctx.attr.commit and rctx.attr.branch)):
        fail("At most one of commit, tag, or branch may be provided")
    git_root = rctx.path(".tmp_git_root") if rctx.attr.strip_prefix else rctx.path(".")
    git_repo(rctx, str(git_root))

    source_root = git_root
    if rctx.attr.strip_prefix:
        source_root = git_root.get_child(rctx.attr.strip_prefix)
        if not source_root.exists:
            fail("strip_prefix at {} does not exist in repo".format(rctx.attr.strip_prefix))
        for entry in source_root.readdir():
            rctx.symlink(entry, entry.basename)

    patch(rctx)
    copy_overlay(rctx, source_root)
    rctx.delete(git_root.get_child(".git"))
    write_llvm_project_files(rctx)

    return rctx.repo_metadata(reproducible = False)

llvm_project_from_git = repository_rule(
    implementation = _llvm_project_from_git_impl,
    attrs = {
        "remote": attr.string(mandatory = True),
        "commit": attr.string(),
        "tag": attr.string(),
        "branch": attr.string(),
        "shallow_since": attr.string(),
        "init_submodules": attr.bool(default = False),
        "recursive_init_submodules": attr.bool(default = False),
        "strip_prefix": attr.string(),
        "patches": attr.label_list(default = []),
        "patch_args": attr.string_list(default = []),
        "patch_cmds": attr.string_list(default = []),
        "patch_cmds_win": attr.string_list(default = []),
        "patch_tool": attr.string(),
        "verbose": attr.bool(default = False),
        "targets": attr.string_list(mandatory = True),
    },
    environ = [DEFAULT_CANONICAL_ID_ENV],
)
