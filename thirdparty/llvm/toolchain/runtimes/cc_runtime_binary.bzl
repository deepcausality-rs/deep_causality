load("@llvm//toolchain/runtimes:with_cfg_runtimes_common.bzl", "configure_builder_for_runtimes")
load("@rules_cc//cc:cc_binary.bzl", "cc_binary")
load("@with_cfg.bzl", "with_cfg")

_builder = with_cfg(
    cc_binary,
)

cc_runtime_stage0_binary, _cc_runtime_stage0_binary_internal = configure_builder_for_runtimes(_builder.clone(), "stage0").build()
cc_runtime_stage1_hosted_binary, _cc_runtime_stage1_hosted_binary_internal = configure_builder_for_runtimes(_builder.clone(), "stage1_hosted").build()
