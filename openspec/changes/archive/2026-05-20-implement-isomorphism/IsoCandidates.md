# Iso Candidates: Multivector, Tensor/Sparse, Topology

Survey of straightforward iso opportunities across `deep_causality_num`,
`deep_causality_multivector`, `deep_causality_tensor`,
`deep_causality_sparse`, and `deep_causality_topology`. Each candidate
is mathematically a genuine iso (or a clear forward `From` paired with
a partial reverse); none requires new heavy infrastructure beyond the
three-tier trait surface already shipped.

## Scoring legend

* **Tier**: 1 = both `From` directions; 2 = witness-typed `Iso<S, T>`;
  Mixed = Tier 1 forward + Tier 2 reverse (orphan-rule case);
  3 = `NaturalIso` at the HKT level.
* **Markers**: which algebraic-structure marker subtraits the iso
  satisfies (`GroupIso`, `RingIso`, `FieldIso`, `DivisionAlgebraIso`).
  `none` means the types don't carry the relevant algebraic structure.
* **Difficulty**: S = small (under ~50 lines), M = medium
  (~50-200 lines), L = large (requires non-trivial algorithm).
* **Value**: which real or planned consumer benefits.

---

## Summary table

| # | Pair | Crates | Tier | Markers | Bijective? | Difficulty | Value |
|---|---|---|---|---|---|---|---|
| 1 | `Complex<F>` <-> `CausalMultiVector<F>` in Cl(0,1) | num + multivector | Mixed | `FieldIso`, `DivisionAlgebraIso` | full | S | Physics; algebraic-trait completeness |
| 2 | `Quaternion<F>` <-> `CausalMultiVector<F>` as Cl(3,0)-even rotor | num + multivector | Mixed (partial reverse) | `DivisionAlgebraIso` | partial (reverse is `TryFrom`) | M | Rotor-based rigid-body work; planned per NumIso.md §7 |
| 3 | `CausalMultiField<T>` <-> `(CausalTensor<T>, Metric, [T;3], [usize;3])` | multivector | Tier 2 | none | full | S | Lets generic tensor ops work on multifield carriers |
| 4 | `CausalTensor<T>` (rank-2) <-> `CsrMatrix<T>` | tensor + sparse | Mixed | none | full (rank-2 only; rank ≠ 2 = panic/`TryFrom`) | S | Memory budget; CDL pipeline. Covered in [`IsoFollowUps.md`] |
| 5 | `Graph<T>` <-> `Hypergraph<T>` (cardinality-2 edges) | topology | Tier 1 forward + `TryFrom` reverse | none | partial (reverse needs cardinality check) | S | Ergonomics; lets graph algorithms accept either |
| 6 | `SimplicialComplex<T>` <-> `CellComplex<Simplex>` | topology | Tier 1 forward + `TryFrom` reverse | none | partial (reverse needs every-cell-is-simplex check) | S | Lets simplicial algorithms run against cell-complex inputs |
| 7 | `LatticeComplex<D>` <-> `DualLatticeComplex<D>` (Poincaré dual) | topology | Tier 2 (named witness `PoincareIso`) | none | full | M | Mathematical correctness pin; consumers of Hodge-star ops |
| 8 | `CsrMatrix<T>` <-> `Graph<T>` adjacency | sparse + topology | Mixed | none | partial (weight semantics: bool / numeric weight) | M | Graph algorithms over sparse linear-algebra backends |
| 9 | `PropagatingEffect<T>` <-> `PropagatingProcess<T, (), ()>` | core (HKT) | 3 (`NaturalIso`) | n/a | full (identity at value level) | S | CDL pipeline; effect-system reuse. Covered in [`IsoFollowUps.md`] |

[`IsoFollowUps.md`]: IsoFollowUps.md

---

## Per-candidate sketch

### 1. `Complex<F>` <-> `CausalMultiVector<F>` in Cl(0,1)

**Math.** Cl(0,1) is the Clifford algebra with one basis vector e₁ such
that e₁² = -1. As an algebra it is isomorphic to ℂ: identify `a + bi`
with `a + b·e₁`. Multiplication agrees on both sides because e₁² = i² = -1.

**Why a real iso (not just an embedding).** Both algebras are 2D over F
and the algebra-homomorphism is bijective. This is the simplest
"Clifford algebra is a generalisation of complex numbers" worked
example; pinning it as a typed iso makes the relationship visible to
the compiler.

**Shape.** Mixed-tier, because `multivector` depends on `num` (not the
other way around):
- `impl From<Complex<F>> for CausalMultiVector<F>` in multivector,
  hard-coded to the Cl(0,1) metric.
