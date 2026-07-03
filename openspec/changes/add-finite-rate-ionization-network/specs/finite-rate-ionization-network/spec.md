## ADDED Requirements

### Requirement: A two-way finite-rate ionization network with detailed balance by construction

`deep_causality_physics` SHALL provide the finite-rate ionization network for RAM-C-class entries as pure
pointwise kernels in the existing `kernels/hypersonic/` domain: (1) associative ionization
`N + O -> NO+ + e-` forward and its **dissociative-recombination** reverse, (2) thresholded electron-impact
ionization of N and O rated at the electron temperature, and (3) a **lagged** neutral atom pool (`N`, `O`) closing the reactant concentrations: atom
fractions relax toward their dissociation equilibrium at the controller temperature with the
dissociation-rate clock `tau_pool = 1/(k_d[M])`, so the pool inherits the same
rate-versus-residence honesty as the electrons (an equilibrium pool would over-predict the
reactants behind the shock, where dissociation is the slowest relaxing process). All forward rates and all
equilibrium constants SHALL come from the Park (1990) rate tables and curve fits, with the backward rates
derived inside the kernels as `k_b = k_f / K_eq`, never independently fitted, so the network's fixed point is
the thermodynamic equilibrium at every temperature by construction. Heavy-particle channels SHALL be rated at
the Park controller `T_a = sqrt(T_tr * T_ve)`; electron-involving channels SHALL be rated at `T_e = T_ve`
(the recorded lever-2 insight). Citations SHALL appear in the kernel docstrings with the source PDF in
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

#### Scenario: The surrogate path is untouched
- **WHEN** the existing QTT corridor and the archived examples run
- **THEN** their `IonizationStage` behavior is bit-identical to before this change
