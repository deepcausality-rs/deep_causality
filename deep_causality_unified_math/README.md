<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified Math

Seventeen crates. One stack, seven tiers tall, rooted in three zero-dependency leaves and
terminating in `deep_causality_topology`. No crate in it contains a line of `unsafe`.


Unified math grew from a small idea. Tensors and multivectors needed to compose, so they were given a shared higher-kinded interface. That interface turned out to want an algebra tower under it, the tower wanted numeric traits under that, and the composition kept finding new things it could reach: geometric algebra, discrete exterior calculus, chain complexes, spectral methods, uncertainty. 

## The stack

Every crate depends only on crates below it. The graph is drawn as its transitive reduction, so
`tensor -> num` is omitted where `tensor -> linear -> haft -> algebra -> num` already implies it.


```
tier 6   topology
tier 5   multivector
tier 4   calculus   fft   homology   tensor
tier 3   linear   num_complex   num_dual   uncertain
tier 2   haft   num_rational   rand
tier 1   algebra
tier 0   num   metric   ast
```

![Dependency graph of the deep_causality mathematics crates: seven tiers, from the roots num and
metric at tier 0 up through algebra and haft, then linear algebra and the number types, then
tensors, to topology at tier 6. The longest chain is highlighted.](graph.png)

The longest chain is seven crates:

```
num -> algebra -> haft -> linear -> tensor -> multivector -> topology
```

Every one of the thirteen crates above tier 1 reaches `algebra`, 
so a change to the tower recompiles the whole stack. `num` and `algebra` are also the two
crates that can never adopt `haft`, because `haft` depends on both.

## The seventeen

| Crate | Tier | What it holds |
|---|---|---|
| `deep_causality_num` | 0 | Numeric traits: casts, identity, float and integer predicates, and the lifts that make precision a parameter. The bottom of the workspace |
| `deep_causality_metric` | 0 | Metric signatures `Cl(p, q, r)` and Lorentzian sign conventions, east coast and west coast |
| `deep_causality_ast` | 0 | The expression AST the tensor and uncertainty layers build on |
| `deep_causality_algebra` | 1 | Groups, rings, fields, algebras, and isomorphism markers |
| `deep_causality_haft` | 2 | Applied category theory: HKT, functor, applicative, monad, foldable, arrow, and a type-encoded effect system |
| `deep_causality_num_complex` | 2 | Complex, quaternion and octonion number types |
| `deep_causality_num_dual` | 2 | Dual numbers, forward-mode automatic differentiation |
| `deep_causality_num_rational` | 2 | Exact rationals over the integers |
| `deep_causality_rand` | 2 | Random number generators and statistical distributions |
| `deep_causality_calculus` | 3 | Arrow-native differentiation and integration operators |
| `deep_causality_fft` | 3 | Fast Fourier transform: FFT, rFFT, N-dimensional |
| `deep_causality_linear` | 3 | Sparse CSR, dense and bit-packed 𝔽₂ matrices and vectors; eliminations, decompositions, conjugate gradient, exact integer path |
| `deep_causality_uncertain` | 3 | A first-order type for uncertain programming |
| `deep_causality_homology` | 4 | Chain complexes, boundary operators and homology over a chosen coefficient field. No geometry |
| `deep_causality_tensor` | 4 | N-index tensors, broadcasting, Einstein summation, the tensor-train stack |
| `deep_causality_multivector` | 5 | Multivectors for geometric algebra. The only crate joining both geometric roots |
| `deep_causality_topology` | 6 | Cell complexes, manifolds, discrete exterior calculus, gauge fields, differential geometry |

Nothing here has a **required** external dependency. Four crates reach crates.io at all: `num` for
`libm`, `rand` for `getrandom`, and `fft` and `topology` for `rayon`. All four sit behind features
that are off by default.

## How they compose

Composition runs through `deep_causality_haft`. A crate that owns a container generic in its element
declares a *witness* type, binds `type Type<T>` to that container, and implements the categorical
traits against the witness. Two mechanisms then fall out, and both are load-bearing in
`examples/mathematics_examples/composable_multi_math/`.

**Nesting.** A witness accepts any element type, including one another crate owns. 
A tensor of multivectors is an ordinary `CausalTensor<CausalMultiVector<T>>`, and
one `fmap` rotates every cell of a vector field by a single Clifford rotor:

