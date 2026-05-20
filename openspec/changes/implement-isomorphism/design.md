## Context

The iso trait surface landed in `2026-05-20-add-iso-traits` and lives at:
- `deep_causality_num::iso` (Tier 1, `From`-based marker subtraits)
- `deep_causality_num::iso::witness` (Tier 2, witness-typed `Iso<S, T>` + `StandardIso<S, T>`)
- `deep_causality_haft::iso` (Tier 3, `NaturalIso<F, G>` through `NaturalIso5`)

Two follow-up notes shape this proposal: [`openspec/notes/IsoCandidates.md`](../../notes/IsoCandidates.md) (full survey, scored) and [`openspec/notes/IsoFollowUps.md`](../../notes/IsoFollowUps.md) (detailed sketches for the tensor/sparse and effect/process pairs). This change instantiates six of those candidates. Three were dropped: Graph <-> Hypergraph and CsrMatrix <-> Graph adjacency (impractical), and `EffectProcessIso` (since `PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type aliases for the same `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`, an iso with identity bodies is just notation; the real property — that the two independently-written `Functor`/`Monad` impls agree — is pinned via a direct consistency test instead).

## Goals / Non-Goals

**Goals:**
- Land six concrete iso instances spanning Tier 1 and Tier 2 so the trait surface is exercised against real codebase types.
- Establish the mixed-tier template (Tier 1 forward + Tier 2 reverse) for future cross-crate isos, using tensor/sparse and num/multivector as the worked examples.
- Pin every iso with property-style tests covering round-trip and (where applicable) homomorphism laws.
- Pin `Functor`/`Monad` consistency between the two propagating-effect HKT witnesses with a direct consistency test (no iso wrapper).
- Keep every instance additive: no breaking changes to existing surfaces.

**Non-Goals:**
- Adding new iso traits or extending the existing trait surface. Every instance consumes what the prior change already shipped.
- Embeddings (e.g. `Complex` -> `Quaternion` with `j=k=0`). These are not bijections and were explicitly excluded in `IsoCandidates.md`.
- Lossy projections (e.g. `CausalMultiField` -> bare `CausalTensor` without metric metadata).
- Octonion isos. Octonions are non-associative; no Clifford-algebra iso preserving multiplication exists.
- An `EffectProcessIso` at any arity for the propagating-effect case. The arity-1 case has identity bodies (both witnesses project to the same concrete type) and adds no information beyond a direct consistency test. A meaningful Tier 3 iso here would target a non-trivial 5-arity relationship between two HKT5 witnesses (e.g. log-strip); deferred until a downstream consumer needs it.

## Decisions

### D1. Capabilities grouped by crate-pair, not by individual iso

Four capabilities (`iso-num-multivector`, `iso-multifield-tensor`, `iso-tensor-sparse`, `iso-topology`) rather than six (one per pair). Rationale: capabilities map to spec files, and per-pair specs would produce six near-identical spec scaffolds. Grouping by destination crate keeps the spec count in line with the actual code locations. The propagating-effect consistency test ships as a test-only addition with no associated capability.

### D2. Mixed-tier as the default for cross-crate pairs

`Complex <-> Cl(0,1) multivector`, `Quaternion <-> Cl(3,0)-even rotor`, and `CausalTensor <-> CsrMatrix` all cross crate boundaries with an asymmetric dependency. Each ships Tier 1 forward `From` in the downstream crate plus Tier 2 reverse `Iso<S, T>` on the downstream type as `Self`. The reverse is paired with an ergonomic inherent-method alias (`to_dense()`, `to_complex()`, `to_quaternion()`) so call sites don't need to write `<CsrMatrix<F> as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(...)`.

This is the template established by `IsoFollowUps.md §2`; subsequent cross-crate isos copy the pattern.

### D3. Partial reverses are `TryFrom`, not panicking `From`

The `Quaternion <-> Cl(3,0)-even rotor` case has a partial reverse (multivectors with non-zero odd-grade coefficients aren't quaternions). Same for `SimplicialComplex <-> CellComplex<Simplex>` (cell complexes with non-simplex cells aren't simplicial). Both ship:
- Total forward via `From`.
- Partial reverse via `TryFrom` with a typed error variant.
- A separate Tier 2 named witness for the always-valid path (callers who have already filtered).

The Tier 2 `to_source` on the always-valid path is allowed to panic on invariant violation (since callers have already filtered); the panic message names the invariant.

The `CausalTensor <-> CsrMatrix` forward direction (`CausalTensor` -> `CsrMatrix`) has the same shape: total reverse, partial forward (only rank-2 inputs). It follows the same convention: panicking `From` plus a TODO to add `TryFrom` later if pipeline code wants the result-typed form. (Decision: ship `From` first; `TryFrom` is a follow-up if downstream code asks for it.)

### D4. Named witnesses for cross-crate isos, no `StandardIso`

`StandardIso<S, T>` only fires when bidirectional `From` exists. None of the cross-crate pairs in this change have bidirectional `From` (that's why they're mixed-tier). Each cross-crate Tier 2 iso therefore uses a dedicated named witness type (`ComplexCl01Iso`, `QuaternionRotorIso`, `CsrMatrix<F>` itself as `Self`, `PoincareIso<D>`). Same-crate isos that *do* have bidirectional `From` can use `StandardIso` — currently only `iso-multifield-tensor` is in this position, and it uses `StandardIso<CausalMultiField<T>, MultiFieldCarrier<T>>` directly.

### D5. Algebraic-marker coverage matches the algebraic structure of the type pair

For each algebraic iso, only the marker subtraits that genuinely apply are impl'd:
- `Complex <-> Cl(0,1) multivector`: both `FieldIso` and `DivisionAlgebraIso`. Complex and Cl(0,1) are both commutative associative fields.
- `Quaternion <-> Cl(3,0)-even rotor`: only `DivisionAlgebraIso`. Quaternions are non-commutative, so `FieldIso` does NOT apply. `RingIso` and `GroupIso` do apply (the additive group is abelian, multiplication is a ring even though non-commutative).
- All structural isos (multifield/tensor, tensor/sparse, simplicial/cell, lattice/dual, effect/process): no algebraic markers. The base `Iso<S, T>` is the right surface.

### D6. `PoincareIso<D>` is generic in dimension

`LatticeComplex<D>` is parameterised by a const generic `const D: usize`. The iso witness mirrors that: `PoincareIso<const D: usize>`. The body algorithm depends on `D` (k-cells swap with (D-k)-cells), so the generic parameter is load-bearing.

### D7. Propagating-effect / propagating-process consistency is a direct test, not an iso

`PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type aliases for `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`. They are literally the same concrete type. An iso with identity bodies adds no information the compiler doesn't already have. Instead, the real property worth pinning — that the two independently-written `Functor` and `Monad` impls produce the same output on the shared carrier — is captured by a one-shot `assert_eq!` test that directly compares `<EffectWitness>::fmap(val, h)` against `<ProcessWitness>::fmap(val, h)`. The test sits under `deep_causality_core/tests/iso/` for discoverability but uses no iso vocabulary.

### D8. Spec files capture requirements only; implementation lives in source

Each capability's spec file lists requirements with WHEN/THEN scenarios. The body algorithms (e.g. the Poincaré dual k-cell -> (D-k)-cell mapping) are implementation details and live in source code, not in the spec.

## Risks / Trade-offs

### R1. Quaternion rotor representation choice

Cl(3,0)-even has two common conventions for which bivector basis maps to which quaternion imaginary unit. The choice locked in here is the "east-coast" convention (`i = e₂e₃`, `j = e₃e₁`, `k = e₁e₂`), matching the existing physics-side code in `deep_causality_multivector`. The opposite convention ("west-coast") would require negating one of `x, y, z`. This is locked in the iso impl and the test fixtures; downstream code that assumes a different convention would need to compose with a sign-flip.

**Mitigation**: doc-comment the convention prominently in the iso impl; the test fixtures pin it.

### R2. Rank-2 precondition on tensor -> sparse

`From<CausalTensor<F>> for CsrMatrix<F>` panics on rank ≠ 2. This is consistent with the rest of the tensor API (which panics on shape mismatches) but breaks the round-trip helper if a caller passes a rank-3 input. The Tier 2 round-trip test uses rank-2 inputs only; the panic case is covered by a separate `#[should_panic]` test.

**Mitigation**: ship `TryFrom` in a follow-up change once a downstream consumer wants graceful failure. Documented in design D3.

### R3. Consistency test pins both propagating-effect `Functor`/`Monad` impls

If a future refactor changes one of the two `Functor` impls (on `PropagatingEffectWitness` or `PropagatingProcessWitness`) in a way that diverges, the consistency test fails. This is the *intended* behaviour — the test pins the consistency — but might surprise a developer touching one of the two impls without realising the other exists. The test failure message should name both witnesses so the relationship is discoverable.

### R4. Poincaré dual algorithm complexity

The body of `PoincareIso<D>::to_target` is the only non-trivial algorithm in this change. For D ≤ 3 (the cases consumers care about) the implementation is mechanical: iterate primal cells, compute dual cells, swap dimension. For D > 3 the algorithm is the same; performance is not a concern at this stage. If a high-D consumer ever shows up, performance is a follow-up.

### R5. `iso-multifield-tensor` round-trip uses a tuple target

`StandardIso<CausalMultiField<T>, MultiFieldCarrier<T>>` where `MultiFieldCarrier<T> = (CausalTensor<T>, Metric, [T; 3], [usize; 3])`. The tuple target is awkward for ergonomics (no method dispatch) but matches the natural shape. Alternative: introduce a dedicated `MultiFieldCarrier<T>` newtype struct. **Decision**: ship the tuple form first; rename to a newtype in a follow-up if downstream code asks for richer ergonomics.

### R6. No domain-specific test helpers

Three pairs (Poincaré dual, simplicial/cell, multifield/tensor) don't fit neatly into the existing `assert_*_law` helpers because they're not algebraic-structure isos. Each ships its own ad-hoc test: e.g. `assert_poincare_dualizes_boundary` for the lattice case. Adding a generic helper for "structural iso with caller-supplied invariant predicate" is tempting but premature; three ad-hoc tests is fine.
