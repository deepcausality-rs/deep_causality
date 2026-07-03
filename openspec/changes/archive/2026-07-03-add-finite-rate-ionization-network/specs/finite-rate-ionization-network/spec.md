## ADDED Requirements

### Requirement: A two-way finite-rate ionization network with detailed balance by construction

`deep_causality_physics` SHALL provide the finite-rate ionization network for RAM-C-class entries as pure
pointwise kernels in the existing `kernels/hypersonic/` domain: (1) associative ionization
`N + O -> NO+ + e-` forward and its **dissociative-recombination** reverse, (2) thresholded electron-impact
ionization of N and O rated at the electron temperature, and (3) a **lagged** neutral atom pool (`N`, `O`) closing the reactant concentrations: atom
fractions relax toward their dissociation equilibria with rate clocks built from the same table
(`tau_O = 1/(k_d_O2[M])`; `tau_N = 1/(k_d_N2[M] + k_z[O])`, where `k_z` is the Zeldovich exchange
`N2 + O -> NO + N`, Table II reaction 6, the low-activation N-production path), so the pool
inherits the same rate-versus-residence honesty as the electrons. All rates SHALL come from the
verified RP-1232 Table II pairs (the source tabulates the backward rate of each forward rate
through its own detailed-balance relation, eq. 5a, valid in this velocity range), so the
network's fixed point recovers the source-consistent equilibrium in the thermal-equilibrium
limit by construction. Every rate SHALL run at its controlling temperature: ionization at the
calibrated geometric mean `sqrt(T_tr * T_ve)`, **dissociation at Park's classic
`T_tr^0.7 * T_ve^0.3`** (the published Park-lineage exponent for the Park rate set, a citation
rather than a fit), electron-involving channels at `T_e = T_ve` (the recorded lever-2 insight). Citations SHALL appear in the kernel docstrings with the source PDF in
`deep_causality_physics/papers/`; the only float literals are constants of nature and the cited Park
coefficients in `constants/`.

#### Scenario: Detailed balance is an identity in the thermal-equilibrium limit
- **WHEN** the network is relaxed to its fixed point from an under-ionized and from an over-ionized initial
  fraction at the same **thermally equilibrated** state (`T_tr = T_ve`, so every channel's rating
  temperature coincides)
- **THEN** both approaches land on the same equilibrium electron density, and that equilibrium equals the
  value implied by the Park equilibrium-constant fit at that temperature within rounding tolerance

#### Scenario: Two-temperature states depart from single-temperature equilibrium by design
- **WHEN** the network is evaluated at a genuine two-temperature state (`T_ve < T_tr`, channels rated per
  their controlling temperatures)
- **THEN** the fixed point differs from the single-temperature equilibrium at either temperature, and no
  test forces single-temperature behavior onto a two-temperature state

#### Scenario: The frozen limit is frozen
- **WHEN** the network is evaluated at a temperature where every forward channel's Arrhenius factor vanishes
- **THEN** the relaxation time exceeds any marching step by orders of magnitude and the carried fraction is
  left unchanged (no spurious jump to equilibrium, matching the LER frozen-chemistry contract)

#### Scenario: Recombination is a real loss channel
- **WHEN** a carried electron population enters a cold dense region (post-peak descent conditions)
- **THEN** dissociative recombination decays it toward the (low) local equilibrium instead of freezing it at
  its hot-region value

### Requirement: An LER-native network stage on the evolved carrier state

`deep_causality_cfd` SHALL provide a `FiniteRateIonizationStage` as a sibling of the existing
`IonizationStage` in the same `PhysicsStage` slot: per cell, it SHALL read the evolved controller and
electron temperatures and the evolved per-cell heavy-particle density, relax the carried ionization fraction
toward the network's closed-form fixed point with the two-way clock `tau = 1/(k_f[M] + beta * n_e)` through
the shipped LER kernel (no stiff ODE integrator, no per-cell iteration), and write `"alpha"` and `"n_e"`.
The stage SHALL offer the same optional sheath-renewal mode as the surrogate stage so the renewal A/B is a
configuration toggle, and the same per-cell/broadcast/config-fallback input resolution. The existing
`IonizationStage` SHALL remain unchanged in behavior and contract.

#### Scenario: The stage runs on evolved fields
- **WHEN** the stage executes in the compressible corridor coupling
- **THEN** every rate and every equilibrium target is evaluated from the per-cell evolved state
  (`T_a`, `T_ve`, `n_tot`), and removing those fields falls back to the configured constants exactly as the
  surrogate stage does

#### Scenario: The stagnation-line age is a field, not a number
- **WHEN** the network is validated on the stagnation line
- **THEN** the parcel age is the transit-age profile `age(xi) = t_res * ln(1/(1-xi))` implied by
  the linear stagnation-line deceleration (geometry and the Rankine-Hugoniot state only, no free
  parameter), and the anchor gate reads the profile's **peak**, matching what the flight
  reflectometers measured

#### Scenario: The surrogate path is untouched
- **WHEN** the existing QTT corridor and the archived examples run
- **THEN** their `IonizationStage` behavior is bit-identical to before this change
