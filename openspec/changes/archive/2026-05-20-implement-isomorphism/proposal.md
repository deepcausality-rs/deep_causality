## Why

The three-tier iso trait surface landed in `2026-05-20-add-iso-traits` but ships **no concrete instances**. Six mathematically clean, low-risk iso pairs across the codebase remain unimplemented: real algebraic isos between `Complex` / `Quaternion` and Clifford-algebra multivectors, structural isos between dense and sparse representations, and topology-side isos for simplicial/cell and Poincaré duality. Until these instances exist, the iso vocabulary cannot be exercised by downstream consumers (CDL pipeline, physics modules, topology workflows) and the trait surface remains a foundation without users.

This change adds **two** concrete iso instances drawn from [`openspec/notes/IsoCandidates.md`](../../notes/IsoCandidates.md), plus a propagating-effect/process `Functor`/`Monad` consistency test. Each iso is independently scoped, mathematically pinned, and either Tier 1 (`From`-based) or Tier 2 (witness-typed `Iso<S, T>`). No new trait machinery; every instance consumes the surface that already shipped.

Four other candidates from the survey were dropped during implementation after feasibility audits exposed structural mismatches: `iso-num-multivector` (Stage C, postponed — `CausalMultiVector<F>` lacks the algebraic-trait stack and `Field` cannot be honestly impl'd because `Commutative` is metric-dependent) and `iso-topology` (Stage E, postponed — simplicial/cell is lossy, lattice/dual is trivial). Both are documented for future follow-up changes that resolve the prerequisites.

The seventh candidate from the survey (`EffectProcessIso`, Tier 3) was dropped after a closer look: `PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type aliases pointing at the same concrete `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`. The compiler already knows they are identical; an iso with identity bodies adds no information. The one real property worth pinning — that the two independently-written `Functor`/`Monad` impls produce the same output on the shared carrier — is captured by a direct consistency test in `deep_causality_core/tests/`, not via the iso vocabulary.

## What Changes

**Two concrete iso instances** (down from six after feasibility audits), grouped into two capabilities by crate-pair, plus a small consistency test in `deep_causality_core`:

**~~Capability `iso-num-multivector`~~ — POSTPONED**
- `Complex<F>` <-> `CausalMultiVector<F>` in Cl(0,1) and `Quaternion<F>` <-> `CausalMultiVector<F>` as Cl(3,0)-even rotor were dropped from this change during Stage C implementation. The spec assumed `CausalMultiVector<F>` implemented the `deep_causality_num` algebraic-trait stack; it does not. Worse, `Field` cannot be honestly impl'd because `Commutative` depends on the runtime metric, not the type. Unlocking these isos requires either phantom-typed metrics (breaking change to all multivector callers) or per-algebra newtype wrappers (`Cl01<F>`, `Cl30Even<F>`) — both of which are separate design exercises out of scope here. The isos remain real and useful; they will land in a follow-up change that ships the prerequisite. See `tasks.md` Part C section.

**Capability `iso-multifield-tensor` (`deep_causality_multivector`):**
- `CausalMultiField<T>` <-> `(CausalTensor<T>, Metric, [T; 3], [usize; 3])`. Same-crate Tier 2 pack/unpack. No algebraic markers.

**Capability `iso-tensor-sparse` (`deep_causality_sparse`):**
- `CausalTensor<F>` (rank-2) <-> `CsrMatrix<F>`. Mixed-tier: forward `From` on rank-2 tensors (panics otherwise), reverse via Tier 2 `Iso<CsrMatrix<F>, CausalTensor<F>>` on `CsrMatrix<F>` as `Self`. Inherent `CsrMatrix::to_dense()` alias. No algebraic markers.

**~~Capability `iso-topology`~~ — POSTPONED**
- `SimplicialComplex<T>` <-> `CellComplex<Simplex>` was dropped because the pair is not an iso: forward is lossy (drops `boundary_operators`, `coboundary_operators`, `hodge_star_operators`), reverse has to fabricate the matrices and loses the `T` parameter. These are useful conversions, but they belong in a separate change focused on inter-topology-type conversions.
- `LatticeComplex<D>` <-> `DualLatticeComplex<D>` (Poincaré) was dropped because `DualLatticeComplex<D>` is a thin `Arc<primal>` wrapper. No cells are materialised in either struct (computed on demand from `shape`), so the data-level iso reduces to wrap/unwrap. The spec scenarios about k-cell <-> (D-k)-cell mapping describe operation semantics, not iso correctness. Adding the witness type doesn't justify its weight given the existing API.

**Out-of-band: consistency test for the two propagating-effect `Functor`/`Monad` impls (`deep_causality_core`)**

A new test under `deep_causality_core/tests/iso/` asserts that the `Functor` and `Monad` impls on `PropagatingEffectWitness<CausalityError, EffectLog>` and on `PropagatingProcessWitness<(), ()>` produce identical results on the shared carrier. This is a direct `assert_eq!` between the two `fmap` / `bind` outputs; no iso wrapper is involved. The test pins the consistency that a future refactor of one impl must not silently break the other.

Each iso ships with property-style tests against the existing helper modules (`assert_iso_from_round_trip`, `assert_witness_iso_round_trip`, `assert_field_iso_from_laws`).

## Capabilities

### New Capabilities
- ~~`iso-num-multivector`~~: **POSTPONED**. The Complex <-> Cl(0,1) and Quaternion <-> Cl(3,0)-even isos were dropped during Stage C implementation due to unmet algebraic-trait prerequisites on `CausalMultiVector` and the structural impossibility of an honest `Field` impl without phantom-typed metrics or per-algebra newtypes. Deferred to a follow-up change. The capability spec file was removed from this change.
- `iso-multifield-tensor`: Structural iso between `CausalMultiField<T>` and its underlying `(CausalTensor<T>, Metric, [T; 3], [usize; 3])` carrier in `deep_causality_multivector`.
- `iso-tensor-sparse`: Cross-crate mixed-tier iso between rank-2 `CausalTensor<F>` and `CsrMatrix<F>`. Establishes the orphan-rule mixed-tier template (Tier 1 forward + Tier 2 reverse) for future cross-crate isos.
- ~~`iso-topology`~~: **POSTPONED**. Both candidates failed the feasibility audit: simplicial-complex <-> cell-complex is lossy (not an iso), and lattice <-> dual-lattice reduces to a trivial Arc-wrap that adds no value. The capability spec file was removed from this change.

### Modified Capabilities

None. This change adds new capabilities without modifying existing ones. The iso trait surfaces (`iso-traits-num`, `iso-traits-haft`) are used as-is; no new requirements on the trait declarations.

## Impact

**Affected crates** (additive; no breaking changes):
- `deep_causality_multivector`: new `src/extensions/iso_multifield/` module with the structural pack/unpack pair. Adds `#[derive(PartialEq)]` to `CausalMultiField<T>` (mechanical, no existing manual impl).
- `deep_causality_sparse`: new `src/extensions/ext_iso.rs` (feature-gated behind `tensor-iso`) with the rank-2 forward `From` and inherent `to_dense()` alias. Cargo.toml gains an optional `deep_causality_tensor` dep.
- `deep_causality_core`: new tests under `tests/iso/` only (no source changes); pins `Functor`/`Monad` consistency between the two propagating-effect witnesses. Bonus fix: missing `use alloc::vec;` for the `vec!` macro under `--no-default-features --features alloc`.

**Crates originally in scope, now untouched** (Stages C and E were postponed):
- `deep_causality_num` / `deep_causality_multivector` (the algebraic isos in Stage C — postponed pending phantom-metric or newtype prerequisite).
- `deep_causality_topology` (Stage E — both candidate isos failed the feasibility audit).

**Dependencies**: no new *external* crates. The only new internal dep is `deep_causality_tensor` added optionally to `deep_causality_sparse` (gated by the `tensor-iso` feature, off by default).

**Tests**: each shipped iso has round-trip tests. The tensor/sparse iso also has `#[should_panic]` rank-mismatch tests.

**Open questions deferred** (for follow-up changes):
- The Complex <-> Cl(0,1) and Quaternion <-> Cl(3,0)-even isos are real and useful but require either phantom-typed metrics or per-algebra newtype wrappers on `CausalMultiVector`. Future change.
- The `SimplicialComplex <-> CellComplex<Simplex>` conversion (lossy projection + best-effort fabrication) could land as a non-iso conversion pair in a topology-conversions change.
