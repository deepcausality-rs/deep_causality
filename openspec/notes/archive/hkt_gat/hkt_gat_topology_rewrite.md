<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Rewriting `CurvatureTensorWitness`, and repairing the HKT test suite

**Scope.** A first-principles rewrite of the most broken HKT implementation in
`deep_causality_topology`, and an audit of `tests/extensions/` with the replacement test design.

**Method.** The proposed design was compiled under `#![forbid(unsafe_code)]`, and each unsound call
it must reject was compiled separately to confirm it now fails. The test harness in §7 was run
against topology as it stands. `hkt_gat_topology.md` is the assessment this follows from.

**Not in scope.** No repository code was changed to produce this note.

---

## 1. Which one is most broken, and why it is that one

Two candidates. `GaugeFieldWitness`'s HKT3 layer is more *thoroughly* fictional: `pure` discards its
argument, `ibind` and `merge` never call their function, and all of it operates on a shadow struct.
`CurvatureTensorWitness` is more *dangerous*: it is unsound, published, and reachable from safe
downstream code.

Soundness wins the ranking, and the choice is also settled by cost. The gauge stubs have **zero**
callers outside topology's own tests, so retiring them is bookkeeping rather than design.
`CurvatureTensorWitness` has a real production caller and needs an actual replacement. It is also the
sole justification for the crate forfeiting `unsafe_code = "forbid"`.

## 2. The diagnosis, from first principles

Start with the mathematics. The Riemann curvature operator is a multilinear map

```
R : V ⊗ V ⊗ V → V
```

over **one** vector space `V`, the tangent space. Its three inputs and its output are the same kind
of object; that is what makes `R(u,v)w` meaningful and what makes antisymmetry `R(u,v)w = -R(v,u)w`
statable at all.

Now look at what the code declares:

```rust
pub struct CurvatureTensor<T, A, B, C, D> {
    components: CausalTensor<T>,
    metric: Metric,
    symmetry: CurvatureSymmetry,
    dim: usize,
    _phantom: PhantomData<(A, B, C, D)>,     // A, B, C, D carry no data
}
```

`A`, `B`, `C` and `D` are pure `PhantomData`, and the crate's own alias instantiates all four
identically:

```rust
pub type CurvatureTensorVector<T> =
    CurvatureTensor<T, TensorVector<T>, TensorVector<T>, TensorVector<T>, TensorVector<T>>;
```

**The four parameters exist for one reason: to fit `HKT4Unbound::Type<A, B, C, D>`.** A data type
grew four phantom parameters so it could be viewed through an arity-4 abstraction, `RiemannMap` then
declared `curvature<A, B, C, D>` generic in all four, and the implementation had to use `unsafe` to
undo a genericity nobody wanted:

```rust
let u_ptr = &u as *const A as *const TensorVector<T>;
```

That is the whole causal chain. The abstraction imposed a shape the mathematics does not have, and
`unsafe` was the cost of pretending otherwise. Every call site in the workspace passes
`TensorVector<T>`; the production one in `deep_causality_physics` even comments "Use TensorVector for
HKT safety contract", which is a human upholding by hand what the compiler was prevented from
checking.

**The principle:** *the domain of an operation belongs to the witness, not to the method's type
parameters.* This is the same move that `ManifoldWitness<C>`, `DeRhamSharpIso<D, R>` and CFD's
`StudyEffectWitness<E, WLog>` already make, and all three are sound and lawful.

## 3. The rewrite

Three changes, in `deep_causality_haft` and `deep_causality_topology`.

**`RiemannMap` names its spaces as associated types.**

```rust
/// A rank-4 multilinear map. Its domain is one vector space, so the space is a
/// property of the implementer rather than four independent method parameters.
pub trait RiemannMap {
    type Tensor;
    type Vector;

    fn curvature(t: &Self::Tensor, u: &Self::Vector, v: &Self::Vector, w: &Self::Vector)
        -> Self::Vector;

    fn scatter(t: &Self::Tensor, a: &Self::Vector, b: &Self::Vector)
        -> (Self::Vector, Self::Vector);
}
```

The trait no longer takes an `HKT4Unbound` parameter, because there is no arity-4 type constructor
here to witness.

**`CurvatureTensor` sheds the four phantom parameters.**

```rust
pub struct CurvatureTensor<T> { components: CausalTensor<T>, metric: Metric, /* … */ }
```

`CurvatureTensorVector<T>` becomes an alias for `CurvatureTensor<T>`, kept for one release so the
physics call site does not have to move in the same commit.

**The impl loses its `unsafe` and its safety contract.**

