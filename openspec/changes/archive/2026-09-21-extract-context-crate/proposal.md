## Why

The typed context ships inside `deep_causality`, which sits above `deep_causality_core`, so a
consumer of the causal monad cannot name it. The consequence is in the tree: **all five**
monad-side classical examples hand-roll a context struct — `TreatmentContext`, `PatientContext`,
`WeatherContext`, `SeriesContext`, `SmokingContext` — and thread each through
`PropagatingProcess<f64, _, TheirOwnStruct>`, while the causaloid-side examples next to them reach
the real `Context` through `deep_causality::*`. Two ways to carry context, because one of them has
no access to the other.

Issue #801 frames this as a typing asymmetry — a Causaloid with "a designated context type" against
a monad that "declares context as a generic parameter". That is not what the code says.
`Causaloid<I, O, STATE, CTX>` bounds `CTX: Clone` and nothing more
(`deep_causality/src/types/causal_types/causaloid/mod.rs:63`); the monad's `Context` channel is
equally open (`deep_causality_core/src/types/causal_effect_propagation_process/mod.rs:57`). Both are
unbounded. The asymmetry is **reachability**, not typing: `Causaloid` and `Context` are compiled
together, so the aliases `BaseContext` and `ContextualCausalFn` wire them up for free, and the monad
gets no such wiring because it lives one crate below.

Extraction fixes reachability. The context layer has **zero** references to `deep_causality_core`
today, so the cut is clean and the new crate can sit beside `deep_causality` rather than beneath it.

## What Changes

- **New crate `deep_causality_context` 0.1.0** at the repository root, holding the whole context
  layer: the `Context` hypergraph, `Contextoid`, `ContextoidType`, `RelationKind`, `TimeScale`, the
  context node types, the `contextuable` traits, `ContextuableGraph` /
  `ExtendableContextuableGraph`, `Adjustable` / `UncertainAdjustable`, the six indexable traits,
  `ScalarProjector` / `ScalarValue`, four error types, and the `BaseContext` / `UniformContext` /
  `BaseContextoid` / `UniformContextoid` aliases, plus the `SymbolicRepresentation` the symbol nodes
  needed. 187 source files and 99 test files moved by `git mv`, so history follows.
- **`Identifiable` moves to `deep_causality_core`.** It returns `IdentificationValue`, which core
  already owns (`deep_causality_core/src/alias/mod.rs`), and it is implemented on both sides of the
  cut — `Contextoid` on the context side, `Causaloid`, `Model`, `Inference`, `Assumption`,
  `Observation` and `ProposedAction` on the causal side. Core is the only place both can reach.
- **BREAKING: `deep_causality` does not re-export the moved surface.** Context is an opt-in
  dependency, declared where it is used. Non-contextual reasoning is the default case and should
  carry no context dependency; a model that does need context adds `deep_causality_context` to its
  `Cargo.toml` and imports from it. The decision becomes explicit, and a dependency search answers
  "where is context actually used?" honestly — which a re-export would have hidden by keeping every
  existing `use deep_causality::Context` compiling.
- **BREAKING: `UpdateError` moves.** It is used by the context node types' `Adjustable` impls *and*
  by `types/csm_types/csm/{state_add,state_remove,state_update}.rs`, which stays behind — so
  `deep_causality` takes a dependency on the context crate for it.
- **`Data<T>` loses its `Copy` bound.** `Data<T>` bounds `T: Default + Copy + Clone + PartialEq`
  (`deep_causality/src/types/context_node_types/data/mod.rs:30`) and is the **only** context node
  type that constrains a payload parameter this way — `EcefSpace`, `GeoSpace`, `EuclideanSpace` and
  `SpaceKind` are `Clone`-only. Nothing about being a context node needs `Copy`: `Datable` requires
  nothing, `Context` requires `D: Datable + Clone`, and `Contextoid`'s `#[derive(Copy, Hash, Eq)]`
  is conditional — and already inapplicable to `BaseContext`, whose `EuclideanSpace` is `Clone`-only.
  The bound is relaxed to `T: Default + Clone + PartialEq`, and `Copy` stays on the `Adjustable`
  impl, which is where `ArrayGrid<T, …>`'s fixed-size array backing genuinely demands it and which
  already restates the bound.
