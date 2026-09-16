# Electroweak Unification: The W Mass, From First Principles

Fix three measured numbers and the Standard Model predicts the W boson mass. The measurement is good
to better than a part in ten thousand. Comparing the two is one of the sharpest tests the theory
faces, and this example runs the comparison.

```bash
cargo run -p quantum_examples --example gauge_electroweak
```

## The problem

Above about 100 GeV the electromagnetic and weak forces are one force, a gauge theory with symmetry
`SU(2) × U(1)`. Below that scale the Higgs field takes a vacuum value and the symmetry breaks: three
of the four gauge bosons acquire mass and become the `W⁺`, `W⁻` and `Z`, and the fourth stays
massless and is the photon.

What makes this testable is how little freedom is left afterwards. Fix `α_EM`, the Fermi constant
and `M_Z`, and the couplings, the remaining masses, the decay widths and the resonance cross-section
all follow.

## Why the tree level is not enough

At tree level `M_W = g·v/2`. The run prints what that gives:

```text
  M_W tree level        78.909302 GeV   from g*v/2
  M_W loop corrected    80.369072 GeV   from the loop solver
  the corrections move it by 1.459770 GeV
```

A 1.5 GeV gap, roughly two percent, and it is not experimental error. It is the one-loop radiative
corrections, dominated by the top quark running around the loop. The `ρ` parameter is exactly `1` at
tree level and the loops move it by `Δρ ≈ 0.0093`.

Printing both is the point. A run that showed only the corrected number would leave the reader with
no way to see what the correction was worth.

## The tolerance is the claim

```rust
pub const W_MASS_TOLERANCE_GEV: FloatType = const_scalar_from_float!(FloatType, 0.020);
```

Twenty MeV is what a one-loop calculation is worth: the terms left out are two-loop and enter at
roughly that size. The summary claims one-loop accuracy and the check tests for exactly that, so
the two cannot drift apart. A tolerance looser than the accuracy being claimed would pass a
calculation that had stopped agreeing with the measurement.

## What the code demonstrates

Four stages, composed as one `CausalFlow`:

| Stage | Adds |
|---|---|
| unification | the couplings `g` and `g'` from `α_EM` and `θ_W` |
| symmetry breaking | the masses the Higgs vacuum value generates, tree and corrected |
| gauge mixing | the `W`/`Z` mass relation and the `ρ` parameter |
| Z resonance | the widths and the peak cross-section |

`bind_or_error` is what makes the chain a chain: a stage that fails stops the ones after it and
carries its reason to the summary, so no stage reads a state an earlier one never filled.

## Output

```text
Stage 3: gauge boson mixing
  rho tree level     1.000000000   the relation M_W = M_Z cos th_W, exactly
  rho effective      1.009326850   with the loop correction
  d_rho = rho - 1    0.009326850   dominated by the top quark in the loop

  M_W computed          80.369072 GeV
  M_W measured          80.377000 GeV   PDG
  deviation                 7.928 MeV   tolerance 20 MeV
  verdict            inside one-loop accuracy

Stage 4: the Z resonance
  peak energy           91.187600 GeV
  total width            2.511175 GeV   G_Z
  hadronic width         1.756205 GeV   G_had
  invisible width        0.502289 GeV   three neutrino generations
  peak cross-section    41.409896 nb
```

The invisible width is a prediction, not an input. Measuring it at LEP is how the number of light
neutrino generations was established to be three.

The two mixing angles in stage 1 differ — `sin²θ_W = 0.2232` on-shell against `sin²θ_eff = 0.2305` —
because they are defined by different measurements, one by the boson masses and one by the Z decay
asymmetries. At tree level they are the same number, and the gap between them is a loop effect.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`, so no conversion runs at any call site. It sits at `Float106` rather
than `f64` on purpose: a hard-coded `f64` is invisible while the alias *is* `f64`, and a compile
error the moment the two differ.

All four scalars run, and the verdict line is where the difference shows:

| Scalar | Deviation | Verdict |
|---|---|---|
| `f32` | 7.927 MeV | inside one-loop accuracy |
| `f64` | 7.928 MeV | inside one-loop accuracy |
| `Float106` | 7.928 MeV | inside one-loop accuracy |
| `BFloat16` | 500.000 MeV | outside one-loop accuracy |

`BFloat16` carries an eight-bit mantissa, so `80.4 GeV` is resolved to about half a GeV and an
8 MeV difference between two such numbers is below the last bit. The run says so rather than
printing a number it cannot support. That is what the tolerance check is for, and it is the reason
it is stated in the same place as the claim.

## What this example covers

The goal is to reformulate the essence of an electroweak precision test as a causal process over the
library's types, and to get precision as a parameter and categorical composition for free once it is
in that form. The essence is that the theory has almost no freedom and the leftover disagreement is
where new physics would have to live. The model keeps that and holds everything else simple: the
on-shell scheme with the library's packaged one-loop corrections, three fermion generations with
their measured masses, and the `Z` treated as a Breit-Wigner resonance at its peak.

A calculation a precision-electroweak group would publish adds what this leaves out: the full
two-loop corrections to `Δr`, QED and QCD corrections to the widths, initial-state radiation
smearing the resonance shape, a full fit over many observables rather than one prediction, and the
correlated experimental uncertainties on the inputs. The CDF 2022 `M_W` measurement sits several
standard deviations from this prediction and from the other measurements, which is the kind of
question a real analysis exists to settle.

## How to grow the example toward a precision fit

Each step keeps the structure already here.

- **Report an uncertainty.** Carry the inputs as `deep_causality_uncertain` distributions instead of
  numbers, and the deviation becomes a pull rather than a difference.
- **Scan the inputs.** The stages are a `CausalFlow`; running it over a grid of top and Higgs masses
  turns the single prediction into the `M_W`-vs-`m_t` band that precision plots show.
- **A second scheme.** The library already carries both mixing angles. Computing the same
  observables in the `MS-bar` scheme and comparing is a direct check on the scheme dependence.
- **More observables.** The widths and asymmetries are already available on `ElectroweakParams`;
  folding them into one figure of merit makes the run a fit instead of a comparison.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the four stages, and the flow that composes them |
| `model.rs` | the constants, the state the stages thread, and the tolerance |
| `utils_print.rs` | the presentation, and the only `lower` calls |