```rust
let rotated: CausalTensor<CausalMultiVector<FloatType>> =
    CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev));
```

**Closure reach.** The comonadic `extend` hands a cursor to a closure, and the closure may call into
any crate it likes. `triple_hkt_stress_field` runs a six-step linear-elastic pipeline over a
tetrahedral mesh inside a single `ManifoldWitness::extend`, with topology supplying the walk, tensor
holding the strain, and multivector applying the material rotor.

Run them:

```bash
cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field_examples
cargo run -p mathematics_examples --example triple_hkt_stress_field_examples
cargo run -p mathematics_examples --example capstone_spinor_minkowski_examples
```

The capstone parallel-transports a unit timelike spinor along a discretized Minkowski worldline in
`Cl(3,1)`. Four crates participate and the final drift against the closed-form `(cosh θ, sinh θ)` is
about `1.7e-31` at `Float106`.

| Trait | Implementers outside `haft` |
|---|---|
| `HKT`, `Functor`, `Foldable` | `linear`, `tensor`, `multivector`, `topology`, `num_complex`, `num_dual` |
| `Applicative`, `Pure`, `CoMonad` | `linear`, `tensor`, `multivector`, `topology` |
| `Monad` | `linear`, `tensor`, `topology` |
| `Semigroupal`, `MonoidalApplicative` | `num_complex`, `num_dual`, `tensor` |
| `Adjunction` | `topology` |
| `Arrow` | `calculus`, `tensor` |
| `Traversable`, `NaturalTransformation`, `Category`, `Kleisli`, `Bifunctor`, `Profunctor` | none |

That table is the work list. `openspec/notes/archive/unified_math/unified_math_gaps.md` carries the full
analysis: which absences are real gaps and which are correct (a `Ratio<A> -> Ratio<B>` under an
arbitrary `f` breaks coprimality, so `num_rational` is right to have none), what each costs, and a
ranking from mechanical to design fork. Two findings there were measured by running code rather than
read off the source: `CausalTensorWitness` and `CausalMultiVectorWitness` both violated monad right
identity. Both are now resolved. `CausalMultiVectorWitness` gave up `Monad`, because no metric choice
satisfies both identity laws, and `CausalTensorWitness::bind` keeps the input's shape when the map is
shape preserving, so a `[2, 3]` no longer comes back `[6]`.


## Precision as a parameter

Every crate above `num` is generic in its scalar. The tower's bounds, `Real`, `RealField` and
`Scalar`, are what a tensor, a manifold or a multivector asks of its element, and the three shipped
real fields `f32`, `f64` and `Float106` all satisfy them. A program therefore names its working type
once and writes everything else against the name:

```rust
type FloatType = f64;
```

Switch the alias and the arithmetic of the whole program changes precision. Nothing else moves,
provided nothing else was ever spelled `f64`.

Three kinds of number cross the boundary of that type. A configuration literal sits in the source as
`f64`, the widest form a source file can hold. A count, such as a shot count or a lattice side,
arrives as `u64` or `usize`. A result on its way to `println!` has to come back out as `f64`. The
obvious spellings each fail on one of the shipped scalars: `x as FloatType` casts between primitives
only, so it stops compiling the day the alias becomes `Float106`, and `FloatType::from(0.5)`
compiles for `f64` and `Float106` but not for `f32`, which has no `From<f64>`. The `lift` module in
`deep_causality_num` writes the crossings once, over `FromPrimitive` and `ToPrimitive`, which
all three scalars implement.

| Crossing | Function | From | To |
|---|---|---|---|
| a configuration literal into the working type | `lift` | `f64` | `T` |
| a count onto the real axis | `lift_count` | `u64` | `T` |
| any primitive float or integer | `lift_f32`, `lift_i32`, … `lift_usize` | that primitive | `T` |
| the display boundary | `lower` | `T` | `f64` |
| a real back to a rounded count | `to_count` | `T` | `u64` |

Every lift has a `try_` form that returns `Option` instead of panicking, and the `Lift` trait offers
the same crossings as methods, so an integer of any width reads as `n.lift::<FloatType>()`.

