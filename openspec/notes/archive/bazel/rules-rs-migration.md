# Migrating to rules_rs

> **Archived, unmodified.** This is the plan as written before implementation, kept as the record
> of what was intended and why. It was archived by a rename, so its predictions were never
> reconciled against what shipped, and a few are wrong: `llvm_version` was already 22.1.8 when the
> note claimed 22.1.6, the `toolchains.toolchain` example predates the `extra_rustc_flags` block
> that silences the darwin `-rtlib` diagnostic, and Stage 2 ended up absorbing the dependency
> derivation this note assigned to Stage 3. For current state read MODULE.bazel and the BUILD
> files; they are the source of truth.

The Bazel build works, but it pays for that with three standing costs: a fork of rules_rust pinned
by commit, 94 MB of vendored crate sources that have to be re-rendered by hand, and a dependency
graph written twice, once in Cargo.toml and once in BUILD.bazel. rules_rs removes the reason for
all three. This note records what the migration actually involves, in the order the work should
land.

| | |
|---|---|
| Issue | [#754](https://github.com/deepcausality-rs/deep_causality/issues/754) |
| From | rules_rust 0.73.0, pinned to `marvin-hansen/rules_rust@47a9fd9` by `git_override` |
| To | `rules_rs` 0.0.107 (BCR), which pins `hermeticbuild/rules_rust@9ec1223` itself |
| Reference port | `../scrith`, built on rules_rs from the start |
| Scale | 76 first-party BUILD files, 45 crates, 496 Rust targets, 94 MB vendored sources |
| Bazel | 9.2.0, unchanged |

## What changes

**Vendoring goes away, and it is not optional.** `crate.from_cargo` accepts `name`, `cargo_toml`,
`cargo_lock`, `cargo_config`, `generate_lint_config`, `use_home_cargo_credentials`,
`platform_triples`, `use_legacy_rules_rust_platforms`, `validate_lockfile` and `debug`. There is no
`vendor_dir`. Adopting rules_rs deletes `thirdparty/crates/` as a consequence, not as a choice.

**The tests/BUILD.bazel split can be folded in.** Each crate currently has two BUILD files because
a BUILD file in `tests/` makes it a separate package, and a glob in the parent package cannot
cross a package boundary. Deleting the child BUILD file dissolves the boundary. Every glob and
`crate_root` in the merged file gains a `tests/` prefix.

**Dependencies get declared once.** Internal path deps and external deps move into root
`[workspace.dependencies]`; members write `dep = { workspace = true }`. In BUILD files the
hand-written dep lists become `all_crate_deps(normal = True)` plus `aliases()` and `lint_config()`
from the generated `@crates//:defs.bzl`.

## What the investigation established

Several assumptions that framed this migration turned out to be wrong. They are corrected here
because they change the shape of the work.

**rules_rs is not a new rule set for BUILD files.** `@rules_rs//rs:rust_library.bzl`,
`rust_test.bzl` and `rust_binary.bzl` are three-line re-exports:

```python
load("@rules_rust//rust:defs.bzl", _rust_library = "rust_library")

rust_library = _rust_library
```

What rules_rs replaces is the dependency-resolution and toolchain layer. `rust_doc`,
`rust_doc_test` and `rust_test_suite` keep coming from `@rules_rust//rust:defs.bzl`, exactly as
scrith loads them today. Rule attributes do not change, so the existing stanzas survive the
migration nearly untouched.

**The rules_rust fork pin can be dropped.** rules_rs provisions its own rules_rust through an
`http_archive` in `rs/rules_rust.bzl`, pinned to `hermeticbuild/rules_rust@9ec1223`, and exposes a
`patch` tag class for anything on top. That fork already carries the doc-test fix this repo's fork
exists for: `rust/private/rustdoc.bzl:111` calls `get_cc_toolchain_runtime_libs`, which returns
`static_runtime_lib` for every non-dylib crate type. It also carries the `lint_config` attribute
(`rust/private/rust.bzl:809`) that `generate_lint_config = True` feeds. The `git_override` block in
MODULE.bazel goes away and nothing replaces it.

**The test-BUILD merge is mechanically safe.** No crate has a target name that appears in both its
BUILD.bazel and its tests/BUILD.bazel, so no merge produces a duplicate. Only one document outside
the build files names a `tests:` label, `Bazel.md:48`.

**The existing rust_test_suite pattern can stay.** Every crate has a `tests/mod.rs` with a mirrored
module tree, which is the layout that forced scrith into a single `rust_test` per crate. It does
not force the same here: no `*_tests.rs` file in this repo references a sibling module, `super::`
or `crate::`, so each one still compiles as its own crate root. The 223 `rust_test_suite` targets
keep working. Switching to scrith's single-target pattern would be a separate decision about test
granularity, not something this migration requires.

**The external dependency surface is larger than the vendored set suggests.** `thirdparty/BUILD.bazel`
declares eight packages and only seven are referenced by a first-party target; `libm` is vendored
but unused, because no Bazel target enables the feature that pulls it. The Cargo graph has more
than either count: `criterion` as a dev-dependency of nine crates,
`rayon` behind the optional `parallel` feature in three, and `candle-core` in
`examples/causal_discovery_examples`. Vendoring hid them because the Bazel build never enabled
those features. `crate.from_cargo` resolves the whole workspace manifest, so the first fetch pulls
a substantially bigger closure than the current 94 MB tree.

**scrith is a target-state reference, not a migration path.** Its Bazel setup was greenfield
(`98b7e3f`, "Minimal bazel config with rules_rs"). There is no migration diff to copy, only an end
state to match.

**Two version pins are already inconsistent.** Root Cargo.toml sets `rust-version = "1.97.1"` while
MODULE.bazel sets `RUST_VERSION = "1.98.0"`. Host `rustc 1.98.0` reports `LLVM version: 22.1.8`,
while MODULE.bazel pins `llvm_version = "22.1.6"`. Neither is caused by this migration, and both
should be reconciled while the file is open.

## Stage 1 — swap the rule set and delete the vendor tree

**Goal:** `bazel test //...` passes with rules_rs resolving crates from the registry, and
`thirdparty/crates/` is gone.

This stage is atomic. The moment `crate.from_cargo` provides `@crates`, every
`//thirdparty/crates:*` label is dead, so the label rewrite and the tree deletion are one change.

1. Spike on one leaf crate first. `deep_causality_unified_math/deep_causality_num` depends on a
   single external package (`libm`) and nothing first-party. Get it building before touching
   anything else.
2. Rewrite MODULE.bazel. Drop `bazel_dep(name = "rules_rust", ...)` and its `git_override`; add
   rules_rs, its rules_rust extension, its toolchains extension and the crate extension:

   ```python
   bazel_dep(name = "rules_rs", version = "0.0.107")

   rules_rs_rules_rust = use_extension("@rules_rs//rs:rules_rust.bzl", "rules_rust")
   use_repo(rules_rs_rules_rust, "rules_rust")

   RUST_EDITION = "2024"

   RUST_VERSION = "1.98.0"

   toolchains = use_extension("@rules_rs//rs/toolchains:module_extension.bzl", "toolchains")
   toolchains.toolchain(
       edition = RUST_EDITION,
       version = RUST_VERSION,
   )
   use_repo(toolchains, "default_rust_toolchains")

   register_toolchains("@default_rust_toolchains//...")

   # The triples the dependency graph is RESOLVED for, which is not the same as cross-compiling.
   # rules_rs evaluates each crate's target-specific dependencies once per triple, so a
   # `[target.'cfg(unix)'.dependencies]` entry is present whichever machine runs the build.
   RUST_PLATFORMS = [
       "aarch64-apple-darwin",
       "aarch64-unknown-linux-gnu",
       "x86_64-unknown-linux-gnu",
   ]

   crate = use_extension("@rules_rs//rs:extensions.bzl", "crate")
   crate.from_cargo(
       name = "crates",
       cargo_lock = "//:Cargo.lock",
       cargo_toml = "//:Cargo.toml",
       generate_lint_config = True,
       platform_triples = RUST_PLATFORMS,
   )
   use_repo(crate, "crates")
   ```

   `extra_target_triples` has no equivalent on `toolchains.toolchain`; the triple list lives on
   `crate.from_cargo` instead.
3. Leave the hermetic LLVM block alone. Keep `toolchain.exec(os = "linux")` and
   `register_toolchains("@llvm_toolchains//:all")` as they are. Switching to scrith's
   `register_toolchains("@llvm//toolchain:all")` also registers darwin cc toolchains, which then
   demand the `osx.from_archive` macOS SDK. That is a separate change with its own download cost
   and belongs in its own commit.
4. Rewrite the third-party labels. Seven packages are actually referenced, mechanically:

   ```bash
   grep -rl '//thirdparty/crates:' --include='BUILD.bazel' . \
     | grep -v '^./thirdparty/' \
     | xargs sed -i '' 's|"//thirdparty/crates:\([a-z0-9_-]*\)"|"@crates//:\1"|g'
   ```

   `rusty-fork` is the one to check by hand: the vendored alias is `rusty-fork` while the crate
   target is `rusty_fork`.
5. Update the `load()` lines to the rules_rs re-exports, matching scrith:

   ```python
   load("@crates//:defs.bzl", "aliases", "all_crate_deps", "lint_config")
   load("@rules_rs//rs:rust_library.bzl", "rust_library")
   load("@rules_rs//rs:rust_test.bzl", "rust_test")
   load("@rules_rust//rust:defs.bzl", "rust_doc", "rust_doc_test", "rust_test_suite")
   ```

6. Delete `thirdparty/crates/`, `thirdparty/BUILD.bazel` and `scripts/vendor.sh`. Drop the
   `thirdparty/*` entries from `.bazelignore`.
7. Decide on `thirdparty/bzlmod/`. The `local_path_override` block that would activate it is absent
   from MODULE.bazel, so the 5.4 MB tree is dormant. Either restore the overrides or remove the
   tree and `scripts/vendor_modules.sh` with it. Leaving dormant vendored sources in the tree is
   the one outcome to avoid.
8. Update `Bazel.md` and `BUILD.md`: the vendored-dependency query section is obsolete, and
   `bazel query "kind('rust_library', //thirdparty/...)"` no longer resolves.

**Exit criteria**

```bash
bazel build -- //... -//lean/...
bazel test  -- //... -//lean/...
bazel test --config=ci -- //... -//lean/...
cargo test --workspace
```

**Rollback:** one `git revert` of the stage commit. Nothing outside the Bazel graph changes, and
Cargo is untouched.

**Risks:** the first fetch is cold and larger than the vendored tree, because the whole Cargo graph
resolves, including `criterion`, `rayon` and `candle-core`. `validate_lockfile` defaults to `True`,
so a Cargo.lock that does not satisfy Cargo.toml fails the build instead of resolving quietly;
run `cargo update --workspace --dry-run` first if the lock is stale.

## Stage 2 — fold the test BUILD files into their crates

**Goal:** one BUILD.bazel per crate, with the same targets and the same tags.

1. For each of the 28 `tests/BUILD.bazel` files: prefix every `srcs` glob pattern and every
   `crate_root` with `tests/`, append the stanzas to the parent BUILD.bazel, delete the child file.
   The transformation is mechanical because no target name collides across the pair, so a script
   over `glob([...])` and `crate_root = "..."` string literals covers it. Review the `deps` lists
   by eye: labels are package-relative only when written as `:name`, and the test files use
   absolute `//crate` labels throughout.
2. Start with `deep_causality_core` (4 suites) and `ultragraph` (small), then sweep. Leave
   `deep_causality_cfd` (369 lines of test targets) for last.
3. Fix `Bazel.md:48`, which documents
   `bazel query 'tests(//deep_causality/tests:ctx_space_time_types_tests)'`. Labels lose the
   `/tests` segment: `//deep_causality_core/tests:errors` becomes `//deep_causality_core:errors`.

**Exit criteria**

```bash
bazel test -- //... -//lean/...
# target count must be unchanged before and after:
bazel query 'kind("rust_test.*", //... except //lean/...)' | wc -l
bazel test //deep_causality/... --test_tag_filters=unit-test
```

**Rollback:** revert the stage commit. This stage touches no MODULE.bazel and no Cargo file.

**Risks:** low. A missed glob prefix produces an empty `srcs` and a target that builds nothing,
which the target-count check above catches. Run that count before and after and compare.

## Stage 3 — declare dependencies once

**Goal:** the dependency graph is written in Cargo.toml only, and BUILD files derive it.

1. Hoist every internal path dependency and every external dependency into root
   `[workspace.dependencies]`, keeping the `version` on each entry. `cargo publish` needs that
   version to rewrite a path dep into a registry dep, and release-plz needs it to bump.
2. Rewrite each member's dependency sections to `dep = { workspace = true }`. Two Cargo rules
   govern the rewrite, and both matter here:
   - `features = [...]` in a member is unioned with the workspace entry's features.
   - `default-features = false` in a member is ignored when the workspace entry enables default
     features. `deep_causality_cfd` sets `default-features = false` on nearly every dependency, so
     the workspace entries those point at must not enable defaults.
3. Replace the hand-written BUILD dep lists with the generated helpers:

   ```python
   rust_library(
       name = "deep_causality_core",
       srcs = glob(["src/**/*.rs"]),
       aliases = aliases(),
       crate_features = [
           "std",
           "alloc",
       ],
       crate_root = "src/lib.rs",
       lint_config = lint_config(),
       tags = ["deep_causality_core"],
       visibility = ["//visibility:public"],
       deps = all_crate_deps(normal = True),
   )
   ```

   `all_crate_deps` takes `normal`, `normal_dev`, `build`, `package_name` and `cargo_only`. Pass
   `cargo_only = True` where a BUILD file also lists first-party `//` labels by hand, otherwise the
   result carries them too and the target fails on a duplicate.
4. Verify the resolved graph did not move: `Cargo.lock` should be byte-identical after the hoist.
5. Once `lint_config()` is wired, the repo-wide `[workspace.lints]` policy reaches Bazel targets
   rather than only `cargo clippy`.

**Exit criteria**

```bash
git diff --exit-code Cargo.lock          # the hoist must not change resolution
cargo test --workspace
cargo publish --dry-run -p deep_causality_core
bazel test -- //... -//lean/...
```

**Rollback:** revert the stage commit. Stages 1 and 2 stand on their own and stay green.

**Risks:** publishing is the real exposure. Run `cargo publish --dry-run` on a leaf crate and on
`deep_causality` before merging, and confirm release-plz still computes per-crate bumps with the
versions living in `[workspace.dependencies]`.

## Risks and open questions

| Item | Severity | Resolution |
|---|---|---|
| release-plz behaviour with `[workspace.dependencies]` | High | Spike in Stage 3: `cargo publish --dry-run` on `deep_causality_core` and `deep_causality`, then a release-plz dry run |
| `default-features = false` silently ignored under workspace inheritance | High | Audit `deep_causality_cfd` first; it has the most overrides |
| Cold remote cache after the toolchain change | Medium | Expected once. Batch every MODULE.bazel change into the Stage 1 commit |
| Larger external closure (`criterion`, `rayon`, `candle-core`) | Medium | Measure the first fetch. If `candle-core` dominates, consider whether `examples/causal_discovery_examples` needs a Bazel target at all |
| darwin cc toolchain | Medium | Deliberately deferred. Keeping `toolchain.exec(os = "linux")` leaves macOS on the autodetected host toolchain, as today |
| `llvm_version = "22.1.6"` against rustc's LLVM 22.1.8 | Low | Reconcile to 22.1.8 in Stage 1 |
| `rust-version = "1.97.1"` against `RUST_VERSION = "1.98.0"` | Low | Pick one. The Kani-bundled toolchain caps the MSRV, so confirm `cargo kani` still runs |
| Air-gapped builds | Low | rules_rs has no `vendor_dir`. If offline builds are ever needed, the fallback is a populated `--repository_cache`, not a vendor tree |

## What does not change

`//lean/...` and rules_lean are untouched; `bazel test -- //... -//lean/...` stays the CI command.
The BuildBuddy endpoints, the `--config=remote` and `--config=ci` blocks, and
`//bazel/platforms` keep their current definitions. Rule attributes are the same, because rules_rs
re-exports rules_rust's rules unchanged. `Cargo.lock` should not move at any point in the
migration. Git history keeps the 94 MB of vendored sources; the win is in the working tree, in
clone size and in review noise, not in repository size.
