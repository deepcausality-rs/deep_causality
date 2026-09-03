## 1. Unblock the current repository state

These stand alone and are worth doing first: they end the `file://` pin regardless of whether the
extraction proceeds.

- [x] 1.1 Decide the artifact host for the olean archive (object storage preferred over GitHub Releases; full Mathlib packs to 1,952 MB against a 2 GB asset cap) and record the decision in `design.md` Open Questions
- [x] 1.2 Upload `lake-oleans-leanprover-lean4-v4.32.0.tar.gz` (431 MB, sha256 `f00fefd9d85142f33beb5ee1a77e8812d10fdce0bca17b39f73557c0f1a23f6b`) and replace the `file://` URL in `lean/lake-lock.json`; the sha256 is unchanged because the pack is reproducible
- [x] 1.3 Verify `scripts/lean_lock.sh check` passes and `bazel build //lean/...` is green from a cold output base with the hosted URL
- [x] 1.4 Confirm the `lean-closure` CI job passes on a PR

## 2. Decide identity and stand up the repository

- [x] 2.1 Settle the repository owner and module name; confirm `rules_lean` is still unclaimed in BCR at that time
- [x] 2.2 Create the repository with an MIT LICENSE at the root and an attribution notice naming `tomato-bazel/rules_lean` as upstream and summarising what changed
- [x] 2.3 Write `MODULE.bazel` with the module name matching the intended registry directory, a semantic version and a `compatibility_level`
- [x] 2.4 Add the `.bcr` templates before any rule code, modelled on `thirdparty/toolchains_buildbuddy/.bcr/`: `metadata.template.json`, `source.template.json`, `presubmit.yml`
- [x] 2.5 Establish the examples split that presubmit scope forces: a toolchain-only module for the registry, Mathlib-dependent examples for the ruleset's own CI

## 3. Port the rules and their fixes

- [x] 3.1 Port the rule sources from `thirdparty/rules_lean/lean/`, adding SPDX headers to every file
- [x] 3.2 Port the `lock` attribute: sha256-pinned `download_and_extract` for every package plus the olean archive, with validation against the manifest and toolchain failing closed on drift
- [x] 3.3 Port the all-or-nothing degradation: a lock with no olean archive falls back to the subprocess path with a diagnostic, since `lake` deletes tarball-sourced packages and `cache get` reads `mathlib/.git`
- [x] 3.4 Port `lean_toolchain_decls` and the per-platform `lean_dist`, with `exec_compatible_with` constraints and lazy fetching of the selected platform only
- [x] 3.5 Port `cache_roots` tree-shaking and the generated package list, keeping the rule that packages building to no oleans get no target
- [x] 3.7 Add a toolchain-only extension tag: the only way to get a toolchain was to declare a `lake.workspace`, which the Mathlib-free presubmit module cannot do
- [x] 3.6 Verify the ported rules build the deep_causality proofs unchanged, locally and under remote execution

## 4. Fold in the surveyed improvements

- [x] 4.1 Invert the hash policy to fail-closed, with per-platform overrides and an explicit development opt-out that announces itself
- [x] 4.2 Replace the full-history clone in the subprocess fallback with a shallow fetch at the pinned revision, retaining `.git` and the `origin` remote that the Mathlib cache client needs, and accepting both tags and bare commit hashes
- [x] 4.3 Add a retry around `lake exe cache get` for the case where it exits zero with gaps
- [x] 4.4 Add post-fetch completeness validation with actionable messages, replacing the bare "no oleans found" failure
- [x] 4.5 Strip macOS quarantine attributes from downloaded toolchains
- [x] 4.6 Keep source-building Mathlib behind an explicit opt-in rather than running it unconditionally after `cache get`

## 5. Move the generic tooling, keep the consumer-side tooling

- [x] 5.1 Move the reproducible archive packer into the ruleset, preserving the three reproducibility fixes (tar mtime/uid/gid, gzip mtime, and the gzip FNAME field that made the digest depend on the output path)
- [x] 5.2 Assert reproducibility in the ruleset's tests by packing twice to different output paths and comparing digests
- [x] 5.3 Keep the repin and drift-check scripts consumer-side in `scripts/`, adjusting them to the published rule surface
- [x] 5.4 Document the pin format and the repack workflow for consumers who are not this repository

## 6. Test the guardrails, not the happy path

- [x] 6.1 CI asserts a missing toolchain hash fails with the message that directs the consumer to supply one
- [x] 6.2 CI verifies every declared platform's toolchain archive downloads and extracts under its recorded sha256
- [x] 6.3 CI asserts Lean/Mathlib version skew fails with a message naming the mismatch
- [x] 6.4 CI asserts lock drift fails: a package revision disagreeing with the manifest, and a lock entry missing its sha256
- [x] 6.5 CI asserts a `file://` pin is rejected
- [x] 6.6 CI covers both macOS and Linux, and exercises remote execution so the platform-selection fix stays fixed
- [x] 6.7 CI builds the Mathlib-dependent examples, which presubmit deliberately does not

## 7. Release and submit

- [x] 7.1 Produce the release archive from the tag so it matches what `source.template.json` resolves to
- [x] 7.2 Wire release automation that opens the registry submission from a published release without hand-editing the integrity hash
- [x] 7.3 Cut the first release and submit to BCR
- [x] 7.4 Confirm presubmit passes and stays fast, having fetched no Mathlib

## 8. Migrate this repository

- [x] 8.1 Replace the `local_path_override` on `thirdparty/rules_lean` with a `bazel_dep` on the published module, updating `use_repo` and `register_toolchains` names
- [x] 8.2 Update `lean/BUILD.bazel`, `lean/lake-lock.json` and `lean/lake_packages.bzl` for any rule-surface change
- [x] 8.3 Verify `bazel build --config=remote //...` is green and the Lean namespaces type-check and cache as before
- [x] 8.4 Move `thirdparty/rules_lean/` aside per repository convention, with a note recording what superseded it
- [x] 8.5 Prepare the commit message; leave committing to the user

## 9. Close out

- [x] 9.1 Update `lean/README.md` with the dependency change and the repin workflow
- [x] 9.2 Record in the proof-header note that a new import needs both a `cache_roots` update and an archive repack
- [x] 9.3 Decide whether to offer the lock design upstream to `tomato-bazel/rules_lean` and `pulseengine/rules_lean`, and open the LICENSE request against the latter
