load("@bazel_lib//lib:copy_file.bzl", "COPY_FILE_TOOLCHAINS", "copy_file_action")
load("@bazel_lib//lib:copy_to_directory.bzl", "copy_to_directory_bin_action")

# echo 'int main() {}' | bazel run //tools:clang -- -x c - -fuse-ld=lld -v --rtlib=compiler-rt -### --target=<triple>
TRIPLE_SELECT_DICT = {
    "@llvm//platforms/config:linux_x86_64": "x86_64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_aarch64": "aarch64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_riscv64": "riscv64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_s390x": "s390x-unknown-linux-gnu",
    "@llvm//platforms/config:linux_armv7": "armv7-unknown-linux-gnueabihf",
    "@llvm//platforms/config:linux_x86_64_gnu": "x86_64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_aarch64_gnu": "aarch64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_riscv64_gnu": "riscv64-unknown-linux-gnu",
    "@llvm//platforms/config:linux_s390x_gnu": "s390x-unknown-linux-gnu",
    "@llvm//platforms/config:linux_armv7_gnu": "armv7-unknown-linux-gnueabihf",
    "@llvm//platforms/config:linux_x86_64_musl": "x86_64-unknown-linux-musl",
    "@llvm//platforms/config:linux_aarch64_musl": "aarch64-unknown-linux-musl",
    "@llvm//platforms/config:linux_riscv64_musl": "riscv64-unknown-linux-musl",
    "@llvm//platforms/config:linux_s390x_musl": "s390x-unknown-linux-musl",
    "@llvm//platforms/config:linux_armv7_musl": "armv7-unknown-linux-musleabihf",
    "@llvm//platforms/config:macos_x86_64": "darwin",
    "@llvm//platforms/config:macos_aarch64": "darwin",
    "@llvm//platforms/config:windows_x86_64": "x86_64-w64-windows-gnu",
    "@llvm//platforms/config:windows_aarch64": "aarch64-w64-windows-gnu",
    "@llvm//platforms/config:none_bpfeb": "bpfeb-unknown-none",
    "@llvm//platforms/config:none_bpfel": "bpfel-unknown-none",
    "@llvm//platforms/config:none_wasm32": "wasm32-unknown-unknown",
    "@llvm//platforms/config:none_wasm64": "wasm64-unknown-unknown",
}

def _copy_to_resource_directory_rule_impl(ctx):
    # Private staging folder inside the output-dir layout before we rewrite prefixes.
    staging_prefix = "_%s_staging" % ctx.label.name

    staged = []
    for src_label, out_basename in ctx.attr.srcs.items():
        src = src_label.files.to_list()[0]
        extension_src = src.path.split(".")[-1]

        # we need to respect the extension since it may differ between platforms.
        out_filename = "%s.%s" % (out_basename, extension_src)
        out = ctx.actions.declare_file("%s/%s" % (staging_prefix, out_filename))
        copy_file_action(
            ctx,
            src = src,
            dst = out,
        )
        staged.append(out)

    copy_to_directory_bin = ctx.toolchains["@bazel_lib//lib:copy_to_directory_toolchain_type"].copy_to_directory_info.bin
    out_dir = ctx.actions.declare_directory(ctx.label.name)
    copy_to_directory_bin_action(
        ctx,
        name = ctx.attr.name,
        copy_to_directory_bin = copy_to_directory_bin,
        dst = out_dir,
        files = staged,
        replace_prefixes = {staging_prefix: "lib/%s" % ctx.attr.target_triple},
        include_external_repositories = ["**"],
        root_paths = ["."],
    )

    return [DefaultInfo(files = depset([out_dir]))]

copy_to_resource_directory_rule = rule(
    doc = "Copies the given srcs into a resource directory layout under lib/<triple>/.",
    implementation = _copy_to_resource_directory_rule_impl,
    attrs = {
        "srcs": attr.label_keyed_string_dict(
            doc = "Dict of label -> basename. Each value is the filename to appear under lib/<triple>/",
            mandatory = True,
            allow_files = True,
        ),
        "target_triple": attr.string(
            doc = "The target triple to use for placing the files.",
        ),
    },
    toolchains = COPY_FILE_TOOLCHAINS + [
        "@bazel_lib//lib:copy_to_directory_toolchain_type",
    ],
)

def _copy_to_resource_directory_macro_impl(name, srcs, target_triple, **kwargs):
    return copy_to_resource_directory_rule(
        name = name,
        srcs = srcs,
        target_triple = target_triple if target_triple else select(TRIPLE_SELECT_DICT),
        **kwargs
    )

copy_to_resource_directory = macro(
    implementation = _copy_to_resource_directory_macro_impl,
    inherit_attrs = copy_to_resource_directory_rule,
)