- `Iso<CausalMultiVector<F>, Complex<F>> for ComplexCl01Iso` (named
  witness) in multivector, with a precondition that the input
  multivector has the Cl(0,1) metric (the alternative — using
  `CausalMultiVector` as `Self` — would require panicking on
  wrong-metric inputs).
- Markers: `FieldIso` and `DivisionAlgebraIso` both apply, because
  `Complex<F>` and the Cl(0,1) multivector both implement the
  matching algebraic-structure traits.

### 2. `Quaternion<F>` <-> `CausalMultiVector<F>` as Cl(3,0)-even rotor

**Math.** The even subalgebra Cl(3,0)⁺ (scalars and bivectors only) is
isomorphic to ℍ. Identify `w + xi + yj + zk` with
`w + x·e₂e₃ + y·e₃e₁ + z·e₁e₂`.

**Catch.** Not every `CausalMultiVector<F>` is a rotor; only those with
zero coefficients on grades 1 and 3 in the Cl(3,0) metric. The reverse
direction is partial.

**Shape.** Mixed-tier with a named witness:
- `impl From<Quaternion<F>> for CausalMultiVector<F>` in multivector
  (always lifts cleanly into the even subalgebra).
- `TryFrom<CausalMultiVector<F>> for Quaternion<F>` for the reverse,
  returning an error when the input has non-zero odd-grade coefficients.
- Tier 2 witness `QuaternionRotorIso` implementing
  `Iso<CausalMultiVector<F>, Quaternion<F>>` only for the
  "always-valid" path (callers must have already filtered via
  `TryFrom`).
- Markers: `DivisionAlgebraIso` applies (both `Quaternion` and the
  Cl(3,0)-even sub-multivector are division algebras over the scalar
  ring); `RingIso` is the highest commutative marker possible because
  quaternions are non-commutative.

Mentioned in NumIso.md §7 as a planned target. Worth landing.

### 3. `CausalMultiField<T>` <-> `(CausalTensor<T>, Metric, [T;3], [usize;3])`

**Math.** `CausalMultiField<T>` is structurally a 4-tuple of
(tensor, metric, grid spacing, grid shape) per the source at
`deep_causality_multivector/src/types/multifield/mod.rs`. The
relationship is a trivial pack/unpack iso; no algebraic structure
involved.

**Shape.** Same-crate Tier 2 to a tuple-typed target. Useful when
generic tensor ops need to consume the underlying carrier without
copying. A `to_tensor()` / `from_tensor(tensor, metric, dx, shape)`
inherent pair on `CausalMultiField` is the natural surface.

**Note.** Could ship as Tier 1 if `(CausalTensor<T>, Metric, [T;3],
[usize;3])` is the target type — Rust allows `From` on tuple targets.
But the named-witness form via Tier 2 reads cleaner.

### 4. `CausalTensor<T>` (rank-2) <-> `CsrMatrix<T>`

Covered in detail in [`IsoFollowUps.md`]. Mixed-tier; sparse depends on
tensor. Forward `From` panics on rank ≠ 2 (or use `TryFrom`); reverse
goes through Tier 2 `Iso<CsrMatrix<F>, CausalTensor<F>>` on
`CsrMatrix<F>` as `Self`. Ergonomic alias `CsrMatrix::to_dense()`.

### 5. `Graph<T>` <-> `Hypergraph<T>`

**Math.** A graph is a hypergraph where every edge has cardinality 2.
The reverse is partial.

**Shape.**
- `impl From<Graph<T>> for Hypergraph<T>` in topology (always works).
- `impl TryFrom<Hypergraph<T>> for Graph<T>`, error variant
  `HypergraphHasNonBinaryEdge`.
- No marker subtraits because `Graph` and `Hypergraph` aren't `Group`s
  or `Ring`s; the iso is structural only.
- Round-trip helper: trivially true for the `Graph` -> `Hypergraph`
  -> `Graph` direction; the reverse direction is locked behind the
  cardinality precondition.

### 6. `SimplicialComplex<T>` <-> `CellComplex<Simplex>`

**Math.** Simplicial complexes are cell complexes whose every cell is
a `Simplex`. The reverse is partial (a `CellComplex<C>` where C is not
`Simplex` cannot become a simplicial complex).

**Shape.** Mirrors candidate 5:
- `impl From<SimplicialComplex<T>> for CellComplex<Simplex>` (always
  works).
