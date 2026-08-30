<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Consolidating the math crates under `deep_causality_unified_math/`

**Scope.** Moving the mathematics crates from the workspace root into one folder, keeping their
package names. What breaks, what does not, and in what order to do it. The assessment below was
written for sixteen crates; seventeen moved. §9 reconciles every count.

**Not in scope.** Renaming any package. Merging crates. Changing any dependency edge. The gap work
in `unified_math_gaps.md`. Publishing.

**Verdict.** Feasible and almost entirely mechanical: roughly 1,100 line edits, nearly all of them
anchored search-and-replace, and no change to a single `src/` file. Two properties make it cheap.
Cargo and Bazel read the layout independently, so each checks the other's work; and the repo's own
tooling already derives crate directories from the workspace manifest rather than assuming them.
Four things fail quietly rather than loudly, and those are the whole risk. §4.

**Status: done, 2026-08-30.** The move landed. Predictions against actuals are in §9. `deep_causality_ast`
was added to the set during execution, so seventeen crates moved rather than sixteen. The website
(§4.4) was carved out into a separate change set and is **not** done.

**Method.** Counts measured against `main`, worktrees under `.claude/` and `thirdparty/` excluded.

---

## 1. The move

Sixteen directories move from the repository root into `deep_causality_unified_math/`:

```
deep_causality_unified_math/
  deep_causality_num          deep_causality_calculus
  deep_causality_metric       deep_causality_fft
  deep_causality_algebra      deep_causality_linear
  deep_causality_haft         deep_causality_uncertain
  deep_causality_num_complex  deep_causality_homology
  deep_causality_num_dual     deep_causality_tensor
  deep_causality_num_rational deep_causality_multivector
  deep_causality_rand         deep_causality_topology
```

Package names do not change. `deep_causality_tensor` stays `deep_causality_tensor` on crates.io and
in every `use` statement. Nothing in any `src/` tree moves relative to its own crate root, and no
math crate contains a `#[path]`, `include!` or `include_str!` that reaches across a crate boundary,
so no Rust source file needs an edit.

Sixteen of the twenty-nine library crates move. Thirteen stay: the causality engine, physics, CFD,
discovery, algorithms, and the infrastructure crates `ast`, `par`, `data_structures`, `file`,
`ultragraph`.

## 2. What survives untouched

Worth listing first, because it is most of the machinery and it sets the size of the job.

| Surface | Why it survives |
|---|---|
| `Cargo.lock` | Records name, version and dependency names. It carries no paths for workspace members. Zero edits |
| crates.io publishing | Path dependencies all carry `version =`, so a published manifest resolves through the registry. Directory layout is not part of a published package. No version bump is owed for the move |
| `build/scripts/crates.sh` | Reads `members` from the root manifest, expands the globs against the repo root, and takes each package's name from its own `Cargo.toml`. It never assumes directory equals package name |
| `check.sh`, `format.sh`, `miri.sh`, `sbom.sh` | All four source `crates.sh` and take `DC_CRATE_DIRS` from it |
| `build/scripts/check_examples.sh` | Runs `bazel query "kind(rust_binary, //$dir:all)"` with `$dir` from `crates.sh`, so the label follows the manifest |
| `build/scripts/test.sh`, `build.sh` | Workspace-wide `cargo test` and `cargo build`. No paths at all |
| Bazel target names | Every math crate's `rust_library` is named for its directory basename, so `//deep_causality_num` keeps resolving once it becomes `//deep_causality_unified_math/deep_causality_num`. Only the prefix changes; no target name does |
| `MODULE.bazel` | Its five `local_path_override` entries all point into `thirdparty/` |
| Bazel third-party crates | No `crate_universe` `manifests` attribute anywhere. Bazel does not read the Cargo manifests, so the Cargo edit and the Bazel edit cannot corrupt each other |
| Root `BUILD.bazel` | One `config_setting`. No `glob` that would capture the new subtree |
| `.bazelignore` | Lists no math crate |
| `release-plz.toml` | One `[[package]]` entry, for `deep_causality`, with no math crate path |
| `bazel test //...` | Recursive target patterns are layout-independent |

The `crates.sh` design is the single largest saving here. Its header records why it exists: four
scripts once carried the crate list by hand and all four had drifted. That decision pays out again
now, because the four consumers absorb a sixteen-crate move without an edit.

## 3. What must change

