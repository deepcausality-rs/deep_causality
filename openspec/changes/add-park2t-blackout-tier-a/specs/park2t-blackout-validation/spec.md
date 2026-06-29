<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Self-verifying blackout verification example

`deep_causality_cfd/verification/` SHALL gain a self-verifying example `qtt_park2t_blackout` in the established
house style (`config.rs` holding only configuration, `main.rs` running the flow + march, `print_utils.rs` doing
the measure/verify, plus `baseline.txt` and `README.md`, mirroring `qtt_taylor_green` and `qtt_cylinder`). It
SHALL be registered in `Cargo.toml` and listed in `verification/README.md`. The example SHALL exit non-zero when
any acceptance gate is broken and emit a human-readable labeled report.

#### Scenario: Example runs and reports
- **WHEN** the verification example is run
- **THEN** it prints a labeled report of the blackout criteria and exits zero only if every gate passes

### Requirement: LER acceptance gates

The verification SHALL gate (exit non-zero on break) on the LER criteria from the resolution notes: (i)
stability at stiffness (`τ = Δt/1000` stays bounded/monotone where explicit Euler diverges); (ii) exactness of
the closed-form exponential on the linear relaxation (equality to round-off); (iii) the Rankine–Hugoniot
temperature magnitude lands peak `T_post` in the ~10⁴ K band at `M ≈ 25`, not the cold isentropic value; (iv)
the ionization lag is real and `τ_ion` is grounded in the dominant rate, with the Saha limit recovered as
`τ → 0`; (v) counterfactual path-dependence (two histories → two blackout outcomes); (vi) ionized species
present so the equilibrium ionization target is nonzero (electron density is not identically zero).

#### Scenario: A broken gate fails the run
- **WHEN** any one of the six gates is violated (e.g. the RH jump is omitted and the temperature is too cold to
  ionize, or `τ_ion` is replaced by a free constant)
- **THEN** the verification exits non-zero and names the failing gate

#### Scenario: All gates pass on the built slice
- **WHEN** the Tier-A slice is run as specified
- **THEN** all six gates pass and the run exits zero

### Requirement: Published reference cross-references with honesty disclaimers

The verification SHALL report cross-references against published reference results — RAM-C II electron density /
blackout onset (and the *Fluid Dynamics* 2022 Park-2T reproduction), Park two-temperature tables, the Saha
equilibrium limit, and Apollo blackout dwell — and SHALL disclaim the Tier-A scope honestly: the slice rides the
incompressible rollout, `T_tr` is a recovery-temperature reconstruction (not a true post-shock thermodynamic
path), and the operator split is first-order. No absolute match to a coupled-CFD result is claimed where the
configuration differs; cross-references are reported as such.

#### Scenario: References reported, not overclaimed
- **WHEN** the verification reports its results
- **THEN** each published reference is shown as a cross-reference with its tolerance and an explicit Tier-A
  disclaimer, and any quantity that is a Tier-A reconstruction is labeled as such

### Requirement: Gap-2 Tier-A closure recorded

On completion the plasma-blackout notes SHALL be updated to mark **Gap 2 Tier-A closed** —
`gap-analysis.md` §4 Gap 2 and the gap-2 note §6 — with Tier-B (compressible QTT shock-capturing marcher,
reacting `*_rhs`, multi-mode relaxation, shock-rank control) explicitly retained as **open**.

#### Scenario: Notes reflect closure
- **WHEN** the change is implemented and verified
- **THEN** the gap analysis marks Tier-A of Gap 2 closed and still labels the Tier-B items open