- **All five monad-side examples migrate.** `rcm`, `cate`, `scm`, `dbn` and `granger` drop their
  hand-rolled context structs and carry a real `Context` through the same `alternate_context`
  chains, which is the demonstration the issue asked for. `granger` is migratable because of the
  bound relaxation above: its `SeriesContext` holds two `Vec<f64>` time series, and `Data<Vec<f64>>`
  becomes expressible.
- **The duplicated primitive aliases are deduplicated onto core.**
  `deep_causality/src/alias/alias_primitives.rs` declares ten aliases that
  `deep_causality_core/src/alias/mod.rs` already declares byte-identically. The file goes; eight
  live aliases are re-exported from core so `deep_causality`'s surface is unchanged, and
  `TeloidTag` / `TeloidID` are dropped rather than re-exported because they are dead there —
  the vocabulary belongs to `deep_causality_ethos`, which declares its own pair.
- **BREAKING: the symbolic dimension is withdrawn.** `Context`, `Contextoid` and `ContextoidType`
  drop from seven type parameters to six; `BaseSymbol`, `SymbolKind`, `SymbolicRepresentation`,
  `SymbolicResult`, the `Symbolic` trait and `ContextoidType::Symboid` are removed. Nothing in the
  workspace ever constructed a symbol contextoid: outside the enum's own arms and a few tests there
  was no construction site at all, so the parameter existed only to be threaded and filled in by
  aliases. `SymbolicTime`, `TimeScale::Symbolic` and the `symbol_spacetime` nodes are temporal and
  spacetime types and are untouched.
- **A migration blog post** ships with the change, published on release date: what moved, why the
  dependency is now explicit, and the one-line `Cargo.toml` plus import edit each consumer needs.
  Drafted at `docs/drafts/context_crate_release_blogpost_draft.md`.
- **Versions are left to release-plz**, which bumps the crates and the workspace constraints from
  the conventional-commit footers. The new crate carries an explicit 0.1.0 because a crate cannot
  be published without a starting version.
- **Out of scope, stated so it is not assumed:** the monad's `Context` parameter stays unbounded.
  Bounding it on a contract trait requires that trait to live in `deep_causality_core` (Tier 3),
  which cannot see a hypergraph that needs `ultragraph`, `deep_causality_uncertain` and `std`. It
  would also need an impl for `()`, and would change every `CausalFlow`, `PropagatingProcess` and
  `Causaloid` signature in the workspace. That is a separate, breaking change and a follow-up.

## Capabilities

### New Capabilities
- `context-crate-identity`: what `deep_causality_context` owns and what it refuses — the context
  hypergraph, its node types and its traits — together with its dependency envelope (it may not
  depend on `deep_causality`) and the reachability guarantee that a consumer depending only on
  `deep_causality_core` and `deep_causality_context` can instantiate the monad's context channel
  with the typed `Context`.
- `context-explicit-dependency`: the no-re-export contract. `deep_causality` SHALL NOT re-export any
  moved item, so reaching context requires a declared dependency on `deep_causality_context`. States
  the migration obligation each consumer carries and what a dependency search is therefore entitled
  to conclude.
- `context-symbolic-dimension-removed`: the withdrawal of the `SYM` parameter, the symbol node
  types, the `Symbolic` trait and the `Symboid` arm. `Context` goes from seven type parameters to
  six. Nothing in the workspace ever constructed a symbol contextoid.
- `context-data-node-payload`: what `Data<T>` requires of its payload — `Clone`, not `Copy` — and
  where the `Copy` requirement properly belongs (the `Adjustable` impl, because `ArrayGrid` is
  array-backed). Makes a sequence-valued context node expressible.
- `core-shared-vocabulary`: `deep_causality_core` is the single home for the vocabulary both layers
  share — the `Identifiable` trait, beside the `IdentificationValue` it returns, and the primitive
  type aliases, each declared exactly once.

### Modified Capabilities
<!-- None. `openspec/specs/context-hypergraph-formalization/` is the Lean model of parent-set
     hyperedge semantics and names no Rust path or crate, so relocating the Rust types does not
     change any requirement it states. No other spec under openspec/specs/ names `Context`,
     `Contextoid` or `BaseContext`. -->

## Impact

**New.** `deep_causality_context` 0.1.0, depending on `deep_causality_core` (aliases,
`Identifiable`), `deep_causality_data_structures` (`ArrayGrid`, `PointIndex`),
`deep_causality_uncertain` (the uncertain node types) and `ultragraph` (the hypergraph backend).
It lands at **Tier 6**, pushing `deep_causality` to Tier 7 and `deep_causality_ethos` /
`deep_causality_quantum` to Tier 8. The crate is std-only; `deep_causality_core` keeps its `no-std`
feature because core does **not** depend on the new crate.

