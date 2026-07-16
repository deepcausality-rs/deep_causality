# Changelog

All notable changes to rules_lean. The format is loosely
[Keep a Changelog](https://keepachangelog.com/) — version headers
mirror the published bazel-registry entries.

## 0.5.4 — tree-shake mathlib's olean download (`cache_roots`)

`lake.workspace(cache_roots = [...])` restricts mathlib's `lake exe cache get`
to the given root modules plus their transitive closure, instead of fetching all
of mathlib. mathlib's cache CLI already supports this (`get [ARGS]` →
`filterByRootModules`); rules_lean simply never passed the args. The fetch stays
sound — you cannot under-fetch a module you import.

Measured against mathlib @ v4.30.0-rc2 with a Lean→SQL emitter's 6 roots
(`Data.Fintype.Basic`, `Data.Fin.Basic`, `Data.List.Basic`, `Data.List.Infix`,
`Order.Basic`, `Order.BoundedOrder.Basic`):

  full fetch        8297 files  (~2 GB)
  6-root closure     595 files  (52 MB)   — 93% fewer files

Empty `cache_roots` (the default) keeps the fetch-everything behaviour, so
existing workspaces are unaffected.

## 0.5.3 — configurable prebuilt-olean cache

- `lake.workspace` can now fetch a package's prebuilt oleans from a **consumer-configurable
  cache** instead of source-building it (e.g. cslib, which Reservoir doesn't serve — a
  ~2.6k-job compile on every cold output base). Declare `olean_cache_packages = ["cslib"]`;
  the cache base is set via the `olean_cache` tag attr (MODULE) or the `LEAN_OLEAN_CACHE`
  repo_env (`--repo_env=...` in .bazelrc — the repo_env wins). Never hardcoded/public.
  Artifact path: `<base>/<pkg>-<rev12>-<leanver>-<platform>.tar.gz` (the package's
  `.lake/build` tree). No base configured → source-build fallback (allow_source_build).
- Each resolved package now also exposes a `:<pkg>_build_tree` filegroup, so producing the
  cache tarball is a hermetic `pkg_tar` over it (no manual host `tar` / AppleDouble cruft).
- Validated: with the cache set, cslib is fetched + unpacked (0 source-build jobs), green.

## 0.5.2 — shared Lean toolchain (dedup)

- The `lake` module extension now extracts the Lean toolchain **once per version**
  into a shared `lean_dist` repo; every `lake.workspace` symlinks it instead of
  extracting its own ~2.5G copy. Previously N workspaces (e.g. a project plus the
  lake workspaces of `rules_lang` / `rules_postgres` / `rules_spec`) each carried a
  full toolchain — 4× the same toolchain ≈ 10G **per output base**, the dominant
  cause of multi-GB Lean checkouts / CI ENOSPC. Now 1× per version.
- Backward-compatible: `@<ws>//:lean_toolchain_def` is still a real `toolchain()`
  (so `register_toolchains(...)` is unchanged) — it now points at the shared
  `lean_dist` toolchain; `@<ws>//:lean_toolchain` is an alias to it. The per-package
  `lean_prebuilt_library` targets and the imports manifest are unchanged.

## 0.4.0 — compiled libraries + cross-repo olean artifacts

- New `lean_library`: compiles `.lean` sources to a persistent `.olean`
  import-root tree (build outputs) and exposes it as `LeanInfo`, so one module
  can be a **compiled** dependency of another (no source re-share, no
  recompile). `DefaultInfo` carries the library's own tree; `LeanInfo` carries
  the transitive closure (own + deps).
- New `lean_olean_archive`: bundles a `lean_library`'s own `.olean` tree into a
  tarball — the deployable cross-repo release artifact.
- New `lean_imported_library`: exposes an unpacked `.olean` tarball (e.g. from
  an `http_archive` of a release asset) as `LeanInfo` with no recompile — the
  consume side. Shares the `lean_prebuilt_library` implementation.
