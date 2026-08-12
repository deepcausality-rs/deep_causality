## Context

The Lean formalization is the L1 verification layer and gates every PR, so its build cost is paid
continuously. Until this week `@lake_deps` was produced by `lake update` (nine full `git clone`s) and
`lake exe cache get` (one blob per Mathlib module) inside `rctx.execute`. Bazel's repository cache
and `--experimental_remote_downloader` both key on sha256 and cannot see into a subprocess, so the
7.5 GB workspace was re-fetched on every cold output base, and the tree it produced was not
byte-reproducible.

Four fixes now live in the vendored copy at `thirdparty/rules_lean/`, each measured:

| fix | measurement |
|---|---|
| content-addressed fetch via a lockfile | `//lean/...` from sha256 archives: 64.6 s cold, 20 s warm |
| lazy per-platform toolchain declarations | cold output base, one Rust crate: 0 bytes of Lean fetched (was 7.5 GB + 2.6 GB) |
| `exec_compatible_with` per execution platform | 8/8 Lean namespaces execute on Linux RBE from a macOS host |
| import minimization + `cache_roots` | closure 8,639 → 1,918 modules; oleans 1,874 MB → 452 MB |

A survey of `pulseengine/rules_lean` (read in full, not from its README) found: `exec_compatible_with`
per platform and a downloadless toolchain hub, arrived at independently — evidence the shape is
right; a shallow `--depth 1` Mathlib fetch, which is a better fallback than a full clone; a
fail-closed hash policy; and CI that tests failure modes. It leaves Mathlib on `rctx.execute` with no
sha256, has no tree-shaking, always runs `lake build Mathlib` after `cache get`, and ships no
LICENSE. `rules_lean` is unclaimed in BCR (probed 404 against a `rules_rust` 200).

Constraints: the repository vendors dependencies under `thirdparty/` with `local_path_override`, is
MIT licensed, never deletes files (moves them aside), and runs `bazel test //...` as its primary
gate.

## Goals / Non-Goals

**Goals:**

- A standalone MIT ruleset carrying the four fixes, publishable to BCR, usable by others.
- BCR shape decided before the first commit — layout, examples, artifact naming and presubmit scope
  all follow from it.
- deep_causality migrated onto the published module with its Lean gate unchanged, as the acceptance
  test for the extraction.
- The three adoptable lessons from pulseengine folded in: fail-closed hashes, shallow fallback
  fetch, failure-mode CI.

**Non-Goals:**

- Merging upstream or waiting on the unresponsive maintainer. This is a fork with attribution.
- Porting pulseengine's Aeneas/Charon (Rust → Lean) integration. Interesting for this project
  eventually; unrelated to the fetch problem and a separate proposal.
- Solving Mathlib artifact hosting for arbitrary consumers. The ruleset defines the pin format; who
  hosts what is a consumer decision, with this repository as the first worked example.
- Changing the proof content or the `lean_test` per-namespace structure.

## Decisions

### Fork rather than adopt pulseengine

Their design is better on toolchain selection and CI, and unfixed on the expensive problem. Adopting
would mean re-porting the lock work onto their codebase while inheriting an unlicensed dependency.
The blocking fact is the missing LICENSE: unlicensed means all rights reserved, and this repository's
model is vendoring. *Alternative considered:* contribute the lock upstream to pulseengine — worth
doing later, and cheap to offer once ours exists and is proven, but it cannot be the path that
unblocks this repository.

### The lockfile is the differentiator, and it stays all-or-nothing

`lake` deletes and re-clones any package whose recorded URL does not match its checkout, and
Mathlib's cache client reads `mathlib/.git`. Both were established empirically here, and both mean a
tarball-sourced workspace cannot fall back to `cache get`. A lock without an olean archive therefore
degrades to the subprocess path rather than half-applying. *Alternative considered:* pin per-module
`.ltar` blobs from Mathlib's own cache — genuinely content-addressed with no hosting, but thousands
of downloads per fetch and an unpack step that depends on `leantar` semantics we have not verified.

### Artifact hosting is the consumer's, with a reproducible packer in the ruleset

