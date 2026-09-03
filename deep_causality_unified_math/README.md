<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified Math


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

The same mechanism composes a tensor with a topology in a few lines. A field of `FloatType`
values arrives as a `CausalTensor`, becomes the data of a line manifold, and the Laplacian is the
comonadic extension of a three-point stencil. The result is another manifold whose data is again
a tensor, so the next crate can pick it up where this one left it:

```rust
// topology: the field on a line manifold; haft: its Laplacian by comonadic extension.
let manifold = line_manifold(phi.clone());
let laplacian = ManifoldWitness::extend(&manifold, |w| {
    let i = w.cursor();
    let d = w.data().as_slice();
    if i >= N {
        return zero;
    }
    let left = if i > 0 { d[i - 1] } else { d[i] };
    let right = if i + 1 < N { d[i + 1] } else { d[i] };
    left + right - two * d[i]
});
let delta = &laplacian.data().as_slice()[..N];
```

The program this is cut from is under "Precision across composition" below.

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

## Precision across composition

The program below carries one field through four different math crates. The values
are sampled into a `CausalTensor`, placed on a line manifold whose complex is typed by the same
alias, differentiated by comonadic extension, paired into `Cl(2,0)` vectors held in a tensor, and
turned through a thousand quarter turns by one rotor pair. Two identities hold in exact
arithmetic, the Laplacian's telescoping sum is zero and four quarter turns are the identity, so
whatever remains is the rounding of the whole pipeline at one precision.

```rust
use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_metric::Metric;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{lift, lift_count, lower};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};

/// One alias for four crates. Switch it to `f32` or `deep_causality_num::Float106`.
type FloatType = f64;

const N: usize = 64;
const TURNS: usize = 250;

fn main() {
    let zero: FloatType = lift(0.0);
    let two: FloatType = lift(2.0);

    // tensor: a field sampled from sin, one value per vertex.
    let step = FloatType::pi() / lift_count::<FloatType>(N as u64);
    let phi: Vec<FloatType> = (0..N)
        .map(|i| Real::sin(lift_count::<FloatType>(i as u64) * step))
        .collect();

    // topology: the field on a line manifold; haft: its Laplacian by comonadic extension.
    let manifold = line_manifold(phi.clone());
    let laplacian = ManifoldWitness::extend(&manifold, |w| {
        let i = w.cursor();
        let d = w.data().as_slice();
        if i >= N {
            return zero;
        }
        let left = if i > 0 { d[i - 1] } else { d[i] };
        let right = if i + 1 < N { d[i + 1] } else { d[i] };
        left + right - two * d[i]
    });
    let delta = &laplacian.data().as_slice()[..N];

    // The stencil telescopes, so Σ Δφ is zero in exact arithmetic; what remains is rounding.
    let drift = Real::abs(delta.iter().fold(zero, |acc, &v| acc + v));

    // multivector: (φ, Δφ) as a vector in Cl(2,0) at every vertex, held in a tensor, and turned
    // through a thousand quarter turns by one rotor pair. Exact arithmetic returns it unchanged.
    let metric = Metric::Euclidean(2);
    let field: CausalTensor<CausalMultiVector<FloatType>> =
        CausalTensor::from_shape_fn(&[N], |idx| {
            CausalMultiVector::new(vec![zero, phi[idx[0]], delta[idx[0]], zero], metric).unwrap()
        });
    let (rotor, rotor_rev) = quarter_turn(metric);
    let mut turned = field.clone();
    for _ in 0..4 * TURNS {
        turned = CausalTensorWitness::fmap(turned, |v| {
            rotor.geometric_product(&v).geometric_product(&rotor_rev)
        });
    }
    let worst = field
        .as_slice()
        .iter()
        .zip(turned.as_slice())
        .map(|(a, b)| {
            let (a, b) = (a.data(), b.data());
            Real::abs(a[1] - b[1]) + Real::abs(a[2] - b[2])
        })
        .fold(zero, |m, e| if e > m { e } else { m });

    // The display boundary.
    println!("Σ Δφ            = {:.1e}", lower(drift));
    println!("{TURNS} full turns  = {:.1e}", lower(worst));
}

/// A line of `N` vertices and `N - 1` edges, the field on the vertices, zero on the edges.
fn line_manifold(vertex_values: Vec<FloatType>) -> SimplicialManifold<FloatType, FloatType> {
    let vertices = (0..N).map(|i| Simplex::new(vec![i])).collect();
    let edges = (0..N - 1).map(|i| Simplex::new(vec![i, i + 1])).collect();
    let mut triplets = Vec::with_capacity(2 * (N - 1));
    for e in 0..N - 1 {
        triplets.push((e, e, -1i8));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(N, N - 1, &triplets).unwrap();
    let complex = SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    );
    let mut data = vertex_values;
    data.extend(std::iter::repeat_n(lift::<FloatType>(0.0), N - 1));
    let data = CausalTensor::new(data, vec![2 * N - 1]).unwrap();
    Manifold::new(complex, data, 0).expect("a manifold")
}

/// The rotor pair for a quarter turn in the e1∧e2 plane, `R = cos θ/2 − sin θ/2 e12` and its
/// reverse.
fn quarter_turn(metric: Metric) -> (CausalMultiVector<FloatType>, CausalMultiVector<FloatType>) {
    let half = FloatType::pi() / lift::<FloatType>(4.0);
    let (c, s) = (Real::cos(half), Real::sin(half));
    let zero: FloatType = lift(0.0);
    (
        CausalMultiVector::new(vec![c, zero, zero, -s], metric).unwrap(),
        CausalMultiVector::new(vec![c, zero, zero, s], metric).unwrap(),
    )
}
```

