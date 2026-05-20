## Why

The three-tier iso trait surface landed in `2026-05-20-add-iso-traits` but ships **no concrete instances**. Six mathematically clean, low-risk iso pairs across the codebase remain unimplemented: real algebraic isos between `Complex` / `Quaternion` and Clifford-algebra multivectors, structural isos between dense and sparse representations, and topology-side isos for simplicial/cell and Poincaré duality. Until these instances exist, the iso vocabulary cannot be exercised by downstream consumers (CDL pipeline, physics modules, topology workflows) and the trait surface remains a foundation without users.

This change adds six concrete iso instances drawn from [`openspec/notes/IsoCandidates.md`](../../notes/IsoCandidates.md). Each is independently scoped, mathematically pinned, and either Tier 1 (`From`-based) or Tier 2 (witness-typed `Iso<S, T>`). No new trait machinery; every instance consumes the surface that already shipped.

The seventh candidate from the survey (`EffectProcessIso`, Tier 3) was dropped after a closer look: `PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type aliases pointing at the same concrete `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`. The compiler already knows they are identical; an iso with identity bodies adds no information. The one real property worth pinning — that the two independently-written `Functor`/`Monad` impls produce the same output on the shared carrier — is captured by a direct consistency test in `deep_causality_core/tests/`, not via the iso vocabulary.

## What Changes

Six concrete iso instances, grouped into four capabilities by crate-pair, plus a small consistency test in `deep_causality_core`:

**~~Capability `iso-num-multivector`~~ — POSTPONED**
- `Complex<F>` <-> `CausalMultiVector<F>` in Cl(0,1) and `Quaternion<F>` <-> `CausalMultiVector<F>` as Cl(3,0)-even rotor were dropped from this change during Stage C implementation. The spec assumed `CausalMultiVector<F>` implemented the `deep_causality_num` algebraic-trait stack; it does not. Worse, `Field` cannot be honestly impl'd because `Commutative` depends on the runtime metric, not the type. Unlocking these isos requires either phantom-typed metrics (breaking change to all multivector callers) or per-algebra newtype wrappers (`Cl01<F>`, `Cl30Even<F>`) — both of which are separate design exercises out of scope here. The isos remain real and useful; they will land in a follow-up change that ships the prerequisite. See `tasks.md` Part C section.

**Capability `iso-multifield-tensor` (`deep_causality_multivector`):**
- `CausalMultiField<T>` <-> `(CausalTensor<T>, Metric, [T; 3], [usize; 3])`. Same-crate Tier 2 pack/unpack. No algebraic markers.

**Capability `iso-tensor-sparse` (`deep_causality_sparse`):**
- `CausalTensor<F>` (rank-2) <-> `CsrMatrix<F>`. Mixed-tier: forward `From` on rank-2 tensors (panics otherwise), reverse via Tier 2 `Iso<CsrMatrix<F>, CausalTensor<F>>` on `CsrMatrix<F>` as `Self`. Inherent `CsrMatrix::to_dense()` alias. No algebraic markers.

**Capability `iso-topology` (`deep_causality_topology`):**
- `SimplicialComplex<T>` <-> `CellComplex<Simplex>`. Same-crate, Tier 1 forward + `TryFrom` reverse. No algebraic markers.
- `LatticeComplex<D>` <-> `DualLatticeComplex<D>` (Poincaré dual). Same-crate Tier 2 with named witness `PoincareIso<D>`. The non-trivial algorithm in the iso bodies pins the duality between primal and dual chain operators.

**Out-of-band: consistency test for the two propagating-effect `Functor`/`Monad` impls (`deep_causality_core`)**

A new test under `deep_causality_core/tests/iso/` asserts that the `Functor` and `Monad` impls on `PropagatingEffectWitness<CausalityError, EffectLog>` and on `PropagatingProcessWitness<(), ()>` produce identical results on the shared carrier. This is a direct `assert_eq!` between the two `fmap` / `bind` outputs; no iso wrapper is involved. The test pins the consistency that a future refactor of one impl must not silently break the other.

Each iso ships with property-style tests against the existing helper modules (`assert_iso_from_round_trip`, `assert_witness_iso_round_trip`, `assert_field_iso_from_laws`).

## Capabilities

### New Capabilities
- ~~`iso-num-multivector`~~: **POSTPONED**. The Complex <-> Cl(0,1) and Quaternion <-> Cl(3,0)-even isos were dropped during Stage C implementation due to unmet algebraic-trait prerequisites on `CausalMultiVector` and the structural impossibility of an honest `Field` impl without phantom-typed metrics or per-algebra newtypes. Deferred to a follow-up change. The capability spec file was removed from this change.
- `iso-multifield-tensor`: Structural iso between `CausalMultiField<T>` and its underlying `(CausalTensor<T>, Metric, [T; 3], [usize; 3])` carrier in `deep_causality_multivector`.
- `iso-tensor-sparse`: Cross-crate mixed-tier iso between rank-2 `CausalTensor<F>` and `CsrMatrix<F>`. Establishes the orphan-rule mixed-tier template (Tier 1 forward + Tier 2 reverse) for future cross-crate isos.
- `iso-topology`: Two structural isos in `deep_causality_topology`: simplicial-complex <-> cell-complex (partial reverse) and primal lattice <-> Poincaré-dual lattice (full bijection via named `PoincareIso<D>` witness).

### Modified Capabilities

None. This change adds new capabilities without modifying existing ones. The iso trait surfaces (`iso-traits-num`, `iso-traits-haft`) are used as-is; no new requirements on the trait declarations.

## Impact

**Affected crates** (additive; no breaking changes):
- `deep_causality_multivector`: new `iso/` module with two pairs plus the multifield/tensor pack.
- `deep_causality_sparse`: new `iso/` module with the rank-2 forward and the inherent `to_dense()` alias.
- `deep_causality_topology`: new `iso/` module with two structural pairs.
- `deep_causality_core`: new test file under `tests/iso/` only (no source changes); pins `Functor`/`Monad` consistency between the two propagating-effect witnesses.

**Dependencies**: no new external crates. Every instance is built against the existing trait surface from `deep_causality_num::iso{::witness,}` and `deep_causality_haft::iso`.

**Tests**: each iso ships round-trip tests; algebraic-marker isos also ship homomorphism tests (group / ring / field / algebra / division-algebra per applicable level).

**Open questions deferred:**
- Symmetric `Iso<CausalMultiVector<F>, Complex<F>>` impl on `CausalMultiVector` as `Self` (alternative to the named witness) — out of scope; named witness keeps the metric assumption explicit.
- A future `NaturalIso5`-based iso that captures a non-trivial relationship between two 5-arity HKT witnesses (e.g. a log-strip transformation) — out of scope; deferred until a downstream consumer needs it.
- Cross-crate Quaternion <-> CausalMultiVector with `TryFrom` for the partial reverse vs. a precondition-panic on the Tier 2 method — design decision documented in design.md; implementation follows the agreed default.
