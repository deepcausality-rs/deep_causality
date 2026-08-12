# GCC Source Repository Guidance

This directory owns GCC source materialization and the libstdc++ source graph
used by the `runtimes/libstdcxx` facade.

## Version Source Of Truth

- `version.bzl` owns `GCC_VERSIONS`, `DEFAULT_GCC_VERSION`, `GCC_RELEASES`,
  constraint names, repository names, and version comparison helpers.
- `DEFAULT_GCC_VERSION` should be the latest declared GCC version.
- `//constraints/cxxstdlib:libstdcxx.<major>.<minor>.<patch>` is generated from
  `GCC_VERSIONS`. Keep the full version string, including patch.

## Repository Layout

- `extension/gcc.bzl` creates one concrete GCC repository per version, named
  from `gcc_repo_name(version)`, and a reproducible `@gcc` trampoline repository.
- `gcc.BUILD.bazel` is loaded inside each concrete GCC repository. Keep
  libstdc++ helper `.bzl` files in the main `@llvm` repository and load them
  with `@llvm//...` from this BUILD file.
- `extension/trampoline.BUILD.bazel` is the stable selected facade. It should
  expose only runtime-facing targets consumed through `@gcc`, plus explicit
  version-matrix aliases needed by tests and audits.
- Do not move version-specific BUILD declarations into a Starlark macro just to
  template the repository. Prefer normal BUILD targets plus local
  `GCC_VERSION` branching or constraint selects.

## Version Differences

- Use `select_for_gcc_version` or `select_gcc_version_at_least` for target
  attributes that can be selected by `//constraints/cxxstdlib`.
- Use `gcc_version_at_least_for(GCC_VERSION, ...)` in `gcc.BUILD.bazel` when the
  loaded source graph itself differs by concrete GCC repository.
- Keep source/header/config changes next to the commit that introduces the GCC
  version that needs them. Avoid mentioning future supported versions in earlier
  plumbing commits.

## Validation

For source graph or version-selection changes, run:

    bazel test --config remote //3rd_party/gcc/libstdcxx/tests:autoconf_inventory_test
    bazel test --config remote //3rd_party/gcc/libstdcxx/tests:config_define_audit_test
    bazel build --config remote //runtimes/libstdcxx/tests:toolchain_dynamic_link_smoke_linux_all_versions
    bazel build --config remote //runtimes/libstdcxx/tests:libstdcxx_cxx26_compile_linux_all_versions

For per-version upstreamability, each commit adding a GCC version should build
the smoke targets for that version and all newer supported versions.