| `FloatType` | Σ Δφ | 250 full turns |
|---|---|---|
| `f32` | `2.0e-7` | `1.5e-7` |
| `f64` | `6.9e-17` | `1.6e-13` |
| `Float106` | `1.7e-32` | `1.3e-29` |

Each row is one precision end to end. The tensor's cells, the complex's coefficients in
`SimplicialManifold<FloatType, FloatType>`, the closure's arithmetic, the rotor's `cos` and `sin`
and the geometric products all took the alias, so the residuals move together: twenty-five orders
of magnitude on the sum from `f32` to `Float106`, and a thousand geometric products at `Float106`
still land thirteen orders below one unit in the last place of `f64`. Nothing was converted
between crates, because there was nothing to convert.

## Mixed precision as a parameter

Mixed precision comes downt to deciding a fundamental engineering trade-off:

Higher precison that is slower and needs more memory vs lower precision that is faster
and uses less memory.

Historically, the decision was made for you by library authors based on best effort to reduce compromises.
Sometimes the decision was right, and in other cases, the decisin had to be revised for any number of reasons. 

In unified math, you, the program author decides the trade-off for each one 
of your use cases because not every part of a simulation requires the same precision,
and the parameter lets each part determines its own precision level. 

The three computations below are bound by three different limits that affect precision.

- **Noise-bound.** A Monte Carlo integral from a hundred thousand stored draws. Its error is the
  statistical 1/√n, near 9e-4, and rounding sits many orders below that at any precision. Extra
  bits buy nothing here. In this case, `f32` saves memory because precision is identical at all float types.
- **Mesh-bound.** A second derivative by a central difference on a fixed step. Truncation costs
  h²/12 and rounding costs ε/h². `f32` loses the value to the second; `Float106` cannot improve
  on the first. `f64` is the sweet spot where both balance.
- **Reference-bound.** The telescoping series from "Precision as a parameter earns a digit
  for roughly every three bits of mantissa, and it is the kind of calculation where higher precision delivers better results.

However, in some cases it is simply not known if a certain precision level meets the error level requirements of the program.
The program below tests each functions at 3 different precison levels and then picks the precision level that meets the error level requirements. Then, the aforementioned engineering trade-off can be decided and each part of a simulation can be parametrized to its optimal precision level to balance precsion and performance per stage. 

