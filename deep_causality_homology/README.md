[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# deep_causality_homology

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_homology

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_homology/latest/deep_causality_homology/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

## Summary

Chain complexes and homology for the [DeepCausality project](http://www.deepcausality.com). The crate owns the
`ChainComplex` trait, the coefficient field a homology rank is taken over, and the bit-packed mod-2 chain. It depends on
`deep_causality_linear` and `deep_causality_num`, and on nothing else.

A chain complex is a sequence of groups `C_k` with maps `∂ₖ : C_k → C_{k−1}` satisfying `∂ₖ ∘ ∂ₖ₊₁ = 0`. That definition
mentions no space, no cell, and no metric. Homology, `H_k = ker ∂ₖ / im ∂ₖ₊₁`, is defined the moment the composite
vanishes, and everything needed to compute it is linear algebra over the boundary matrices.

Geometry supplies chain complexes. It is not the only thing that does. A quantum error-correcting code is a chain
complex with no cells: `H_X` and `H_Z` are parity-check matrices whose product vanishes over 𝔽₂, and their homology is
the code's logical space. Before this crate existed, reaching 419 lines of chain-complex machinery meant depending on
`deep_causality_topology` and its 27,317 lines of geometry. That is the split this crate makes.

`deep_causality_topology` keeps the geometric half on `CellularComplex: ChainComplex` and re-exports the three names
that moved, so existing code that uses only homology compiles unchanged.

## The law, and who owes it

Every implementor owes `∂ₖ ∘ ∂ₖ₊₁ = 0`. The trait cannot check it, and nothing here assumes it silently.

The Betti number this crate computes is

```text
β_k = (n_k − rank ∂ₖ) − rank ∂ₖ₊₁
```

Three integers: one cell count and two ranks. No kernel is built and no quotient is formed. Substituting
`n_k − rank ∂ₖ` for `dim ker ∂ₖ` is rank–nullity, and reading the result as `dim H_k` needs the image to sit inside the
kernel, which is what the law says. Break the law and the formula still returns a number; it stops being a Betti number.

Both steps are machine-checked. See [Verification](#verification).

## Implementing the trait

Four methods, and two more come free:

```rust
use deep_causality_homology::ChainComplex;
use deep_causality_linear::CsrMatrix;
use std::borrow::Cow;

struct TwoPoints;

impl ChainComplex for TwoPoints {
    fn num_cells(&self, k: usize) -> usize {
        if k == 0 { 2 } else { 0 }
    }

    fn max_dim(&self) -> usize {
        0
    }

    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        // ∂₀ has no rows and one column per vertex; above the top there are no columns.
        let (rows, cols) = if k == 0 { (0, 2) } else { (2, 0) };
        Cow::Owned(CsrMatrix::from_triplets(rows, cols, &[]).unwrap())
    }

    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        Cow::Owned(self.boundary_matrix(k + 1).transpose())
    }
}

// Two points are two components, so β₀ = 2.
assert_eq!(TwoPoints.betti_number(0), 2);
```

Note the shape of `∂₀`. The degenerate grades carry the shape their dimension implies rather than an empty matrix:
`∂₀` is `(0, n₀)` and `∂_{max+1}` is `(n_max, 0)`. This keeps `cols(∂ₖ) == rows(∂ₖ₊₁)` true at every grade, so the
composite in the law is always formable. An empty matrix at either end breaks that, and the Betti computation survives
the breakage only because it saturates its subtractions.

## The coefficient field is an argument

Rank is a property of a matrix over a field, and a boundary matrix has a different rank over ℚ than over 𝔽₂. So `β_k` is
not a number until the field is named. `HomologyField` names it at the call site, and there is no other way to set it:
no default, no feature flag, no global.

The two answers genuinely differ. Real projective space has 2-torsion, and the universal coefficient theorem makes it
visible over 𝔽₂ and invisible over ℚ:

| space | β over ℚ | β over 𝔽₂ |
|---|---|---|
| torus `T²` | 1, 2, 1 | 1, 2, 1 |
| real projective plane `ℝP²` | 1, 0, 0 | 1, 1, 1 |
| Klein bottle | 1, 1, 0 | 1, 2, 1 |

```rust
use deep_causality_homology::{ChainComplex, HomologyField};
use deep_causality_homology::utils_tests::reference_spaces;

let (rp2, _, _) = reference_spaces()
    .into_iter()
    .find(|(c, _, _)| c.name() == "real_projective_plane")
    .unwrap();

assert_eq!(rp2.betti_number_over(1, HomologyField::Rational).unwrap(), 0);
assert_eq!(rp2.betti_number_over(1, HomologyField::Gf2).unwrap(), 1);
```

`Rational` runs fraction-free elimination over ℤ, which never leaves the integers and so never rounds; rank is a
fraction-field notion, so the answer over ℤ is the answer over ℚ. `Gf2` runs packed mod-2 elimination. Only the
first can fail, because its fraction-free intermediates are minors of the whole matrix and can overflow `i64`;
reporting that is what keeps a wrapped intermediate from being returned as a rank.

## Boundary matrices carry `i8`

`boundary_matrix` returns `Cow<'_, CsrMatrix<i8>>`, and that is not a storage convenience. The entries are incidence
numbers, and they lie in `{−1, 0, 1}` by construction: a face is dropped from a cell at most once, with one sign. The
type records an invariant of the boundary operator.

So the trait takes no coefficient parameter. The coefficient field belongs to the computation rather than to the
complex, and `HomologyField` carries it where the choice is actually made.

## Mod-2 chains

`Gf2Chain<W>` is an element of `C_k` over 𝔽₂, one bit per cell, packed into words of type `W`.

```rust
use deep_causality_homology::Gf2Chain;

let a = Gf2Chain::<u64>::from_support(130, 1, &[0, 5, 70]).unwrap();
let b = Gf2Chain::<u64>::from_support(130, 1, &[5, 70, 129]).unwrap();

// Addition is mod 2, so a shared support cancels.
assert_eq!(a.add(&b).unwrap().support().collect::<Vec<_>>(), vec![0, 129]);

// Intersection keeps it, and the pairing is the parity of its weight.
assert_eq!(a.intersect(&b).unwrap().support().collect::<Vec<_>>(), vec![5, 70]);
assert_eq!(a.inner(&b).unwrap().bit(), false);
```

The support is enumerable as elements, as unordered pairs, and as unordered triples. Those three shapes are what the
gate decompositions in Haruna's Table 1 range over: single-qubit factors over `supp(γ)`, two-qubit factors over its
pairs, `CCZ` factors over its triples. Enumeration walks set bits, so it costs the weight rather than the length.

### What identifies a chain group

The pair `(degree, len)`, and nothing else. `C_k = 𝔽₂^{n_k}` is fixed by the cell count, so two complexes with twelve
1-cells have the same `C₁`, and a sum of two of its elements is right whichever complex produced them.

That is why the type holds no complex handle. Every operation it offers belongs to the group rather than to a complex,
and both halves of the identity are checked in one place:

```rust
use deep_causality_homology::{Gf2Chain, HomologyError, HomologyErrorEnum};

let deg_1 = Gf2Chain::<u64>::zeros(130, 1);
let deg_2 = Gf2Chain::<u64>::zeros(130, 2);
let shorter = Gf2Chain::<u64>::zeros(129, 1);

// A degree mismatch and a length mismatch raise the same error.
for r in [deg_1.add(&deg_2), deg_1.add(&shorter)] {
    assert!(matches!(r, Err(HomologyError(HomologyErrorEnum::ChainGroupMismatch(_)))));
}
```

The complex enters with `∂`, and the compatibility check belongs there, made against the complex being applied. A handle
remembered at construction cannot do that, because it can go stale.

### Reading a basis vector

`kernel_basis_gf2` and `image_basis_gf2` in `deep_causality_linear` write their bases down **columns**. Use
`from_column`. Reading such a basis with `from_row` yields a vector whose length is the number of basis vectors rather
than the dimension they live in, and those two numbers differ whenever the matrix is not square.


## Verification

The suite is checked against published values, not against itself.

`openspec/notes/archive/homology/reference/reference.py` builds ten spaces in Python, computes their Betti numbers with
exact arithmetic, and compares them against Hatcher, *Algebraic Topology*. It imports nothing from this workspace. The
fixtures in `utils_tests` are an independent construction of the same spaces, checked against the same published values.
Two implementations agreeing with a source is a different claim from one implementation agreeing with itself.

`ℝP²` and the Klein bottle are in that set for a specific reason. Every complex this workspace shipped before the crate
existed is orientable and torsion-free, so ℚ and 𝔽₂ agreed at every grade of every fixture and the coefficient field was
never discriminated. These two separate them.

The Euler characteristic is computed twice per space, once from cell counts and once from Betti numbers. The first never
reaches the rank routine, so their agreement is evidence rather than arithmetic.

Two statements are machine-checked in Lean 4 with Mathlib, and carry Rust witnesses under
`tests/formalization_lean/`:

| id | statement |
|---|---|
| `homology.chain.dd_zero_implies_range_le_ker` | `∂ₖ ⬝ ∂ₖ₊₁ = 0 → im ∂ₖ₊₁ ⊆ ker ∂ₖ` |
| `homology.chain.betti_from_dd_zero` | `dim H_k = (n_k − rank ∂ₖ) − rank ∂ₖ₊₁`, given the chain condition |

The first exists because the second needs it. `linear.gf2.betti_from_ranks` had proved the Betti identity under the
subspace inclusion as an unproved hypothesis, so every Betti number the workspace computed rested on an assumption
written down once, as an argument to a theorem. The inclusion cannot be tested directly. The matrix identity can, and
the implication is what lets the testable statement stand in for the one the proof needs.

`LEAN_HOMOLOGY.md` in this directory carries the details, including what is left unformalized and why.

Mutation testing over `src/`: 50 mutants, 40 caught, 10 unviable, no survivors.

## Dependency

```toml
[dependencies]
deep_causality_homology = "0.1"
```

`no-std` with `alloc`:

```toml
[dependencies]
deep_causality_homology = { version = "0.1", default-features = false, features = ["no-std"] }
```

Both `Gf2Chain` and the boundary matrices allocate, so `alloc` is required in either configuration.

## Licence

MIT. See [LICENSE](https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE).
