# qtt-immersed-body Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-immersed-body. Update Purpose after archive.
## Requirements
### Requirement: Rank-controlled body-mask tensor train

The `tensor_bridge` module SHALL provide a way to encode an immersed-body indicator as a
`CausalTensorTrain` on the `2^Lx × 2^Ly` grid: a **smoothed volume-fraction** field `χ_body ∈ [0, 1]`
(1 inside the body, 0 outside, smeared over a few cells) quantized and rounded, so its bond dimension
stays bounded. It SHALL provide a `body_mask_2d` helper for the analytic cylinder, and SHALL report the
resulting bond dimension so the smoothing width can be tuned against rank.

The `χ_body ∈ [0, 1]` invariant SHALL be enforced on the quantized mask **to the extent a lossy tensor
train permits**, and a mask outside the range by more than a stated fraction SHALL be rejected.
Tensor-train rounding drives the quantized mask outside `[0, 1]` — measured `min χ = −1.78e-3` across
188 cells at bond cap 4, and `−6.5e-5` at cap 8 — and a negative `χ` inverts the sign of the
penalization forcing.

**A fixed-rank tensor train cannot represent an arbitrary clamped field exactly**, so clamping the
dequantized mask and re-quantizing removes the bulk of the excursion but reintroduces a smaller one
(measured `−1.78e-3 → −1.21e-3` at bond cap 4): pointwise `[0, 1]` on the *stored* train is not
achievable at a coarse cap. The enforceable contract is therefore:

- the construction SHALL clamp the dequantized mask to `[0, 1]` and re-quantize, removing the gross
  excursion;
- a raw excursion beyond a stated fraction of the range (a wrong mask, not rounding noise) SHALL fail
  construction, naming the violation;
- the residual after clamping SHALL be bounded by the truncation tolerance — noise orders below the
  body value `χ = 1`, not a modelling sign error.

The immersed-cylinder ladder's η sweep (its acceptance test) runs at the **highest** bond cap, at which
the mask is in range to the truncation tolerance — on the shipped `L = 8` ladder `sweep_cap = 48` gives
`min χ ≈ −7e-7`, a bounded truncation-noise negative that reaches the forcing but is orders below the
body value, not a sign-flipping error. Exact non-negativity is not claimed on any cap the ladder runs.

#### Scenario: Cylinder mask is bounded-rank
- **WHEN** a smoothed cylinder mask is built on the grid
- **THEN** it quantizes to a tensor train whose bond dimension is bounded (far below the dense element
  count), and dequantizing recovers the smoothed volume fraction within rounding tolerance