```rust
use deep_causality_algebra::{Real, Scalar};
use deep_causality_num::{Float106, Lift, ToPrimitive, lift, lift_count, lower};
use deep_causality_rand::{Distribution, Rng, StandardUniform, Xoshiro256};

/// The master precision, where all the parts meet. It must be no narrower than the widest part.
type Master = Float106;

const SAMPLES: u64 = 100_000;
const SEED: u64 = 7;
const STEP: f64 = 1e-3;
const TERMS: u64 = 1_000_000;

fn main() {
    println!("            noise-bound  mesh-bound  reference-bound");
    for (name, [a, b, c]) in [
        ("f32     ", errors_at::<f32>()),
        ("f64     ", errors_at::<f64>()),
        ("Float106", errors_at::<Float106>()),
    ] {
        println!("{name}    {a:>9.1e}   {b:>9.1e}   {c:>9.1e}");
    }

    // The pick: each part at the precision its error budget asks for.
    let (integral, bytes) = monte_carlo::<f32>(SAMPLES, SEED);
    let curvature = second_derivative::<f64>(STEP);
    let reference = series::<Float106>(TERMS);

    // They meet at the master precision. Each crossing is a lift; nothing is thrown away.
    let total: Master = integral.lift::<Master>() + curvature.lift::<Master>() + reference;
    let one: Master = lift(1.0);
    let exact: Master = one / lift_count::<Master>(3) - Real::sin(one) + one
        - one / lift_count::<Master>(TERMS + 1);
    println!(
        "\ncomposed at the master precision: error {:.1e}",
        lower(Real::abs(total - exact))
    );
    let widest = bytes / core::mem::size_of::<f32>() * core::mem::size_of::<Float106>();
    println!("the draws took {bytes} bytes at f32; {widest} at Float106");
}

/// Noise-bound: a Monte Carlo estimate of ∫₀¹ x² dx from stored draws. The statistical error
/// is about 1/√n and buries rounding at every precision. Returns the estimate and the bytes
/// the draws occupied.
fn monte_carlo<S: Scalar>(samples: u64, seed: u64) -> (S, usize)
where
    StandardUniform: Distribution<S>,
{
    let mut rng = Xoshiro256::from_seed(seed);
    let draws: Vec<S> = (0..samples).map(|_| rng.random::<S>()).collect();
    let bytes = core::mem::size_of_val(draws.as_slice());
    let sum = draws.iter().fold(lift::<S>(0.0), |acc, &x| acc + x * x);
    (sum / lift_count::<S>(samples), bytes)
}

/// Mesh-bound: the second derivative of sin at 1 by a central difference on a fixed step.
/// Truncation costs h²/12, rounding costs ε/h². f64 is where neither dominates.
fn second_derivative<S: Scalar>(step: f64) -> S {
    let x = lift::<S>(1.0);
    let h = lift::<S>(step);
    let two = lift::<S>(2.0);
    (Real::sin(x + h) - two * Real::sin(x) + Real::sin(x - h)) / (h * h)
}

/// Reference-bound: the telescoping series from the previous section. It earns a digit for
/// roughly every three bits of mantissa it is given.
fn series<S: Scalar>(terms: u64) -> S {
    let one = lift::<S>(1.0);
    let mut sum = lift::<S>(0.0);
    for k in 1..=terms {
        let k = lift_count::<S>(k);
        sum += one / (k * (k + one));
    }
    sum
}

/// The three errors against closed forms, all at one precision. `lower` asks for `ToPrimitive`,
/// which `Scalar` does not carry.
fn errors_at<S: Scalar + ToPrimitive>() -> [f64; 3]
where
    StandardUniform: Distribution<S>,
{
    let one = lift::<S>(1.0);
    let error = |value: S, exact: S| lower(Real::abs(value - exact));
    [
        error(monte_carlo::<S>(SAMPLES, SEED).0, one / lift_count::<S>(3)),
        error(second_derivative::<S>(STEP), -Real::sin(one)),
        error(series::<S>(TERMS), one - one / lift_count::<S>(TERMS + 1)),
    ]
}

```

| | noise-bound | mesh-bound | reference-bound |
|---|---|---|---|
| `f32` | `5.9e-4` | `1.3e-1` | `1.5e-4` |
| `f64` | `5.8e-4` | `7.0e-8` | `4.8e-14` |
| `Float106` | `9.3e-4` | `7.0e-8` | `9.8e-31` |

The noise-bound column does not move with precision: all three rows sit inside
the 1/√n band, and which of them lands closest is the luck of the stream, since the `Float106`
sampler consumes more random bits per draw and walks a different sequence. The mesh-bound column
moves once, from `f32` to `f64`, and then stops. The reference-bound column moves with every row,
by ten digits and then by seventeen.