```rust
impl<T: Field + Float + Copy + PartialOrd> RiemannMap for CurvatureTensorWitness<T> {
    type Tensor = CurvatureTensor<T>;
    type Vector = TensorVector<T>;

    fn curvature(t: &CurvatureTensor<T>, u: &TensorVector<T>, v: &TensorVector<T>,
                 w: &TensorVector<T>) -> TensorVector<T> {
        Self::geodesic_deviation_impl(t, u, v, w)   // the existing private body, unchanged
    }
}
```

The private `geodesic_deviation_impl` and `scatter_impl` already have exactly this signature. The
rewrite deletes the generic wrapper that existed only to cast down to them.

**Measured.** The design compiles under `#![forbid(unsafe_code)]`, and each call that is undefined
behaviour today becomes a compile error:

| Call | Today | After |
|---|---|---|
| `curvature(&ct, &vec![1.0], …)` with `Vec<f64>` | compiles, UB | **E0308** |
| `curvature` with a zero-sized type | compiles, UB | **E0308** |
| `TensorVector<f32>` against an `f64` witness | compiles, UB | **E0308** |
| `curvature(&ct, &u, &v, &w)` with `TensorVector<T>` | compiles | compiles |

## 4. What it costs

| Consumer | Change |
|---|---|
| `deep_causality_physics/src/theories/general_relativity/gr_ops_impl.rs:142` | pass by reference; drop the five-parameter type annotation |
| `deep_causality_topology/tests/extensions/hkt_curvature_tests.rs` | three call sites lose their turbofish |
| `deep_causality_haft/tests/algebra/riemann_map_tests.rs` | one test witness, which passes `(1.0, 2, 3)` as three different types and is a test of the fiction itself |
| `deep_causality_cfd` | **none.** CFD uses its own witnesses, not topology's |
| examples | **none.** No example names `CurvatureTensorWitness` or `RiemannMap` |

Four files. `RiemannMap` has one real implementer in the workspace, so the haft change is cheap. The
`HKT4Unbound` impl on `CurvatureTensorWitness` goes away, and with it the workspace's only use of
arity-4 HKT.

## 5. What it buys

Three silent-UB paths become compile errors. The crate can restore `unsafe_code = "forbid"` once the
`Send`/`Sync` pair in `lattice_cell.rs` gets a scoped `#[allow]`. A safety contract enforced by a
docstring becomes one enforced by the type checker. Four phantom parameters, a `HKT4Unbound` impl and
a generic wrapper disappear, and the physics call site stops hand-upholding an invariant.

Against that: `RiemannMap` can no longer express a curvature operator whose three inputs are three
different types. Nothing in the workspace wants that, and the multilinear map it models does not have
it.

## 6. The test suite: what is wrong with it

`tests/extensions/`, 13 files, 2,250 lines, **115 tests**.

**Law coverage is inverted relative to risk.** Fourteen law tests exist, and all fourteen sit on
witnesses that are already correct:

| Witness | Tests | Law tests | Status |
|---|---|---|---|
| `Graph`, `Hypergraph`, `MixedGraph`, `PointCloud`, `Topology` | 28 | **13** | correct, well covered |
| `DeRhamSharpIso` | 1 | 1 | correct |
| **`Manifold`** — the crate's only `Monad` and `Applicative` | 8 | **0** | **right identity broken** |
| **`ChainWitness` + `StokesAdjunction`** — both `Adjunction` impls | 38 | **0** | untested as adjunctions |
| **`CurvatureTensorWitness`** | 7 | **0** | unsound |
| **`GaugeFieldWitness`** | 25 | **0** | three law-free stubs |

The comonad laws are tested six times over on the witnesses least likely to be wrong. The monad,
applicative and adjunction laws are tested nowhere. The comonad tests exist because someone found the
cursor-reset bug in `extend`; the identical bug in `bind` has no test, because nobody wrote monad law
tests.

**Five tests assert the broken implementation as the specification.** In `hkt_gauge_field_tests.rs`:

```rust
let result: GaugeFieldHKT<(), (), i32, f64> = GaugeFieldWitness::pure(42);
assert!(!result.has_data());                       // asserts that pure(42) loses 42

let merged = GaugeFieldWitness::merge(pa, pb, |a: f64, b: f64| a + b);
assert_eq!(conn[0], 2.0);                          // (1+3)/2. Passes `a + b`, asserts the average

// "ibind propagates data unchanged (placeholder impl)"
assert_eq!(result.connection_data().unwrap(), &[1.0, 2.0]);   // asserts f was ignored
```

Each passes a function and then asserts that the function was not applied. These tests do not protect
the behaviour; they pin the defect in place, and they are why the stubs survived review.

**The curvature file tests something else.** Four of its seven tests exercise `TensorVector`
constructors and `From` impls rather than curvature. Of the three that remain:

