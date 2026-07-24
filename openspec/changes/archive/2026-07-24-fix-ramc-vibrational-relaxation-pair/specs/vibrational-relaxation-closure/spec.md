## ADDED Requirements

### Requirement: The relaxation reduced mass is derived from a named collision pair

The Millikan–White reduced mass `μ_sr` SHALL be derived from the constituent masses of an explicitly
named collision pair, never written as a bare literal. The relaxing species `s` MUST be a diatomic —
a monatomic species has no vibrational mode and cannot be the relaxing partner — and the derivation
`μ_sr = m_s·m_r / (m_s + m_r)` MUST be visible where the value is defined, with each constituent mass
stated in amu.

This exists because the defect it replaces was not a wrong number so much as an unchecked one. The
committed `REDUCED_MASS_AMU = 7.0` carried the comment `N₂–N₂ ≈ 14·14/28 = 7`, which uses nitrogen's
*atomic* mass 14 where N₂'s molecular mass 28 belongs. 7.00 amu is the N–N pair — two nitrogen atoms,
not a vibrational-relaxation pair at all. No candidate pair yields it: N₂–N₂ = 14.00, N₂–O₂ = 14.93,
N₂–O = 10.18, N₂–N = 9.33. Nothing in the codebase related the constant to a pair, so an arithmetic
slip in a comment became the physics.

#### Scenario: The pair and its masses are recoverable from the definition

- **WHEN** a reader opens the site where `μ_sr` is defined
- **THEN** the collision pair is named, both constituent masses are stated in amu, and the reduced-mass
  formula relating them to the value is present

#### Scenario: A monatomic relaxing species is rejected

- **WHEN** the named relaxing species `s` has no vibrational mode
- **THEN** the pair is rejected as invalid rather than accepted with a computed `μ`

#### Scenario: The derived value matches the named pair

- **WHEN** `μ_sr` is checked against `m_s·m_r / (m_s + m_r)` for the named pair
- **THEN** they agree, and a test pins the relationship so a future edit to one without the other fails

### Requirement: The collision pair is justified against the flight condition

The choice of collision partner `r` SHALL be justified against the conditions the closure is applied
at, and the alternatives considered SHALL be recorded. Selecting the partner is a physical judgement,
not a default: at RAM-C post-shock conditions the air is partially dissociated, so N₂–N₂ is not
automatically dominant, and N₂–N (`μ = 9.33`), N₂–O (`μ = 10.18`) and N₂–O₂ (`μ = 14.93`) are live
alternatives that move `τ_vt` materially.

Where a single pair is used to stand in for a mixture, that simplification SHALL be stated as a
modelling choice with its expected direction of error, alongside the closure's other recorded
limitations.

#### Scenario: The selection rationale is recorded

- **WHEN** the closure's documentation is read
- **THEN** it names the chosen pair, states why that partner dominates at the applied condition, and
  lists the alternatives with their reduced masses

#### Scenario: A single-pair stand-in is labelled as such

- **WHEN** one pair represents a chemically mixed bath
- **THEN** the documentation says so and states which way the simplification biases `τ_vt`

### Requirement: The constant has exactly one definition

`μ_sr` SHALL have a single definition in the workspace. Consumers — the verification harness, the
shared example world, and the closure's own documentation — SHALL refer to that definition rather than
restating the value.

The constant was previously defined independently in
`deep_causality_cfd/verification/qtt_ramc_stagline/config.rs` and
`examples/avionics_examples/src/shared/constants.rs`, both at 7.0 and both labelled N₂–N₂. That
duplication is how one arithmetic slip reached both the verification harness and all three
plasma-blackout examples.

#### Scenario: Correcting the value corrects every consumer

- **WHEN** `μ_sr` is changed at its definition
- **THEN** every consumer observes the new value with no second edit, and no site restates the literal

#### Scenario: Doc comments do not restate the number

- **WHEN** `Park2tClosure::reduced_mass_amu` and the blackout stage document the parameter
- **THEN** they describe what it is and point at the definition, rather than quoting a value that can
  drift from it

### Requirement: The closure carries its citation

The Millikan–White correlation SHALL be cited where it is implemented, with the form of `A_sr`, `B_sr`
and the `−18.42` constant recorded together with the units each term expects (`μ` in amu, `p` in atm,
`τ` in seconds, `θ_v` in K).

#### Scenario: The implemented form is checkable against the source

- **WHEN** a reviewer compares the implementation against the cited correlation
- **THEN** every coefficient and the unit convention are stated in the code, so agreement can be
  checked without reconstructing the derivation
