<!-- Generated with Stardoc: http://skydoc.bazel.build -->

Lake integration for rules_lean.

`lake_workspace` is a repository rule that materializes any Lake workspace
(lakefile + lake-manifest.json) into a Bazel-managed external repo,
downloads the matching Lean toolchain, resolves Lake packages, and exposes
each resolved package as its own `lean_prebuilt_library` target.

Generated targets in `@<name>//:`:

  - `:lean_toolchain` / `:lean_toolchain_def` — register via
    `register_toolchains(...)`.
  - `:<package>` — one `lean_prebuilt_library` per Lake package found under
    `.lake/packages/<package>/`. Target names preserve Lake's directory
    casing (e.g. `:mathlib`, `:batteries`, `:Cli`, `:LeanSearchClient`).
    Consumers depend on multiple packages by listing all needed names.

Fast path for mathlib-based workspaces: if `.lake/packages/mathlib/` is
present after `lake update`, the rule runs `lake exe cache get` to pull
prebuilt oleans from the Reservoir cache (covering mathlib + its transitive
deps). For non-mathlib packages and workspaces, `lake build` produces
oleans from source.

Use via the module extension:

    lake = use_extension("@rules_lean//lean:lake.bzl", "lake")
    lake.workspace(
        name = "lake_deps",
        lean_toolchain = "//:lean-toolchain",
        lakefile = "//:lakefile.lean",
        lake_manifest = "//:lake-manifest.json",
    )
    use_repo(lake, "lake_deps")
    register_toolchains("@lake_deps//:lean_toolchain_def")

    # In a BUILD.bazel:
    lean_test(
        name = "smoke",
        srcs = ["Smoke.lean"],
        entry = "Smoke.lean",
        deps = ["@lake_deps//:mathlib", "@lake_deps//:batteries"],
    )

Hermeticity:
  - The Lean toolchain is downloaded with a known sha256 (see
    private/known_lean_versions.bzl) when the version is pinned there.
    Unpinned versions download unverified (warning emitted).
  - Lake dep revs are pinned by the user's committed lake-manifest.json.
  - Mathlib oleans (when applicable) are content-addressed by mathlib's
    commit hash in the upstream Reservoir cache; integrity is verified by
    Lake.

Constraints on the lakefile passed in:
  - Should be a *deps-only* lakefile (the rule creates a placeholder
    package source). Build directives (`lean_lib`, `lean_exe`) for the
    user's own code don't belong here — those live in Bazel BUILD files
    via the `lean_test` / `lean_emit` rules.

<a id="lake_workspace"></a>

## lake_workspace

<pre>
load("@rules_lean//lean:lake.bzl", "lake_workspace")

lake_workspace(<a href="#lake_workspace-name">name</a>, <a href="#lake_workspace-allow_source_build">allow_source_build</a>, <a href="#lake_workspace-lake_manifest">lake_manifest</a>, <a href="#lake_workspace-lakefile">lakefile</a>, <a href="#lake_workspace-lean_toolchain">lean_toolchain</a>)
</pre>

Materializes a Lake workspace as a Bazel external repo. Produces `:lean_toolchain_def` + one `lean_prebuilt_library` per resolved Lake package (target name = Lake's directory name).

**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lake_workspace-name"></a>name |  A unique name for this repository.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lake_workspace-allow_source_build"></a>allow_source_build |  If True, run `lake build <pkg>` for every package whose oleans aren't covered by `lake exe cache get`. Slow for large packages (mathlib from source is ~30 min); fast and necessary for custom Lake deps that have no upstream cache.   | Boolean | optional |  `False`  |
| <a id="lake_workspace-lake_manifest"></a>lake_manifest |  The committed lake-manifest.json (pins git revs of every Lake dep).   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lake_workspace-lakefile"></a>lakefile |  The lakefile (deps-only — no library/exe directives for the user's own code).   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lake_workspace-lean_toolchain"></a>lean_toolchain |  The `lean-toolchain` file. Drives both Lake's toolchain choice and the Lean binary Bazel downloads.   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |


<a id="lake"></a>

## lake

<pre>
lake = use_extension("@rules_lean//lean:lake.bzl", "lake")
lake.workspace(<a href="#lake.workspace-name">name</a>, <a href="#lake.workspace-allow_source_build">allow_source_build</a>, <a href="#lake.workspace-lake_manifest">lake_manifest</a>, <a href="#lake.workspace-lakefile">lakefile</a>, <a href="#lake.workspace-lean_toolchain">lean_toolchain</a>)
</pre>


**TAG CLASSES**

<a id="lake.workspace"></a>

### workspace

**Attributes**

| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lake.workspace-name"></a>name |  -   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lake.workspace-allow_source_build"></a>allow_source_build |  -   | Boolean | optional |  `False`  |
| <a id="lake.workspace-lake_manifest"></a>lake_manifest |  -   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lake.workspace-lakefile"></a>lakefile |  -   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lake.workspace-lean_toolchain"></a>lean_toolchain |  -   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |


