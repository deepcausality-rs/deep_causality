<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Scoping: `Chain<R, G>` and `Topology<R, G>`

**Status: implemented.** The recommendation below was taken. The tree carries `Chain<R, G>`,
`Topology<R, G>`, `ChainWitness<R>`, `TopologyWitness<R>`, `StokesContext<R>` and the
`UniformChain<T>` / `UniformTopology<T>` aliases, and item 10 of `hkt_gat_topology.md` is marked
done. The recommendation sections that follow are kept in their original tense as the record of the
argument that was made at the time; read them as history, not as work outstanding.

**Purpose.** Size the fix for item 10 of `hkt_gat_topology.md`: `fmap` drops the Hodge ⋆ operators,
so the functor identity law fails for any complex carrying geometry. The cause is a conflated type
parameter, not the `fmap` bodies.

**Method.** Counts from the tree as it stands. Every claim about which operation needs which bound
was read off the impl blocks, not inferred.

---

## 1. The defect, stated precisely

```rust
pub struct Chain<T> {
    complex: Arc<SimplicialComplex<T>>,   // T here is the metric precision
    grade: usize,
    weights: CsrMatrix<T>,                // T here is the coefficient group
}
```

Mathematically these are independent. A simplicial chain group is `C_k(K; G)`: formal sums of
`k`-simplices with coefficients in an abelian group `G`, over a complex `K`. The functor is
`C_k(K; −)`, acting in the coefficient slot with `K` fixed. The Hodge ⋆ is determined by the metric
on `K` and has nothing to do with `G`.

The code agrees. In `SimplicialComplex<T>`, `T` occurs in exactly two fields, both geometric:

```rust
hodge_star_operators: OnceLock<Vec<CsrMatrix<T>>>,
geometric_data:       Option<GeometricData<T>>,   // coords
```

The combinatorics (`skeletons`, `boundary_operators`, `coboundary_operators`) are `CsrMatrix<i8>`,
payload-independent and correct.

So `fmap` maps the coefficients, which forces it to change the complex's precision parameter as
well. It cannot do that meaningfully, so it rebuilds the complex with `..Default::default()` and the
geometry is lost. **The geometry drop is a symptom; the conflation is the defect.**

`Topology<T>` has the identical shape. `DifferentialForm<T>` does **not**: it holds only a degree, a
dimension and a coefficient tensor, so its `T` is already unambiguously the coefficient.

## 2. This is the design that was already intended

Three places in the workspace describe the separation, and one implements it.

- `Manifold<K, F>` / `SimplicialManifold<R, F>` implements it, with the doc "R is the precision of
  the simplicial complex (and its `ReggeGeometry<R>` metric); F is the field-data type on simplices
  **and may differ from R**".
- `deep_causality_homology/src/lib.rs` describes it: "`Chain<T>` where `T: AbelianGroup` or
  `SimplicialComplex<T>` where `T: RealField`". It presents the two as instances of one pattern; in
  fact `Chain` uses a single parameter for both roles, which is what this note fixes.
- `deep_causality_linear/src/types/csr_matrix/compat.rs` refers twice to "`deep_causality_topology`'s
  `Chain<T, S>`", a two-parameter chain that does not exist yet.

`ManifoldWitness` is therefore the existence proof. Measured on a complex built with real
coordinates:

```
Manifold before fmap: hodge ok = true
Manifold after  fmap: hodge ok = true
Manifold fmap(id, m) == m : true
```

`Chain` and `Topology` fail the identical test. Same crate, same operation, same complex; the only
difference is whether precision and coefficient share a parameter.

## 3. The finding that makes this cheap

**No operation on `Chain` or `Topology` consumes the metric.** `hodge_star_operators()` has four
production call sites workspace-wide, none of them in `Chain`, `Topology` or `cup_product`:

| Call site | Crate |
|---|---|
| `kernels/mhd/ideal.rs:165` | `deep_causality_physics` |
| `kernels/mhd/grmhd.rs:66` | `deep_causality_physics` |
| `types/manifold/constructors/constructors_impl.rs:102` | `deep_causality_topology` |
| `types/regge_geometry/has_hodge_star.rs:39` | `deep_causality_topology` |

An earlier revision of this note said "exactly three places" and listed
`point_cloud/ops/op_triangulate_delaunay.rs` among them. That was wrong twice: the point-cloud file
mentions the accessor only in doc comments and never calls it, and the count missed the two MHD
kernels entirely, which sit in a different crate. The conclusion is unchanged — the split stays
cheap because none of the four touches `Chain` or `Topology` — but the blast radius crosses a crate
boundary, which the original count hid.

Every bound on every `Chain` impl is a coefficient bound:

| Operation | Current bound | Belongs to |
|---|---|---|
| `Add`, `Sub`, `Neg` (6 impls) | `T: AbelianGroup + Copy + PartialEq + Default + Neg` | `G` |
| `Mul<S>`, `scale<S>` | `T: Module<S>`, `S: Ring` | `G`, **already separated** |
| `Display` | `T: Display + Clone` | `G` |
| `SimplicialComplex::boundary` | `T: Copy + Num + Default + From<i8> + Debug` | `G` (`From<i8>` for incidence signs) |
| `Topology::cup_product` | `T: Field + Copy + Clone + Zero + Mul + Debug` | `G` |
| `new`, `complex`, `grade`, `weights` | none | — |

`Chain::scale<S>` is the tell: it already takes the scalar ring as a **separate parameter** with
`T: Module<S>`. The coefficient/scalar distinction is half-built already.

So the two parameters never need to interact. Splitting them requires no new trait, no module
structure relating `R` to `G`, and no change to any operation body. The `Module<R>` and
`AbelianGroup` traits the tower already provides cover everything.

