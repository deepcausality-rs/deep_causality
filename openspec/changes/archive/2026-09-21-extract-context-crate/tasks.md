## 1. Scaffold the crate

- [x] 1.1 Create `deep_causality_context/` with `Cargo.toml`: version 0.1.0, workspace-inherited
      edition/rust-version/license/repository/homepage, `[lints] workspace = true`, the standard
      `exclude` list, and dependencies on `deep_causality_core`, `deep_causality_data_structures`,
      `deep_causality_uncertain` and `ultragraph`
- [x] 1.2 Add `deep_causality_context` to `members` in the root `Cargo.toml` and add
      `deep_causality_context = { path = "deep_causality_context", version = "0.1" }` to
      `[workspace.dependencies]` (two-digit precision, per repo convention)
- [x] 1.3 Add `BUILD.bazel` modelled on `deep_causality/BUILD.bazel` (`rust_library`, `rust_doc`,
      `rust_doc_test`, plus a `rust_test_suite` per test folder). Corrected during apply: the source
      crate keeps its test suites in its own `BUILD.bazel` and has no separate `tests/BUILD.bazel`,
      so the homology layout the task first named does not apply here
- [x] 1.4 Add `README.md`; do **not** create `CHANGELOG.md` by hand — release-plz generates it
- [x] 1.5 Create an empty `src/lib.rs` with the SPDX header and confirm
      `cargo build -p deep_causality_context` succeeds
- [x] 1.6 Verify `source scripts/crates.sh` now lists the crate in `DC_CRATES` and `DC_CRATE_DIRS`

## 2. Move the files (rename-only commit)

- [x] 2.1 `git mv deep_causality/src/types/context_types` and `context_node_types` into
      `deep_causality_context/src/types/`
- [x] 2.2 `git mv` `traits/{contextuable,contextuable_graph,adjustable,indexable,scalar}` into
      `deep_causality_context/src/traits/`
- [x] 2.3 `git mv` `errors/{context_index_error,index_error,adjustment_error,update_error}.rs` into
      `deep_causality_context/src/errors/`
- [x] 2.4 `git mv` the 62 context test files into `deep_causality_context/tests/`, mirroring the src
      layout, including `formalization_lean/context_graph_tests.rs`
- [x] 2.5 Commit the renames with no content edits, and confirm `git log --follow` resolves history
      for a sample file from each group. The tree does not build at this commit — that is expected

## 3. Move `Identifiable` into core

- [x] 3.1 Add `deep_causality_core/src/traits/identifiable/mod.rs` declaring
      `fn id(&self) -> IdentificationValue`, register the module, and export it from core's `lib.rs`
- [x] 3.2 Remove `deep_causality/src/traits/identifiable/` and its `lib.rs` export; point the six
      causal-side impls (`Causaloid`, `Model`, `Inference`, `Assumption`, `Observation`,
      `ProposedAction`) at `deep_causality_core::Identifiable`
- [x] 3.3 Add a core test asserting two implementors standing in for the causal and context sides
      satisfy one `Identifiable`-bounded generic function. Corrected during apply: core cannot see
      the real `Causaloid` or `Contextoid` (both crates depend on core), so the genuine cross-crate
      assertion is added in `deep_causality`'s tests under task 5.6, which is the only place both
      types are visible
- [x] 3.4 `cargo test -p deep_causality_core` green

## 4. Make the new crate compile

- [x] 4.1 Rewrite the moved files' imports: `crate::` paths that resolve inside the new crate stay,
      `Identifiable` comes from `deep_causality_core`, and `FloatType`, `ContextId`, `ContextoidId`
      and `NumericalValue` come from `deep_causality_core` — the new crate declares no alias of its
      own for them
- [x] 4.2 Add `BaseContext`, `BaseContextoid`, `UniformContext` and `UniformContextoid` to the new
      crate, lifted from `deep_causality/src/alias/alias_base.rs` and `alias_uniform.rs` (only these
      four; the `BaseCausaloid`/`UniformCausaloid` family stays behind)
