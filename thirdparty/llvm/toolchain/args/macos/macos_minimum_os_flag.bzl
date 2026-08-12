load("@bazel_skylib//rules:common_settings.bzl", "BuildSettingInfo")

def _macos_minimum_os_flag_impl(ctx):
    value = ctx.fragments.apple.macos_minimum_os_flag
    if value == None:
        value = "14.0"
    else:
        value = str(value)

    return [BuildSettingInfo(value = value)]

macos_minimum_os_flag = rule(
    implementation = _macos_minimum_os_flag_impl,
    fragments = ["apple"],
)
