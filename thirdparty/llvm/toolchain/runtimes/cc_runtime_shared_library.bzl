load("@llvm//toolchain/runtimes:with_cfg_runtimes_common.bzl", "configure_builder_for_runtimes")
load("@rules_cc//cc:cc_shared_library.bzl", "cc_shared_library")
load("@rules_cc//cc/common:cc_shared_library_info.bzl", "CcSharedLibraryInfo")
load("@with_cfg.bzl", "with_cfg")

_builder = with_cfg(
    cc_shared_library,
    extra_providers = [CcSharedLibraryInfo],
)

cc_runtime_stage0_shared_library, _cc_stage0_shared_library_internal = configure_builder_for_runtimes(_builder.clone(), "stage0", "dynamic").build()
cc_runtime_stage1_shared_library, _cc_stage1_shared_library_internal = configure_builder_for_runtimes(_builder.clone(), "stage1", "dynamic").build()
cc_runtime_stage1_asan_shared_library, _cc_stage1_asan_shared_library_internal = configure_builder_for_runtimes(
    _builder.clone(),
    "stage1",
    "dynamic",
    inherit_asan = True,
).build()
cc_runtime_stage2_shared_library, _cc_stage2_shared_library_internal = configure_builder_for_runtimes(_builder.clone(), "stage2", "dynamic").build()
cc_runtime_stage3_shared_library, _cc_stage3_shared_library_internal = configure_builder_for_runtimes(_builder.clone(), "stage3", "dynamic").build()