- [x] 4.3 Move `get_context`, `get_base_context` and `get_test_context` into
      `deep_causality_context/src/utils_test/`, and add tests for them (they count toward coverage)
- [x] 4.4 Write `src/lib.rs`: module declarations plus the full public export list. No prelude
- [x] 4.5 Fix the `Causaloid` "See Also" line in `traits/scalar/scalar_projector.rs` so it does not
      name a type from a crate this one cannot see
- [x] 4.6 Register every moved test file in its `mod.rs` with `#[cfg(test)]`, declare the folder
      modules in `BUILD.bazel`, and confirm `cargo test -p deep_causality_context` and
      `bazel test //deep_causality_context/...` report the same count as before the move

## 4b. Remove the symbolic (SYM) dimension

Added during apply, on the user's decision: nothing in the workspace ever constructs a
`ContextoidType::Symboid`. The only non-test references were the enum's own arms, so the parameter
existed solely to be threaded through signatures and filled in by aliases.

- [x] 4b.1 Remove `context_node_types/symbol/` (`BaseSymbol`, `SymbolKind`),
      `types/symbolic_types/` (`SymbolicRepresentation`, `SymbolicResult`) and
      `traits/contextuable/symbolic.rs` (`Symbolic`), with their tests
- [x] 4b.2 Drop `SYM` from `Context`, `Contextoid`, `ContextoidType`, `Contextuable`,
      `ContextuableGraph` and `ExtendableContextuableGraph`: seven type parameters become six
- [x] 4b.3 Remove the `ContextoidType::Symboid` variant, `ContextKind::Symboid`, the `symboid()`
      accessor and the Display arm
- [x] 4b.4 Update `BaseContext`, `BaseContextoid`, `UniformContext`, `UniformContextoid`
- [x] 4b.5 Update `deep_causality_ethos` (`BaseTeloidStore`, `Teloid`, `TeloidStore`, `EffectEthos`)
      and `examples/csm_examples/csm_effect_ethos` for the six-parameter context
- [x] 4b.6 `SymbolicTime`, `SymbolicTimeUnit`, `TimeScale::Symbolic` and `symbol_spacetime`
      (`CausalSetSpacetime`, `ConformalSpacetime`) are unaffected — they are temporal and spacetime
      nodes that never referenced the `Symbolic` trait

## 5. Rewrite `deep_causality`

- [x] 5.1 Add `deep_causality_context` to `deep_causality/Cargo.toml`
- [x] 5.2 Remove every moved item from `src/lib.rs` and add **no** re-export of the new crate
      (`context-explicit-dependency`)
- [x] 5.3 Fix the 9 named-import files in `src/`: `Model`, the generative interpreter,
      `UncertainActivationPredicate`, `model_validation_error`, the CSM `UpdateError` call sites,
      and the `Base*`/`Uniform*` causal aliases
- [x] 5.4 Fix the 12 `use crate::*` files in `src/` by adding explicit
      `use deep_causality_context::{…}` imports
- [x] 5.5 Split `src/utils_test/test_utils.rs`: keep the Causaloid/Model/Inference/Observation
      builders, import the three context builders from `deep_causality_context`
- [x] 5.6 Fix the 33 staying test files (6 named imports, 27 `use deep_causality::*` globs) and the
      3 benches
- [x] 5.7 `cargo test -p deep_causality` and `cargo bench -p deep_causality --no-run` green

## 6. Deduplicate the primitive aliases onto core

- [x] 6.1 Confirm all ten declarations in `deep_causality/src/alias/alias_primitives.rs` are still
      byte-identical to `deep_causality_core/src/alias/mod.rs` before removing anything
- [x] 6.2 Delete `alias_primitives.rs`, its `pub(crate) mod` declaration and its glob re-export in
      `deep_causality/src/alias/mod.rs`
- [x] 6.3 Re-export the eight live aliases from core in `deep_causality/src/lib.rs`:
      `IdentificationValue`, `ContextId`, `ContextoidId`, `CausaloidId`, `DescriptionValue`,
      `NumericalValue`, `NumberType`, `FloatType`
