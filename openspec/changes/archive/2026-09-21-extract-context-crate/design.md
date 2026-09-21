## Context

The context layer — `Context`, `Contextoid`, the twenty-four node types, the `contextuable` traits —
is compiled into `deep_causality`. The causal monad is compiled into `deep_causality_core`, one tier
below. Nothing links them, and the result is visible in the classical examples: the five
causaloid-side ones reach the typed `Context`, the five monad-side ones each declare a private
struct and thread it through the monad's `Context` channel.

Three facts, all verified against the tree, set the shape of the solution:

1. **The context subtree names nothing from `deep_causality_core`.** Zero references across all 182
   files. The cut needs no untangling.
2. **The context subtree names nothing from the causal side either.** The only mention of
   `Causaloid` anywhere in it is a plain-backtick "See Also" line in
   `traits/scalar/scalar_projector.rs:28` — prose, not an intra-doc link, so no rustdoc target
   breaks. What it *does* name from `deep_causality` is `Identifiable`, four error types, the
   `FloatType` / `ContextId` / `ContextoidId` aliases, and `BaseContext` — all of which either move
   with it or already exist in core.
3. **`Context` needs `ultragraph`, `deep_causality_uncertain` (Tier 5) and `std`.** That is what
   fixes where the crate can sit.

## Goals / Non-Goals

**Goals:**

- A consumer depending on `deep_causality_core` and `deep_causality_context`, and *not* on
  `deep_causality`, can write `PropagatingProcess<f64, (), BaseContext>` and carry a real context
  through a monad chain.
- The dependency on context is declared by whoever uses it, so `cargo tree` and a `Cargo.toml` grep
  answer "which crates use context?" correctly.
- History is preserved across the move; no file is deleted.
- Four of the five monad examples demonstrate the typed context end to end.

**Non-Goals:**

- Bounding the monad's `Context` parameter on a contract trait. Out of scope, named as a follow-up.
- **Feature-gating `deep_causality`'s own dependency on the context crate.** Considered and
  rejected: the gate would have to cover `Model`, the generative interpreter,
  `UncertainActivationPredicate`, the `Base*`/`Uniform*` aliases and `UpdateError`, which is a large
  amount of `#[cfg]` surface and a second build configuration to test, to save a pure-causaloid user
  two transitive dependencies. Not worth it.
- Moving the dead `TeloidTag` / `TeloidID` pair out of `deep_causality_core` into
  `deep_causality_ethos`, which is where that vocabulary belongs.
- Adding a *new* context node type for sequences. None is needed: relaxing `Data<T>` is sufficient
  (Decision 9).
- Auditing the `Copy` / `Hash` / `Eq` derives on `Contextoid` and `ContextoidType`. They are
  conditional and already inapplicable to `BaseContext`, so they are inert rather than wrong.
- Any behavioural change. Every moved item keeps its semantics, signature and tests.

## Decisions

### 1. The crate sits beside `deep_causality`, not beneath `deep_causality_core`

`deep_causality_context` depends on `deep_causality_core`, `deep_causality_data_structures`,
`deep_causality_uncertain` and `ultragraph`, landing at Tier 6.

*Alternative considered: put it below core, so core itself can name `Context`.* Rejected. `Context`
needs `ultragraph`, `deep_causality_uncertain` (Tier 5) and `std`. Core today has exactly one
dependency (`deep_causality_haft`) and a working `no-std` feature; dragging a hypergraph and an
uncertainty engine into it would destroy both to serve a requirement that does not need it. The goal
is that a *consumer* can depend on both crates — not that core depends on context. Core stays
untouched apart from gaining `Identifiable`.

### 2. No re-export from `deep_causality`

Reaching a moved item requires a declared dependency on `deep_causality_context`.

*Alternative considered: re-export the moved surface, making the change non-breaking.* Rejected.
Non-contextual reasoning is the default case, and a re-export would let every existing
`use deep_causality::Context` keep compiling — which means the coupling this change exists to expose
would survive it, invisibly. The dependency graph would go on reporting that `deep_causality` owns
context, and no consumer would ever have to decide whether it actually needs it. The breakage is the
mechanism, not a side effect: it forces each consumer to answer the question once, in its
`Cargo.toml`, where the answer is durable and searchable. The cost is 99 files across the workspace
and one migration blog post; the numbers are in the proposal's impact table.

### 3. `Identifiable` moves to `deep_causality_core`, not to the context crate

*Alternative considered: let it travel with the other moved traits and have `deep_causality` import
it back.* Rejected. `Causaloid`, `Model`, `Inference`, `Assumption`, `Observation` and
`ProposedAction` all implement it, and none of them is a context type. Defining the identity of the
causal types in the context crate inverts the relationship, and under Decision 2 it would also force
`deep_causality` to depend on the context crate for a trait that has nothing to do with context.
Core already owns `IdentificationValue`, which is what `id()` returns.

