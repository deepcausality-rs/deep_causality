## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Rank-controlled body-mask tensor train

The `tensor_bridge` module SHALL provide a way to encode an immersed-body indicator as a
`CausalTensorTrain` on the `2^Lx × 2^Ly` grid: a **smoothed volume-fraction** field `χ_body ∈ [0, 1]`
(1 inside the body, 0 outside, smeared over a few cells) quantized and rounded, so its bond dimension
stays bounded. It SHALL provide a `body_mask_2d` helper for the analytic cylinder, and SHALL report the
resulting bond dimension so the smoothing width can be tuned against rank.

The `χ_body ∈ [0, 1]` invariant SHALL hold for the mask a solver actually consumes, not only for the
mask before truncation. Tensor-train rounding drives the quantized mask outside that range — measured
`min χ = −1.78e-3` across 188 cells at bond cap 4, and `−6.5e-5` across 84 cells at cap 8, two of the
four caps the shipped immersed-cylinder ladder runs — and a negative `χ` inverts the sign of the
penalization forcing, so the term amplifies instead of damping. The invariant SHALL therefore be
established after quantization, by construction, by clamping, or by rejection.

#### Scenario: Cylinder mask is bounded-rank
- **WHEN** a smoothed cylinder mask is built on the grid
- **THEN** it quantizes to a tensor train whose bond dimension is bounded (far below the dense element
  count), and dequantizing recovers the smoothed volume fraction within rounding tolerance

#### Scenario: Sharper masks cost more rank
- **WHEN** the smoothing width is reduced and the mask is quantized at a fixed bond cap
- **THEN** the reconstruction error (vs. the accurately-quantized mask) increases — i.e. a sharper body
  needs more rank to represent at the same fidelity, making the rank/accuracy trade-off explicit
  (the resolution-robust form, since bonds saturate the grid's rank ceiling at a fixed tolerance)

#### Scenario: The consumed mask satisfies its documented range
- **WHEN** a mask is quantized at any bond cap the harnesses run, including the coarsest
- **THEN** the mask the solver consumes satisfies `χ ∈ [0, 1]`, or its construction fails with an error
  naming the violation

#### Scenario: A truncation-induced negative cell cannot reach the forcing term
- **WHEN** rounding at an aggressive bond cap would drive `χ` negative
- **THEN** the negative value does not reach the penalization forcing, so the term cannot change sign
