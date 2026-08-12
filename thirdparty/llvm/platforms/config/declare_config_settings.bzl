load("@bazel_skylib//lib:selects.bzl", "selects")
load("//constraints/libc:libc_versions.bzl", "GLIBCS", "LIBCS")
load("//platforms:common.bzl", "LIBC_SUPPORTED_TARGETS", "SUPPORTED_TARGETS")
load("//toolchain:bootstrap_stages.bzl", "BOOTSTRAP_STAGES")

def declare_config_settings():
    for (target_os, target_cpu) in SUPPORTED_TARGETS:
        native.config_setting(
            name = "{}_{}".format(target_os, target_cpu),
            constraint_values = [
                "@platforms//cpu:" + target_cpu,
                "@platforms//os:" + target_os,
            ],
            visibility = ["//visibility:public"],
        )

        for bootstrap_stage in BOOTSTRAP_STAGES:
            native.config_setting(
                name = "%s_%s_%s" % (target_os, target_cpu, bootstrap_stage),
                constraint_values = [
                    "@platforms//cpu:" + target_cpu,
                    "@platforms//os:" + target_os,
                ],
                flag_values = {
                    "//toolchain:bootstrap_stage": bootstrap_stage,
                },
                visibility = ["//visibility:public"],
            )

    declare_config_settings_libc_aware()

def declare_config_settings_libc_aware():
    for (target_os, target_cpu) in LIBC_SUPPORTED_TARGETS:
        for libc in LIBCS + ["unconstrained"]:
            native.config_setting(
                name = "{}_{}_{}".format(target_os, target_cpu, libc),
                constraint_values = [
                    "@platforms//cpu:{}".format(target_cpu),
                    "@platforms//os:{}".format(target_os),
                    "//constraints/libc:{}".format(libc),
                ],
                visibility = ["//visibility:public"],
            )

        selects.config_setting_group(
            name = "{}_{}_gnu".format(target_os, target_cpu),
            match_all = [
                "@platforms//cpu:{}".format(target_cpu),
                "@platforms//os:{}".format(target_os),
                ":gnu",
            ],
            visibility = ["//visibility:public"],
        )

    selects.config_setting_group(
        name = "gnu",
        match_any = [
            "//constraints/libc:{}".format(libc)
            for libc in GLIBCS
        ] + [
            "{}_{}_unconstrained".format(target_os, target_cpu)
            for (target_os, target_cpu) in LIBC_SUPPORTED_TARGETS
        ],
        visibility = ["//visibility:public"],
    )

    native.config_setting(
        name = "musl",
        constraint_values = [
            "//constraints/libc:musl",
        ],
        visibility = ["//visibility:public"],
    )