| # | Surface | Count | Edit |
|---|---|---|---|
| C1 | Root `Cargo.toml` `members` | 16 entries | Path prefix, or one `"deep_causality_unified_math/*"` glob |
| C2 | Cargo path deps, non-math into math | 105 | `../deep_causality_x` becomes `../deep_causality_unified_math/deep_causality_x` |
| C3 | Cargo path deps, math out to non-math | 4 | `../deep_causality_ast` becomes `../../deep_causality_ast` |
| B1 | Bazel labels `"//deep_causality_<math>"` | 853 across 59 files | Prefix insertion |
| W1 | `.github/workflows/formalization.yml:82-88` | 11 of 13 directory arguments | `grep -Frl` target list |
| W2 | `.github/workflows/rust_deps.yml:30` | 16 of 29 directory arguments | `cargo machete` argument list |
| S1 | `build/scripts/mutants.sh:60-62` | 1 path glob | `deep_causality_linear/src/algorithms/*.rs` |
| D1 | `lean/THEOREM_MAP.md` | 82 path references | Documentation |
| D2 | **Published website GitHub links** | 36 in 11 source files | **404 on the live site**, §4.4 |
| D3 | Live markdown | 88 files | Documentation, §5.2 |

C2 and C3 total 109 manifest edits. C3 is the small surprise: four math crates depend on a crate
that is staying outside, and those four gain a level.

| Crate | Depends on | New path |
|---|---|---|
| `fft` | `par` | `../../deep_causality_par` |
| `topology` | `par` | `../../deep_causality_par` |
| `tensor` | `ast` | `../../deep_causality_ast` |
| `uncertain` | `ast` | `../../deep_causality_ast` |

C2 concentrates in a handful of manifests. `deep_causality_physics` and
`examples/mathematics_examples` carry twelve each, `deep_causality_cfd` and
`examples/physics_examples` ten each, `deep_causality_quantum` eight.

## 4. Four ways this fails quietly

Every item in §3 except these four announces itself: Cargo refuses to resolve a path, and Bazel
refuses to resolve a label. These do not.

### 4.1 `formalization.yml` fails with the wrong message

```
rust_hit=$(grep -Frl "THEOREM_MAP: $id" \
           deep_causality_algebra deep_causality_num deep_causality_num_complex \
           ... --include='*.rs' || true)
if [ -z "$rust_hit" ]; then echo "MISSING Rust witness for id: $id"; fail=1; fi
```

Eleven of the thirteen directory arguments are math crates. If the list goes stale, `grep` writes
"No such file or directory" to stderr, `|| true` discards the exit status, `rust_hit` comes back
empty, and CI reports `MISSING Rust witness` for every theorem id in the file. The build fails, so
nothing ships broken; the message names the wrong cause, and someone will go looking for a deleted
witness that is still there.

### 4.2 `cargo machete` may go dark

`rust_deps.yml:30` passes twenty-nine directory paths, sixteen of them math crates. Whether
`cargo machete` errors on a missing path or skips it has to be established before the move, not
after. If it skips, the unused-dependency guard silently stops covering the entire math stack.

That guard is load-bearing rather than decorative. `deep_causality_homology/src/lib.rs:52` names it
as the reason the crate carries no unused dependency on the algebra tower, which is one of the two
arguments in `unified_math_gaps.md` §3.7.

### 4.3 `lean/THEOREM_MAP.md` rots without complaint

The file holds 82 path references of the form
`deep_causality_algebra/tests/formalization_lean/monoid_tests.rs`. The CI check at
`formalization.yml:89` greps the map for the theorem *id* and never for the path. Every path in that
file can be wrong and the formalization job still passes.

### 4.4 The website ships GitHub deep links that will 404

Scouted on request. `website/` carries **36 links of the form
`github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_<crate>`** across 11 source
files. These are published URLs on a live site. Every one of them 404s the moment the directory
moves, and nothing in the repo tests them.

| Source file | Links |
|---|---|
| `docs/src/content/docs/getting-started/install.md` | 6 |
| `docs/src/content/docs/concepts/uniform-math.md` | 5 |
| `docs/src/content/docs/concepts/uncertainty.md` | 3 |
| `docs/src/content/docs/formalization/linear.md` | 2 |
| `docs/src/content/docs/concepts/{hkt,glossary,cdl}.md` | 1 each |
| `web/src/content/blog/en/announcement-{maybe-uncertain,haft-hkt}.md` | 1 each |
| `web/src/content/examples/en/{physics-maxwell,event-horizon-probe}.mdx` | 1 each |
| `quantum/src/consts.ts` | 1+ |

