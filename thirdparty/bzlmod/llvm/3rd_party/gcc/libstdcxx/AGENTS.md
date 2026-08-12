# libstdc++ Bazel Porting Guide

This file defines the protocol for porting GCC libstdc++ source and configure
logic into Bazel. It applies to files under `3rd_party/gcc/libstdcxx` and the
matching GCC source declarations in `3rd_party/gcc/gcc.BUILD.bazel`.

The scope here is the libstdc++ Bazel port itself: source lists, generated
headers, header overlays, configure checks, and configure-derived policy. This
file does not describe C++ standard library selection, cc toolchain declaration,
or downstream toolchain integration.

The GCC source graph is versioned. `3rd_party/gcc/version.bzl` is the source of
truth for `GCC_VERSIONS`, release metadata, constraint names, and version helper
functions. The module extension materializes one concrete GCC repository per
version and a reproducible `@gcc` trampoline repository that selects the right
concrete target from the active `//constraints/cxxstdlib:libstdcxx.<version>`
constraint. Keep helper `.bzl` files in the main repository and load them from
the concrete GCC build with `@llvm//...`; `3rd_party/gcc/gcc.BUILD.bazel`
should remain the BUILD file loaded inside each concrete GCC source repository.

## Core Rule

Do not port checks by memory or by manually scanning one file. First build a
mechanical inventory from the GCC sources, then classify every item, then port
the active supported behavior, then validate that the inventory and Bazel model
still agree.

Every GCC check, macro, define, substitution, conditional, and target branch
must end up in exactly one of these states:

- `probe-modeled`: represented by a Bazel compile, link, declaration, header,
  or other toolchain-backed probe.
- `policy-modeled`: represented by a fixed or target-derived Bazel policy.
- `target-derived`: selected from Bazel target/platform information.
- `build-setting-later`: currently fixed, but should become an explicit private
  build setting before claiming full knob parity.
- `not-needed`: configure/build/install/testsuite plumbing that Bazel replaces.
- `unsupported-target`: target family not supported by this libstdc++ port.
- `unsupported-feature`: optional libstdc++ feature not built by this port.

Avoid a vague `unsupported` state. Say whether the item is unsupported because
the target family is out of scope or because the feature is out of scope.

## Source Counterparts

Keep Bazel files shaped like the GCC files they port:

- `3rd_party/gcc/libstdcxx/configure.ac.bzl` is the active
  counterpart of `libstdc++-v3/configure.ac`. It composes the supported
  configure flow.
- `3rd_party/gcc/libstdcxx/acinclude.m4.bzl` is the counterpart of
  `libstdc++-v3/acinclude.m4`.
- `3rd_party/gcc/libstdcxx/crossconfig.m4.bzl` is the counterpart of
  `libstdc++-v3/crossconfig.m4`.
- `3rd_party/gcc/libstdcxx/gcc_config_checks.bzl` is the counterpart
  for GCC top-level `config/*.m4` checks used by libstdc++.
- `3rd_party/gcc/libstdcxx/linkage.m4.bzl` is the counterpart of
  `libstdc++-v3/linkage.m4`.
- `3rd_party/gcc/libstdcxx/autoconf/checks.bzl`,
  `3rd_party/gcc/libstdcxx/autoconf/autoconf_config.bzl`,
  `3rd_party/gcc/libstdcxx/autoconf/autoconf_hdr.bzl`,
  `3rd_party/gcc/libstdcxx/autoconf/cc_configure_probe.bzl`, and
  `3rd_party/gcc/libstdcxx/autoconf/providers.bzl` are local generic
  autoconf mechanics. They should stay free of libstdc++ source-policy
  decisions so a future external autoconf ruleset migration is a thin adapter
  change.
- `libstdc++-v3/linkage.m4` helpers are represented in
  `3rd_party/gcc/libstdcxx/linkage.m4.bzl`, even when they are generic math or
  stdlib declaration/linkage checks.
- `3rd_party/gcc/libstdcxx/target_config.bzl`,
  `3rd_party/gcc/libstdcxx/libstdcxx_cxxconfig_header.bzl`,
  `3rd_party/gcc/libstdcxx/libstdcxx_gthr_headers.bzl`,
  `3rd_party/gcc/libstdcxx/libstdcxx_largefile_config_header.bzl`,
  `3rd_party/gcc/libstdcxx/libstdcxx_symbols_version_script.bzl`, and
  `3rd_party/gcc/libstdcxx/BUILD.bazel` may consume configure-derived policy,
  but should not hide new configure semantics without updating the inventories.
  `runtimes/libstdcxx/BUILD.bazel` is only the public runtime facade.