The pick reads off the table: `f32` for the draws, `f64` for the difference, `Float106` for the
series. The composed value lands at 5.9e-4, which is the noise-bound part's own error, and the
draws took 400 000 bytes where `Float106` would have taken 1 600 000. The master precision has
one job, to be no narrower than the widest contributor, so that the crossings lose nothing. The
accuracy of the composed value is then set by its parts, each at the precision its physics asked
for. That is the trade-off made explicit: a precision chosen per part against a requirement.

## Economic impact

An estimate, from stated assumptions, of what the pick in the previous section is worth on a
simulation that conventionally runs on a supercomputer: a convection-permitting ensemble forecast
of local weather. Nothing in this section was measured. Every number follows from the assumption
table by the arithmetic shown, so a reader with a different model can substitute their own.

**The simulation.** A limited-area model on a 1000 km square at 1 km spacing with 100 levels, run
as a 40-member ensemble, cycled with an ensemble-variational assimilation, and verified against
conservation budgets. Its parts are bound the way the three computations above were bound.

- The ensemble members are noise-bound. Their spread is the signal, and rounding at `f32` sits
  far below the perturbations that make them differ. That finding is what let ECMWF move its
  operational forecast model to single precision in 2021 (Váňa et al., 2017). The members hold
  the fields, and the fields are where the memory goes.
- The assimilation is mesh-bound in the sense of the previous section: a minimisation whose
  gradients cancel and whose conditioning asks for `f64`. It stays at `f64` in both
  configurations.
- The conservation budgets are reference-bound: mass, energy and moisture integrals accumulated
  over a whole forecast, where a drift of 1e-12 per step is the quantity being measured. They go
  to `Float106`. They are reductions, so they cost almost no memory.

| Assumption | Value |
|---|---|
| grid | 1000 × 1000 columns × 100 levels = 10⁸ points |
| resident 3D fields per member | 48: 12 prognostic at two time levels, 12 tendencies, 12 diagnostics |
| halo cells and exchange buffers | 15 % on top of field memory |
| ensemble | 40 members |
| assimilation working set | 10 model states at `f64` |
| ensemble statistics | mean and spread of 20 fields at `f64` |
| reference budgets | 4 budgets × 100 levels per member |
| `Float106` cost per operation | 10 to 20 × `f64`, so it is spent on reductions and never on fields |
| instance headroom | 10 % of node memory for the operating system, MPI and I/O |
| the run | one assimilation cycle and its 40 forecasts: 1 hour at `f64`, of which the members take 80 % and the assimilation and I/O 20 % |
| `f32` time on the members | 40 % less than `f64`, transferred from ECMWF's measurement on the IFS (Váňa et al., 2017) |
| budget reductions | per step at `f64`, accumulated across steps at `Float106`; no measurable time |
| billing | per second of use, so a shorter run bills fewer hours |
| prices | AWS on-demand Linux list prices, US East (N. Virginia), instances.vantage.sh, 2026-09-03 |
| GB | 10⁹ bytes; 1 GiB = 1.074 GB |

**The memory.** One member's fields are 48 × 10⁸ × 8 B = 38.4 GB at `f64` and 19.2 GB at
`f32`; with halos, 44.16 GB and 22.08 GB.

| Component | blanket `f64` | mixed, per part | saving |
|---|---|---|---|
| 40 ensemble members, with halos | 1766.4 GB | 883.2 GB at `f32` | 883.2 GB |
| assimilation working set | 384.0 GB | 384.0 GB at `f64` | 0 |
| ensemble statistics | 32.0 GB | 32.0 GB at `f64` | 0 |
| reference budgets, 16 000 accumulators | 0.13 MB | 0.26 MB at `Float106` | −0.13 MB |
| **total** | **2182.4 GB** | **1299.2 GB** | **883.2 GB, 40 %** |

**The price.** On rented nodes the memory has an hourly price, and it is bought in rungs. A cloud
user fixes the node count, which fixes the cores and the wall clock, then takes the lowest rung
whose memory holds that node's share of the state with headroom. The ladders below share the
384, 768 and 1536 GiB rungs and differ in processor: c7a, m7a and r7a on one 192-vCPU AMD
processor; c8i, m8i and r8i as 384-vCPU Intel bare metal; c8g, m8g and r8g as Graviton4 bare
metal, where every one of the 192 vCPUs is a core because the processor has no simultaneous
multithreading. Each rung costs more for the same cores, and the price per terabyte-hour falls
as the rung rises: $23.90 on c7a, $13.49 on m7a, $8.86 on r7a, $6.07 on the high-memory x2iedn.