- These three form the cross-repo compiled-olean seam (split a monolithic Lean
  library into modules that publish/consume prebuilt oleans). `.olean` is
  neither Lean-version- nor architecture-portable, so artifacts are built
  per-`(lean-version, os, arch)` and consumers pin the matching toolchain;
  Lean rejects a mismatched olean loudly at use.
- Round-trip example under `examples/olean_roundtrip/`.
- **Cross-namespace deps.** A `lean_library` dep that shares the consumer's
  top-level namespace (e.g. two libs both under `Aion/`) is staged into the
  single compile root, since Lean commits to the first `LEAN_PATH` root owning a
  namespace and won't fall through to siblings. Disjoint deps (Mathlib, …) stay
  on `LEAN_PATH`, uncopied. This makes `lean_library`→`lean_library` deps within
  one namespace work (the basis for splitting a monolith in place).
- **Shell-free compile.** All four rules (`lean_library`, `lean_test`,
  `lean_emit`, `lean_main_test`) now drive the compiler through a self-contained
  Lean topo-compile driver (`lean/private/topo_compile.lean`, invoked
  `lean --run …` via `ctx.actions.run`) instead of a `run_shell` `tsort`
  pipeline — staging/copying uses native `IO.FS`; the only subprocess is `lean`.
  `lean_test`/`lean_main_test` now type-check / run at build time (a failure
  fails the build); their test executable is a trivial pass.

## 0.3.9 — import-topological compile order (`glob()`-safe srcs)

- `lean_test`, `lean_emit`, and `lean_main_test` now compile their
  `srcs` in **import-topological order** instead of literal list
  order. Previously, Lean's requirement that a module's imports be
  compiled to `.olean` first meant `srcs` had to be hand-ordered
  with dependencies before dependents — and a natural
  `glob(["**/*.lean"])` would fail, because a root file like
  `Trading.lean` sorts before `Trading/Fx/Basic.lean` (`.` < `/`)
  yet imports it. Now the generated runner derives the order at
  execution time: it parses each staged file's `import` lines,
  keeps edges to modules that are themselves in `srcs`, and
  `tsort`s the graph. **`srcs = glob([...])` now Just Works**;
  explicit ordered lists keep working unchanged (any valid manual
  order is already a valid topological order).
- Implementation: a portable bash helper (`__lean_topo_compile`,
  shared via `_topo_compile_block`) using only
  `grep`/`sed`/`cut`/`tsort`/`mktemp` — no bash-4 associative
  arrays, so it runs on macOS's stock bash 3.2. Out-of-`srcs`
  imports (Mathlib, dep packages) are ignored; genuine import
  cycles still fail the build (Lean rejects them downstream).

## 0.3.5 — `lean_main_test` rule

- New `lean_main_test(name, srcs, entry, deps, data)` rule in
  `lean/lean.bzl`. Compiles + runs a Lean entry whose
  `main : IO UInt32` returns the test result via its exit code
  (0 = pass, non-zero = fail). No expected-output diff required —
  use when the Lean script self-validates (round-trip stability,
  structural equivalence) and you'd otherwise need a committed
  `expected.txt` fixture just to flag drift. Accepts the same
  `deps` (LeanInfo) + `data` (workspace-relative staging) attrs
  as `lean_emit` / `lean_regen_test`.
- New smoke `examples/regen_smoke/regen_smoke_exit` runs
  `ExitZero.lean` (`pure 0`) to exercise the happy path. A
  companion `ExitOne.lean` (`pure 1`) is committed for manual
  negative testing.

## 0.3.4 — `lean_emit.data` accepts external-repo files

- `lean_emit.data` now stages files at their workspace-relative path
  (e.g. `examples/regen_smoke/fixture.txt`) instead of the package-
  relative path the 0.3.3 release used. Externally-sourced data
  (`@some_repo//path:file`) is staged at `path/file` — the `..//<canon>`
  prefix in bazel's external-repo short_path is stripped. This lets
  consumers pull fixtures directly from upstream repos
  (e.g. `@postgres_src//:src/include/catalog/pg_namespace.dat`)
  instead of vendoring them.
- Smoke updated: `examples/regen_smoke/EchoFixture.lean` reads the
  full workspace-relative path.

## 0.3.3 — `lean_emit.data` attr

