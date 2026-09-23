# The IKKT Matrix Model: Spacetime as a Property of Matrices

This example relaxes the IKKT matrix model toward the minimum of its action at fixed norm, where the
matrices come to commute and acquire a spacetime. The IKKT model is a candidate non-perturbative
formulation of type IIB superstring theory, and it contains no spacetime of its own.

```bash
cargo run -p quantum_examples --example ikkt_matrix_model
```

## The problem

The model holds a set of matrices `X_μ` and an action

```text
S = Σ_{μ<ν} ‖[X_μ, X_ν]‖²
```

which is zero exactly when every pair of them commutes. Commuting matrices can be simultaneously
diagonalised, and their joint eigenvalues form a set of points. That set is the emergent
spacetime, a property of the configuration at the minimum.

## Relaxing along the equation of motion

Varying the action gives

```text
Σ_ν [X_ν, [X_μ, X_ν]] = 0
```

so the double commutator is zero exactly at a solution, and moving against it drives the
configuration toward one. It takes two nested calls to `commutator_kernel`. Its zeros are the
configurations the model is about: mutually commuting matrices, and the fuzzy spheres where the
double commutator cancels while the single one does not.

The run prints the action at every step it shows and reports whether it fell at each one. A step
that raised it would mean the step length was too long, and the run shows that openly.

## The norm has to be held fixed

This part is easy to get wrong, and the wrong version produces output that looks correct.

The action is **quartic** in the coordinates. Multiplying every matrix by `1 − η` multiplies the
action by `(1 − η)⁴` whatever the matrices do. A run that shrinks everything toward the origin
reports an action falling smoothly to zero and demonstrates nothing: the limit is an empty vacuum,
no matrices commute that did not commute before, and no spacetime emerges.

Each step here restores the norm the configuration started with, so the action can fall only if the
matrices commute. The norm column in the output shows the constraint: it holds at `1.356465997` from
the first step to the last.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fold` over pairs `(μ, ν)` | the action, over every commutator in the configuration |
| `fold` over `ν` | the double commutator at one coordinate: the equation of motion |
| `fold` over coefficients | the norm each step restores |

Each reduces a structure the model already has, so the relaxation step stays one call and the
loop carries only the configuration and what to report.

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

The action drops by five orders of magnitude while the norm stays fixed, so the matrices commute
without shrinking.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, so no conversion runs at
any call site. The alias defaults to `Float106` so that a hard-coded `f64`, invisible while the alias
*is* `f64`, becomes a compile error.

All four scalars run, and they disagree:

| Scalar | Steps | Final action | Fell every step |
|---|---|---|---|
| `f32` | 40 | 8.20e-7 | yes |
| `f64` | 40 | 8.20e-7 | yes |
| `Float106` | 40 | 8.20e-7 | yes |
| `BFloat16` | 200 | 1.11e-4 | no |

`BFloat16` has an eight-bit mantissa. Once the action is down around `1e-4` the step it needs is
smaller than the last bit of the coordinates, so the descent stalls and the monotonicity check
reports it. The check exists to catch a step length that overshoots; here it catches a scalar that
has run out of room.

## What this example covers

The example restates the essence of the model as a relaxation over the library's types; precision
as a parameter and categorical composition then come for free. The essence: commuting minimises an
action built only from commutators, and the minimum has a spectrum where the model had none. The
example keeps that and holds everything else simple: four coordinates instead of ten, `Cl(2)`
multivectors standing in for `N × N` matrices, the bosonic action with no fermions, and a fixed step
length instead of a line search.

A calculation a researcher would quote adds what this leaves out: the full ten coordinates of type
IIB, genuine large-`N` Hermitian matrices, the fermionic determinant that makes the model finite and
picks out four large dimensions from ten, and Monte Carlo sampling of the partition function in
place of relaxation to one solution. The joint spectrum is the observable of interest, and reading it
needs a simultaneous diagonalisation this example does not perform.

## How to grow the example toward a research calculation

Each step keeps the structure already here.

- **Read the emergent geometry.** Diagonalise the converged configuration and plot the joint
  eigenvalues: the spacetime, which the run stops short of.
- **Ten coordinates.** `N_MATRICES` is a constant and every loop is written over it.
- **Real matrices.** Replace the `Cl(2)` multivectors with `deep_causality_tensor` rank-2 tensors.
  `commutator_kernel` is the only call that needs a counterpart; the action and the equation of
  motion are written in terms of it.
- **A line search.** Choose the step length by testing the action instead of fixing it, which is
  also what would carry `BFloat16` past the stall in the table above.
- **Fuzzy-sphere initial data.** Start from an `SU(2)` representation and watch the run hold a
  non-zero action, the other kind of solution the equation of motion admits.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the relaxation loop, and the monotonicity check |
| `model.rs` | the configuration, the action, the equation of motion, and one relaxation step |
| `utils_print.rs` | the presentation, and the only `lower` calls |

## Key APIs used

- `commutator_kernel()`: `[A, B] = AB − BA`, called twice per coordinate per step
- `HilbertState` (as `Operator`): the coordinate matrices
- `Metric::Euclidean(dim)`: the algebra signature they share