#### Scenario: Sharper masks cost more rank
- **WHEN** the smoothing width is reduced and the mask is quantized at a fixed bond cap
- **THEN** the reconstruction error (vs. the accurately-quantized mask) increases — i.e. a sharper body
  needs more rank to represent at the same fidelity, making the rank/accuracy trade-off explicit
  (the resolution-robust form, since bonds saturate the grid's rank ceiling at a fixed tolerance)

#### Scenario: The consumed mask is in range to the truncation tolerance
- **WHEN** a mask is quantized at any bond cap the harnesses run, including the coarsest
- **THEN** the gross excursion is removed by the construction clamp and the residual is bounded
  truncation noise well within a stated fraction of the `[0, 1]` range

#### Scenario: A grossly out-of-range mask fails construction
- **WHEN** the analytic mask, or its quantization, leaves `[0, 1]` by more than the stated fraction
- **THEN** construction fails with an error naming the excursion, rather than the value being silently
  clamped — distinguishing a wrong mask from rounding noise

#### Scenario: The forcing at the acceptance-test cap sees a mask in range to truncation tolerance
- **WHEN** the η ladder runs at its (highest) bond cap
- **THEN** the mask the penalization forcing consumes there is within `[0, 1]` to the truncation
  tolerance — its residual negative, if any, is bounded truncation noise orders below the body value
  (measured `min χ ≈ −7e-7` at the shipped `L = 8` sweep cap 48), not a sign-flipping modelling error.
  Exact pointwise non-negativity is not claimed: a lossy tensor train cannot hold it, and the
  penalization forcing multiplies the mask train directly with no per-use clamp

### Requirement: Brinkman-penalized no-slip tensor-train marcher

The `solvers/qtt` module SHALL provide a 2-D incompressible marcher that enforces an immersed body by
**volume penalization**: each step SHALL add the forcing `−(1/η)·χ_body ⊙ (u − u_body)` to the velocity
rate (via the fused Hadamard product + round), driving the velocity toward the body velocity inside the
body, then project as the body-free solver does. With no body (or `χ_body = 0`) it SHALL reduce to the
existing `QttIncompressible2d` behavior.

#### Scenario: No-slip is enforced inside the body
- **WHEN** the penalized marcher advances a flow past a static body to a quasi-steady state
- **THEN** the velocity magnitude inside the body region falls toward zero (to the penalization floor),
  while the field stays divergence-free and bond-bounded

#### Scenario: Reduces to the body-free solver
- **WHEN** the body mask is identically zero
- **THEN** the marched result matches `QttIncompressible2d` for the same configuration

### Requirement: Immersed-cylinder validation constrains the reported drag

The `qtt_cylinder_verification` gate set SHALL include at least one gate that constrains the reported drag
coefficient against a parameter it actually depends on. No gate in the present set does: one cannot fail,
one is provably invariant under the parameter that dominates the answer, and one carries eleven orders of
margin.

Specifically the harness SHALL gate a **smoothing-width ladder** and an **η ladder** as first-class checks
alongside the existing bond ladder, and SHALL tighten `CONVERGENCE_BOUND` to the scale of the difference it
measures.

Measured behaviour that motivates this, recorded so the gates are interpretable:

- `C_d` moves **6.1×** with the mask smoothing width — 7.70, 12.33, 23.76, 35.81, 47.27 at 0.5, 1, 2, 3, 4
  cells — while the no-slip gate passes identically across that entire range.
- `C_d` is **non-monotone** in the penalization parameter — 17.39, 24.02, 26.25, 23.76, 21.40 at
  η = 0.128, 0.064, 0.032, 0.016, 0.008 — and is still drifting at the finest η, so no η → 0 limit is
  demonstrated. Establishing that limit is what would license calling the penalization integral a drag
  (Angot, Bruneau & Fabrie 1999).
- `CONVERGENCE_BOUND = 0.10` gates a measured successive difference of `1.89e-11`.

Where a ladder does not converge, the harness SHALL report the non-convergence as its result rather than
passing silently. Establishing the physical η envelope is out of scope here and is handled by the Phase 2
remediation; this requirement makes the harness capable of *detecting* the condition.

#### Scenario: Smoothing-width ladder is gated

- **WHEN** the harness runs its smoothing-width ladder
- **THEN** it reports `C_d` at each width and gates the trend, so a result that scales with a purely
  numerical parameter cannot pass unremarked

#### Scenario: Penalization ladder is gated

- **WHEN** the harness runs its η ladder
- **THEN** it reports `C_d` and the interior slip at each η and gates whether the sequence is converging,
  failing or explicitly reporting non-convergence when it is not

#### Scenario: Bond-convergence bound matches the phenomenon

- **WHEN** the bond ladder completes
- **THEN** the successive-difference bound is set at the scale of the measured differences rather than
  orders of magnitude above them, so a solver that had not saturated in bond would fail

#### Scenario: A parameter-dependent result cannot pass silently

- **WHEN** the reported `C_d` changes materially under a parameter the gate set covers
- **THEN** at least one gate responds to that change, and the harness output records which parameter moved
  the answer

#### Scenario: Cross-references state their configuration

- **WHEN** the harness prints the DEC isolated-cylinder cross-reference
- **THEN** the Reynolds number of both cases is shown — the QTT case runs at `Re = 37.7` against a DEC
  reference at `Re = 100` — and the value is marked as a disclaimed cross-reference, not a gate

### Requirement: The penalization parameter is chosen from a wall-error target

`η` SHALL be chosen from a stated no-slip error target and the resolution constraint relating it to
the grid, not from the explicit-stability ratio. Where a configuration cannot satisfy the constraint,
that SHALL be documented at the configuration site with the factor by which it is violated.

Brinkman penalization converges to the no-slip solution as `η → 0` at `O(η^{3/4})` (Angot, Bruneau &
Fabrie 1999). That limit is what licenses reading the penalization integral as a drag. Two conditions
bound a usable `η`:

- **the wall-error target** — the slip error scales as `√(ην)`, so a target error fixes an upper bound
  on `η`;
- **the resolution constraint** — the penalization layer of thickness `√(ην)` must be resolved by the
  grid, giving `η ≥ dx²/ν`.

The shipped immersed-cylinder configuration satisfies neither in a compatible way: `η = 0.016` gives
`√(ην) = 0.144·dx`, a layer ~7× thinner than one cell, against a resolution criterion of
`η ≥ dx²/ν = 0.771` — violated by **48×**. `η` is instead pinned by `dt/η = 0.25`, an explicit-stability
ratio with no physical content, which is why the measured `C_d` tracks the mask smoothing width rather
than `η`.

Satisfying both conditions at once may require a finer grid or a softer wall. That trade-off SHALL be
stated rather than resolved by leaving the constraint unmentioned.

#### Scenario: The configured η is traceable to a wall-error target

- **WHEN** `η` is defined for a case
- **THEN** the target no-slip error and the resulting bound are recorded at the definition, and `η` is
  not derived from `dt`

#### Scenario: The resolution constraint is stated and its standing reported

- **WHEN** a configuration is documented
- **THEN** it records `√(ην)` against `dx`, the criterion `η ≥ dx²/ν`, and whether the configuration
  satisfies it — including the violation factor when it does not

#### Scenario: A configuration that cannot satisfy both conditions says so

- **WHEN** the wall-error target and the resolution constraint cannot both be met at the affordable
  grid
- **THEN** the conflict is documented with its cost, rather than one condition being silently dropped

#### Scenario: The η ladder is the acceptance test

- **WHEN** the envelope is resolved and `qtt_cylinder_verification` runs
- **THEN** its η ladder converges, which is the condition under which the reported drag has a limit —
  the ladder having been added, and observed failing, before this requirement existed
