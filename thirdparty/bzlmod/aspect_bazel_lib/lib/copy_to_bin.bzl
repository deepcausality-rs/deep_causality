"""A rule that copies source files to the output tree.

This rule uses a Bash command (diff) on Linux/macOS/non-Windows, and a cmd.exe
command (fc.exe) on Windows (no Bash is required).

Originally authored in rules_nodejs
https://github.com/bazel-contrib/rules_nodejs/blob/8b5d27400db51e7027fe95ae413eeabea4856f8e/internal/common/copy_to_bin.bzl
"""

load(
    "@bazel_lib//lib:copy_to_bin.bzl",
    _copy_file_to_bin_action = "copy_file_to_bin_action",
    _copy_files_to_bin_actions = "copy_files_to_bin_actions",
    _copy_to_bin = "copy_to_bin",
)

# bazel-lib 3.x COPY_FILE_TO_BIN_TOOLCHAINS references COPY_FILE_TOOLCHAINS.
# bazel-lib 2.x COPY_FILE_[TO_BIN_]TOOLCHAINS must only expose the @aspect_bazel_lib toolchain
# names, which may alias to bazel-lib 3.x
load(":copy_file.bzl", "COPY_FILE_TOOLCHAINS")

copy_file_to_bin_action = _copy_file_to_bin_action
copy_files_to_bin_actions = _copy_files_to_bin_actions
copy_to_bin = _copy_to_bin
COPY_FILE_TO_BIN_TOOLCHAINS = COPY_FILE_TOOLCHAINS