- `test_geodesic_deviation_flat` uses `CurvatureTensor::flat(4)`, whose components are all zero, and
  asserts the result is zero. **Any implementation returning zeros passes**, including one that
  ignores `u`, `v` and `w`. The oracle cannot distinguish a correct contraction from a stub.
- `test_curved_tensor_contraction` sets exactly one component, `R^0_010 = 1`, and passes basis
  vectors with `u == w`. With `u` and `w` equal, transposing those indices changes nothing, so a
  whole class of index bugs is invisible.
- `test_scatter_vectors` is circular. Its comment derives the expected value from what the
  implementation does ("out1[c] += 1.0 * 0.5 * (dim=2 for d) = 1.0") and its own header calls the
  code "placeholder logic".

**Adjunctions are never tested as adjunctions.** Thirty-eight tests across two impls check `unit`,
`counit`, `left_adjunct` and `right_adjunct` one at a time on one fixture each. The defining property
is that the adjuncts are mutually inverse and that the triangle identities hold. Neither is asserted
anywhere.

**Everything runs on one fixture.** `create_line_manifold()`, `simple_complex()`,
`create_test_manifold()`, `create_simple_complex()`: each file has a single hand-built input, usually
a 2-vertex line or a 3-element vector, and a cursor of 0. The cursor bug lives at cursor 1 and 2, and
no manifold test ever sets a non-zero cursor.

## 7. The replacement test design, and what it caught

Four changes: assert laws rather than outputs, sweep inputs rather than fixture them, use metamorphic
properties where a pointwise oracle is degenerate, and self-check the generators.

Run against topology as it stands today:

```
ManifoldWitness (8 tests, 0 law tests today):
  [PASS] Functor identity: fmap(id, m) == m                12/12 cases
  [PASS] Functor composition: fmap(g)∘fmap(f) == fmap(g∘f) 24/24 cases
  [PASS] Monad left identity: bind(pure(a), f) == f(a)     24/24 cases
  [FAIL] Monad right identity: bind(m, pure) == m           1/3 cases
         first counterexample: cursor 1: bind(m,pure).cursor = 0, m.cursor = 1
  [PASS] Applicative identity: apply(pure(id), v) == v     12/12 cases
  [PASS] CoMonad left identity: extend(w, extract) == w      3/3 cases

CurvatureTensorWitness (7 tests today, 4 of which test TensorVector instead):
  [PASS] Metamorphic: R(u,v)w == -R(v,u)w                  40/40 cases
```

**One new law test found the cursor bug on its first run**, where eight existing tests had not. It
reports the counterexample and the cursor that produced it, because every case is generated from a
seeded LCG rather than hand-written.

**The antisymmetry result is worth reading carefully, and it is a lesson about this kind of test.**
My first version of that check generated random tensor components and asserted
`R(u,v)w == -R(v,u)w`. It failed 40 of 40, which looked like a serious find. It was not: a random
component array is not antisymmetric in its own `(a, b)` indices, so the property does not follow and
the oracle was simply wrong. Corrected to generate components that *are* antisymmetric in `(a, b)`,
and with a self-check asserting the generator produced what it claimed, the property passes 40 of 40.

The contraction arithmetic is correct. Its only defect is the type dispatch in §2. A property test
with a wrong oracle manufactures false findings as readily as a fixture test hides real ones, so the
generator self-check is not optional decoration.

## 8. What to land

1. **Delete the five tests that assert stub behaviour**, together with the stubs they pin
   (`hkt_gat_topology.md` §10 step 4). A test asserting that `pure` discards its argument is worse
   than no test.
2. **Add monad, applicative and adjunction law tests** for `ManifoldWitness`, `ChainWitness` and
   `StokesAdjunction`, swept over cursors, widths and grades. The manifold set fails until the
   `cursor: 0` line in `bind` is fixed, which is the point.
3. **Replace the curvature tests.** Move the four `TensorVector` tests to their own file, drop the
   degenerate `flat` oracle in favour of antisymmetry and linearity in each slot, and generate the
   tensor rather than placing one component by hand. Use `u ≠ w` so index transposition is visible.
4. **Give every fixture a generator.** One seeded builder per container, sweeping size, cursor and
   grade, replacing `create_line_manifold()` and its four siblings.
5. **Add the negative compile tests** from §3 as `compile_fail` doctests once the rewrite lands, so
   the three UB paths stay closed.
6. **Self-check every generator** that encodes a mathematical precondition, as §7 shows.

Steps 1 and 2 are independent of the rewrite and can land first. Step 2 will fail CI until the
one-line cursor fix goes in, so they belong in the same commit.

---

## 12. Implemented

Applied 2026-08-30, tests first. Each law test was written and run against the unchanged
implementation, and the implementation was changed only where a law failed.

**What the new tests found.**

