<!-- Generated with Stardoc: http://skydoc.bazel.build -->

Bazel rules for Lean 4.

Four user-facing rules:
  lean_toolchain         — registers a Lean compiler binary + runtime tree.
                           Normally produced by `lake_workspace` (see lake.bzl);
                           can also be declared by hand against a hermetic
                           lean tarball.
  lean_prebuilt_library  — exposes a tree of prebuilt .olean files as a
                           LeanInfo provider consumable via the `deps` attr.
                           The `path_marker` file's parent directory becomes
                           the LEAN_PATH entry.
  lean_test              — stages a set of .lean sources into a module-path
                           layout and invokes the compiler on an entry point.
                           Returns 0 if all type-check, nonzero otherwise.
                           Accepts `deps = [LeanInfo]` and prepends each
                           dep's import root to LEAN_PATH.
  lean_emit              — like lean_test, but the entry file defines
                           `main : IO Unit`; runs it and captures stdout to
                           a declared output file. The Lean kernel becomes
                           the source of truth for emitted artifacts (SQL,
                           TTL, Markdown). Same `deps` plumbing as lean_test.

Design choice: one bundled lean_test per package rather than a per-file
lean_library + transitive .olean tracking. Lean already does fast
incremental type-checking; the value of fine-grained Bazel actions is not
worth the staging-tree complexity at small-to-medium scale.

<a id="lean_emit"></a>

## lean_emit

<pre>
load("@rules_lean//lean:lean.bzl", "lean_emit")

lean_emit(<a href="#lean_emit-name">name</a>, <a href="#lean_emit-deps">deps</a>, <a href="#lean_emit-srcs">srcs</a>, <a href="#lean_emit-out">out</a>, <a href="#lean_emit-entry">entry</a>)
</pre>



**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lean_emit-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lean_emit-deps"></a>deps |  -   | <a href="https://bazel.build/concepts/labels">List of labels</a> | optional |  `[]`  |
| <a id="lean_emit-srcs"></a>srcs |  -   | <a href="https://bazel.build/concepts/labels">List of labels</a> | required |  |
| <a id="lean_emit-out"></a>out |  The emitted artifact (one file). Filename should reflect the artifact kind.   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lean_emit-entry"></a>entry |  Path of the entry-point .lean file (relative to the package) defining `main : IO Unit`. Stdout is captured to `out`.   | String | required |  |


<a id="lean_prebuilt_library"></a>

## lean_prebuilt_library

<pre>
load("@rules_lean//lean:lean.bzl", "lean_prebuilt_library")

lean_prebuilt_library(<a href="#lean_prebuilt_library-name">name</a>, <a href="#lean_prebuilt_library-srcs">srcs</a>, <a href="#lean_prebuilt_library-path_marker">path_marker</a>)
</pre>



**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lean_prebuilt_library-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lean_prebuilt_library-srcs"></a>srcs |  All files in the prebuilt-olean tree (typically `glob(["lib/**"])`).   | <a href="https://bazel.build/concepts/labels">List of labels</a> | required |  |
| <a id="lean_prebuilt_library-path_marker"></a>path_marker |  Anchor file inside the import-root directory. The marker's parent is the LEAN_PATH entry.   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |


<a id="lean_test"></a>

## lean_test

<pre>
load("@rules_lean//lean:lean.bzl", "lean_test")

lean_test(<a href="#lean_test-name">name</a>, <a href="#lean_test-deps">deps</a>, <a href="#lean_test-srcs">srcs</a>, <a href="#lean_test-entry">entry</a>)
</pre>



**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lean_test-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lean_test-deps"></a>deps |  Prebuilt Lean libraries. Each dep's import root is prepended to LEAN_PATH.   | <a href="https://bazel.build/concepts/labels">List of labels</a> | optional |  `[]`  |
| <a id="lean_test-srcs"></a>srcs |  All .lean files in the proof tree. Module path is derived from the file's path relative to this BUILD.bazel's package.   | <a href="https://bazel.build/concepts/labels">List of labels</a> | required |  |
| <a id="lean_test-entry"></a>entry |  Path of the entry-point .lean file relative to the package.   | String | required |  |


<a id="lean_toolchain"></a>

## lean_toolchain

<pre>
load("@rules_lean//lean:lean.bzl", "lean_toolchain")

lean_toolchain(<a href="#lean_toolchain-name">name</a>, <a href="#lean_toolchain-lean">lean</a>, <a href="#lean_toolchain-runtime">runtime</a>)
</pre>



**ATTRIBUTES**


| Name  | Description | Type | Mandatory | Default |
| :------------- | :------------- | :------------- | :------------- | :------------- |
| <a id="lean_toolchain-name"></a>name |  A unique name for this target.   | <a href="https://bazel.build/concepts/labels#target-names">Name</a> | required |  |
| <a id="lean_toolchain-lean"></a>lean |  -   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |
| <a id="lean_toolchain-runtime"></a>runtime |  -   | <a href="https://bazel.build/concepts/labels">Label</a> | required |  |


<a id="LeanInfo"></a>

## LeanInfo

<pre>
load("@rules_lean//lean:lean.bzl", "LeanInfo")

LeanInfo(<a href="#LeanInfo-markers">markers</a>, <a href="#LeanInfo-files">files</a>)
</pre>

A Lean library: a directory of importable .olean files, exposed via a marker file whose parent directory is the LEAN_PATH entry.

**FIELDS**

| Name  | Description |
| :------------- | :------------- |
| <a id="LeanInfo-markers"></a>markers |  depset[File]: each marker's parent directory IS a LEAN_PATH entry.    |
| <a id="LeanInfo-files"></a>files |  depset[File]: all .olean files (and the marker) needed when this lib is consumed.    |


<a id="LeanToolchainInfo"></a>

## LeanToolchainInfo

<pre>
load("@rules_lean//lean:lean.bzl", "LeanToolchainInfo")

LeanToolchainInfo(<a href="#LeanToolchainInfo-lean">lean</a>, <a href="#LeanToolchainInfo-runtime">runtime</a>)
</pre>

Lean 4 compiler binary + runtime tree.

**FIELDS**

| Name  | Description |
| :------------- | :------------- |
| <a id="LeanToolchainInfo-lean"></a>lean |  File: the lean compiler binary (executable).    |
| <a id="LeanToolchainInfo-runtime"></a>runtime |  depset[File]: stdlib oleans, shared libs, etc.    |