- [x] 6.4 Do **not** re-export `TeloidTag` or `TeloidID` — both are dead in `deep_causality` and
      `deep_causality_ethos` declares its own pair
- [x] 6.5 Confirm no call site changed: `cargo test -p deep_causality` green with zero import edits
      in the 202 example files naming `FloatType` and the 25 test files naming `NumericalValue`

## 7. Relax the `Data<T>` payload bound

- [x] 7.1 Change the `Data<T>` where-clause to `T: Default + Clone + PartialEq` and drop `Copy` from
      its `#[derive(...)]`, in `deep_causality_context/src/types/context_node_types/data/mod.rs`
- [x] 7.2 Change `Datable for Data<T>` to the same bound and return `self.data.clone()` from
      `get_data`
- [x] 7.3 Leave `Adjustable<T> for Data<T>` untouched — it already restates `Copy` alongside
      `Hash + Eq + PartialOrd + Add + Sub + Mul`, which `ArrayGrid<T, …>` genuinely requires
- [x] 7.4 Build the workspace to size the blast radius of `Data<T>` no longer being `Copy`, and
      replace each former implicit copy with an explicit `clone()`. Do not widen any other bound to
      make an error go away
- [x] 7.5 Add a test constructing a `Data<Vec<f64>>`, reading it back through `Datable`, and placing
      it in a `Contextoid` inside a `Context`
- [x] 7.6 Confirm `BaseContextoid` is not `Copy` before and after, so nothing above `Data` changed
      shape
- [x] 7.7 Confirm the existing `Data<f64>` update/adjust tests still pass unchanged, including the
      zero-value rejection in `update`

## 8. Rewrite `deep_causality_ethos`

- [x] 8.1 Add `deep_causality_context` to `deep_causality_ethos/Cargo.toml`
- [x] 8.2 Fix the 27 files importing `Context`, `BaseContext`, `Datable`, `SpaceTemporal`,
      `Spatial`, `Symbolic`, `Temporal`, `BaseSymbol`, `Euclidean*` and `Identifiable`
- [x] 8.3 Confirm its own `TeloidTag`/`TeloidID` still resolve from its own `alias` module,
      unaffected by task 6.4
- [x] 8.4 `cargo test -p deep_causality_ethos` green

## 9. Rewrite the example packages

- [x] 9.1 Add `deep_causality_context` to the manifests of `classical_causality_examples`,
      `csm_examples`, `tokio_example` and `avionics_examples`
- [x] 9.2 Fix imports in `csm_examples` (4 files), `tokio_example` (2) and `avionics_examples`
      (2 globs)
- [x] 9.3 Fix imports in the causaloid-side `classical_causality_examples` (11 files, 7 of them
      `use deep_causality::*` globs)
- [x] 9.4 Run every example in the four packages and confirm output is unchanged

## 10. Migrate the monad examples onto the typed context

- [x] 10.1 `rcm_via_monad`: replace `TreatmentContext` with a `BaseContext` of `Data` contextoids;
      keep the `alternate_context` factual/counterfactual structure and the same printed outcome
- [x] 10.2 `cate_via_monad`: same treatment for `PatientContext` (two `f64`, one `bool`)
- [x] 10.3 `scm_via_monad`: same treatment for `SmokingContext` (two `f64`)
- [x] 10.4 `dbn_via_monad`: same treatment for `WeatherContext` (`&'static str`, two `f64`), leaving
      `WeatherState` in the monad's State channel where it belongs
- [x] 10.5 `granger_via_monad`: migrated onto `Context<Data<Vec<FloatType>>, …>` — the acceptance
      test for the group 7 relaxation. Its two series are `Data<Vec<FloatType>>` contextoids and the
      counterfactual world is the same builder with `oil_prices` empty
- [x] 10.6 Confirm each migrated example still has a `rust_binary` in `BUILD.bazel` and that
      `make check_examples` passes
- [x] 10.7 Confirm no monad-side example declares a context struct of its own any more, which is
      the demonstration issue #801 asked for