The tree-shaken archive encodes *a consumer's* import closure, so it cannot be a ruleset constant:
another consumer would silently receive an archive cut to someone else's imports, and a proof-file
edit would require a ruleset release. The ruleset ships the packer, the pin format and the drift
check; the artifact lives with whoever's imports define it. A generic per-revision archive could sit
alongside as a ruleset-level default. *Sizing:* full Mathlib packs to 1,952 MB — 48 MB under the
2 GB GitHub asset cap, so object storage is the safer default; the tree-shaken archive is 431 MB
with 4.5× headroom.

### Reproducibility is a property of the packer, not a convention

Three separate defects broke it here: tar mtime/uid/gid, the gzip header timestamp, and — least
obvious — `GzipFile` storing the *output filename* in the FNAME field, so the same content packed to
two paths hashed differently. All three are fixed and the property is asserted by packing twice to
different names. This matters because a pin nobody can reproduce cannot be audited, and because
re-hosting must not invalidate the pin.

### Presubmit is toolchain-only

BCR presubmit runs on the registry's infrastructure. A test module that fetches Mathlib would make
every submission slow and flaky — and would demonstrate the exact cost this ruleset removes. The
Mathlib examples run in the ruleset's own CI where caches exist. This constraint is why BCR shape has
to be decided first: it dictates that examples are split by weight, not by topic.

### Hash policy inverts to fail-closed

Currently a missing sha256 warns and downloads unverified. For a toolchain whose purpose is
trustworthy proof checking that is the weakest possible link, and a warning is not a control. Taken
from pulseengine: fail by default, allow a per-platform override, allow an explicit development
opt-out that announces itself.

## Risks / Trade-offs

- **A third ruleset fragments a small ecosystem further.** → The differentiator is narrow and
  verifiable, the work already exists and is measured, and the fork is attributed. Offer the lock to
  both upstreams once it is proven in the wild.
- **Maintenance burden lands on this project.** → Scope is ~1,700 lines of Starlark plus tooling
  already written and running. The alternative is an unmaintained vendored copy that cannot receive
  fixes.
- **BCR submission needs maintainer identity and a hosting story.** → Both are prerequisites, not
  discoveries; the checklist is derived from the templates in `thirdparty/toolchains_buildbuddy/.bcr/`.
- **The tree-shaken archive goes stale on any import change.** → The drift check runs in CI on a bare
  checkout in about a second, and names the offending import and the repair command.
- **GitHub source tarballs are not guaranteed byte-stable forever.** Commit-SHA archives have been
  stable, and the ecosystem-wide churn incident was reverted, but the pin depends on it. → If it
  matters, mirror the package sources alongside the olean archive and re-point the lock.
- **Olean portability rests on one observation.** The archive packed on macOS type-checked on Linux
  RBE workers, which is strong evidence but a single data point. → CI covers both platforms
  explicitly.
- **Migrating this repository is a real cutover.** → The vendored copy moves aside rather than being
  deleted, so reverting is a one-line `MODULE.bazel` change.

## Migration Plan

1. Extract into the new repository, MIT licensed, BCR scaffolding present from the first commit.
2. Fold in the three pulseengine lessons; make CI assert each failure mode.
3. Host the olean archive and replace the `file://` pin — this unblocks the current repository state
   regardless of the rest.
4. Release, submit to BCR, and consume the registry module here behind a `bazel_dep`.
5. Keep `thirdparty/rules_lean/` in place until the registry path is green, then move it aside.

Rollback at any point is restoring the `local_path_override`.

## Open Questions

- Repository owner and module name: `deepcausality-rs/rules_lean` claims the free BCR name; a
  neutral home may age better if others adopt it.
- Where the olean archives are hosted, and whether the ruleset ships a default base URL for
  generic per-revision archives or stays hosting-agnostic.
- Whether to keep the `oleanImports` manifest and per-namespace `lean_test` in the public surface,
  or treat them as consumer-side.
- Whether to offer the lock design to `pulseengine/rules_lean` and `tomato-bazel/rules_lean` after
  it ships.