Deep links reach past the crate root in places: `deep_causality_uncertain/examples/gps`,
`deep_causality_uncertain/examples/clinical_trial`, `deep_causality_haft/examples`.

The same files carry a further 26 plain repository paths in prose and tables, mostly pointing at
`tests/formalization_lean/` directories and crate `README.md` files.

Two things make this the most exposed surface in the move. It is public, so a reader hits the
failure rather than a maintainer. And `quantum/src/consts.ts` is TypeScript, so a Markdown-only
sweep misses it.

Build output under `website/*/dist/` and `website/*/.astro/` carries copies. Those regenerate and
need no edit.

**The first three have the same fix, and it is worth doing on its own merits.** W1 and W2 should read
their crate list from `crates.sh` rather than carrying it inline, exactly as `check.sh` and
`sbom.sh` already do. D1 wants a check that each path in `THEOREM_MAP.md` resolves to a file.
Landing that hardening *before* the move converts two silent failures and one misleading one into
ordinary loud ones.

## 5. Friction that is not a path

### 5.1 The Bazel replacement must be anchored

`deep_causality_num` is a prefix of three other package names, and all four appear as labels:

```
 129  "//deep_causality_num"
  44  "//deep_causality_num_complex"
  13  "//deep_causality_num_dual"
   4  "//deep_causality_num_rational"
```

An unanchored `s|//deep_causality_num|//deep_causality_unified_math/deep_causality_num|` corrupts
the other three into `..._unified_math/deep_causality_num_complex` only by luck, and mangles any
label written in long form. Every occurrence in the tree is quoted, so anchoring on the closing
quote and on `:` is both possible and sufficient. Run the replacement per crate name, longest first.

### 5.2 The historical record should not be rewritten

Of 212 markdown files carrying a math crate path, **124 live under `openspec/changes/archive/`**.
Those are records of what was decided when, and rewriting paths inside them falsifies the record. A
line in the archived change that reads `deep_causality_topology/src/...` was true when it was
written.

The 88 live files are the real surface: 15 crate `README.md`, 19 `openspec/notes`, 13
`openspec/specs`, 11 under `website/`, 9 live `openspec/changes`, 8 `openspec/audits`, plus
`AGENTS.md`, `README.md`, `README_NO_STD.md` and `lean/THEOREM_MAP.md`.

Two of the 24 crate-level documents must not be hand-edited: `deep_causality_tensor/CHANGELOG.md` is
generated by release-plz. Eight `LEAN_*.md` files sit alongside the crate READMEs and travel with
their crates.

`AGENTS.md` needs more than a path substitution. Its "Project Structure" and "Internal Dependencies"
sections describe the layout in prose, and the tier block is the document most agents read first.

### 5.3 Worktrees, resolved

**Done.** Three worktrees held the old layout and would each have produced a sixteen-directory
rename conflict. All three carried zero commits not in `main`, and their uncommitted work was
superseded: the algebra-tower experiments landed as `semiring.rs`, `domain_euclidean.rs` and
`invertible.rs` (the integer arm was deliberately rejected), and the `proto/` prototype was already
preserved under `openspec/notes/archive/linear/prototype/`. Removed. The three
`worktree-wf_*` branches remain and point at commits already in `main`.

### 5.4 Git history

`git mv` preserves history, and `git log --follow` keeps working per file. `git blame` is unaffected
because file contents do not change. Rename detection is cleanest when the commit contains renames
and nothing else, so the move should be its own commit with no content edits in it.

## 6. Sequence

The order is chosen so that each step is verified by a tool before the next one starts, and so the
two silent failures are removed before they can occur.

| Step | Action | Verified by |
|---|---|---|
| 0 | Land or discard the three worktrees | `git worktree list` |
| 1 | Harden W1 and W2 to read `crates.sh`; add a path check for `THEOREM_MAP.md` | CI green on the current layout |
| 2 | `git mv` the sixteen directories. Nothing else in this commit | `git status` shows renames only |
| 3 | Root `Cargo.toml` members (C1) | — |
| 4 | The 109 manifest path edits (C2, C3) | `cargo build --all-features`, then `cargo test` |
| 5 | The 853 Bazel labels (B1), anchored, longest name first | `bazel build //...` then `bazel test //...` |
| 6 | `mutants.sh` glob (S1) | `build/scripts/mutants.sh` starts |
| 7 | `make check_examples`, `make format`, `make fix` | — |
| 8 | Documentation: `AGENTS.md` first, then the other 87 live files | Reading |