- [x] 10.8 Added during apply: every migrated example declares `type FloatType = f64;` locally,
      before `main`, rather than importing the shared alias. An alias change in
      `deep_causality_core` must not silently reconfigure every example that names one. The
      pre-existing `deep_causality::FloatType` in `csm_effect_ethos/model.rs` was localized too

## 11. Update the formalization to match the moved tree

- [x] 11.1 **No-op, resolved during apply.** `context_graph_tests.rs` pins the real `Context` AND
      the real `CausalEffectPropagationProcess::bind`, so it needs both crates and stays in
      `deep_causality`, the only crate that sees both. The path in
      `lean/DeepCausalityFormal/Core/ContextGraph.lean:52` is therefore still correct and unchanged
- [x] 11.2 **No-op, same reason.** Both `THEOREM_MAP.md` rows still name a path that exists
- [x] 11.3 Leave the Lean namespace where it is: `Core/` already carries witnesses into two crates,
      so a third needs no reorganisation and no `lean/BUILD.bazel` change
- [x] 11.4 Confirm the formalization workflow reports no `MISSING Rust witness`, and that every
      named test function in those three references still exists at the new path

## 12. Repair the remaining cross-cutting references

- [x] 12.1 Re-derive the tier block in `AGENTS.md` from the `Cargo.toml` files: add the crate to the
      index, 30 crates becomes 31, `deep_causality` to Tier 7, `deep_causality_ethos` and
      `deep_causality_quantum` to Tier 8
- [x] 12.2 **No-op, verified during apply.** That README's tier block covers only the mathematics
      crates. `deep_causality_context` is not one, and no mathematics crate changed tier (topology
      stays 7, multivector 6), so the block is still correct
- [x] 12.3 Generate the SBOM pair `deep_causality_context_sbom.spdx.json` and its `.sha`

## 13. Verify the whole change

- [x] 13.1 `make format && make fix` — clippy lints fixed by rewriting, not by `#[allow]`
- [x] 13.2 `bazel test //...` green across the workspace
- [x] 13.3 Assert the no-re-export contract: `deep_causality/src/lib.rs` contains no `pub use` of
      `deep_causality_context`, and `use deep_causality::Context;` fails to compile
- [x] 13.4 Assert reachability: a scratch crate depending only on `deep_causality_core` and
      `deep_causality_context` builds a `PropagatingProcess<f64, (), BaseContext>`
- [x] 13.5 Confirm `cargo build -p deep_causality_core --no-default-features --features no-std`
      still succeeds
- [x] 13.6 Confirm each of the eight deduplicated aliases is declared exactly once in the workspace
- [x] 13.7 Confirm the workspace dependency search returns exactly the crates that use context
- [x] 13.8 Confirm `Data<Vec<f64>>` compiles and that no bound other than `Data<T>`'s was widened to
      get there

## 14. Release communication

- [x] 14.1 **Corrected during apply: release-plz owns versions.** Commit `7585f4d42` shows it
      bumping both the crate versions and the two-digit workspace constraints itself, so the manual
      bumps were rolled back. The `BREAKING CHANGE:` footers drive thecomputation.
      `deep_causality_context` keeps an explicit 0.1.0, which a new crate needs in order to publish
      at all. Verified: the new crate is in `Cargo.lock`, carries full crates.io metadata, has no
      `publish = false`, and the release workflow reads no hardcoded crate list
- [x] 14.2 Draft at `docs/drafts/context_crate_release_blogpost_draft.md`. Migration blog post: what moved, why the context dependency is now declared
      rather than inherited, the `Cargo.toml` line and import rewrite a consumer makes, the removal
      of `deep_causality::TeloidTag`/`TeloidID`, the relaxed `Data<T>` bound (`Data<T>` is no longer
      `Copy`), and the note that `deep_causality_ethos` is
      breaking for its own consumers
- [x] 14.3 Put the breaking-change and migration detail in the commit messages, since release-plz
      generates the changelogs from them
- [x] 14.4 Prepare the commit messages and hand them to the user to commit