Each source-counterpart `.bzl` file should start with a short comment naming
the GCC file(s) it was ported from. When a group of upstream macros is collapsed
into one Bazel helper, leave an anchor comment listing the upstream macro names.

## Required Tracking Files

Maintain these human-readable tracking files in `3rd_party/gcc/libstdcxx/docs`:

- `autoconf.checks.md`: checklist of check definitions available from GCC
  sources. This covers GCC top-level `config/*.m4` macros, libstdc++ custom
  `GLIBCXX_*` macros, and helper macros such as those in `linkage.m4`.
- `autoconf.usage.md`: checklist of configure usage, in the order
  `configure.ac` uses checks and macros. This file explains which definitions
  are actually reached for the current supported Linux GNU configuration and
  which branches are inactive.
- `autoconf.README.md`: glossary and report for the configure model. For every
  check, it records what the upstream check does, when it runs, what outputs it
  produces, and the Bazel status.

The lower-level machine status files, such as `config_define_status.txt` and
`config_macro_status.txt`, also live in `3rd_party/gcc/libstdcxx/docs` as audit
inputs. They should agree with the three Markdown files, but they are not a
substitute for the glossary.

## Inventory Scripts

The inventory must be scriptable and reproducible. Scripts should live in
`3rd_party/gcc/libstdcxx/tests` and be exposed through Bazel tests or runnable
targets.

The current entry point is
`3rd_party/gcc/libstdcxx/tests/autoconf_inventory.sh`.
Its `inventory` mode prints raw discoveries: macro definitions, macro uses,
config defines, check form counts, and check arguments. Raw discoveries are not
checklist entries by themselves. Run
`bazel run //3rd_party/gcc/libstdcxx/tests:autoconf_inventory -- inventory` to
inspect that raw queue through Bazel runfiles for the selected libstdc++
constraint. The aggregate Bazel targets
`//3rd_party/gcc/libstdcxx/tests:config_define_audit_test` and
`//3rd_party/gcc/libstdcxx/tests:autoconf_inventory_test` run the audit for every
version in `GCC_VERSIONS`. The first uses `check-status` mode to verify status
coverage and modeled-source references. The second uses `check-docs` mode to
verify that the Markdown checklists and glossary mention every status-tracked
configure macro and every reviewed concrete `AC_ARG_*`, `AC_CHECK_*`, and
`AC_COMPUTE_INT` argument.

The scripts should read the fetched GCC source files from Bazel runfiles, not
from an arbitrary local GCC checkout. They should cover at least:

- `libstdc++-v3/configure.ac`
- `libstdc++-v3/acinclude.m4`
- `libstdc++-v3/linkage.m4`
- `libstdc++-v3/crossconfig.m4`
- `libstdc++-v3/configure.host`
- selected top-level GCC `config/*.m4` files exported by the version-specific
  GCC repositories through `@gcc` facade aliases

The inventory must extract:

- macro definitions such as `AC_DEFUN([GLIBCXX_*])` and `AC_DEFUN([GCC_*])`;
- macro uses, including direct calls, `AC_REQUIRE`, and `AC_BEFORE`;
- config defines from `AC_DEFINE`, `AC_DEFINE_UNQUOTED`, and `AH_VERBATIM`;
- header, function, declaration, type, member, compile, link, run, and compute
  checks;
- substitutions and conditionals from `AC_SUBST`, `AM_CONDITIONAL`, and
  `GLIBCXX_CONDITIONAL`;
- target branches and option branches that change source lists, headers, flags,
  ABI, or generated config output.

Scripts must fail when GCC adds a discovered status-tracked item that is missing
from the tracking files. They should print the missing upstream symbol or macro
name and the source file where it was discovered. Raw concrete check arguments
are a review queue until they have been mapped to an implementation or explicit
classification.

Do not run `make`, `./configure`, autoconf, automake, libtool, or GCC build
scripts to produce the inventory. The inventory is a static source audit.

## Porting Protocol

Follow this sequence for every configure-check change and every GCC update:

1. Refresh the GCC sparse source list only as needed. Do not fetch all GCC
   sources just to inspect configure logic. Add explicit files to
   `3rd_party/gcc/extension/gcc.bzl` and export them from
   `3rd_party/gcc/gcc.BUILD.bazel`.
2. Run the inventory scripts. Treat their output as a review queue, not as a
   completed checklist. Update
   `3rd_party/gcc/libstdcxx/docs/autoconf.checks.md`,
   `3rd_party/gcc/libstdcxx/docs/autoconf.usage.md`, and
   `3rd_party/gcc/libstdcxx/docs/autoconf.README.md` only for checks that are
   implemented, target-derived, deliberately deferred, not needed, or
   explicitly out of scope.