Its signature is written `fn id(&self) -> u64` today and becomes `fn id(&self) -> IdentificationValue`.
`IdentificationValue = u64`, so this is the same type spelled by its name — not a signature change,
and no implementation needs editing.

### 4. All four error types move; `UpdateError` costs `deep_causality` a dependency edge

`ContextIndexError`, `IndexError` and `AdjustmentError` are context-only. `UpdateError` is not: the
context node types' `Adjustable` impls raise it, and so do
`types/csm_types/csm/{state_add,state_remove,state_update}.rs`, which stay behind.

*Alternative considered: duplicate `UpdateError` on both sides.* Rejected — two distinct types with
one name, and a CSM error that cannot be compared with a context error, is worse than the edge.
`deep_causality` depends on `deep_causality_context` anyway, for `Model`, the generative interpreter
and the `Base*` aliases.

### 5. Move in two commits: `git mv` first, imports second

The first commit is `git mv` only — 182 source files, 62 test files, no content edits — so `git log
--follow` keeps working and the diff is reviewable as a rename set. The second rewrites imports and
adds the manifests. AGENTS.md authorises `git mv` and forbids deletion.

### 6. Context test utilities move; the causal ones stay and import them

`get_context`, `get_base_context` and `get_test_context` in
`deep_causality/src/utils_test/test_utils.rs` build only contexts and move to the new crate's
`src/utils_test`. The Causaloid, Model, Inference and Observation builders in the same file stay and
import the three back.

They live under `src/`, not `tests/`, because Bazel cannot reach a `tests/` helper from another
target — and, per AGENTS.md, that placement means they count toward coverage and must themselves be
tested.

### 7. All five examples migrate

`rcm` (`bool`, `f64`), `cate` (two `f64`, one `bool`), `scm` (two `f64`) and `dbn` (`&'static str`,
two `f64`) hold payloads that already satisfy `Data<T>`, so each hand-rolled struct becomes `Data`
contextoids in a context and the `alternate_context` chains keep their shape.

`granger` holds two `Vec<f64>` time series and becomes migratable through Decision 9. It is the
acceptance test for that relaxation: if `Data<Vec<f64>>` cannot be built and read back, the
relaxation did not achieve what it was for.

### 8. The duplicated primitive aliases collapse onto core, eight by re-export and two by removal

`deep_causality/src/alias/alias_primitives.rs` declares ten aliases that
`deep_causality_core/src/alias/mod.rs` declares byte-identically. The extraction is what makes this
worth fixing now rather than later: the context crate needs `FloatType`, `ContextId`, `ContextoidId`
and `NumericalValue`, and it should take them from the one crate that defines them rather than
becoming a third declaration site.

Eight are live and are re-exported from core at `deep_causality`'s root, so every call site keeps
its spelling — including the 202 example files naming `FloatType` and the 25 test files naming
`NumericalValue`. Not one needs an edit.

`TeloidTag` and `TeloidID` are removed rather than re-exported. They have no user in
`deep_causality` or in `deep_causality_core`; `deep_causality_ethos` declares its own identical
pair and uses that. Re-exporting a dead alias to preserve a surface nothing consumes would be
deduplication in name only. Removing a public item is breaking, which is affordable in a release
that is already breaking.

*Alternative considered: re-export all ten uniformly.* Rejected — simpler to write, but it keeps
dead vocabulary alive at two sites and leaves the same cleanup to be done again later.

### 9. `Data<T>` requires `Clone`, not `Copy`; `Copy` moves to the `Adjustable` impl

`Data<T>` bounds `T: Default + Copy + Clone + PartialEq` and derives `Copy`. The bound is relaxed to
`T: Default + Clone + PartialEq`, the `Copy` derive goes, and `Datable::get_data` returns
`self.data.clone()` instead of `self.data`. The `Adjustable<T> for Data<T>` impl is untouched: it
already restates `Copy` alongside `Hash + Eq + PartialOrd + Add + Sub + Mul`, and that is where the
requirement actually originates — `ArrayGrid<T, W, H, D, C>` is backed by fixed-size arrays and
bounds `T: Copy + Default`.

Four measurements say the struct-level bound is incidental rather than load-bearing:

1. **Nothing above it asks for `Copy`.** `Datable` requires nothing of `Self::Data` — it is
   `type Data; fn get_data(&self) -> Self::Data; fn set_data(&mut self, _)`, and returning by value
   needs `Clone`. `Context` requires `D: Datable + Clone`.
2. **`Contextoid`'s `#[derive(Copy, Hash, Eq)]` is conditional and already inert.** `BaseContext`
   instantiates `S = EuclideanSpace`, which derives only `Debug, Clone, PartialEq`, so
   `BaseContextoid` is already not `Copy`, not `Hash` and not `Eq`. `Data<T>: Copy` buys nothing in
   the canonical configuration.