Needs `BUILD.bazel` and `tests/BUILD.bazel`, an SBOM pair, a README, `[lints] workspace = true`, a
root `Cargo.toml` member plus workspace-dependency entry, and an AGENTS.md crate-index entry with
the tier block re-derived (30 crates becomes 31). `scripts/crates.sh` reads the member list from the
manifest, so the formalization and coverage workflows pick the crate up with no hand edit.

**Breaking.** `deep_causality` 0.17.0 → **0.18.0**. 182 files leave `src/` and 62 test files leave
`tests/`. Every remaining consumer of a moved item declares the new dependency and edits its
imports:

| Consumer | Files to edit | Note |
|---|---|---|
| `deep_causality_ethos` | 27 src/test files + `Cargo.toml` | 0.3.x → **0.4.0**; its public API names `Context`, so its own consumers need the crate too |
| `deep_causality/src` | 21 (9 named imports, 12 `use crate::*`) | `Model`, the generative interpreter, `UncertainActivationPredicate`, the `Base*`/`Uniform*` aliases, `utils_test` |
| `deep_causality/tests` | 33 that stay (6 named, 27 glob) | a further 62 move with the types |
| `deep_causality/benches` | 3 (`use deep_causality::*`) | |
| `examples/classical_causality_examples` | 11 + `Cargo.toml` | includes the four monad examples being migrated |
| `examples/csm_examples` | 4 + `Cargo.toml` | |
| `examples/tokio_example` | 2 + `Cargo.toml` | |
| `examples/avionics_examples` | 2 (`use deep_causality::*`) + `Cargo.toml` | |

The 46 `use deep_causality::*;` glob imports are the sharp edge: they do not name the moved symbols,
so they fail only at the use site, with errors that point at the identifier rather than the import.
Each needs a matching `use deep_causality_context::*;` or explicit imports.

**Modified, additive.** `deep_causality_core` 0.12.1 gains `Identifiable`.

**The `Data<T>` relaxation is widening, but it drops a `Copy` impl.** Every `T` in use today is
`Copy`, so no instantiation stops compiling. `Data<T>` itself stops being `Copy`, so any site
relying on an implicit copy needs a `.clone()`. `BaseContextoid` is unaffected — it is already not
`Copy`, because `EuclideanSpace` is `Clone`-only. The blast radius needs a build to size and is
budgeted as its own task group.

**Alias deduplication costs no call site.** All ten declarations are byte-identical to core's, and
the eight live ones keep their names at `deep_causality`'s root via re-export, so the 202 example
files naming `FloatType` and the 25 test files naming `NumericalValue` need no edit. Only
`TeloidTag` / `TeloidID` leave the surface, and nothing imports them from `deep_causality`.

**Unaffected.** `deep_causality_physics` and `deep_causality_cfd` do not use these types; their
`Coordinate` and `Spatial` matches are their own `SpaceTimeCoordinate` and `BodyFittedCoordinate`.
`deep_causality_quantum` and `deep_causality_discovery` have no context usage. Eleven of the fifteen
example packages name no moved item: `causal_correction`, `causal_counterfactual`,
`causal_discovery`, `causal_uncertain`, `core`, `material`, `mathematics`, `medicine`, `physics`,
`quantum` and `starter`.

**Formalization follows the move.** `lean/DeepCausalityFormal/Core/ContextGraph.lean:52` and
`lean/THEOREM_MAP.md` rows `core.context_graph.threading_bind` and
`core.context_graph.acyclicity_separable` all name
`deep_causality/tests/formalization_lean/context_graph_tests.rs` as their Rust witness. That test
moves with the type, so all three references are updated **after** the move lands, in the same
commit as the test relocation. The formalization workflow greps these paths out of the Lean and
theorem-map sources, so a stale path reports a false `MISSING Rust witness`. The Lean namespace
itself does not move: `Core/` already spans two crates — nine of its files witness into
`deep_causality_core` and six into `deep_causality` — so a third witness crate is consistent with
how that namespace is already organised.

**Noted, not fixed here.** `TeloidTag` and `TeloidID` remain declared in `deep_causality_core` with
no user in core or in `deep_causality`, because `deep_causality_ethos` declares its own pair. That
leaves one dead copy after this change. Moving the Teloid vocabulary to the crate that owns it is a
separate change.
