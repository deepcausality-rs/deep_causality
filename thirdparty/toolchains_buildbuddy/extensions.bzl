load("//:rules.bzl", "DEFAULT_IMAGE", "UBUNTU16_04_IMAGE", "UBUNTU20_04_IMAGE", "UBUNTU22_04_IMAGE", "UBUNTU24_04_IMAGE", bb_macro = "buildbuddy")

def _ext_impl(mctx):
    found_root_module = False
    root_module = struct()
    for m in mctx.modules:
        if m.is_root:
            found_root_module = True
            root_module = m
            break
    if not found_root_module:
        fail("buildbuddy must be used in root module")

    tags = root_module.tags
    if len(tags.platform) > 1:
        fail("buildbuddy.platform can only be specified once per module")
    if len(tags.gcc_toolchain) > 1:
        fail("buildbuddy.gcc_toolchain can only be specified once per module")
    if len(tags.msvc_toolchain) > 1:
        fail("buildbuddy.msvc_toolchain can only be specified once per module")
    platform_tag = tags.platform[0] if tags.platform else None
    gcc_toolchain_tag = tags.gcc_toolchain[0] if tags.gcc_toolchain else None
    msvc_toolchain_tag = tags.msvc_toolchain[0] if tags.msvc_toolchain else None

    macro_args = dict()
    if platform_tag:
        if platform_tag.buildbuddy_container_image and platform_tag.container_image:
            fail("buildbuddy.platform.container_image and buildbuddy.platform.buildbuddy_container_image cannot be specified together")
        if platform_tag.buildbuddy_container_image:
            if platform_tag.buildbuddy_container_image == "DEFAULT_IMAGE":
                image = DEFAULT_IMAGE
            elif platform_tag.buildbuddy_container_image == "UBUNTU16_04_IMAGE":
                image = UBUNTU16_04_IMAGE
            elif platform_tag.buildbuddy_container_image == "UBUNTU20_04_IMAGE":
                image = UBUNTU20_04_IMAGE
            elif platform_tag.buildbuddy_container_image == "UBUNTU22_04_IMAGE":
                image = UBUNTU22_04_IMAGE
            elif platform_tag.buildbuddy_container_image == "UBUNTU24_04_IMAGE":
                image = UBUNTU24_04_IMAGE
            else:
                fail("Unknown buildbuddy.platform.buildbuddy_container_image value: %s" % platform_tag.buildbuddy_container_image)
        else:
            image = platform_tag.container_image
        macro_args |= {
            "container_image": image,
        }
    if gcc_toolchain_tag:
        macro_args |= {
            "gcc_version": gcc_toolchain_tag.gcc_major_version,
            "extra_cxx_builtin_include_directories": gcc_toolchain_tag.extra_cxx_builtin_include_directories,
        }
    if msvc_toolchain_tag:
        macro_args |= {
            "msvc_edition": msvc_toolchain_tag.msvc_edition,
            "msvc_release": msvc_toolchain_tag.msvc_release,
            "msvc_version": msvc_toolchain_tag.msvc_version,
            "windows_kits_release": msvc_toolchain_tag.windows_kits_release,
            "windows_kits_version": msvc_toolchain_tag.windows_kits_version,
        }

    bb_macro(
        name = "buildbuddy_toolchain",
        **macro_args
    )

buildbuddy = module_extension(
    implementation = _ext_impl,
    tag_classes = {
        "gcc_toolchain": tag_class(
            attrs = {
                "gcc_major_version": attr.string(),
                "extra_cxx_builtin_include_directories": attr.string_list(),
            },
        ),
        "msvc_toolchain": tag_class(
            attrs = {
                "msvc_edition": attr.string(values = ["Community", "Professional", "Enterprise"]),
                "msvc_release": attr.string(),
                "msvc_version": attr.string(),
                "windows_kits_release": attr.string(),
                "windows_kits_version": attr.string(),
            },
        ),
        "platform": tag_class(
            attrs = {
                # Only one of these can be specified
                "container_image": attr.string(doc = "custom container image to use as execution sandbox"),
                "buildbuddy_container_image": attr.string(
                    values = [
                        "DEFAULT_IMAGE",
                        "UBUNTU16_04_IMAGE",
                        "UBUNTU20_04_IMAGE",
                        "UBUNTU22_04_IMAGE",
                        "UBUNTU24_04_IMAGE",
                    ],
                    doc = "buildbuddy's pre-built container image to use as execution sandbox",
                ),
            },
        ),
    },
)