3. **`Data<T>` is the only node type that constrains a payload parameter.** `EcefSpace`,
   `GeoSpace`, `EuclideanSpace`, `SpaceKind`, `SymbolKind`, `CausalSetSpacetime` and
   `ConformalSpacetime` are all `Clone`-only. The types that do derive `Copy` are concrete and hold
   `f64` or `u64`, where `Copy` is free.
4. **`Data` is the only generic `ArrayGrid` user.** Of the 28 `Adjustable` impls, 22 take
   `ArrayGrid<f64, …>` and 4 take `ArrayGrid<u64, …>`; only the 2 in `Data` take `ArrayGrid<T, …>`.
   So `Data<T>` is the sole path by which `ArrayGrid`'s `Copy` requirement reaches a type parameter.

*Alternative considered: add a new sequence-valued node type and leave `Data<T>` alone.* Rejected —
it would add a type to work around a bound that is not justified in the first place, and the same
wall would be hit by the next non-`Copy` payload.

*Alternative considered: defer this to a follow-up change.* Rejected — it is what blocks `granger`,
and deferring it means shipping an example that documents a limitation the repository has already
decided is a defect.

## Risks / Trade-offs

- **46 `use deep_causality::*;` glob imports fail at the use site, not the import.** 27 staying
  tests, 12 `src` files, 3 benches, 2 avionics examples and 7 classical ones. The compiler points at
  the identifier, so the error volume is large and the cause is one line away. → Fix the manifests
  first, then work package by package, adding `use deep_causality_context::*;` beside the existing
  glob before touching anything else.
- **`deep_causality_ethos` is breaking for its own consumers, not just for itself.** Its public API
  names `Context` in signatures, so anyone calling it needs the new crate too. → Bump it 0.3.x →
  0.4.0 and cover it in the blog post rather than treating it as a transitive detail.
- **The tier block is hand-maintained and now wrong in two places.** `deep_causality` moves to Tier
  7 and `deep_causality_ethos` / `deep_causality_quantum` to Tier 8, in AGENTS.md and in
  `deep_causality_unified_math/README.md`. → Re-derive both from the `Cargo.toml` files; `crates.sh`
  handles the CI side automatically.
- **A stale Lean witness path reports a false `MISSING Rust witness`.** Three references name the
  moving test: `lean/DeepCausalityFormal/Core/ContextGraph.lean:52` and the `THEOREM_MAP.md` rows
  `core.context_graph.threading_bind` and `core.context_graph.acyclicity_separable`. → Update all
  three **after** the test has moved, in the same commit as the relocation, so the path written is
  the path that exists. The Lean namespace stays put: `Core/` already carries witnesses into two
  crates, so a third needs no reorganisation.
- **62 moved test files must be re-registered.** Every test file needs its `mod.rs` entry with the
  right `#[cfg(test)]`, and the folder modules need declaring in the new crate's `BUILD.bazel`, or
  Bazel silently runs fewer tests than Cargo. → Compare the Cargo and Bazel test counts before and
  after; they must match and must equal the pre-move total.
- **A two-commit move means the tree does not build at the first commit.** → Land both together on
  one branch; do not merge the rename commit alone.
- **Dropping `Copy` from `Data<T>` breaks any site relying on an implicit copy.** Every `T` in use
  today is `Copy`, so no instantiation stops compiling, but `Data<T>` itself is no longer `Copy` and
  a moved-then-reused value now needs `.clone()`. The count is not greppable. → Size it with a build
  rather than by inspection, and do it in its own task group so the errors are not tangled with the
  move. `BaseContextoid` is already not `Copy`, so nothing above `Data` changes shape.

## Migration Plan

1. Scaffold the crate (manifests, `BUILD.bazel`, SBOM, README, lints) with an empty `lib.rs`.
2. `git mv` the 182 source and 62 test files. Tree does not build.
3. Move `Identifiable` into core; rewrite the new crate's imports; publish its `lib.rs` surface.
4. Rewrite `deep_causality`: imports, the removed re-exports, the split `utils_test`.
5. Deduplicate the primitive aliases onto core (Decision 8); relax `Data<T>` (Decision 9).
6. Rewrite `deep_causality_ethos` (27 files), then the four example packages (19 files).
7. Migrate all five monad examples onto the typed `Context`.
8. Re-derive tier blocks, update the three Lean/theorem-map witness references, verify Bazel and
   Cargo test counts match.
9. Write the migration blog post.

Rollback is `git revert` of the branch merge; no data, schema or published artifact is involved.

## Open Questions

- **Is a time series better carried in the monad's `State` channel than in `Context`?** Decision 9
  makes `Data<Vec<f64>>` expressible, so `granger` migrates either way, but the modelling question
  stands: `Context` is read-only reference data and `State` is what evolves. A fixed historical
  series is reference data, which is why `granger` is migrated as `Context`. A series that grows as
  the chain runs would belong in `State`.
