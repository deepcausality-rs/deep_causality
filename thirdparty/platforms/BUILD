load("@package_metadata//licenses/rules:license.bzl", "license")
load("@package_metadata//rules:package_metadata.bzl", "package_metadata")
load("@rules_license//rules:license.bzl", deprecated_license = "license")

package(
    default_applicable_licenses = [
        ":license",
        ":package_metadata",
    ],
    default_visibility = ["//visibility:public"],
)

package_metadata(
    name = "package_metadata",
    attributes = [
        ":package_metadata_license",
    ],
    purl = "pkg:bazel/{}@{}".format(
        module_name(),
        module_version(),
    ) if module_version() else "pkg:bazel/{}".format(module_name()),
    visibility = ["//visibility:public"],
)

license(
    name = "package_metadata_license",
    kind = "@package_metadata//licenses/spdx:Apache-2.0",
    text = "LICENSE",
)

deprecated_license(
    name = "license",
    license_kinds = [
        "@rules_license//licenses/spdx:Apache-2.0",
    ],
    license_text = "LICENSE",
)

exports_files([
    "LICENSE",
    "MODULE.bazel",
])

filegroup(
    name = "srcs",
    srcs = [
        "BUILD",
        "WORKSPACE",
        "//cpu:srcs",
        "//os:srcs",
        "//host:srcs",
    ],
)

# For use in Incompatible Target Skipping:
# https://docs.bazel.build/versions/main/platforms.html#skipping-incompatible-targets
#
# Specifically this lets targets declare incompatibility with some set of
# platforms. See
# https://docs.bazel.build/versions/main/platforms.html#more-expressive-constraints
# for some more details.
constraint_setting(name = "incompatible_setting")

constraint_value(
    name = "incompatible",
    constraint_setting = ":incompatible_setting",
)