## 4. Proposed shape

No struct-level bounds, so the types stay witnessable (`hkt_gat.md` §7); bounds go on the impls,
which is where they already are.

```rust
pub struct Chain<R, G> {
    complex: Arc<SimplicialComplex<R>>,   // R: RealField at the impls that need it
    grade: usize,
    weights: CsrMatrix<G>,                // G: AbelianGroup at the impls that need it
}

pub struct Topology<R, G> {
    complex: Arc<SimplicialComplex<R>>,
    grade: usize,
    data: CausalTensor<G>,
    cursor: usize,
}

pub struct ChainWitness<R>(PhantomData<R>);
impl<R> HKT for ChainWitness<R> { type Type<G> = Chain<R, G>; }

pub struct TopologyWitness<R>(PhantomData<R>);
impl<R> HKT for TopologyWitness<R> { type Type<G> = Topology<R, G>; }
```

Structure first, payload second, matching `Manifold<K, F>`. The witnesses gain the precision
parameter exactly as `ManifoldWitness<C>` carries it.

`fmap` then clones the complex instead of rebuilding it, which is what makes the identity law hold:

```rust
Chain { complex: fa.complex.clone(), grade: fa.grade, weights: CsrMatrixWitness::fmap(fa.weights, f) }
```

Two incidental cleanups fall out. `StokesContext<T>` becomes `StokesContext<R>`, its `T` having
always been the precision. And the `ChainWitness` adjunction impl currently reads
`Adjunction<ChainWitness, ChainWitness, (Arc<SimplicialComplex<T>>, usize)>` with a `T` unrelated to
the `A` and `B` it maps; under the split that stray parameter becomes the honest `R`.

## 5. Blast radius

138 raw `Chain<` hits and 22 `Topology<` hits; 31 of the `Chain` hits are vendored crates
(`bytes`, `tokio`, `futures-util`, `hashbrown`) and irrelevant. Real total: **107 + 22**.

**Source, 16 files, ~75 sites**

| File | Sites |
|---|---|
| `extensions/hkt_simplicial_complex/mod.rs` | 17 |
| `types/chain/arithmetic/mod.rs` | 16 |
| `extensions/hkt_gauge/hkt_adjunction_stokes.rs` | 16 |
| `extensions/hkt_topology/mod.rs` | 6 |
| `types/chain/mod.rs` | 4 |
| `types/chain/algebra/group.rs` | 3 |
| `types/simplicial_complex/ops/ops_boundary.rs` | 2 |
| `types/topology/` (7 files: mod, api, clone, constructors, display, getters, ops/cup_product) | 7 |
| `types/chain/{algebra/module.rs, display/mod.rs}`, `types/cup_product/mod.rs`, `utils_tests/hkt_law_utils.rs` | 4 |

**Tests, 6 files, 47 sites.** `hkt_simplicial_complex_tests.rs` 28, `adjunction_stokes_tests.rs` 6,
`hkt_topology_tests.rs` 5, `hkt_adjunction_law_tests.rs` 5,
`cup_product/implementation_agreement_tests.rs` 2, `types/topology/topology_tests.rs` 1. Almost all
are type annotations: `Chain<f64>` becomes `Chain<f64, f64>`.

**Downstream: one file.** `examples/mathematics_examples/topology/chain_algebra.rs`, 1 site.

**Not affected.** `deep_causality_physics`, `deep_causality_cfd`, and every other example.
`deep_causality_homology` is **below** topology in the graph (it depends only on `num` and `linear`),
so nothing here can reach it; its `Gf2Chain<W>` is a different type over a fixed `𝔽₂` coefficient,
documented as such. Its two `Chain<T>` mentions are prose, and one of them wants the correction in §2.
`deep_causality_linear`'s two mentions are prose that already says `Chain<T, S>`.

## 6. Cost, risk and the payoff

**Mechanical.** ~124 sites, of which 47 are test annotations and ~75 are source. No operation body
changes except the four `fmap`-family bodies that currently rebuild the complex, which get shorter.

**Risk: low, and bounded by the type checker.** Every site that conflates the two parameters becomes
a compile error naming itself. The one judgement call per site is which of `R` or `G` a given `T`
meant, and §3 answers it: if the bound is algebraic it is `G`; if the value came from the complex it
is `R`. There is no silent-wrong-answer failure mode.

**A convenience alias absorbs most call sites.** Every current use has `R == G`, so
`pub type UniformChain<T> = Chain<T, T>;` (and the same for `Topology`) keeps the common spelling
available and shrinks the test diff.

**Breaking.** `Chain<T>` and `Topology<T>` gain a parameter, and the two witnesses gain one. Given
the downstream surface is a single example file, the migration note is one line.

**Payoff.** The functor identity law holds for `Chain` and `Topology`, provable with the generated
law harness already in `tests/extensions/`. The Hodge ⋆ survives `fmap`, so a mapped chain remains
usable in DEC pipelines rather than being silently degraded. `hkt_gat_topology.md` item 10 closes,
and the three-way disagreement in §2 between the code, the homology docs and the linear docs resolves
in favour of what all three intended.

**What it does not do.** It does not touch `hkt_gat.md` item 9 (the GAT where-clause) or item 5
(population B). Those stay open and independent.

## 7. Recommendation

Take it. The mathematics is unambiguous, the operation-level analysis shows the parameters never
interact, the crate already implements the target design in `Manifold`, and two other crates document
it as the intended shape. The cost is an afternoon of type annotations that the compiler drives, and
the alternative is keeping a `Functor` instance that is measurably not one.

Suggested order: split `Chain` first (it carries the adjunctions and the larger test surface), verify
the law harness goes green on it, then `Topology`, which is a quarter the size and has no adjunction.
