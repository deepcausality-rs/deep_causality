load(
    "@bazel_tools//tools/jdk:default_java_toolchain.bzl",
    %{jvm_opts_import}
    "default_java_toolchain",
    "java_runtime_files",
)
load("@rules_cc//cc:defs.bzl", "cc_toolchain", "cc_toolchain_suite")
load("@rules_java//java/toolchains:java_runtime.bzl", "java_runtime")
load("@rules_java//toolchains:default_java_toolchain.bzl", "default_java_toolchain")
load(":gcc_config.bzl", "GCC_BUILTIN_INCLUDE_PATHS")
load(":msvc_config.bzl", "MSVC_BUILTIN_INCLUDE_PATHS")
load(":cc_toolchain_config.bzl", "cc_toolchain_config")
load(":windows_cc_toolchain_config.bzl", windows_cc_toolchain_config = "cc_toolchain_config")

package(default_visibility = ["//visibility:public"])

# Some targets may need to directly depend on these files.
exports_files(glob(["bin/*", "lib/*"], allow_empty = True))

## Platforms

alias(
    name = "platform",
    actual = "%{default_platform}",
)

platform(
    name = "platform_linux",
    constraint_values = [
        "@platforms//cpu:%{default_arch_constraint}",
        "@platforms//os:linux",
        "@bazel_tools//tools/cpp:gcc",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "Arch": "%{default_arch_exec_property}",
        "container-image": "%{default_container_image}",
        "dockerNetwork": "%{default_docker_network}",
    },
)

platform(
    name = "platform_linux_x86_64",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
        "@bazel_tools//tools/cpp:gcc",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "Arch": "amd64",
        "container-image": "%{default_x86_64_container_image}",
        "dockerNetwork": "%{default_docker_network}",
    },
)

platform(
    name = "platform_linux_arm64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:linux",
        "@bazel_tools//tools/cpp:gcc",
    ],
    exec_properties = {
        "OSFamily": "Linux",
        "Arch": "arm64",
        "container-image": "%{default_arm64_container_image}",
        "dockerNetwork": "%{default_docker_network}",
    },
)

platform(
    name = "platform_darwin",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:macos",
        "@bazel_tools//tools/cpp:gcc",
    ],
    exec_properties = {
        "OSFamily": "Darwin",
        "container-image": "none",
        "dockerNetwork": "%{default_docker_network}",
    },
)

platform(
    name = "platform_darwin_arm64",
    constraint_values = [
        "@platforms//cpu:aarch64",
        "@platforms//os:osx",
        "@bazel_tools//tools/cpp:gcc",
    ],
    exec_properties = {
        "OSFamily": "Darwin",
        "Arch": "arm64",
        "container-image": "none",
        "dockerNetwork": "%{default_docker_network}",
    },
)

platform(
    name = "platform_windows",
    constraint_values = [
        "@platforms//cpu:x86_64",
        "@platforms//os:windows",
        "@bazel_tools//tools/cpp:msvc",
    ],
    exec_properties = {
        "OSFamily": "Windows",
    },
)

## Java %{java_version}

java_runtime(
    name = "javabase",
    srcs = [],
    java_home = "/usr/lib/jvm/java-%{java_version}-openjdk-amd64",
)

default_java_toolchain(
    name = "java_toolchain",
    jvm_opts = %{jvm_opts},
    source_version = "%{java_version}",
    target_version = "%{java_version}",
)

## Defaults

alias(
    name = "toolchain",
    actual="%{default_cc_toolchain_suite}"
)

alias(
    name = "cc_toolchain",
    actual="%{default_cc_toolchain}"
)


## CC

filegroup(
    name = "empty",
    srcs = [],
)

filegroup(
    name = "compiler_deps",
    srcs = [
        # This file is not expected to be used by any of the CC actions but is used to ensure that
        # these paths are part of the Action's cache key calculation.
        ":gcc_config.bzl",
    ],
)