| Test | Result on the unchanged code |
|---|---|
| `monad_right_identity` on `ManifoldWitness` | **failed** at cursor 1 and 2 of 3; `bind(m, pure).cursor = 0` |
| The other five manifold laws | passed |
| Four `ChainWitness` adjunction laws | passed; the adjunction was already lawful |
| Six curvature properties | passed; the contraction arithmetic was already correct |

The manifold failure is the bug that eight fixture tests had missed. Everything else the new tests
established was already right, which is worth stating: the suite's problem was that it could not
tell the two cases apart.

**Implementation changes.**

1. `ManifoldWitness::bind` preserves the focus (`cursor: m_a.cursor`) instead of resetting it to 0.
   One line. The six `extend` implementations already carried the reasoning in a comment.
2. `RiemannMap` in `deep_causality_haft` names its spaces as associated types, `Tensor` and
   `Vector`, and no longer takes an `HKT4Unbound` parameter.
3. `CurvatureTensor<T, A, B, C, D>` became `CurvatureTensor<T>`. The four parameters were
   `PhantomData`; `deep_causality_physics` was instantiating all four as `()`. `cast`, which existed
   only to re-tag them, went with them, and `CurvatureTensorVector<T>` is now an alias.
4. Both `unsafe` blocks in `hkt_curvature.rs` are gone, and the two private `*_impl` functions with
   them: they existed only as the safe landing point for the cast, so with the cast removed the
   indirection had no callers and no purpose. Their bodies are now the trait methods.
5. The two `unsafe impl Send/Sync for LatticeCell<D>` were removed as unnecessary: the struct is
   `{ [usize; D], u32 }`, so both are derived. **`deep_causality_topology` now compiles under the
   workspace `unsafe_code = "forbid"` policy with no exemption**, and its `[lints]` section is back
   to `workspace = true`.
6. `GaugeFieldHKT`, `GaugeFieldData` and the `HKT3Unbound`, `MonoidalMerge` and `ParametricMonad`
   implementations were removed, along with a vacuous `Satisfies<NoConstraint>` bound on the
   surviving inherent impl. `MonoidalMerge` and `ParametricMonad` keep lawful witnesses in haft's
   own tests, so both traits stay exercised; topology held their only production implementers and
   none of the three was lawful.

**Test changes.**

- `src/utils_tests/hkt_law_utils.rs`: seeded generators seeding manifolds at every legal cursor,
  chains across grades and sparsity patterns, and graphs at every width, plus tolerance helpers.
- `hkt_manifold_law_tests.rs`: nine laws, generated (functor identity and composition, monad left
  identity, right identity and associativity, applicative identity and homomorphism, comonad left
  identity, right identity and associativity).
- `hkt_adjunction_law_tests.rs`: four laws for `ChainWitness`, including both adjunct round-trips,
  which no existing test asserted.
- `hkt_curvature_tests.rs`: rewritten as six properties over generated antisymmetric tensors with
  `u`, `v` and `w` distinct. The degenerate `flat` oracle is now paired with a curved case, so an
  implementation returning zeros fails.
- `tensor_vector_tests.rs`: the four `TensorVector` tests that were inflating the curvature file.
- Three `compile_fail` doctests on `hkt_curvature`, one per call that used to be undefined
  behaviour, plus one that must keep compiling.
- `deep_causality_haft/tests/algebra/riemann_map_tests.rs`: the witness that returned `tensor.3`
  and panicked in `scatter` became a real contraction with antisymmetry, vanishing, homogeneity and
  additivity tests.
- Eight gauge tests removed with the stubs, five of which asserted the defect as the specification.

**Two generator bugs, both mine, both worth recording.**

The first version of the antisymmetry check drew tensor components from the generator that
deliberately injects `1e12` and subnormals. Summing terms across twenty orders of magnitude lost
about `1e12 * 2^-52 ≈ 2e-4` to cancellation, and `R(u,u)w` came back as `-4e-4` instead of zero.
That reads as an implementation bug and is not one. Numeric laws now draw from `well_scaled`, which
omits the extremes; structural laws keep them, because `fmap(id) == m` holds bit-for-bit whatever
the payload is.

The second: `chain_cases` drew weights that could be exactly `0.0`, and a CSR matrix drops explicit
zeros, so one generated chain came back empty and `right_adjunct` panicked on it as documented. The
generator now draws away from zero and asserts that no entry was lost, so the empty-chain case
appears only in the tests that mean to exercise it.

Both were caught because the failure was reproducible from its seed. A property test is only as
sound as the precondition its generator establishes, which is why every generator here self-checks.

**Verification.** `cargo test -p deep_causality_topology`: 1585 pass. `cargo test -p
deep_causality_haft`: doctests and the rewritten Riemann tests pass. `cargo clippy --all-targets
--all-features` clean on topology, haft and physics, with no new `allow`.
