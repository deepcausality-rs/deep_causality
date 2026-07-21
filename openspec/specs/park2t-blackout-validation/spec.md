# park2t-blackout-validation Specification

## Purpose
TBD - created by archiving change add-park2t-blackout-tier-a. Update Purpose after archive.
## Requirements
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
stability at stiffness (`τ = Δt/1000` stays bounded/monotone where explicit Euler diverges); (ii) the
closed-form relaxation kernel agrees with an **independently-derived** reference — a converged
sub-stepped integration of `dx/dt = (x_eq − x)/τ`, compared within a documented tolerance, never an
equality check against a re-transcription of `ler_step`'s own body; (iii) the Rankine–Hugoniot
temperature magnitude lands peak `T_post` in the ~10⁴ K band at `M ≈ 25`, not the cold isentropic value;
(iv) the ionization lag is real and `τ_ion` is grounded in the dominant rate, and the lag reproduces the
qualitative electron-density **overshoot** signature (`n_e` rising above its local-equilibrium value on a
rise-then-relax temperature history; Lin et al. 1962); (v) counterfactual path-dependence (two histories →
two blackout outcomes); (vi) ionized species present so the equilibrium ionization target is nonzero
(electron density is not identically zero).

Gate (ii) previously required "exactness of the closed-form exponential on the linear relaxation (equality
to round-off)". As implemented that compared `ler_step(x, x_eq, tau, dt)` against a character-for-character
restatement of its own definition at the same monomorphization, so the two sides were bit-identical by
construction and the gate could not fail. It is replaced by the independent-reference form above.

Gate (iv) previously also required "the Saha limit recovered as `τ → 0`". As implemented that invoked the
`tau = 0` branch, which is an explicit early `return x_eq`, and then asserted the result equalled `x_eq` —
a check of a hard-coded return statement. That conjunct is removed; the remaining conjuncts of gate (iv)
are retained because they are falsifiable.

Every bound used by these gates SHALL declare its evidence class, and no bound pinned from this code's own
measurement SHALL be presented as agreement with published two-temperature data.

#### Scenario: A broken gate fails the run
- **WHEN** any one of the six gates is violated (e.g. the RH jump is omitted and the temperature is too cold to
  ionize, or `τ_ion` is replaced by a free constant)
- **THEN** the verification exits non-zero and names the failing gate

#### Scenario: All gates pass on the built slice
- **WHEN** the Tier-A slice is run as specified
- **THEN** all six gates pass and the run exits zero

#### Scenario: Gate (ii) fails when the relaxation kernel is wrong
- **WHEN** `ler_step` is mutated — for example the sign of the exponent is flipped, or `τ` is scaled
- **THEN** gate (ii) reports FAIL against the independently sub-stepped reference and the run exits
  non-zero, where the previous transcription form would have continued to pass

#### Scenario: Gate count matches the gates that carry information
- **WHEN** the harness prints its gate block
- **THEN** the number of gates it advertises equals the number that can fail, and each carries its
  evidence class

### Requirement: Published reference cross-references with honesty disclaimers

The verification SHALL report cross-references against published reference results — RAM-C II electron density /
blackout onset (and the *Fluid Dynamics* 2022 Park-2T reproduction and the Aiken–Carter–Boyd 2025 review), Park
two-temperature tables, the Saha equilibrium limit, and Apollo blackout dwell — and SHALL disclaim the Tier-A
scope honestly: the slice rides the incompressible rollout, `T_tr` is a recovery-temperature reconstruction (not
a true post-shock thermodynamic path), and the operator split is first-order. It SHALL additionally name the
known **quantified** physics biases of the Tier-A closure: the two-temperature (`T_ve = T_e`) lumping that
over-predicts peak `n_e` by ~2× (Farbar–Boyd–Martin 2013; the 3T fix is an LER-native deferral), the
non-Maxwellian-EEDF error in weakly-ionized air, and the Gupta-vs-Park rate-set sensitivity. No absolute match
to a coupled-CFD result is claimed where the configuration differs; cross-references are reported as such.

#### Scenario: References reported, not overclaimed
- **WHEN** the verification reports its results
- **THEN** each published reference is shown as a cross-reference with its tolerance and an explicit Tier-A
  disclaimer, any quantity that is a Tier-A reconstruction is labeled as such, and the named quantified biases
  (the ~2× two-temperature lumping, non-Maxwellian EEDF, rate-set sensitivity) are reported as the reason the
  `n_e` tolerance is what it is

### Requirement: Gap-2 Tier-A closure recorded

On completion the plasma-blackout notes SHALL be updated to mark **Gap 2 Tier-A closed** —
`gap-analysis.md` §4 Gap 2 and the gap-2 note §6 — with Tier-B (compressible QTT shock-capturing marcher,
reacting `*_rhs`, multi-mode relaxation, shock-rank control) explicitly retained as **open**.

#### Scenario: Notes reflect closure
- **WHEN** the change is implemented and verified
- **THEN** the gap analysis marks Tier-A of Gap 2 closed and still labels the Tier-B items open

### Requirement: Uncalibrated stagnation-line prediction

The stagnation-line verification (`qtt_ramc_stagline`) SHALL be re-measured with the finite-rate network and
**no Saha calibration target**: the peak electron density SHALL be a prediction from the Park rate data and
the evolved thermodynamic state alone. The acceptance band SHALL be pinned from that measurement against the
RAM-C II anchor and justified against the production-code context (DPLR, LAURA, US3D land 2x to 3x on this
anchor with a 2x to 5x chemistry-model spread); the expectation is the ~3x band. The previously calibrated
surrogate result (the 12x-to-1.1x lever-1 history) SHALL remain recorded as history, not overwritten. If the
uncalibrated prediction misses the production band and the miss traces to the partial-equilibrium atom pool,
the designed response is the rated-dissociation promotion, not a re-calibration.

#### Scenario: The anchor is predicted, not reproduced
- **WHEN** the stagnation-line verification runs with the network
- **THEN** no calibrated equilibrium target enters the computation, the peak `n_e` is compared against the
  RAM-C II anchor, and the report labels the band as an uncalibrated prediction with its measured value

#### Scenario: The sheath-renewal A/B is re-run under a loss channel
- **WHEN** the network (with dissociative recombination) is run with and without explicit sheath renewal
- **THEN** both peak values are measured and recorded, the mode matching the stagnation-line closure is
  kept, and the decision with both numbers is written into the example's model labels, superseding the
  first A/B's record
