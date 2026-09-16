# The IKKT Matrix Model: Spacetime as a Property of Matrices

The IKKT model is a candidate non-perturbative formulation of type IIB superstring theory. It has no
spacetime in it. Spacetime is what the matrices acquire at the minimum of the action.

```bash
cargo run -p quantum_examples --example ikkt_matrix_model
```

## The problem

The model holds a set of matrices `X_μ` and an action

```text
S = Σ_{μ<ν} ‖[X_μ, X_ν]‖²
```

which is zero exactly when every pair of them commutes. Commuting matrices can be simultaneously
diagonalised, and their joint eigenvalues are then a set of points. That set is the emergent
spacetime: it is a property of the configuration at the minimum, not a stage the matrices were
placed on.

## Relaxing along the equation of motion

Varying the action gives

```text
Σ_ν [X_ν, [X_μ, X_ν]] = 0
```

so that double commutator is zero exactly at a solution, and moving against it drives the
configuration toward one. It is two nested calls to `commutator_kernel`, and its zeros are the
configurations the model is about: mutually commuting matrices, and the fuzzy spheres where the
double commutator cancels while the single one does not.

The run prints the action at every step it shows, and reports whether it fell at each one. A step
that raised it would mean the step length was too long, and that is worth seeing rather than hiding.

## The norm has to be held fixed

This is the part that is easy to get wrong, and getting it wrong produces output that looks correct.

The action is **quartic** in the coordinates. Multiplying every matrix by `1 − η` multiplies the
action by `(1 − η)⁴` regardless of what the matrices are doing. So a run that simply shrinks
everything toward the origin reports an action falling smoothly to zero while demonstrating nothing
whatsoever: the limit is an empty vacuum, no matrices commute that did not commute before, and no
spacetime emerges.

Each step here restores the norm the configuration started with. The only way left for the action to
fall is for the matrices to genuinely commute. The norm column in the output is there so the
constraint is visible rather than asserted — it holds at `1.356465997` from the first step to the
last.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fold` over pairs `(μ, ν)` | the action, over every commutator in the configuration |
| `fold` over `ν` | the double commutator at one coordinate: the equation of motion |
| `fold` over coefficients | the norm each step restores |

Each is a reduction over a structure the model already has, so the relaxation step stays one call
and the loop carries nothing but the configuration and what to report.

## Output

```text
Start
  action S             0.048000000
  configuration norm   1.356465997

Relaxing against the equation of motion
  step        action S    largest [X,X]    configuration norm
     1     0.045250486     0.142697997     1.356465997
     5     0.024817603     0.105678386     1.356465997
    10     0.007037002     0.056273004     1.356465997
    15     0.001634298     0.027118888     1.356465997
    20     0.000361390     0.012752474     1.356465997
    25     0.000079040     0.005963879     1.356465997
    30     0.000017245     0.002785732     1.356465997
    35     0.000003761     0.001300874     1.356465997
    40     0.000000820     0.000607444     1.356465997

Outcome
  steps taken                         40
  final action S             0.000000820
  largest [X_mu, X_nu]       0.000607444
  configuration norm         1.356465997
  action fell every step             yes
```

The action drops by five orders of magnitude while the norm does not move at all. The matrices
commute because they turned into commuting matrices, not because they shrank.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, so no conversion runs at
any call site. It sits at `Float106` rather than `f64` on purpose: a hard-coded `f64` is invisible
while the alias *is* `f64`, and a compile error the moment the two differ.

All four scalars run, and they do not all agree, which is the interesting part:

| Scalar | Steps | Final action | Fell every step |
|---|---|---|---|
| `f32` | 40 | 8.20e-7 | yes |
| `f64` | 40 | 8.20e-7 | yes |
| `Float106` | 40 | 8.20e-7 | yes |
| `BFloat16` | 200 | 1.11e-4 | no |

`BFloat16` has an eight-bit mantissa. Once the action is down around `1e-4` the step it needs is
smaller than the last bit of the coordinates, so the descent stalls and the monotonicity check
reports it. That line exists to catch a step length that overshoots; here it catches a scalar that
has run out of room, which is the same question asked of a different part of the setup.

## What this example covers

The goal is to reformulate the essence of the model as a relaxation over the library's types, and to
get precision as a parameter and categorical composition for free once it is in that form. The
essence is that an action built only from commutators is minimised by commuting, and that the
minimum has a spectrum where the model had none. The example keeps that and holds everything else
simple: four coordinates rather than ten, `Cl(2)` multivectors standing in for `N × N` matrices, the
bosonic action with no fermions, and a fixed step length instead of a line search.

A calculation a researcher would quote adds what this leaves out: the full ten coordinates of type
IIB, genuine large-`N` Hermitian matrices, the fermionic determinant that makes the model finite and
picks out four large dimensions from ten, and Monte Carlo sampling of the partition function rather
than relaxation to one solution. The joint spectrum is also the observable of interest, and reading
it needs a simultaneous diagonalisation this example does not perform.

## How to grow the example toward a research calculation

Each step keeps the structure already here.

- **Read the emergent geometry.** Diagonalise the converged configuration and plot the joint
  eigenvalues. That is the spacetime, and it is the one thing the run currently stops short of.
- **Ten coordinates.** `N_MATRICES` is a constant and every loop is written over it.
- **Real matrices.** Replace the `Cl(2)` multivectors with `deep_causality_tensor` rank-2 tensors.
  `commutator_kernel` is the only call that needs a counterpart; the action and the equation of
  motion are written in terms of it.
- **A line search.** Choose the step length by testing the action instead of fixing it, which is
  also what would carry `BFloat16` past the stall in the table above.
- **Fuzzy-sphere initial data.** Start from an `SU(2)` representation and watch the run hold a
  non-zero action, which is the other kind of solution the equation of motion admits.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the relaxation loop, and the monotonicity check |
| `model.rs` | the configuration, the action, the equation of motion, and one relaxation step |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `commutator_kernel()` — `[A, B] = AB − BA`, called twice per coordinate per step
- `HilbertState` (as `Operator`) — the coordinate matrices
- `Metric::Euclidean(dim)` — the algebra signature they share
