# A Topological Insulator: The Chern Number, Two Independent Ways

Some insulators cannot be turned into others without closing their gap. What separates them is not
any local property but an integer, and this example computes that integer twice by routes that share
almost nothing.

```bash
cargo run -p quantum_examples --example topological_insulator
```

## The problem

The **Chern number** is the Berry curvature integrated over the whole Brillouin zone:

```text
C = (1 / 2π) ∫∫ Ω(kx, ky) dkx dky
```

It comes out an integer however the material is deformed, and changes only when the gap closes. That
is what "topological" means here, and it is why the quantised Hall conductance of such a material
survives disorder that changes everything else about it.

The model is Qi-Wu-Zhang, `H(k) = d(k)·σ` with

```text
d(k) = (sin kx,  sin ky,  u + cos kx + cos ky)
```

whose phase depends on the single mass parameter `u`.

## Two routes, sharing nothing but the d-vector

| Route | How | What it needs |
|---|---|---|
| Quadrature | `Ω` from exact `∂d/∂k`, integrated by nested composite Simpson | derivatives, no spinors |
| Wilson loop | Berry flux `Im ln W` around each plaquette of a k-grid, summed | spinors, no derivatives |

The first differentiates and never forms a spinor. The second forms spinors and never
differentiates. They can only agree by both being right, which is what makes the second calculation
worth doing.

The derivatives come from the tangent functor, so there are no finite differences anywhere and no
step size to tune.

## The mass is an argument, not a field

`DComponent` carries no numbers at all — only which component it is:

```rust
pub struct DComponent {
    pub component: usize,
}

impl DifferentiableField<3> for DComponent {
    fn run<S: Scalar>(&self, at: &[S; 3]) -> S {
        match self.component {
            0 => Real::sin(at[KX]),
            1 => Real::sin(at[KY]),
            _ => at[MASS] + Real::cos(at[KX]) + Real::cos(at[KY]),
        }
    }
}
```

A struct that stored `u` at one concrete type would have to widen it inside `run`, and the model
would then be evaluated at whatever precision `u` was written down in no matter what `S` the caller
asked for. Passing it in as a third coordinate leaves every number in the model at the caller's
precision. The extra slot costs one derivative nobody reads, and buys a model with no concrete type
written into it anywhere.

## Two bounds, for two reasons

The quadrature route is generic over `Scalar`. The Wilson route asks for `Scalar + RealField`,
because it builds `Complex` numbers and a complex number needs a field underneath it. `Scalar` alone
admits the dual numbers the tangent functor runs on, and those are not a field — which is exactly
what lets the quadrature route be differentiated through.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fmap` | one phase → its two Chern numbers, either of which may fail |
| `sequence` | a tensor of fallible phases → one fallible tensor |
| `bind` | the analysis as a stage that short-circuits on a non-finite integral |

The nested `quadrature` is worth noting on its own: the two-dimensional integral is the
one-dimensional operator applied to itself, not a second routine written for the purpose.

## Output

```text
     u    | C (quadrature) | C (Wilson loop) |  C  | phase
  --------+----------------+-----------------+-----+------------------------
     3.0  |      0.000000  |      -0.000000  | +0  | trivial (|u| > 2)
     1.0  |      1.000000  |       1.000000  | +1  | topological (0 < u < 2)
    -1.0  |     -1.000000  |      -1.000000  | -1  | topological (-2 < u < 0)

Agreement
  largest gap between the two routes      1.190e-15
  largest departure from an integer       1.190e-15
```

The second number is the one that matters. Nothing in either calculation rounds or snaps to an
integer; both integrate a smooth function over a closed surface, and an integer is what comes out.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`. It sits at `Float106`
rather than `f64` on purpose: a hard-coded `f64` is invisible while the alias *is* `f64`, and a
compile error the moment the two differ. All four scalars run:

| Scalar | Gap between routes | Departure from integer |
|---|---|---|
| `BFloat16` | 7.85e-1 | 2.34e-2 |
| `f32` | 2.38e-6 | 1.31e-6 |
| `f64` | 4.22e-15 | 1.22e-15 |
| `Float106` | 1.19e-15 | 1.19e-15 |

`BFloat16` is where the two routes stop agreeing, and the way they fail is informative. The
quadrature still lands the right integers — `1.015625` and `-1.023438` — while the Wilson sum
collapses to `0.231` and `-0.285`. The Wilson route adds ten thousand small `atan2` values into one
accumulator, and at an eight-bit mantissa each addition loses most of what it was adding. Simpson's
rule works with far fewer, far larger contributions and survives. Same integral, same answer in
exact arithmetic, and very different tolerance for a short mantissa.

### The bug this rewrite found

Making the Wilson route precision-generic turned up a defect in `Float106::atan2`: it returned
`NaN` when both arguments were zero, where IEEE 754 specifies `atan2(±0, +0) = ±0`. `f32`, `f64` and
`BFloat16` all reach the platform's `atan2` and answer correctly, so the same source computed a
finite angle at three precisions and a `NaN` at the fourth.

The origin is reached here as a matter of course: at `u = 3` and `k = (±π, ±π)` the d-vector points
straight along `z`, its in-plane part is exactly zero, and the azimuthal angle is undefined. A `NaN`
there propagated through the whole flux sum. The fix and its regression tests are in
`deep_causality_num`.

## What this example covers

The goal is to reformulate the essence of a topological invariant as a composition over the
library's types, and to get precision as a parameter and categorical composition for free once it is
in that form. The essence is that an integer falls out of integrating a smooth curvature, and that
two unrelated routes to it agree. The model keeps that and holds everything else simple: a two-band
model with an analytic d-vector, a clean gap at every phase probed, no disorder and no interactions,
and the lower band only.

A calculation a condensed-matter group would run adds what this leaves out: a tight-binding or
continuum Hamiltonian diagonalised numerically rather than an analytic `d`, several occupied bands
with a non-Abelian Berry connection, disorder averaging, and a check near the transition at
`|u| = 2` where the gap closes and both methods are expected to struggle.

## How to grow the example toward a band-structure calculation

Each step keeps the structure already here.

- **Sweep `u` through a transition.** The phase list is a constant array and `fmap` already walks
  it. Watching the integral lose quantisation as `u → 2` is the honest way to see where the methods
  break.
- **Several occupied bands.** Replace the lower-band spinor with the projector onto the occupied
  subspace; the Wilson loop becomes a determinant and the rest of the sum is unchanged.
- **A numerical Hamiltonian.** Drop the analytic `d` and diagonalise `H(k)` at each point. The
  quadrature route needs the derivative of the eigenvector, which is what the tangent functor is
  for.
- **The Hall conductance.** `C` times `e²/h` is the observable; adding the unit conversion turns the
  table into a prediction about a measurement.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the phase list, and the analysis as a `CausalFlow` stage |
| `model.rs` | the d-vector as a differentiable field, the Berry curvature, and both integrals |
| `utils_print.rs` | the presentation, and the only `lower` calls |