Steps 4 and 5 are independent. Cargo cannot see a broken Bazel label and Bazel cannot see a broken
Cargo path, so a mistake in one is caught by its own build rather than masked by the other. That
independence is the main reason this is a low-risk refactor despite its size.

Step 1 is the one step that is worth doing whether or not the move happens.

## 7. Cost

| Category | Edits | Character |
|---|---|---|
| Manifests | 125 | Anchored replace |
| Bazel | 853 | Anchored replace, per crate name |
| CI and scripts | 3 files | Hand, and better rewritten than patched |
| Documentation | 88 files | Mixed; `AGENTS.md` needs prose |
| Rust source | 0 | — |

The judgement calls are few: whether `members` becomes a glob or stays an explicit sixteen-line
list, and how much of `AGENTS.md` to restructure.

## 8. Recommendation

Do the move, and do it before the gap work rather than after.

The gap work in `unified_math_gaps.md` touches `src/` in six crates and will add files. Running it
first means the move later collides with fresh code, and it means the 88 live documentation files
get rewritten twice. The move touches no `src/` at all, so the two changes do not overlap except in
time.

One consequence to accept: `unified_math_gaps.md` itself carries about twenty-five path citations
that go stale on the day of the move. Updating one note once is the cheaper end of that trade.

`deep_causality_sparse` is settled: it was retired to `yanked/` on 2026-08-30, so the folder holds
seventeen live crates and no dead one. One thing is left to settle:

1. Whether the folder gets a `README.md` carrying the tier diagram and the composition story, which
   is the stated reason for consolidating in the first place. If it does, that README becomes the
   natural home for material now spread across `AGENTS.md`, the gap note and the
   `composable_multi_math` examples README.


## 9. Outcome

Landed 2026-08-30. Every count below was predicted before the move and measured after.

| Item | Predicted | Actual | Note |
|---|---|---|---|
| Cargo path deps, non-math into math | 105 | 104 | `sparse` retired in between |
| Cargo path deps, math out to non-math | 4 | 4 | `fft`/`topology` to `par`, `tensor`/`uncertain` to `ast` |
| Bazel labels | 853 | 852 | `sparse` again |
| Rust source files | 0 | 0 | for compilation; 17 docstring paths in 16 files were stale and fixed |
| Renames staged | — | 1,856 | 1,843 for the sixteen, 13 more for `ast` |

**`deep_causality_ast` joined the move.** Its consumers are `tensor` and `uncertain`, both in the
folder, plus `deep_causality` itself, which is not. That leaves one cross-boundary edge rather than
none. The two math edges became siblings again, so `tensor` and `uncertain` went from
`../../deep_causality_ast` back to `../deep_causality_ast`.

**Two things the assessment missed.**

`build/scripts/mutants.sh` assumed a package name equals its directory. It passes `$CRATE` to both
`cargo mutants -p` and `--file "$CRATE/$FILE"`, and only the second is a path. A path substitution
would have produced `-p deep_causality_unified_math/deep_causality_linear`, which is not a package
name. Fixed with a `crate_dir` helper that asks `cargo metadata` for the manifest directory, so the
assumption cannot come back.

`openspec/changes/archive-notes.md` is a table whose left column is *supposed* to hold old paths. A
blanket documentation sweep rewrote 22 of them and destroyed the mapping. Reverted and edited by
hand. Any future relocation must exclude that file from bulk passes.

**What was deliberately not rewritten.** `openspec/changes/archive/` and `openspec/notes/archive/`
by the argument in `archive-notes.md`; `openspec/changes/reverted/` and `openspec/audits/` for the
same reason, since both are dated records rather than live documents; every `CHANGELOG.md`, which
release-plz generates; and all of `website/`, carved out per §4.4.

**Still outstanding.** The website's 36 GitHub deep links now 404 and are the separate change set.
The hardening in step 1 of §6 was skipped: `formalization.yml` and `rust_deps.yml` still carry
inline crate lists rather than reading `crates.sh`, and `lean/THEOREM_MAP.md` still has no check
that its paths resolve. Both lists were updated by hand and verified to resolve, so the traps did
not fire this time. They remain traps.