- `lean_emit` (and `lean_regen_test`) gain a `data` attr — non-Lean
  fixture files staged alongside `srcs` in the action's work directory
  without being compiled. The entry runs from that work dir, so it
  can `IO.FS.readFile` them by their package-relative path. Typical
  use: `.dat` / `.txt` / `.json` inputs the entry parses. Enabled
  rules_postgres' Lean-native `Pg.Catalog.Dat` round-trip gate
  against the vendored `pg_namespace.dat` sample.
- New smoke `examples/regen_smoke/regen_smoke_data` exercises the
  attr end-to-end: a Lean main reads `fixture.txt` and echoes it;
  the diff_test verifies the captured stdout matches the same
  `fixture.txt` (proving the data file is reachable from the Lean
  entry's relative-path `readFile`).

## 0.3.2 — `lean_regen_test` macro

- New `lean_regen_test(name, srcs, entry, expected, ...)` macro in
  `lean/lean.bzl`. Wraps `lean_emit` + skylib `diff_test` to assert a
  committed artifact matches the current Lean-emit output for a given
  Lean main. Captures the "Lean spec is source-of-truth; emitted X
  was generated from it" pattern that rules_postgres' Pg.Ir cluster
  Gate 1 was building on top of `lean_emit` + `diff_test` by hand.
- Smoke test under `examples/regen_smoke/` exercises the macro
  end-to-end against a tiny `Hello.lean` and a committed
  `expected.txt`.

## 0.3.1 — External-repo Lean sources

- `_module_path` and `_lean_test_impl` now handle external-repo
  source layouts (`../<repo>+/<package>/<file>` short_paths). Lets
  `lean_library` and `lean_test` targets in a consumer module
  reference Lean sources from a `bazel_dep` repo without copying
  the files into the consumer's tree. Used by rules_postgres'
  `lean/Pg/Ir/Emit/` modules when consumed through the registry
  rather than through a `local_path_override`.

## 0.3.0 — RulesLean Lean library + lake_imports_manifest

- Promote `v0.3.0-rc1` and pin to Lean `v4.30.0-rc2` for cslib compatibility.
- Add `RulesLean.Internal.Closure` (transitive olean closure computed from the
  Lake manifest) and `RulesLean.Internal.AxiomDeps` (`declaredAxioms` +
  `isAxiom`, Internal v0.1).
- CI: add a `ruleslean_library` matrix job so the in-tree Lean library is
  built + tested on every PR.
- Untrack `.vscode/` and notebook scratch artifacts; tighten `.gitignore`.

### 0.3.0-rc1 — RulesLean scaffold + manifest tooling

- Introduce the `RulesLean` Lean library under `lean/lib/` (Olean + Lake) and
  wire it through Bazel.
- Add the `lake_imports_manifest` target: workspace API,
  `exportedConstants` + `containsConstant`, and the `Internal` namespace
  convention with `namespacePackageIndex`.
- Add `tools/reservoir_manifest.py` — a stdlib-only Reservoir index fetcher.
- Update the install snippet to point at `fastverk/bazel-registry`.

## 0.2.2 — Un-dev bazel_skylib

- Promote `bazel_skylib` out of `dev_dependency` so downstream consumers can
  actually `load()` `lean/BUILD.bazel` without re-declaring it.

## 0.2.1 — README, license, CI, smoke test

- Bump module version to 0.2.1.
- Add `README.md`, MIT `LICENSE`, and the PR-gate CI workflow.
- Add a Batteries-only `lake_workspace` smoke test.
- Apply buildifier formatting fixes across the tree.

## 0.2.0 — Generalized Lake integration + stardoc

- Generalize the Lake integration so `lake_workspace` works for arbitrary
  Lake projects instead of being hard-coded to a single layout.
- Add stardoc generation for the public rules.

## 0.1.0 — Initial release

- First public cut of `rules_lean`: `lean_test`, `lean_emit`,
  `lean_prebuilt_library`, `lean_toolchain`, and the initial `lake_workspace`
  repository rule + `lake` module extension reusing Mathlib's Reservoir
  cache.
