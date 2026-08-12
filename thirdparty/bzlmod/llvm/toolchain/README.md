# Toolchain definition

Toolchains are defined as a composition of canonical argument groups, each representing a stable semantic aspect of the toolchain:
- Linker choice.
- Sysroot.
- Resource directory.
- Default startfiles.
- Hermetic compile and link flags.
- Deterministic compile and link flags.
- Default compile and link flags.
- Default libs.
- Sanitizers.

Each group has a stable meaning across all platforms and targets, even if its concrete flags differ or the group is empty.

Groups are defined at the level of the most constrained targets, so that more feature-rich or hosted environments compose or extend existing groups rather than redefining their semantics. For example, default_libs may be empty for freestanding or embedded targets, while being non-empty for hosted environments.

### Package structure

**//toolchain/args:BUILD.bazel**:
- Defines the canonical meaning of each argument group.
- Groups may be empty, but must exist.
- No platform select() logic.

This package defines what each group means, independent of platform or environment.

**//toolchain/args/\<platform\>:BUILD.bazel**:
- Defines platform-specific implementations of argument groups.
- May replace or extend canonical groups where platform semantics differ.
- No platform select() logic.

These packages adjust how a canonical group is implemented on a given platform, without changing its semantic intent.

**//toolchain:BUILD.bazel**:
- Assembles the final toolchain by selecting between argument groups.
- Contains only cc_args_list targets.
- No raw flags or action bindings.
- **All platform selection lives here.**

This package answers which groups apply on which platforms, and nothing else.

TODO(cerisier): Support macOS specific flags (objc and frameworks). Still needed ?

# Other Resources

https://github.com/bazelbuild/rules_cc/blob/main/docs/toolchain_api.md
https://github.com/CACI-International/cpp-toolchain/blob/74efb5bc636f48db86652f0cfdb7d46af100e51f/bazel/toolchain.bzl#L31
https://github.com/cortecs-lang/cortecs-cc-toolchain/blob/78792fba9eec75bfac14a0cd20cb0e4973175871/sysroot/alpine/BUILD
https://github.com/lukasoyen/bazel_linux_packages/blob/c50a9bf22122a507d2bb5e348231413a05e6d90f/e2e/toolchains/gcc/BUILD.bazel
https://github.com/lowRISC/opentitan/blob/6d29bb86581892c43d3dea8856b275dc3a40c575/toolchain/README.md
https://cs.opensource.google/pigweed/pigweed/+/main:pw_toolchain/cc/args/BUILD.bazel
https://github.com/envoyproxy/envoy/blob/main/bazel/rbe/toolchains/configs/linux/clang/cc/cc_toolchain_config.bzl
