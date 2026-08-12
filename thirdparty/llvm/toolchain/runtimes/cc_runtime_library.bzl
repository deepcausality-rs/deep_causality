load("@llvm//toolchain/runtimes:with_cfg_runtimes_common.bzl", "configure_builder_for_runtimes")
load("@rules_cc//cc:cc_library.bzl", "cc_library")
load("@with_cfg.bzl", "with_cfg")

_builder = with_cfg(
    cc_library,
)

# TODO(cerisier): Can we remove this so that only root targets are transitioned ?
cc_runtime_stage0_library, _cc_stage0_library_internal = configure_builder_for_runtimes(_builder, "stage0").build()