| Instance | vCPU | memory | list price |
|---|---|---|---|
| c7a.48xlarge, also c7a.metal-48xl | 192 | 384 GiB | $9.85 |
| m7a.48xlarge | 192 | 768 GiB | $11.13 |
| r7a.48xlarge | 192 | 1536 GiB | $14.61 |
| x2idn.32xlarge | 128 | 2048 GiB | $13.34 |
| x2iedn.32xlarge | 128 | 4096 GiB | $26.68 |
| c8i.metal-96xl | 384 | 768 GiB | $17.99 |
| m8i.metal-96xl | 384 | 1536 GiB | $20.32 |
| r8i.metal-96xl | 384 | 3072 GiB | $26.67 |
| c8g.metal-48xl | 192 cores | 384 GiB | $7.66 |
| m8g.metal-48xl | 192 cores | 768 GiB | $8.62 |
| r8g.metal-48xl | 192 cores | 1536 GiB | $11.31 |

The blanket `f64` state needs 2401 GB of instance memory with headroom and the mixed state
1429 GB. The best value for money that holds the `f64` state with 384 cores is two r8g nodes at
$22.62 an hour; the alternatives are one r8i node at $26.67 with half the cores, or two r7a
nodes at $29.21 with half the cores. On the same two Graviton nodes the mixed state fits one
rung lower, on m8g. That pair is the master comparison.

| | fixed `f64` | mixed precision | difference |
|---|---|---|---|
| state in memory | 2182 GB | 1299 GB | −883 GB, −40 % |
| instance memory needed | 2401 GB | 1429 GB | −972 GB |
| best-value configuration | 2 × r8g.metal-48xl | 2 × m8g.metal-48xl | one rung lower |
| cores | 384 | 384 | same |
| memory bought | 3072 GiB, 3299 GB | 1536 GiB, 1649 GB | half |
| bytes moved per member per step, members at `f32` throughout | 1 | 0.5 | half |
| bytes moved per run, members 80 % and assimilation 20 % | 1 | 0.8 × 0.5 + 0.2 = 0.6 | −40 % |
| members that fit on eight 512 GB nodes | 74 | 148 | twice |
| accuracy of the composed result | set by the noise-bound part | set by the noise-bound part | same |
| list price per hour | $22.62 | $17.23 | −$5.39, −24 % |
| wall clock per run, members at 0.6 and assimilation at 1 | 1.00 h | 0.8 × 0.6 + 0.2 = 0.68 h | −32 % |
| **total cost per run** | **$22.62** | **$11.72** | **−$10.90, −48 %** |

The difference is a rung on the ladder and a third off the clock: the same cores, half the memory
bought, a quarter off the hourly rate, a run that ends 32 % sooner, and the conservation budgets
carried at `Float106` for a quarter of a megabyte. The 48 % is the product of two effects with
different standing. The 24 % on the rate is arithmetic on list prices. The 32 % on the clock is
the members' 80 % of the run running in 60 % of the time, and that 60 % is ECMWF's figure for
the IFS on its own machines, carried over here as the best available measurement rather than
as one made on this model. The assimilation keeps its `f64` and its 20 %.
The saving is a step function of the ladder, so it is not the same at every node count. It is
largest where the `f64` state is pushed onto the memory-priced instances, 45 % on a single
virtual node where no 192-vCPU instance holds the `f64` state at all, and it is zero at a node
count where both states land on the same rung, where the halved byte traffic is what remains.
Reserved and spot pricing discount both columns alike and leave the percentages where they are.

The rule is the one the previous section stated, precision per part against a requirement. Here
the requirement has a price, and the price is in the table.

Váňa, F., Düben, P., Lang, S., Palmer, T., Leutbecher, M., Salmond, D., Carver, G. (2017). Single
Precision in Weather Forecasting Models: An Evaluation with the IFS. *Monthly Weather Review*,
145(2), 495–502.

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