3. Classify every new or changed item before editing probe behavior. Use the
   status vocabulary from this file.
4. Port active Linux GNU behavior into the source-counterpart `.bzl` file that
   matches the upstream source. Keep unsupported target branches documented as
   inactive notes with the upstream condition and a reason.
5. Prefer Bazel-native structure. Use normal Bazel targets, `cc_library` deps,
   generated headers, and existing helper rules before adding a custom rule.
6. For configure probes, use the selected Bazel C/C++ toolchain. Do not
   manually synthesize libc include paths or compiler command lines. If a probe
   rule needs to compile or link, it must get compiler, linker, system include,
   and target flags from the toolchain.
7. Keep temporary shell actions small. `run_shell` is acceptable for now, but
   design probe data and actions so the shell runner can later be replaced for
   Windows portability.
8. Do not use Python for configure inventory or probe execution unless this
   policy is explicitly changed. Prefer shell plus standard text tools for the
   audit scripts.
9. Update `3rd_party/gcc/libstdcxx/docs/config_define_status.txt` and
   `3rd_party/gcc/libstdcxx/docs/config_macro_status.txt` only after the
   Markdown checklists and glossary explain the semantic status. Do not mark a
   check `modeled` merely because the inventory found it.
10. Keep version-specific source, header, and configure differences adjacent to
    the commit that introduces the corresponding GCC version. Do not land broad
    plumbing that mentions future versions before those versions exist in
    `GCC_VERSIONS`; model differences with `select_for_gcc_version`,
    `select_gcc_version_at_least`, or a local `GCC_VERSION` branch in
    `gcc.BUILD.bazel` only where BUILD loading needs that shape.
11. Validate with the package audit tests, selected runtime wrapper targets,
    all-version libstdc++ smoke builds, and a real `e2e/rules_cc` smoke test
    when behavior changes.

## Modeling Rules

Use a probe when GCC determines the answer by compiling, linking, checking a
declaration, or inspecting headers for the target. Use policy only when GCC's
answer is a configure option, an installation/build-system concern, or a target
decision that Bazel already knows through constraints.

When GCC uses an aggregate check, preserve the aggregate shape unless there is
a clear reason to split it. If a split is needed, the checklist and glossary
must explain how the split maps back to upstream.

When GCC has native and cross behavior, document both. Only activate the
current supported Linux GNU path unless the libstdc++ support matrix has been
expanded deliberately.

When GCC substitutes source paths, version scripts, headers, or flags, treat
that as configure behavior even if it does not create a `config.h` define. It
still belongs in the usage checklist and glossary.

## Validation

Before committing configure-check changes, run from the repository root:

    bazel run //internal_tools:buildifier.check
    bazel build --config remote --platforms=//runtimes/libstdcxx/tests:linux_x86_64_libstdcxx.17.0.0 //runtimes/libstdcxx:headers_include_search_directory //runtimes/libstdcxx:abi_headers_include_search_directory //runtimes/libstdcxx:libstdcxx.shared //runtimes/libstdcxx:libstdcxx_library_search_directory //3rd_party/gcc/libstdcxx:libstdcxx_config_h //3rd_party/gcc/libstdcxx:target_config //3rd_party/gcc/libstdcxx/autoconf:autoconf_config //3rd_party/gcc/libstdcxx/autoconf:autoconf_hdr //3rd_party/gcc/libstdcxx/autoconf:checks //3rd_party/gcc/libstdcxx/autoconf:cc_configure_probe //3rd_party/gcc/libstdcxx:gcc_config_checks //3rd_party/gcc/libstdcxx:linkage_checks //3rd_party/gcc/libstdcxx:configure_ac_checks
    bazel test --config remote //3rd_party/gcc/libstdcxx/tests:autoconf_inventory_test
    bazel test --config remote //3rd_party/gcc/libstdcxx/tests:config_define_audit_test
    bazel build --config remote //runtimes/libstdcxx/tests:toolchain_dynamic_link_smoke_linux_all_versions
    bazel build --config remote //runtimes/libstdcxx/tests:libstdcxx_cxx26_compile_linux_all_versions

When the change can affect real runtime behavior, also run from
`e2e/rules_cc`:

    bazel test --config remote //:libstdcxx_main_dynamic_output_test //:libstdcxx_main_dynamic_with_linkopts_output_test

For a GCC update, the acceptance condition is not only that Bazel builds. The
inventory scripts must show that every upstream configure item is classified in
the checklists and described in `autoconf.README.md`, and every commit adding a
new GCC version must still build the smoke targets for that version and all
newer supported versions.
