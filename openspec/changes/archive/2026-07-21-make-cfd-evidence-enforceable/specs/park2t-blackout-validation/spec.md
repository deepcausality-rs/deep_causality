## MODIFIED Requirements

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
