<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: An iterative solver that stops at its cap says so

Each of the four solvers that reach an iteration cap and return their last iterate SHALL report that it did not converge, and SHALL NOT present an unconverged iterate as a result.

The four, all with the same shape — a bounded `for` loop, a `break` on tolerance, and an
unconditional `Ok(value)` after it:

| Site | Iteration cap |
|---|---|
| `physics/theories/electroweak/radiative.rs:116-155` | 20 |
| `physics/kernels/astro/two_body.rs:186-192` | 100 |
| `physics/kernels/astro/ks_propagator.rs:189-196` | 100 |
| `algorithms/causal_discovery/brcd/brcd_gate.rs:124-162` | IRLS, ridge-penalised |

An earlier draft of this change named only the first and called it "the one real defect", contrasting
it with "six further solvers [that] share the shape without the same silence". That contrast is
inverted for two of them. The count is four.

Note that the last is not a scalar root finder at all: it is a `(p+1)`-dimensional Newton assembling
a Hessian and calling a dense solve. It shares the silence, not the shape.

#### Scenario: Each of the four signals non-convergence
- **WHEN** any of the four reaches its cap without meeting its stopping test
- **THEN** it returns a typed error rather than a value

#### Scenario: The error names the site's own failure
- **WHEN** the error is constructed
- **THEN** it carries enough to distinguish non-convergence from the site's other failure modes, which for the physics kernels already include a negative discriminant and a failed conversion

#### Scenario: Each error path is reachable in a test
- **WHEN** the suite runs
- **THEN** each of the four has a test that drives it to its cap and asserts the specific error

### Requirement: Signalling non-convergence does not regress a site that is already correct

Where a solver reaches its cap with an iterate that already satisfies the problem, the change SHALL accept that iterate rather than reject it, and the stopping test SHALL be widened rather than the result discarded.

This is the trap in the obvious fix. `two_body`'s Kepler Newton reaches its 100-iteration cap at
`e = 0.9999, M = 1e-6`, and its iterate is right: `0.008846308180179127` against a high-precision
bisection root of `0.008846308180176104`, a relative error of `3.4e-13` with a residual near `1e-18`.
Erroring at the cap would turn a correct answer into a failure on a case that works today.

The step-size test is what runs out there, not the accuracy. So the repair at that site is a
residual-based acceptance — converged when the equation is satisfied, not merely when the step
stopped shrinking — and the error is reserved for the case where neither holds.

#### Scenario: A correct iterate at the cap is accepted
- **WHEN** Kepler's equation is solved at `e = 0.9999, M = 1e-6`
- **THEN** the result is the root to the precision in use, not an error
- **AND** a test pins that case with the root taken from an independent bisection, not from this solver

#### Scenario: A genuinely unconverged iterate is refused
- **WHEN** a solver reaches its cap with neither its step test nor its residual test satisfied
- **THEN** the typed error is returned

#### Scenario: The distinction is recorded per site
- **WHEN** the stage's notes are read
- **THEN** each of the four records which test it converged on and whether widening was needed

### Requirement: The severity of each silence is stated as measured, not assumed

The stage SHALL record, for each of the four, whether its non-convergence is reachable at the inputs its callers actually supply.

The electroweak solver is the one this change first called live. It is not: at the constants its only
in-workspace caller supplies — `Z_MASS 91.1876`, `TOP_MASS 172.52`, `ALPHA_EM_MZ 1/127.95`,
`FERMI_CONSTANT 1.1663787e-5` — it converges in 5 of its 20 iterations, with the step contracting by
about 17.7× each time. It is a latent defect on a public function, which is worth fixing and is not
the same claim.

This change has the vocabulary for that distinction already: the Meek stage measured whether R4 ever
fires and recorded a bound rather than a conclusion. The same standard applies here.

#### Scenario: Reachability is measured, not asserted
- **WHEN** the stage's notes are read
- **THEN** each of the four is marked live or latent, with the inputs that decide it

#### Scenario: A latent defect is still fixed
- **WHEN** a solver's non-convergence is unreachable at its shipped inputs
- **THEN** it is still made to signal, because the function is public and its callers are not fixed

### Requirement: Adding an error variant to a published enum is treated as breaking

Where signalling requires a new variant on `PhysicsErrorEnum` or `BrcdErrorEnum`, the stage SHALL record it as a breaking change and enumerate what matches on it.

Neither enum is `#[non_exhaustive]`, so a downstream `match` breaks on a new variant. Both crates are
published.

There is a second, smaller consequence to record rather than discover: every numerical variant of
`PhysicsErrorEnum` carries a `String`. Constructing one from a no-allocation kernel either formats a
message — allocating where the kernel does not today — or the variant is shaped to avoid it.

#### Scenario: The break is recorded before the variant lands
- **WHEN** a variant is added
- **THEN** the stage's notes name it as breaking and list the matches affected

#### Scenario: The allocation consequence is decided, not inherited
- **WHEN** a kernel that does not allocate today gains an error path
- **THEN** the stage records whether it now allocates, and if so that this was chosen

### Requirement: No general root-finding surface is built in this change

This change SHALL NOT add a bisection, Newton, dual-number Newton, fixed-point or line-search operator to `deep_causality_calculus`, and SHALL NOT add an arrow form for any of them.

An earlier draft did, at roughly 3100 lines. Measured against the tree, a generic scalar root finder
could replace 55 lines — the three bisections and the two scalar Newtons. `brcd_gate` is
multidimensional and `radiative` is a fixed point, so neither is in that set. That is a ratio near
25:1 against code it does not replace, for a stage whose actual defect is the silence above.

Three further reasons, each independently sufficient:

- **The bisections need nothing.** All three already validate their bracket before iterating and
  already return `PhysicsError::NumericalInstability`. All three break on interval exhaustion at
  54–66 `f64` iterations against a cap of 200, so the cap is unreachable. A spec requiring them to
  do what they already do is a spec for existing code.
- **Dual-number Newton has no caller.** Both shipped Newtons supply closed-form derivatives by
  hand — `1 − e·cos E` and `dt/ds = |u|²`. `Dual` appears in neither. D6 forbids building it.
- **The arrow form cannot be composed.** `Arrow::compose` requires `G: Arrow<In = Self::Out>`, and a
  root finder's `Out` is a `Result`; every existing downstream arrow takes `In = S` or
  `In = Dual<R>`. The scenario an earlier draft wrote for this could not have been satisfied without
  building a bridge nobody asked for.

If a root-finding surface is wanted later, it is its own change, with the consumer named first.

#### Scenario: The calculus crate is unchanged
- **WHEN** `deep_causality_calculus` is compared before and after this change
- **THEN** its public surface is identical

#### Scenario: The reduction is recorded
- **WHEN** the stage's notes are read
- **THEN** they record what the operator family would have replaced and why it was cut
