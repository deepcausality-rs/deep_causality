LLVM_PROJECT_OVERLAY = "utils/bazel/llvm-project-overlay"

_MAX_OVERLAY_DIRECTORIES = 1000000

def _extract_cmake_settings(rctx, path):
    settings = {
        "CMAKE_CXX_STANDARD": None,
        "LLVM_VERSION_MAJOR": None,
        "LLVM_VERSION_MINOR": None,
        "LLVM_VERSION_PATCH": None,
        "LLVM_VERSION_SUFFIX": None,
    }

    for line in rctx.read(path).splitlines():
        set_call = line.partition("(")
        if set_call[1] != "(" or set_call[0].strip().lower() != "set":
            continue

        key_value = set_call[2].strip()
        separator = key_value.find(" ")
        if separator < 0:
            continue
        key = key_value[:separator]

        if key == "LLVM_REQUIRED_CXX_STANDARD":
            key = "CMAKE_CXX_STANDARD"
            settings[key] = None
        if key not in settings or settings[key] != None:
            continue

        settings[key] = key_value[separator:].strip().partition(")")[0].partition(" ")[0]

    return settings

def _write_llvm_vars(rctx):
    settings = _extract_cmake_settings(rctx, "llvm/CMakeLists.txt")
    version_settings = _extract_cmake_settings(rctx, "cmake/Modules/LLVMVersion.cmake")
    settings.update({key: value for key, value in version_settings.items() if value != None})
    settings["LLVM_VERSION_SUFFIX"] = settings["LLVM_VERSION_SUFFIX"] or ""
    settings["LLVM_VERSION"] = "{}.{}.{}".format(
        settings["LLVM_VERSION_MAJOR"],
        settings["LLVM_VERSION_MINOR"],
        settings["LLVM_VERSION_PATCH"],
    )
    settings["PACKAGE_VERSION"] = settings["LLVM_VERSION"] + settings["LLVM_VERSION_SUFFIX"]

    content = "# Generated from llvm/CMakeLists.txt\n\n"
    dictionary = "\nllvm_vars = {\n"
    for key, value in settings.items():
        content += '{} = "{}"\n'.format(key, value)
        dictionary += '    "{}": "{}",\n'.format(key, value)
    rctx.file("vars.bzl", content + dictionary + "}\n", executable = False)

def _write_llvm_targets(rctx):
    rctx.file(
        "llvm/targets.bzl",
        "llvm_targets = {}\n".format(rctx.attr.targets),
        executable = False,
    )

    bolt_targets = [target for target in rctx.attr.targets if target in ["AArch64", "X86", "RISCV"]]
    rctx.file(
        "bolt/targets.bzl",
        "bolt_targets = {}\n".format(bolt_targets),
        executable = False,
    )

def _expose_third_party_build_files(rctx):
    # The rules_foreign_cc pfm repository uses pfm.BUILD from
    # utils/bazel/third_party_build. Replace the utils/bazel .bazelignore entry
    # with LLVM_PROJECT_OVERLAY so that file remains visible.
    bazelignore = rctx.read(".bazelignore")
    rctx.delete(".bazelignore")
    rctx.file(
        ".bazelignore",
        bazelignore.replace(
            "# Ignore the utils/bazel directory when this is overlayed onto the repo root.\nutils/bazel\n",
            "# Ignore {} after merging it into the repository root.\n{}\n".format(LLVM_PROJECT_OVERLAY, LLVM_PROJECT_OVERLAY),
        ),
        executable = False,
    )
    rctx.delete("utils/bazel/third_party_build/BUILD.bazel")
    rctx.file("utils/bazel/third_party_build/BUILD.bazel", """exports_files(["pfm.BUILD"])\n""", executable = False)

def write_llvm_project_files(rctx):
    _expose_third_party_build_files(rctx)
    rctx.file("BUILD.bazel", """\
load("@bazel_lib//:bzl_library.bzl", "bzl_library")

bzl_library(
    name = "version",
    srcs = ["vars.bzl"],
    visibility = ["//visibility:public"],
)
""", executable = False)
    _write_llvm_vars(rctx)
    _write_llvm_targets(rctx)

def _relative_path(relative_dir, basename):
    if relative_dir == ".":
        return basename
    return relative_dir + "/" + basename

def copy_overlay(rctx, source_root):
    overlay_root = source_root.get_child(LLVM_PROJECT_OVERLAY)
    stack = ["."]
    for _ in range(_MAX_OVERLAY_DIRECTORIES):
        relative_dir = stack.pop()
        for entry in overlay_root.get_child(relative_dir).readdir():
            relative_path = _relative_path(relative_dir, entry.basename)
            if entry.is_dir:
                stack.append(relative_path)
            else:
                rctx.file(relative_path, rctx.read(entry), executable = False)
        if not stack:
            return
    fail("LLVM overlay contains more than {} directories".format(_MAX_OVERLAY_DIRECTORIES))

def symlink_source_and_overlay(rctx, source_root):
    overlay_root = source_root.get_child(LLVM_PROJECT_OVERLAY)
    target_root = rctx.path(".")
    stack = ["."]

    for _ in range(_MAX_OVERLAY_DIRECTORIES):
        relative_dir = stack.pop()
        overlay_dirs = {}

        for entry in overlay_root.get_child(relative_dir).readdir():
            relative_path = _relative_path(relative_dir, entry.basename)
            if entry.is_dir:
                stack.append(relative_path)
                overlay_dirs[entry.basename] = None
            else:
                rctx.symlink(entry, target_root.get_child(relative_path))

        for entry in source_root.get_child(relative_dir).readdir():
            if entry.basename not in overlay_dirs:
                relative_path = _relative_path(relative_dir, entry.basename)
                rctx.symlink(entry, target_root.get_child(relative_path))

        if not stack:
            return
    fail("LLVM overlay contains more than {} directories".format(_MAX_OVERLAY_DIRECTORIES))