cc_toolchain_suite(
    name = "ubuntu_cc_toolchain_suite",
    toolchains = {
        "k8|compiler": ":ubuntu_local_cc_toolchain",
        "k8": ":ubuntu_local_cc_toolchain",
    },
)

toolchain(
    name = "ubuntu_cc_toolchain",
    exec_compatible_with = [
        "@bazel_tools//tools/cpp:gcc",
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    target_compatible_with = [
        "@platforms//cpu:x86_64",
        "@platforms//os:linux",
    ],
    toolchain = ":ubuntu_local_cc_toolchain",
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)

toolchain(
    name = "ubuntu_cc_toolchain_arm64",
    exec_compatible_with = [
        "@bazel_tools//tools/cpp:gcc",
        "@platforms//cpu:aarch64",
        "@platforms//os:linux",
    ],
    target_compatible_with = [
        "@platforms//cpu:aarch64",
        "@platforms//os:linux",
    ],
    toolchain = ":ubuntu_local_cc_toolchain",
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)

cc_toolchain(
    name = "ubuntu_local_cc_toolchain",
    toolchain_identifier = "local",
    toolchain_config = ":ubuntu_cc_toolchain_config",
    all_files = ":compiler_deps",
    ar_files = ":compiler_deps",
    as_files = ":compiler_deps",
    compiler_files = ":compiler_deps",
    dwp_files = ":empty",
    linker_files = ":compiler_deps",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 1,
)

cc_toolchain_config(
    name = "ubuntu_cc_toolchain_config",
    cpu = "k8",
    compiler = "compiler",
    toolchain_identifier = "local",
    host_system_name = "local",
    target_system_name = "local",
    target_libc = "local",
    abi_version = "local",
    abi_libc_version = "local",
    cxx_builtin_include_directories = GCC_BUILTIN_INCLUDE_PATHS,
    tool_paths = {
        "ar": "/usr/bin/ar",
        "ld": "/usr/bin/ld",
        "cpp": "/usr/bin/cpp",
        "gcc": "/usr/bin/gcc",
        "dwp": "/usr/bin/dwp",
        "gcov": "/usr/bin/gcov",
        "nm": "/usr/bin/nm",
        "objcopy": "/usr/bin/objcopy",
        "objdump": "/usr/bin/objdump",
        "strip": "/usr/bin/strip"
    },
    compile_flags = [
        "-U_FORTIFY_SOURCE",
        "-fstack-protector",
        "-Wall",
        "-Wunused-but-set-parameter",
        "-Wno-free-nonheap-object",
        "-fno-omit-frame-pointer"
    ],
    opt_compile_flags = [
        "-g0",
        "-O2",
        "-D_FORTIFY_SOURCE=1",
        "-DNDEBUG",
        "-ffunction-sections",
        "-fdata-sections"
    ],
    dbg_compile_flags = ["-g"],
    cxx_flags = ["-std=c++17"],
    link_flags = [
        "-fuse-ld=gold",
        "-Wl,-no-as-needed",
        "-Wl,-z,relro,-z,now",
        "-B/usr/bin",
        "-pass-exit-codes",
        "-lstdc++",
        "-lm"
    ],
    link_libs = [],
    opt_link_flags = ["-Wl,--gc-sections"],
    unfiltered_compile_flags = [
        "-fno-canonical-system-headers",
        "-Wno-builtin-macro-redefined",
        "-D__DATE__=\"redacted\"",
        "-D__TIMESTAMP__=\"redacted\"",
        "-D__TIME__=\"redacted\""
    ],
    coverage_compile_flags = ["--coverage"],
    coverage_link_flags = ["--coverage"],
    supports_start_end_lib = True,
)

## Windows MSVC Toolchain

filegroup(
    name = "msvc_compiler_files",
    srcs = [
        # This file is not expected to be used by any of the CC actions but is used to ensure that
        # these paths are part of the Action's cache key calculation.
        ":msvc_config.bzl",
    ],
)

windows_cc_toolchain_config(
    name = "msvc_toolchain_config",
    cpu = "x64_windows",
    compiler = "msvc-cl",
    host_system_name = "local",
    target_system_name = "local",
    target_libc = "msvcrt",
    abi_version = "local",
    abi_libc_version = "local",
    toolchain_identifier = "msvc_toolchain_config",
    msvc_env_tmp = "C:\\Users\\User\\AppData\\Local\\Temp",
    msvc_env_path = ";".join([
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\CommonExtensions\\Microsoft\\FSharp\\Tools",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\CommonExtensions\\Microsoft\\TeamFoundation\\Team Explorer",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\CommonExtensions\\Microsoft\\TestWindow",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\VC\\Linux\\bin\\ConnectionManagerExe",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\IDE\\VC\\VCPackages",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Common7\\Tools\\",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\MSBuild\\Current\\bin\\Roslyn",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\Team Tools\\DiagnosticsHub\\Collector",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\VC\\Tools\\MSVC\\%{msvc_version}\\bin\\HostX64\\x64",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\\\MSBuild\\Current\\Bin\\amd64",
    ]),
    msvc_env_include = ";".join([
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Include\\%{windows_kits_version}\\cppwinrt",
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Include\\%{windows_kits_version}\\shared",
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Include\\%{windows_kits_version}\\um",
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Include\\%{windows_kits_version}\\winrt",
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Include\\%{windows_kits_version}\\ucrt",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\VC\\Auxiliary\\VS\\include",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\VC\\Tools\\MSVC\\%{msvc_version}\\include",
    ]),
    msvc_env_lib = ";".join([
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Lib\\%{windows_kits_version}\\um\\x64",
        "C:\\Program Files (x86)\\Windows Kits\\%{windows_kits_release}\\Lib\\%{windows_kits_version}\\ucrt\\x64",
        "C:\\Program Files\\Microsoft Visual Studio\\%{msvc_release}\\%{msvc_edition}\\VC\\Tools\\MSVC\\%{msvc_version}\\lib\\x64",
    ]),
    msvc_cl_path = "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/cl.exe",
    msvc_ml_path = "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/ml64.exe",
    msvc_link_path = "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/link.exe",
    msvc_lib_path = "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/lib.exe",
    cxx_builtin_include_directories = MSVC_BUILTIN_INCLUDE_PATHS,
    tool_paths = {
        "ar": "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/lib.exe",
        "ml": "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/ml64.exe",
        "cpp": "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/cl.exe",
        "gcc": "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/cl.exe",
        "gcov": "wrapper/bin/msvc_nop.bat",
        "ld": "C:/Program Files/Microsoft Visual Studio/%{msvc_release}/%{msvc_edition}/VC/Tools/MSVC/%{msvc_version}/bin/HostX64/x64/link.exe",
        "nm": "wrapper/bin/msvc_nop.bat",
        "objcopy": "wrapper/bin/msvc_nop.bat",
        "objdump": "wrapper/bin/msvc_nop.bat",
        "strip": "wrapper/bin/msvc_nop.bat",
    },
    archiver_flags = ["/MACHINE:X64"],
    default_link_flags = ["/MACHINE:X64"],
    dbg_mode_debug_flag = "/DEBUG:FULL",
    fastbuild_mode_debug_flag = "/DEBUG:FASTLINK",
    supports_parse_showincludes = True,
)

cc_toolchain(
    name = "msvc_toolchain",
    toolchain_identifier = "msvc_toolchain_config",
    toolchain_config = ":msvc_toolchain_config",
    all_files = ":empty",
    ar_files = ":empty",
    as_files = ":msvc_compiler_files",
    compiler_files = ":msvc_compiler_files",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = True,
)

toolchain(
    name = "windows_msvc_cc_toolchain",
    exec_compatible_with = [
        "@platforms//cpu:x86_64",
        "@platforms//os:windows",
    ],
    target_compatible_with = [
        "@platforms//cpu:x86_64",
        "@platforms//os:windows",
    ],
    toolchain = ":msvc_toolchain",
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
)