- `impl TryFrom<CellComplex<Simplex>> for SimplicialComplex<T>`
  (validates cell structure; can also recover `T` only if it was
  carried through, which depends on the simplicial-complex API
  details).

Markers don't apply; chain-complex laws hold but `ChainComplex` is
not a marker subtrait of the iso family.

### 7. `LatticeComplex<D>` <-> `DualLatticeComplex<D>` (Poincaré dual)

**Math.** For a D-dimensional cubical (lattice) complex, the Poincaré
dual swaps k-cells with (D-k)-cells; the boundary operator on the
primal becomes the coboundary on the dual, and vice versa. This is a
genuine algebraic iso of chain complexes when expressed over a
coefficient field.

**Shape.** Tier 2 with a named witness:

```rust,ignore
pub struct PoincareIso<const D: usize>;

impl<const D: usize> Iso<LatticeComplex<D>, DualLatticeComplex<D>> for PoincareIso<D> {
    fn to_target(s: LatticeComplex<D>) -> DualLatticeComplex<D> { /* ... */ }
    fn to_source(t: DualLatticeComplex<D>) -> LatticeComplex<D> { /* ... */ }
}
```

No algebraic markers from the iso family apply directly (neither
`LatticeComplex` nor `DualLatticeComplex` is a `Group` in
`deep_causality_num`'s sense). The naturality with respect to the
boundary/coboundary operators is the law worth pinning; it doesn't
have a generic helper today but a domain-specific
`assert_poincare_iso_dualizes_boundary` test is cheap to write.

Of the topology candidates this is the most mathematically interesting
and the only one with a non-trivial algorithm in the bodies; everything
else in this section is structural.

### 8. `CsrMatrix<T>` <-> `Graph<T>` adjacency

**Math.** A finite graph with numeric (or boolean) edge weights is a
sparse square matrix; conversely a square sparse matrix indexed by
vertex pairs is a graph. Symmetric / asymmetric matrix corresponds to
undirected / directed graph.

**Catch.** Two semantic choices the user must pick:
1. Boolean adjacency (`CsrMatrix<bool>`) versus numeric weights
   (`CsrMatrix<T>`).
2. Symmetric storage versus asymmetric.

These aren't iso failures; they're API decisions for how the iso
witness is parameterised. A `Graph<T>` <-> `CsrMatrix<T>` with
symmetric storage is the natural default.

**Shape.** Mixed-tier across `sparse` and `topology`. Pick one
direction as the local `Self`; given that `sparse` and `topology` are
roughly parallel in the dep graph, either side works (depending on
which crate already depends on the other). Worth checking before
deciding.

### 9. `PropagatingEffect<T>` <-> `PropagatingProcess<T, (), ()>`

Covered in [`IsoFollowUps.md`]. Tier 3 `NaturalIso` between the two
HKT witnesses; identity at the value level because both type aliases
project to the same underlying carrier.

---

## Recommended landing order

If we're picking a small number to land:

1. **Candidate 9** (effect <-> process). Smallest diff; first real
   exercise of `NaturalIso` against codebase types.
2. **Candidate 4** (tensor <-> sparse). Canonical mixed-tier template;
   establishes the `to_dense()` ergonomics pattern.
3. **Candidate 1** (complex <-> Cl(0,1) multivector). First real
   exercise of marker-subtrait inheritance (`FieldIso` +
   `DivisionAlgebraIso` both apply).
4. **Candidate 2** (quaternion <-> Cl(3,0)-even rotor). Was already
   flagged in NumIso.md §7; mathematically meaty but with a clear
   partial-reverse story.
5. **Candidate 7** (Poincaré dual). Topology-side highlight;
   non-trivial algorithm but the math is clean.

Candidates 3, 5, 6, 8 are ergonomic wins of varying weight; can land
opportunistically when downstream code starts needing them.

## What this list does *not* include

* **Lossy "isos"** like `CausalMultiField<T>` <-> bare `CausalTensor<T>`
  (drops metric and grid metadata). These are projections, not isos.
* **Embeddings** like `Complex<F>` -> `Quaternion<F>` with `j=k=0`
  (complex is a 2D subalgebra of the 4D quaternions). Forward `From`
  is fine; reverse is partial and the algebra structure isn't
  preserved in a way that gives a marker.
* **Octonions** anywhere with anyone else. Octonions are
  non-associative; no `RingIso` can apply because the target side
  would have to be non-associative too, and the only non-associative
  algebra we ship is itself. The Cl(N,0) Clifford algebras are
  associative, so an octonion / multivector iso of multiplications
  cannot exist.