The program below sums a million terms of a series whose value is known in closed form, so the
rounding the loop accumulates can be measured in the working type and read out as digits. Each
index is a count lifted onto the real axis. The program compiles unchanged at all three precisions.

```rust
use deep_causality_algebra::{Real, Scalar};
use deep_causality_num::{lift, lift_count, lower, to_count};

/// The working type. Switch it to `f32` or `deep_causality_num::Float106`; nothing below changes.
type FloatType = f64;

/// Σ 1/(k(k+1)) for k = 1..=terms, summed forward at whatever precision the caller works.
/// `Scalar` carries `FromPrimitive`, so a library function lifts the same way a program does.
fn telescoping_sum<S: Scalar>(terms: u64) -> S {
    let one = lift::<S>(1.0);
    let mut sum = lift::<S>(0.0);
    for k in 1..=terms {
        // Each index is a count lifted onto the real axis.
        let k = lift_count::<S>(k);
        sum += one / (k * (k + one));
    }
    sum
}

fn main() {
    let terms: u64 = 1_000_000;
    let sum = telescoping_sum::<FloatType>(terms);

    // The series telescopes: its exact value is 1 − 1/(terms + 1), one division away.
    let one: FloatType = lift(1.0);
    let exact = one - one / lift_count::<FloatType>(terms + 1);

    // The rounding the loop accumulated, measured in the working type. `abs` and `log10` go
    // through `Real`, which every shipped scalar implements.
    let error = Real::abs(sum - exact);

    // The display boundary: f64 appears here and nowhere else.
    println!("error after {terms} terms: {:.1e}", lower(error));

    // A real back to a count: the digits the sum earned.
    match to_count(-Real::log10(error)) {
        Some(digits) => println!("{digits} correct digits"),
        None => println!("exact at this precision"),
    }
}
```

| `FloatType` | error after a million terms | correct digits |
|---|---|---|
| `f32` | `1.5e-4` | 4 |
| `f64` | `4.8e-14` | 13 |
| `Float106` | `9.8e-31` | 30 |

Three things can be read off that table.

- `f32` stops adding early. A term below half a unit in the last place of a sum near one vanishes
  on addition, and 1/(k(k+1)) falls below that once k passes four thousand, so the rest of the
  series is dropped and the sum stalls four digits short.
- `f64` keeps every term but rounds on each of a million additions. The roundings partly cancel
  and leave 4.8e-14, thirteen digits.
- `Float106` runs the same loop on a 106-bit mantissa and lands at 9.8e-31, thirty digits. The
  seventeen digits it gains over `f64` are what the type is for, and the program earned them by
  changing one alias.

`to_count` turns the error into that digit count. The negative logarithm lives in the working type
until it is rounded, and an error of zero, which has no logarithm, comes back as `None` rather than
a panic.

One rule of inference. `let one: FloatType = lift(1.0)` resolves the target from the annotation, and
a lifted value handed to a typed parameter or field resolves it from the slot. A lifted value that
first meets an operator does not: `lift(0.1) * x` leaves `T` open across several candidate `Mul`
impls, and the compiler will not choose. Name the target there, as in `lift::<FloatType>(0.1)`, or
bind it first under an annotation. Inside a generic function the target is the type parameter,
`lift::<S>(0.1)`, and any bound that carries `FromPrimitive` is enough; `Scalar` does.

The examples under `examples/` are written this way. Each keeps its alias in `main.rs`, none carries
a conversion helper of its own, and the three QCL examples under `examples/quantum_examples/`
run at all three precisions.

## Further reading

| Document | What it holds |
|---|---|
| `openspec/notes/archive/unified_math/unified_math_gaps.md` | Where the categorical layer stops, and the ranked work to close it |
| `openspec/notes/archive/unified_math/deep_causality_unified_math.md` | The assessment for this consolidation, and what it predicted against what happened |
| `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md` | Why the shaped witnesses stop at `Applicative` |
| `openspec/changes/archive-notes.md` | Reading an archived change whose paths predate a move |
| `examples/mathematics_examples/composable_multi_math/README.md` | Seven worked cross-crate compositions |
| `examples/quantum_examples/qcl_examples/` | Three programs that run unchanged at `f32`, `f64` and `Float106` |
| `lean/THEOREM_MAP.md` | Lean theorems and the Rust witnesses bound to them |
