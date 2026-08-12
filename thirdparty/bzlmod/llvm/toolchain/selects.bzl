load("//platforms:common.bzl", "SUPPORTED_EXECS")

def _tool_repo(exec_os, exec_cpu):
    os_part = "darwin" if exec_os == "macos" else exec_os
    cpu_part = "amd64" if exec_cpu == "x86_64" else "arm64"
    return "@llvm-toolchain-minimal-%s-%s//" % (os_part, cpu_part)

def _platform_bootstrap_stage(exec_os, exec_cpu, bootstrap_stage):
    return "@llvm//platforms/config:%s_%s_%s" % (exec_os, exec_cpu, bootstrap_stage)

def platform_llvm_binary(binary):
    binaries = {
        _platform_bootstrap_stage(exec_os, exec_cpu, "stage0_prebuilt_seed"): Label(
            "%s:bin/%s%s" % (_tool_repo(exec_os, exec_cpu), binary, ".exe" if exec_os == "windows" else ""),
        )
        for exec_os, exec_cpu in SUPPORTED_EXECS
    }
    binaries["@llvm//toolchain:bootstrap_stage1_from_source"] = Label(
        "@llvm//toolchain/bootstrap/stage1:" + binary,
    )
    binaries["@llvm//toolchain:bootstrap_stage2_lto_and_fdo_instrumented"] = Label(
        "@llvm//toolchain/bootstrap/stage2:" + binary,
    )
    binaries["@llvm//toolchain:bootstrap_stage3_lto_and_fdo_applied"] = Label(
        "@llvm//toolchain/bootstrap/stage3:" + binary,
    )
    return select(binaries)

def platform_extra_binary(binary):
    return select({
        "@llvm//platforms/config:macos_x86_64": Label("@toolchain-extra-prebuilts-darwin-amd64//:%s" % binary),
        "@llvm//platforms/config:macos_aarch64": Label("@toolchain-extra-prebuilts-darwin-arm64//:%s" % binary),
        "@llvm//platforms/config:linux_x86_64": Label("@toolchain-extra-prebuilts-linux-amd64//:%s" % binary),
        "@llvm//platforms/config:linux_aarch64": Label("@toolchain-extra-prebuilts-linux-arm64//:%s" % binary),
        # TODO(zbarsky): should we suffix these with `.exe` in the dist?
        "@llvm//platforms/config:windows_aarch64": Label("@toolchain-extra-prebuilts-windows-arm64//:%s" % binary),
        "@llvm//platforms/config:windows_x86_64": Label("@toolchain-extra-prebuilts-windows-amd64//:%s" % binary),
    })

def platform_module_map(exec_os, exec_cpu):
    return Label(_tool_repo(exec_os, exec_cpu) + ":module_map")

def resource_dir_arg(exec_os, exec_cpu):
    return Label(_tool_repo(exec_os, exec_cpu) + ":compile_resource_dir")

def platform_cc_tool_map(exec_os, exec_cpu):
    tool_repo = _tool_repo(exec_os, exec_cpu)

    # Even though `tool_map` is exec-configured, this `select` happens under the target configuration.
    # That's because Bazel resolves the select before applying the exec transition, but if these targets
    # point at further aliases that use `select`, those will resolve according to the exec platform.
    # See https://github.com/bazelbuild/bazel/issues/27623#issuecomment-3529439585 for more details.
    return select({
        "@llvm//toolchain:linux_complete": Label(tool_repo + ":tools_with_interface_libraries"),
        "@llvm//toolchain:macos_complete_with_libtool": Label(tool_repo + ":tools_with_dsym_and_libtool"),
        "@llvm//toolchain:macos_complete": Label(tool_repo + ":tools_with_dsym"),
        "@rules_cc//cc/toolchains/args/archiver_flags:use_libtool_on_apple_setting": Label(tool_repo + ":tools_with_libtool_for_runtime"),
        "//conditions:default": Label(tool_repo + ":default_tools_for_runtime"),
    })
