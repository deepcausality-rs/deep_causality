# deep_causality_cfd — reacting-plasma / blackout physics (blackout.rs, finite_rate_ionization.rs, corridor/{regime,envelope,gate,branch}.rs, Park2tClosure in solvers/qtt/compressible/fitting.rs) and the deep_causality_physics kernels + constants they call

**Production readiness: `not-ready`**

The plumbing is unusually clean — the plasma-frequency kernel, the Rankine–Hugoniot jump, the Saha quadratic, the Millikan–White functional form, the Knudsen bands, the network fixed-point quadratic, and the rad/s-vs-rad/s comms comparison (COMMS_BAND_RAD_S = 9.899e9 = 2π·1.57542 GHz) are all algebraically correct, and I reproduced the shipped qtt_ramc_stagline baseline to 4 significant digits from an independent Python re-implementation, which means the code does what its own equations say. The problem is the equations' inputs. The Millikan–White reduced mass is 7.0 amu labelled "N₂–N₂"; the reduced mass of an N₂–N₂ pair is 14 amu (7 is the N-atom pair), and the shipped docstring's own arithmetic "14·14/28 = 7" uses atomic-nitrogen masses for a molecular collision. Setting μ = 14 moves the crate's flagship calibrated result from n_e = 1.085e19 (+0.04 dec, the "+0.0 decades vs the RAM-C II anchor" headline) to 5.32e17 (−1.27 dec), which fails the crate's own NE_LO = 3e18 gate; the independent check that μ = 14 is correct is that it reproduces the literature pτ(N₂–N₂, 300 K) ≈ 3.7e3 atm·s while μ = 7 gives 3.1 atm·s, three orders too fast. Separately, the "uncalibrated network" prediction is not converged: its peak is read off the deepest sample of a 64-cell transit-age profile whose exposure is t_res·ln(cells+1), so the reported +0.48 dec becomes +0.58 at 256 cells, +0.68 at 4096 and +0.72 at 65536 — outside the ±0.70 acceptance band whose own docstring says it was "pinned from the measurement". That combination — a wrong physical constant carrying the headline agreement, a resolution-dependent prediction, and a gate band admittedly back-fitted to that prediction — is disqualifying for a pre-certification bar. The corridor safety gate and the classifier are in better shape but have real enforcement gaps (descent-rate and propellant axes silently inert without a throttle channel; negative throttle unbounded).

- Files read: **36**
- Findings raised: **15** — surviving adversarial verification: **15** (refuted: 0)
- Surviving by severity: critical 1, major 3, minor 11
- Independently confirmed-correct items: **13**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Angular plasma frequency kernel and its constants | `deep_causality_physics/src/kernels/mhd/plasma.rs:125` | ω_p = sqrt(n_e e²/(ε₀ m_e)) rad/s (Chen, Introduction to Plasma Physics, eq. 4-25). CODATA: e = 1.602176634e-19 C (exact), ε₀ = 8.8541878188e-12 F/m, m_e = 9.1093837139e-31 kg. |
| GNSS cutoff criterion is unit-consistent (rad/s vs rad/s, no missing 2π) | `deep_causality_cfd/src/types/flow/blackout.rs:513` | A wave at f is reflected when f_p > f_signal, equivalently ω_p > ω_signal. GPS L1 = 1.57542 GHz → ω = 2π·1.57542e9 = 9.8987e9 rad/s. |
| Millikan–White τ_vt functional form and the −18.42 constant | `deep_causality_physics/src/kernels/hypersonic/thermochemistry.rs:98-100` | Millikan & White, J. Chem. Phys. 39:3209 (1963), as rearranged in Park (1990): τ_sr·p = exp[A_sr(T^(−1/3) − 0.015 μ^(1/4)) − 18.42], A_sr = 1.16e−3 μ^(1/2) θ_v^(4/3), p in atm, τ in s. |
| Rankine–Hugoniot normal-shock temperature ratio | `deep_causality_physics/src/kernels/hypersonic/shock.rs:60-62` | T₂/T₁ = [2γM² − (γ−1)][(γ−1)M² + 2] / [(γ+1)² M²] (Anderson, Modern Compressible Flow, eq. 3.60). |
| Saha quadratic solution and the 3/2 exponent | `deep_causality_physics/src/kernels/hypersonic/ionization.rs:87-92` | n_e n_i/n_n = (2 g_i/g_n)(2π m_e k_B T/h²)^(3/2) exp(−E_i/k_B T); with n_e=n_i=αn, n_n=(1−α)n this is α²/(1−α) = K/n, so α = (−x + sqrt(x²+4x))/2, x = K/n. |
| Park rate-controlling temperature T_q = T_tr^q · T_v^(1−q), with q=1/2 for ionization and q=0.7 for dissociation | `deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:324` | Park (1990), Nonequilibrium Hypersonic Aerothermodynamics: T_a = T^q T_v^(1−q); the geometric mean q = 1/2 and the classic dissociation q = 0.7. |
| Finite-rate network fixed point and its quadratic solution | `deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:242-243` | Steady state of dn_e/dt = p + k_lin·n_e − β·n_e² under quasi-neutrality: β x² − k_lin x − p = 0, positive root x* = (k_lin + sqrt(k_lin² + 4βp))/(2β). |
| Dissociation-equilibrium closed form and the K = k_f/k_b detailed-balance construction | `deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:191` | A₂ ⇌ 2A at fixed nuclei density n: [A]²/[A₂] = K with [A₂] = (n − [A])/2 ⇒ 2[A]² + K[A] − K n = 0 ⇒ [A] = (−K + sqrt(K² + 8Kn))/4. |
| Knudsen number and the standard rarefaction bands | `deep_causality_cfd/src/types/flow/corridor/regime.rs:179-181, 266-276` | Kn = λ/L; continuum Kn < 0.01, slip 0.01–0.1, transitional 0.1–10, free-molecular Kn > 10 (Bird, Molecular Gas Dynamics; Schaaf & Chambre classification). Hard-sphere λ = 1/(√2 π d² n). |
| Dynamic thrust-coefficient ceiling in the safety gate | `deep_causality_cfd/src/types/flow/corridor/gate.rs:275` | C_T = T/(q∞·S_ref) with T = throttle·T_full; the throttle at which C_T reaches max_ct is throttle* = max_ct·q∞·S_ref/T_full. |
| Bank-rotated lift decomposition in the 3-DOF aero producer | `deep_causality_cfd/src/types/flow/corridor/branch.rs:198-206` | Bank-to-turn point mass: D along −v̂, L of magnitude (L/D)·D rotated about v̂ by bank φ in the plane spanned by the velocity-normal radial n̂ and b̂ = v̂ × n̂: a_L = L(cos φ n̂ + sin φ b̂). |
| RP-1232 Table II rate coefficients as shipped | `deep_causality_physics/src/constants/hypersonic.rs:38-46, 108-176, 353-361` | Gupta, Yos, Thompson & Lee, NASA RP-1232 (1990), Table II (Dunn–Kang-lineage 11-species air set): N+O→NO⁺+e⁻ 9.03e9 T^0.5 exp(−32400/T); NO⁺+e⁻→N+O 1.80e19 T^−1; O+e→O⁺+2e 3.6e31 T^−2.91 exp(−158000/T |
| Characteristic vibrational temperatures and the NO ionization energy | `deep_causality_physics/src/constants/hypersonic.rs:47, 88, 92, 96` | θ_v = 1.4388·ω_e[cm⁻¹]: N₂ ω_e = 2358.6 → 3393 K; O₂ ω_e = 1580.2 → 2274 K; NO ω_e = 1904.2 → 2740 K (Huber & Herzberg). NO first ionization energy 9.2642 eV (NIST). |

## Findings

### 10.1 [CRITICAL] Millikan–White reduced mass is 7.0 amu for a pair documented as N₂–N₂; the correct value is 14 amu, and the error is what produces the crate's headline RAM-C agreement

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:72`
- **Auditor confidence:** confirmed

**Claim.** μ_sr = 7.0 amu is wrong for an N₂–N₂ collision pair. The reduced mass of two 28-amu molecules is 28·28/56 = 14 amu; 7 amu is the N-atom/N-atom pair. Because A_sr ∝ μ^(1/2) and the μ^(1/4) offset sits inside the exponent, this makes τ_vt roughly 1.9× too short, so T_ve over-relaxes, T_a is too hot, and n_e is over-predicted by ~1.3 decades. The flagship 'peak n_e = 1.08e19, +0.0 decades vs the RAM-C II anchor 1e19' result exists because of this error.

**Code evidence.**

```
fitting.rs:72  `/// Reduced mass `μ_sr` of the dominant relaxing collision pair, in **amu** (N₂–N₂ ≈ 7).`
blackout.rs:160 `/// in atm, the reduced mass of the dominant colliding pair in amu (N₂-N₂ ≈ 7), the`
examples/avionics_examples/src/shared/constants.rs:127-128
  `/// Reduced mass of the dominant relaxing collision pair, amu (N₂-N₂).`
  `pub const REDUCED_MASS_AMU: f64 = 7.0;`
verification/qtt_ramc_stagline/config.rs:35-37
  `/// Reduced mass `μ_sr` of the dominant relaxing collision pair (N₂–N₂ ≈ 14·14/28 = 7), in amu — sets the`
  `pub const REDUCED_MASS_AMU: f64 = 7.0;`
consumed at thermochemistry.rs:98 `let a_sr = a * reduced_mass_amu.powf(half) * theta_vib.powf(four_thirds);`
```

**Reference form.** Millikan & White (1963): μ_sr is the reduced mass of the colliding pair in amu. For N₂–N₂, m₁ = m₂ = 28 amu ⇒ μ = 28·28/(28+28) = 14 amu. The docstring's own arithmetic '14·14/28 = 7' substitutes atomic-nitrogen masses (14 amu) into a molecule–molecule pair. Independent check: MW predicts pτ(N₂–N₂, 300 K) with μ = 14 as exp(1.16e-3·√14·3393^(4/3)·(300^(-1/3) − 0.015·14^(1/4)) − 18.42) = 3.70e3 atm·s, consistent with the measured N₂ self-relaxation time of order 10³–10⁴ atm·s; with μ = 7 the same formula gives 3.12 atm·s, three orders too fast.

**Impact.** I re-implemented the exact stagnation-line code path in Python and reproduced the shipped baseline to 4 significant digits (T₂ = 8043.6 K, ρ₂/ρ₁ = 20.349, n₂ = 2.6453e22, t_res = 2.0216e-5 s, α = 4.1013e-4, n_e = 1.0849e19, ω_p = 1.8582e11 — baseline.txt reports 8044, 20.349, 2.645e22, 2.022e-5, 4.101e-4, 1.085e19, 1.858e11). Changing only μ from 7 to 14: τ_vt 1.869e-5 → 3.516e-5 s, T_ve 5401 → 3658 K, T_a 6591 → 5424 K, α 4.10e-4 → 2.01e-5, n_e 1.085e19 → 5.32e17, i.e. +0.04 dec → −1.27 dec versus the RAM-C II anchor. That FAILS the crate's own gate `NE_LO = 3.0e18` (verification/qtt_ramc_stagline/print_utils.rs:20, 64-67). The finite-rate network arm moves 2.991e19 → 2.252e19 (+0.48 → +0.35 dec) and the corridor's peak-passage n_e would shift comparably. An avionics engineer reading 'peak n_e = 1.08e19, +0.0 decades vs the RAM-C II anchor' would conclude the two-temperature closure is validated against flight data; it is not — the agreement is produced by a reduced mass that is half the physical value.

**Recommended fix.** Set REDUCED_MASS_AMU = 14.0 in examples/avionics_examples/src/shared/constants.rs:128 and verification/qtt_ramc_stagline/config.rs:37, and correct the four docstrings (fitting.rs:72, blackout.rs:160, and the two constants) to read 'N₂–N₂ ⇒ μ = 28·28/56 = 14'. Then re-derive the acceptance gates from the corrected physics rather than re-tuning them: the −1.27 dec result at μ = 14 is the honest statement of where the current closure lands, and the remaining gap should be attributed (T_e = T_ve lumping, effective γ, the exposure choice) rather than absorbed. Add a unit test pinning pτ(N₂–N₂, 300 K, 1 atm) to the literature 10³–10⁴ atm·s range — that single assertion would have caught this.

**Adversarial check.** Every cited site exists verbatim. The reference form is right: μ(N₂–N₂) = 28·28/56 = 14 amu; 7 amu is the N–N atomic pair, and atoms have no vibrational mode, so no legitimate reading of the code's own documented pair yields 7. Independent cross-check that does not depend on the auditor's Python: Park's tabulated Millikan–White prefactor for N₂–N₂ is A_sr = 221. With μ = 14 and θ_v = 3393 the shipped formula (thermochemistry.rs:98, a_sr = 1.16e-3·μ^0.5·θ^(4/3)) gives 1.16e-3·3.742·5.10e4 = 221.4 — the published value. With μ = 7 it gives 156.5, which matches no tabulated pair. The 300 K sanity check also reproduces: μ=14 → pτ ≈ 3.8e3 atm·s (physical for N₂ self-relaxation), μ=7 → 3.1 atm·s (three decades too fast). Nothing downstream compensates: reduced_mass_amu is passed straight through wrappers.rs and used only at thermochemistry.rs:98–99, and the sign/direction of the error is unambiguous (τ_vt too short → T_ve too hot → T_a too hot → n_e over-predicted). I could not independently re-run the stagnation line, so the precise claim that the calibrated arm lands at −1.27 dec with μ=14 is unverified; the wrong constant and its direction are not.

> Evidence re-read: deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:72-73 `/// Reduced mass `μ_sr` ... (N₂–N₂ ≈ 7).` / `pub reduced_mass_amu: R`; verification/qtt_ramc_stagline/config.rs:35-37 `(N₂–N₂ ≈ 14·14/28 = 7)` / `pub const REDUCED_MASS_AMU: f64 = 7.0;`; examples/avionics_examples/src/shared/constants.rs:127-128; src/types/flow/blackout.rs:160; deep_causality_physics/src/kernels/hypersonic/thermochemistry.rs:98-100 (a_sr, exponent, tau = exponent.exp()/pressure_atm — no correction elsewhere)

---

### 10.2 [MAJOR] The 'uncalibrated network' peak n_e is not converged: it grows monotonically with the transit-age profile resolution, and the ±0.70-decade band it is gated against was pinned from the 64-cell value

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:54`
- **Auditor confidence:** confirmed

**Claim.** The reported network n_e is bounded (it converges logarithmically to the network fixed point), but its value at the shipped 64 cells is set by a discretization parameter with no physical cutoff criterion, and the ±0.70-dec band gating it is documented twice as pinned from that same measurement. Whether the converged value breaches the band is unverified.

**Code evidence.**

```
verification/qtt_ramc_stagline/main.rs:128-135
  `let profile_cells = 64_usize;`
  `let xi = j as FloatType / (profile_cells as FloatType + 1.0);`
  `let age = residence_time * (1.0 / (1.0 - xi)).ln();`
verification/qtt_ramc_stagline/config.rs:51-54
  `/// Acceptance band of the uncalibrated finite-rate network prediction, in decades around the`
  `/// flight anchor. Pinned from the measurement recorded in baseline.txt; the production-code`
  `/// context (DPLR/LAURA/US3D at 2x to 3x, rate-set spread 2x to 5x) justifies the width.`
  `pub const NETWORK_BAND_DECADES: f64 = 0.7;`
print_utils.rs:116-123 gate text: `"full network {:+.2} dec vs the flight anchor (band +-{:.2} dec, pinned from the measurement; production codes sit at 2x to 3x)"`
examples/avionics_examples/src/shared/constants.rs:141
  `pub const SHEATH_PEAK_AGE_S: f64 = RESIDENCE_TIME_S * 4.174;`   // 4.174 = ln(65), i.e. the 64-cell artifact hard-coded into the production corridor
```

**Reference form.** A reported physical prediction must be independent of a purely numerical discretization parameter, and an acceptance band must come from a source independent of the measurement it gates (here: the cited DPLR/LAURA/US3D spread of 2×–3×, i.e. ±0.30–0.48 dec, or the stated rate-set spread of 2×–5×, i.e. ±0.30–0.70 dec).

**Impact.** Re-running my validated re-implementation of the network stage at other resolutions (identical in every other respect; it reproduces the shipped 64-cell values x_N = 4.617e-01, x_O = 6.364e-01, n_e = 2.9908e19 exactly): cells = 16 → +0.247 dec; 64 → +0.476; 256 → +0.582; 1024 → +0.641; 4096 → +0.677; 65536 → +0.715; 1048576 → +0.732. At 65,536 cells the gate `dec_network.abs() <= 0.7` FAILS. The shipped 'PASS' is therefore a property of the choice of 64, not of the chemistry. Worse, the corridor bakes this in: SHEATH_PEAK_AGE_S = t_res·4.174 hard-codes ln(65) as a *physical* sheath exposure in the production example, so a diagnostic's grid count is now a load-bearing constant in the flagship run. Meanwhile the band it is checked against is documented, twice, as 'pinned from the measurement', and at ±0.70 dec (a factor of 5) it is wider than the stated DPLR/LAURA/US3D context of 2×–3×.

**Recommended fix.** Replace 'peak over the sampled profile' with a resolution-independent observable — for example the mass- or path-averaged n_e over the sheath, or n_e at a physically specified depth (the reflectometer sampling depth), or an explicit exposure cap justified by the boundary-layer thickness where the linear u(ξ) ≈ u₂(1−ξ) model stops holding. Add a convergence gate that asserts the reported peak changes by less than X% between profile_cells = N and 4N. Derive NETWORK_BAND_DECADES from the cited rate-set spread alone and state the derivation, removing the phrase 'pinned from the measurement'; if the corrected prediction then falls outside, report that as the result. Replace the literal 4.174 in shared/constants.rs:141 with a named exposure that has its own physical justification.

**Adversarial check.** All structural facts check out in source. profile_cells = 64 is a bare literal with no convergence argument; age(ξ) = t_res·ln(1/(1−ξ)) with ξ = j/(cells+1) so the deepest sample's exposure is t_res·ln(cells+1), i.e. set by the discretization; and the reported ne_network is `peak` over the profile, which is necessarily the deepest cell because both the atom pool (ler_step from 0 toward x_eq) and α (ler_step from 0 toward alpha_target) are monotone increasing in exposure. The circularity half is admitted in the code itself: config.rs:52-53 says the band is 'Pinned from the measurement recorded in baseline.txt'. The corridor does hard-code the artifact: SHEATH_PEAK_AGE_S = RESIDENCE_TIME_S * 4.174 with a doc that spells out 'the oldest sampled parcel (ξ = 64/65) ... ln(65) ≈ 4.174'. Corrections to the auditor: n_e does not diverge — it is bounded by the network fixed point and converges to it logarithmically slowly — and I could not verify without running that the asymptote exceeds +0.70 dec (the claimed failure at 65,536 cells is unconfirmed). The defect that stands on source alone is that the cutoff exposure is set by a grid count rather than by a physical criterion (e.g. boundary-layer thickness), and the band gating it was pinned from the number it gates.

> Evidence re-read: verification/qtt_ramc_stagline/main.rs:128-135 (`let profile_cells = 64_usize;`, `let xi = j as FloatType / (profile_cells as FloatType + 1.0);`, `let age = residence_time * (1.0 / (1.0 - xi)).ln();`) and main.rs:160-168 (`if ne > peak.0 { peak = (...) }`); config.rs:51-54 NETWORK_BAND_DECADES = 0.7 'Pinned from the measurement'; print_utils.rs:116-123 gate text; examples/avionics_examples/src/shared/constants.rs:134-141 SHEATH_PEAK_AGE_S doc naming ξ = 64/65 and ln(65); src/types/flow/finite_rate_ionization.rs:242-292 (monotone ler_step from zero)

---

### 10.3 [MAJOR] Saha statistical-weight factor 2·g_i/g_n is hard-coded to 2 for the NO/NO⁺ channel; the correct value is 0.5

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_physics/src/kernels/hypersonic/ionization.rs:128`
- **Auditor confidence:** likely

**Claim.** The Park-2T surrogate passes partition_ratio = 2.0, asserting g_i/g_n ≈ 1 for NO → NO⁺ + e⁻. The electronic degeneracy of ground-state NO (X²Π, Λ-doubling × spin) is 4 and of NO⁺ (X¹Σ⁺) is 1, so g_i/g_n = 1/4 and the full statistical factor 2·g_i/g_n = 0.5 — a factor of 4 too large in K_Saha, hence a factor of 2 too large in α and n_e in the dilute limit where α ≈ sqrt(K/n).

**Code evidence.**

```
ionization.rs:128-131
  `// Statistical-weight factor 2·g_i/g_n ≈ 2 for the NO/NO⁺ channel (Tier-A).`
  `let two = R::from_f64(2.0)`
  `    .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;`
  `saha_ionization_fraction_kernel(temperature, total_number_density, e_ion, two)`
kernel doc line 33: `* `partition_ratio` — statistical-weight factor `g = 2 g_i / g_n`.`
```

**Reference form.** Saha: n_e n_i/n_n = 2 (g_i/g_n) (2π m_e k_B T/h²)^(3/2) exp(−E_i/k_B T), where the leading 2 is the electron spin degeneracy and g_i, g_n are the electronic partition functions of the ion and neutral. Huber & Herzberg: NO ground state X²Π_r, g_e = 4; NO⁺ ground state X¹Σ⁺, g_e = 1. Excited states (NO A²Σ⁺ ~5.5 eV, NO⁺ a³Σ⁺ ~6.5 eV) are negligible at k_BT ≈ 0.6 eV. Hence 2 g_i/g_n = 2·(1/4) = 0.5.

**Impact.** In the validated re-implementation, changing only the partition ratio from 2.0 to 0.5 moves the calibrated stagnation-line result from n_e = 1.085e19 (+0.04 dec) to 5.55e18 (−0.26 dec) — a factor of 1.95. Combined with the reduced-mass error above, the flagship result becomes 2.67e17 (−1.57 dec). Every consumer of park2t_ionization_surrogate_kernel is affected: Park2tClosure::stagnation_blackout, ::stagnation_line_blackout, ::stagnation_line_blackout_2t, ::relaxation_profile_bond, and blackout.rs::IonizationStage. (FiniteRateIonizationStage does not use Saha and is unaffected.)

**Recommended fix.** Pass 0.5 instead of 2.0 at ionization.rs:131 and update the comment to state the spectroscopic source: g(NO, X²Π) = 4, g(NO⁺, X¹Σ⁺) = 1, so 2·g_i/g_n = 0.5 (Huber & Herzberg, Constants of Diatomic Molecules). If the intent is a lumped multi-channel air ionization rather than the NO channel specifically, say so explicitly and cite the effective degeneracy used; do not leave a factor labelled as the NO/NO⁺ ratio while carrying a different value.

**Adversarial check.** Code is exactly as quoted and the reference derivation is correct. The kernel's own doc (ionization.rs:33) defines the argument as g = 2·g_i/g_n, and the surrogate passes 2.0 with a comment asserting g_i/g_n ≈ 1. For the documented NO → NO⁺ + e⁻ channel the ground-state electronic degeneracies are g(NO, X²Π) = 4 (Λ-doubling × spin; both spin-orbit components populated at ~8000 K, splitting ≈ 121 cm⁻¹) and g(NO⁺, X¹Σ⁺) = 1, so 2·g_i/g_n = 0.5 — the shipped value is 4× too large in K_Saha. Nothing compensates: partition_ratio enters K linearly (k_saha = partition_ratio · thermal_db · exp(...)) and the α solve is the standard α²/(1−α) = K/n_tot, so in the dilute limit α scales as √K, i.e. 2× too high. The 'Tier-A' label does not rescue it, because the comment asserts the value is approximately right rather than flagging it as a placeholder. Only the downstream magnitudes (5.55e18 etc.) are unverified.

> Evidence re-read: deep_causality_physics/src/kernels/hypersonic/ionization.rs:33 (`* `partition_ratio` — statistical-weight factor `g = 2 g_i / g_n`.`), :86-88 (`let k_saha = partition_ratio * thermal_db * (-(e_ion_j / (kb * t))).exp();`), :127-131 (`// Statistical-weight factor 2·g_i/g_n ≈ 2 for the NO/NO⁺ channel (Tier-A).` then `saha_ionization_fraction_kernel(temperature, total_number_density, e_ion, two)`)

---

### 10.4 [MINOR] The Park high-temperature limiting vibrational correction is documented as 'applied' but is never applied anywhere in the crate

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_physics/src/constants/hypersonic.rs:70`
- **Auditor confidence:** confirmed

**Claim.** The constants file states that the Park (1990) limiting relaxation τ_park = 1/(σ_v·c̄·N) is 'applied ... to correct the Millikan–White under-prediction above ~8000 K', and ships the two constants it needs. Neither constant is referenced by any code in the workspace; the correction is not applied. The regime it is documented to correct (T₂ ≈ 8044 K) is exactly the regime the blackout physics runs in.

**Code evidence.**

```
constants/hypersonic.rs:70-79
  `// Park (1990) high-temperature limiting vibrational relaxation, applied as`
  `//   τ_park = 1 / (σ_v · c̄ · N),   σ_v = σ_ref · (T_ref / T)²`
  `// to correct the Millikan–White under-prediction above ~8000 K.`
  `pub const PARK_LIMITING_CROSS_SECTION: f64 = 1.0e-21;`
  `pub const PARK_LIMITING_REFERENCE_TEMP: f64 = 50_000.0;`
Workspace-wide grep for PARK_LIMITING_CROSS_SECTION / PARK_LIMITING_REFERENCE_TEMP / park_limiting returns exactly these two definition sites and no use site. vibrational_relaxation_kernel (thermochemistry.rs:45-111) computes tau from Millikan–White only: `let tau = exponent.exp() / pressure_atm;` with no additive Park term.
```

**Reference form.** Park (1990), Nonequilibrium Hypersonic Aerothermodynamics: above roughly 8000 K the Millikan–White correlation under-predicts τ_vt, and the total relaxation time is τ = τ_MW + τ_park with τ_park = 1/(σ_v c̄ N), σ_v = σ_ref (T_ref/T)².

**Impact.** Omitting τ_park makes τ_vt too short, so T_ve relaxes faster than physical, T_a = sqrt(T_tr·T_ve) is too hot, and n_e is over-predicted — the same direction as the reduced-mass error, so the two compound. An engineer reading the constants file will believe the correction is in the model. Order of magnitude at the RAM-C post-shock state (T = 8044 K, n = 2.645e22 m⁻³, c̄ ≈ 2.7e3 m/s, σ_v = 1e-21·(50000/8044)² = 3.9e-20 m²): τ_park ≈ 1/(3.9e-20·2.7e3·2.645e22) ≈ 3.6e-7 s, small against τ_MW ≈ 1.9e-5 s, so the numerical effect here is modest — but the docs assert an applied correction that does not exist, and the constants become dead weight a reader will assume is live.

**Recommended fix.** Either implement the correction (add τ_park to the Millikan–White τ inside vibrational_relaxation_kernel, gated on the documented ~8000 K threshold, and add the mean thermal speed c̄ = sqrt(8k_BT/πm) it needs), or rewrite the block comment to say the correction is *not* applied and state the resulting bias direction and magnitude. Do not leave 'applied as' in the file while no call site exists.

**Adversarial check.** The comment block is verbatim as quoted and states the correction is 'applied'. A workspace-wide grep for PARK_LIMITING_CROSS_SECTION, PARK_LIMITING_REFERENCE_TEMP and park_limiting returns exactly the two definition sites at hypersonic.rs:76 and :79 and no use site. I read vibrational_relaxation_kernel in full: tau is computed solely from the Millikan–White exponential (`let tau = exponent.exp() / pressure_atm;`) with no additive term and no cross-section path. The auditor's reference form (τ = τ_MW + τ_park, σ_v = σ_ref(T_ref/T)²) is the standard Park limiting-rate correction and matches the comment's own statement. The auditor also honestly bounds the numerical effect as small at the RAM-C state, so this is a documentation defect plus two dead constants, not a wrong number — severity minor is the accurate grading.

> Evidence re-read: deep_causality_physics/src/constants/hypersonic.rs:70-79 (comment 'applied as τ_park = 1/(σ_v·c̄·N) ... to correct the Millikan–White under-prediction above ~8000 K' + both consts); grep -rn 'PARK_LIMITING|park_limiting' over the repo → only lines 76 and 79; kernels/hypersonic/thermochemistry.rs:96-106 (MW-only tau)

---

### 10.5 [MINOR] Two of the RAM-C stagnation-line gates contain halves that are algebraically unfalsifiable

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/print_utils.rs:150`
- **Auditor confidence:** likely

**Claim.** `verify_renewal_ab` asserts `ne_carried <= ne_renewal`, and `verify_network`'s second gate asserts `ne_network >= ne_channel1`. Both inequalities follow from the definitions of the quantities being compared and cannot fail for any parameter values, yet both are presented as the evidence for design decisions ('the property the recombination channel was added for', 'electron impact is a refinement, not the driver').

**Code evidence.**

```
print_utils.rs:150
  `let pass = ne_carried <= ne_renewal && dec_carried.abs() <= config::CARRIED_ARM_BAND_DECADES;`
print_utils.rs:126-127
  `"electron impact is a refinement, not the driver",`
  `ne_network >= ne_channel1 && ne_network < ne_channel1 * 10.0,`
the reason the first is guaranteed is stated in the code itself, finite_rate_ionization.rs:274-292:
  `Some(_) => { if target_conc < conc { target_conc } else { conc } }`   // renewal clock at the fixed point
  `None => alpha_prev * conc,`                                            // carried clock at the smaller carried population
  `let denom = k_f * conc + beta * e_for_tau;`
and the second from finite_rate.rs:243 `let x = (linear_coefficient + disc.sqrt()) / (two * loss_coefficient);`
```

**Reference form.** A verification gate must be able to fail on some reachable input; otherwise it records a theorem about the code, not a check on it. Here: (a) since alpha_prev ≤ alpha_target ≤ min(target_conc,conc)/conc at every step, the carried clock's denominator is never larger than the renewal clock's, so the carried arm relaxes no faster toward the same (indeed, a smaller, because its atom pool also lags) target from the same initial value — carried ≤ renewal identically. (b) x*(k_lin) = (k_lin + sqrt(k_lin² + 4βp))/(2β) is monotonically increasing in k_lin ≥ 0, so dropping the electron-impact term can only lower the fixed point — ne_network ≥ ne_channel1 identically.

**Impact.** The verification README and baseline present seven PASSes as evidence that the recombination channel and the renewal mode were the right calls. Two of those PASSes are guaranteed. An engineer counting green gates over-estimates the coverage; a regression that inverted the intended physics would still have to breach the ±0.70-dec band half to be caught, and that band is itself back-fitted (see the resolution finding).

**Recommended fix.** Replace the unfalsifiable halves with quantitative assertions: pin the expected *ratio* ne_carried/ne_renewal (measured 4.699e18/2.991e19 = 0.157) to a band derived from the clock analysis, and pin the electron-impact contribution fraction (measured (2.991−2.599)/2.991 = 13%) to a band derived from the published statement that associative ionization dominates below ~8 km/s. Those can fail; the current inequalities cannot.

**Adversarial check.** Both quoted conjuncts exist verbatim and both are structurally guaranteed for the shipped code. (a) In finite_rate_ionization.rs the renewal arm evaluates its clock at min(target_conc, conc) while the carried arm evaluates it at alpha_prev·conc with alpha_prev ≤ alpha_target at every step, so the carried denominator is never larger, the carried τ never smaller; the carried pools (x_N, x_O) likewise lag, so the carried target is no larger. Both arms start from zero and ler_step never overshoots its target, so ne_carried ≤ ne_renewal for all parameter values. (b) x(k_lin) = (k_lin + √(k_lin² + 4βP))/(2β) is monotone increasing in k_lin ≥ 0, and the channel-1 arm additionally uses the shorter exposure (residence_time vs age = 4.174·t_res), so ne_network ≥ ne_channel1 identically. Two qualifications the auditor already scoped correctly in the title: each gate retains a second, genuinely falsifiable conjunct (the ±band, and ne_network < 10·ne_channel1), and both conjuncts can still fail if the implementation is changed, so they function as invariant regression pins. As evidence that a design decision was correct they contribute nothing, which is the finding's substance; that makes this minor rather than major.

> Evidence re-read: verification/qtt_ramc_stagline/print_utils.rs:150 `let pass = ne_carried <= ne_renewal && dec_carried.abs() <= config::CARRIED_ARM_BAND_DECADES;`; print_utils.rs:124-127 `ne_network >= ne_channel1 && ne_network < ne_channel1 * 10.0`; src/types/flow/finite_rate_ionization.rs:274-296 (e_for_tau renewal = min(target_conc, conc), carried = alpha_prev*conc; denom = k_f*conc + beta*e_for_tau; ler_step from R::zero()/alpha_prev); deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:241-243 `let x = (linear_coefficient + disc.sqrt()) / (two * loss_coefficient);`

---

### 10.6 [MAJOR] The D7 attribution gate compares the two arms at different exposure times (t_res versus 4.174·t_res), so the 13% electron-impact contribution it reports is not measured

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/main.rs:213`
- **Auditor confidence:** confirmed

**Claim.** Correct as stated except for one internally inconsistent sentence: the exposure confound inflates the reported gap rather than partially cancelling it, so the printed 13% is an upper bound on the electron-impact contribution.

**Code evidence.**

```
main.rs:135  `let age = residence_time * (1.0 / (1.0 - xi)).ln();`     // used for ne_network via `.with_sheath_renewal(config::ft(age))` at line 151
main.rs:212-214
  `let tau_c1 = 1.0 / (k_f * conc + beta * target_c1);`
  `let alpha_c1 = (target_c1 / conc) * (1.0 - (-(residence_time / tau_c1)).exp());`
  `let ne_channel1 = alpha_c1 * post.n_tot2;`
print_utils.rs:128-131 prints `"channel 1 + pool {:.3e} vs full network {:.3e} m^-3 (the associative channel carries the prediction at RAM-C speeds)"`
```

**Reference form.** An attribution measurement isolates one variable. To attribute a difference to the electron-impact channel, both arms must be evaluated at identical state, identical pool fractions, and identical exposure, with only the k_lin term set to zero in one of them.

**Impact.** The reported 'channel 1 + pool 2.599e19 vs full network 2.991e19' (a 13% gap) is a mixture of two effects pulling in the same direction, so the true electron-impact contribution is smaller than 13% — the confound partially cancels the difference the gate is trying to measure. The design-D7 conclusion 'if the full network ever leaves its band, the two numbers say which channel moved' does not hold, because a change in exposure moves the two numbers differently. Note the arm does at least use the peak's x_N, x_O and t_ve, so the pool state is shared; only the exposure differs.

**Recommended fix.** Evaluate `ne_channel1` at the same `age` used for the profile peak (store the peak's age alongside x_n/x_o/t_ve at main.rs:150-167 and use it at line 213), or better, compute both arms inside the profile loop by running FiniteRateIonizationStage twice per ξ with and without the electron-impact term. Then the printed difference is the electron-impact contribution.

**Adversarial check.** Verified directly in main.rs. The network arm's per-cell exposure is `age = residence_time * (1.0/(1.0-xi)).ln()`, passed both to with_sheath_renewal and to the StepContext, and ne_network is the profile peak, i.e. the ξ = 64/65 cell at age = 4.174·t_res. The channel-1 arm at main.rs:211-214 relaxes over `residence_time` only. So the two arms differ in exposure by 4.17× as well as in the k_lin term, while print_utils.rs:126-131 labels the gap 'electron impact is a refinement, not the driver' and the design-D7 doc comment (print_utils.rs:110-115) claims the two numbers say which channel moved. Since both differences push the same way, the true electron-impact share is smaller than the printed 13% — the auditor's conclusion is right even though the phrase 'the confound partially cancels the difference' contradicts it (the confound inflates it). The auditor is also correct that the pool state (x_N, x_O, t_ve) is shared from the peak, so exposure is the sole remaining confound.

> Evidence re-read: verification/qtt_ramc_stagline/main.rs:135 `let age = residence_time * (1.0 / (1.0 - xi)).ln();`, :150-151 `FiniteRateIonizationStage::new(post.n_tot2).with_sheath_renewal(config::ft(age))`, :156 `StepContext::qtt(config::ft(age), 1)`, :160-168 peak selection, :211-214 `let tau_c1 = 1.0 / (k_f * conc + beta * target_c1); let alpha_c1 = (target_c1 / conc) * (1.0 - (-(residence_time / tau_c1)).exp());`; print_utils.rs:110-131

---

### 10.7 [MINOR] The Park-2T verification's 'closed-form exponential exactness' gate compares ler_step against a retyped copy of its own body

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/print_utils.rs:38`
- **Auditor confidence:** confirmed

**Claim.** Gate (ii) asserts that ler_step(x, x_eq, tau, dt) equals `x_eq - (x_eq - x) * (-(dt / tau)).exp()`, which is character-for-character the body of ler_step. The gate verifies nothing about the correctness of the closed-form relaxation; it is a self-comparison against the same code path.

**Code evidence.**

```
print_utils.rs:36-39
  `fn gate_exponential_exactness() -> bool {`
  `    let (x, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);`
  `    ler_step(x, x_eq, tau, dt) == x_eq - (x_eq - x) * (-(dt / tau)).exp()`
versus src/types/flow/blackout.rs:46
  `    x_eq - (x_eq - x) * (-(dt / tau)).exp()`
```

**Reference form.** The claim to verify is that the discrete update is exact on dx/dt = (x_eq − x)/τ, i.e. that x(t+Δt) from the analytic ODE solution x(t+Δt) = x_eq − (x_eq − x(t))e^(−Δt/τ) is reproduced. That requires an independent reference — a high-order numerical integration of the ODE, or a many-substep composition identity — not a restatement of the implementation.

**Impact.** This gate is one of six presented in verification/README.md as 'Gap-2 Tier-A verified'. It cannot fail short of an editor accident, so it contributes no assurance. Gate (vi) ('n_e > 0 somewhere') is similarly near-unfalsifiable, since the Saha target is strictly positive for any T > 0.

**Recommended fix.** Replace with a property that can fail: integrate dx/dt = (x_eq − x)/τ with a small-step RK4 over the same Δt and assert ler_step matches to RK4's truncation error; and/or assert the semigroup identity ler_step(ler_step(x,·,τ,Δt/2),·,τ,Δt/2) == ler_step(x,·,τ,Δt) to round-off. Either would catch a sign flip, a swapped argument, or a missing negation.

**Adversarial check.** The gate body is verbatim as quoted (at print_utils.rs:34-37, two lines off the cited 36-39) and the expression is character-for-character the τ > 0 branch of ler_step (blackout.rs:47, cited as 46). For the chosen inputs (tau = 0.01 > 0) the two sides are the same expression evaluated twice, so the equality holds by construction and the gate proves nothing about exactness on dx/dt = (x_eq − x)/τ. The auditor's stated reference form (compare against an independent integration or a substep-composition identity) is the right one. Two mitigations that make this minor rather than major: the gate still pins the implementation against accidental edits, and gate (iv) does exercise the non-trivial τ → 0 limit against the Saha target. The aside about gate (vi) is fair but overstated — gate_electrons_produced reads the marched Report series, so it would fail if the pipeline never published n_e; it is weak, not unfalsifiable.

> Evidence re-read: verification/qtt_park2t_blackout/print_utils.rs:33-37 `fn gate_exponential_exactness() ... ler_step(x, x_eq, tau, dt) == x_eq - (x_eq - x) * (-(dt / tau)).exp()`; src/types/flow/blackout.rs:41-48 `pub fn ler_step ... x_eq - (x_eq - x) * (-(dt / tau)).exp()`; print_utils.rs:99-105 gate (vi) reads `report.series("n_e")`

---

### 10.8 [MINOR] The verification README's summary row for qtt_park2t_blackout reports 'Reference: all gates pass' and 'Gap-2 Tier-A verified' for a run whose electron density is fully saturated at n_e = n_tot, three decades above the flight anchor

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/README.md:39`
- **Auditor confidence:** confirmed

**Claim.** In a table whose stated purpose is to hold 'the compared values' in Measured and Reference and 'their exact difference' in Divergence, this row's Reference is 'all gates pass' (a restatement of the Measured column) and its Divergence is 'Gap-2 Tier-A verified' (a conclusion). The measured ω_p 5.6e12 rad/s corresponds to n_e = 1.000e22 m⁻³ = n_tot exactly — complete ionization, ~1000× the RAM-C II anchor of 1e19 — which the row does not state.

**Code evidence.**

```
verification/README.md:39
  `| `qtt_park2t_blackout` | 6 LER gates (stability, exactness, RH band, lag+Saha, path-dependence, n_e>0) | all 6 PASS; ω_p 5.6e12 ≫ band | all gates pass | Gap-2 Tier-A verified (cross-refs, Tier-A disclaimers) | 32², 40 steps | ~1 s |`
verification/qtt_park2t_blackout/baseline.txt:
  `  peak electron density n_e   : 1.000e22 m^-3`
  `  peak plasma frequency w_p   : 5.641e12 rad/s`
config.rs:49 `pub const NUMBER_DENSITY: f64 = 1.0e22;`  // so alpha = 1.000 exactly
config.rs:43 `pub const GAMMA: f64 = 1.4;`   // gives T_post ≈ 30,600 K, which qtt_ramc_stagline/config.rs:16-19 itself documents as a bad over-prediction for reacting air
```

**Reference form.** The README's own convention, stated at lines 32-33: 'The Measured and Reference columns hold the compared values; Divergence is their exact difference.' The physical reference available for this quantity is the RAM-C II peak n_e ≈ 1e19 m⁻³, which the example itself carries as RAMC_NE_REFERENCE (config.rs:57).

**Impact.** The row reads as a verified result in a table where every other row carries a genuine external or analytic reference. The honest content — a ~1000× over-prediction caused by perfect-gas γ = 1.4 driving T_post to ~30,600 K and saturating Saha — is present only in baseline.txt's Notes and in print_utils.rs's DISCLAIMER block, neither of which a reader of the summary table sees. A reviewer scanning the table concludes the Tier-A blackout slice reproduces flight-relevant electron densities. It does not.

**Recommended fix.** Fill Reference with 'RAM-C II ~1e19 m⁻³' and Divergence with '+3.0 dec (Tier-A, perfect-gas γ=1.4, Saha-saturated)', and add the row to the 'Validation scope labels' paragraph under a fourth tier: internal-invariant only, no physical-accuracy claim. Alternatively raise the example's γ to the effective 1.1 the sibling stagnation-line config documents, so the two verification programs do not disagree about the post-shock temperature of the same vehicle.

**Adversarial check.** The row at README.md:39 is verbatim as quoted, and the column convention it violates is stated two paragraphs above ('The Measured and Reference columns hold the compared values; Divergence is their exact difference'). The Reference cell restates the Measured cell and the Divergence cell holds a conclusion. The saturation claim checks out arithmetically: config.rs:49 NUMBER_DENSITY = 1.0e22 and baseline.txt reports peak n_e = 1.000e22, i.e. α = 1 exactly, ~1000× the RAMC_NE_REFERENCE = 1.0e19 the same config carries; config.rs:43 GAMMA = 1.4, and qtt_ramc_stagline/config.rs:12-16 independently documents that perfect-gas 1.4 over-predicts T₂ at ≈30 000 K. I also checked the mitigation the crate might claim: the 'Validation scope labels' paragraph immediately below the table names qtt_sod, qtt_ramc_stagline, qtt_blunt_body_2d and qtt_reentry_3d — it does not mention qtt_park2t_blackout, so the over-prediction is disclosed only in baseline.txt's Notes block, which a table reader does not see. Severity minor rather than major: the disclosure exists and is candid where it appears, and the row does not assert a flight-data match.

> Evidence re-read: deep_causality_cfd/verification/README.md:31-33 (column convention), :39 (the row, quoted correctly), :45-52 (scope-labels paragraph, park2t absent); verification/qtt_park2t_blackout/baseline.txt (`peak electron density n_e : 1.000e22 m^-3`, Notes block naming the over-prediction); qtt_park2t_blackout/config.rs:43 GAMMA=1.4, :49 NUMBER_DENSITY=1.0e22, :57 RAMC_NE_REFERENCE=1.0e19

---

### 10.9 [MINOR] The corridor's blackout-exit acceptance band was pinned from the code's own measured output

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/constants.rs:57`
- **Auditor confidence:** confirmed

**Claim.** The exit-altitude gate is a regression detector rather than a flight-anchored validation, and its constants.rs doc says so and gives the ballistic-coefficient reason the RAM-C window does not apply. The unsupported claim is the crate README's unqualified 'validate ... against RAM-C II flight data', which should be scoped to the n_e anchor rather than the corridor's exit-altitude gate.

**Code evidence.**

```
corridor/constants.rs:52-58
  `/// Pinned acceptance band for the corridor's flow-resolved blackout-exit altitude, km.`
  `/// Measured 47.0 km with the uncalibrated finite-rate network (see output.txt). ...`
  `pub const EXIT_ALTITUDE_BAND_KM: (f64, f64) = (40.0, 50.0);`
corridor/constants.rs:48-51
  `/// The RAM-C II blackout-exit flight window, km: the flight signal recovery fell at 25 to 30 km`
  `/// on descent. Reported for comparison; the corridor's own gate band is [`EXIT_ALTITUDE_BAND_KM`]`
deep_causality_cfd/README.md:218-219
  `The plasma-blackout examples validate an uncalibrated finite-rate ionization network`
  `against RAM-C II flight data.`
```

**Reference form.** A gate whose bound is derived from the measurement it gates is a regression detector, not a validation. Validation requires an independently sourced bound — here, either the RAM-C II window itself, or a predicted exit altitude derived from the vehicle's ballistic coefficient before the run.

**Impact.** Six of the corridor's gates are presented in output.txt as evidence; this one can only detect a change in the code's own behaviour. It is honestly labelled inside constants.rs ('The band catches regressions in either'), but the crate README's claim of validation against RAM-C II flight data is not supported by it. An avionics reviewer taking the README at face value would credit the exit-altitude gate as flight-anchored.

**Recommended fix.** Either (a) predict the exit altitude independently — the documented mechanism is 'the light ballistic bundle β ≈ 170 kg/m² decelerates the probe below the ionization threshold higher', which is a computable prediction from the ballistic coefficient and the ionization threshold, so compute it and gate against that; or (b) relabel the gate in output and README as a regression pin rather than a validation, and amend README:218-219 to say which specific quantity (peak n_e against the ±0.7-dec anchor band) is the flight-data comparison and which are regression pins.

**Adversarial check.** The factual core is confirmed verbatim: EXIT_ALTITUDE_BAND_KM = (40.0, 50.0) is documented as 'Measured 47.0 km with the uncalibrated finite-rate network (see output.txt)', RAMC_EXIT_WINDOW_KM is explicitly 'Reported for comparison' and not the gate, and deep_causality_cfd/README.md:218-219 says the plasma-blackout examples 'validate an uncalibrated finite-rate ionization network against RAM-C II flight data'. Where the finding overreaches is the framing that this is a hidden circularity: constants.rs states in the same doc block why the RAM-C window is not the gate (the corridor probe flies a deliberately light ballistic bundle, β ≈ 170 kg/m², so it exits above the RAM-C window; 'the offset is ballistics, not chemistry') and explicitly labels the band's purpose as catching regressions. That is a disclosed, physically reasoned design choice, and the auditor concedes it. The residual defect is narrow and real: the crate README's blanket 'validate ... against RAM-C II flight data' is not supported by this particular gate. That is a README wording defect, not a circular-verification defect in the gate.

> Evidence re-read: examples/avionics_examples/cfd/plasma_blackout/corridor/constants.rs:48-58 (RAMC_EXIT_WINDOW_KM 'Reported for comparison'; EXIT_ALTITUDE_BAND_KM doc 'Measured 47.0 km ... The band catches regressions in either'); deep_causality_cfd/README.md:218-219

---

### 10.10 [MINOR] The safety gate's descent-rate, propellant-floor and ignition-window axes are silently inert unless a throttle command is present on the channel, and a negative throttle passes through unbounded

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/corridor/gate.rs:188`
- **Auditor confidence:** confirmed

**Claim.** The descent-rate axis is scoped to throttle recoverability at its check site but its BurnEnvelope doc omits that precondition, so the axis reads as unconditional; the enforcement gap is limited to the unpowered/no-command case (the powered-but-scalar-only case is caught by the blind-gate diagnostic). The negative-throttle passthrough is real but latent for shipped producers.

**Code evidence.**

```
gate.rs:188  `if let Some(commanded) = commanded_channel {`
gate.rs:213  `if commanded > R::zero() && propellant <= burn.propellant_floor {`
gate.rs:228  `if descent_rate > burn.max_descent_rate {`         // inside the same `if let`
gate.rs:292-299
  `if commanded <= R::zero() {`
  `    // Engine off is always admissible. ...`
  `    bounded_throttle = Some(commanded);`
envelope.rs:34-35
  `/// Maximum admissible descent rate (m/s).`
  `pub max_descent_rate: R,`
envelope.rs:32-33
  `/// Minimum admissible propellant mass (kg); a positive throttle at or below it is a breach.`
The blind-gate detection at gate.rs:168-186 only fires when the *published scalar* `commanded_throttle` is > 0; a world with neither channel nor published scalar gets no diagnostic at all.
```

**Reference form.** A safety envelope axis stated without precondition must either enforce unconditionally or document its precondition at the axis definition. Descent rate is a vehicle-state limit, not a throttle limit, so conditioning it on the presence of a throttle command is a semantic mismatch. A clamp into [floor, ceiling] must reject values outside the domain of the commanded quantity; throttle is physically non-negative.

**Impact.** In an unpowered or pre-ignition phase with burn axes attached but nothing written to the throttle channel, the vehicle can exceed the certified descent-rate bound and the gate reports no breach and returns Ok — the coupling continues. A producer emitting a negative throttle (the shipped ThrottleGuidance clamps to [0,1] at throttle_guidance.rs:500-507, but the gate is generic over producers) would have that value written back to the channel by `field.set_throttle_action(bounded)` at gate.rs:343 and consumed by the thrust stages as negative thrust. Note the propellant_floor doc does state the 'positive throttle' precondition; max_descent_rate does not.

**Recommended fix.** Hoist the descent-rate check out of the `if let Some(commanded)` block so it enforces whenever burn axes are attached, and document at envelope.rs:34 that it is a whole-profile state limit. Add an explicit rejection of `commanded < R::zero()` as a misconfiguration breach rather than folding it into the engine-off branch at gate.rs:292. Document the throttle-channel precondition on BurnEnvelope itself, not only in gate.rs's implementation comments.

**Adversarial check.** The control flow is exactly as claimed: the propellant-floor, descent-rate and ignition-window checks all sit inside `if let Some(commanded) = commanded_channel` (gate.rs:187), so with burn axes attached and nothing on the throttle channel a descent-rate breach is not raised, and BurnEnvelope's max_descent_rate doc carries no precondition while propellant_floor's does. The negative-throttle path is also real: `if commanded <= R::zero() { bounded_throttle = Some(commanded); }` returns the value unclamped and it is written back by set_throttle_action. Two corrections to the severity and framing. First, the intent is documented at the check site, not silently: the check is commented 'Descent rate: above the bound with no admissible throttle correction' and its breach string is 'descent-rate bound breached, no recoverable throttle' — the axis is scoped to throttle recoverability, so the gap is a doc-parity defect at the axis definition rather than an unconditional-enforcement bug. Second, the blind-gate detection at gate.rs:168-186 is broader than the finding allows: it fires whenever the channel is absent and the published scalar is positive, which covers the powered-but-unsensed case; only the genuinely unpowered case (no channel, no positive scalar) is undiagnosed, and there a throttle-based correction does not exist. The negative-throttle passthrough is a latent hazard against a hypothetical producer — the shipped ThrottleGuidance clamps to [0,1].

> Evidence re-read: src/types/flow/corridor/gate.rs:156-186 (blind-gate diagnostic on the published scalar), :187 `if let Some(commanded) = commanded_channel {`, :212-219 propellant floor, :226-236 descent rate with its comment and message, :292-299 `if commanded <= R::zero() { ... bounded_throttle = Some(commanded); }`, :337-344 writeback; src/types/flow/corridor/envelope.rs:31-35 (propellant_floor doc states the positive-throttle precondition; max_descent_rate doc does not)

---

### 10.11 [MINOR] EosStage is documented as writing a 'two-temperature pressure' consumed by the Tier-B compressible marcher; it computes a single-temperature ideal-gas pressure at a configuration constant, and nothing in the crate reads its output field

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/blackout.rs:434`
- **Auditor confidence:** confirmed

**Claim.** Two of the three sub-claims hold: the closure is single-temperature despite the '2T' label, and it uses a frozen configured number density rather than the evolved field. The third is overstated — the doc explicitly says the Tier-A marcher does not read the field, so the unread 'pressure' scalar is disclosed rather than hidden; what remains is a name mismatch with the carrier's 'pressure_atm'.

**Code evidence.**

```
blackout.rs:434-437
  `/// A two-temperature ideal-gas pressure closure `p = n·k_B·T_tr` written into a`
  `/// per-cell `"pressure"` scalar — the interface the Tier-B compressible marcher`
  `/// reuses.`
blackout.rs:468-470
  `let n_tot = self.number_density;`
  `let pressure: Vec<R> = t_tr.iter().map(|&t| n_tot * kb * t).collect();`
  `field.set_scalar(self.pressure_field, pressure);`
Workspace grep for the literal `"pressure"` in deep_causality_cfd/src returns only blackout.rs:20, 435, 437 and 450 — the definition and its own docs. The carrier writes a different name: compressible_march_run.rs:448 `field.set_scalar("pressure_atm", p_atm);`, and that is what VibrationalLagStage consumes (world.rs:169 `.with_pressure_field("pressure_atm")`).
```

**Reference form.** Two-temperature partial-pressure closure for weakly ionized air (Park 1990): p = (n_a + n_i) k_B T_tr + n_e k_B T_e. A doc claiming an interface is 'reused' by a named component must have a call site in that component.

**Impact.** A reader wiring a Tier-B coupling will expect EosStage to feed the compressible marcher and will not notice that the marcher never reads `"pressure"`, and that if it did it would receive a pressure computed from a frozen configuration density rather than the evolved field. The stage is currently exercised only by the Tier-A verification and unit tests.

**Recommended fix.** Rename the docstring to 'single-temperature ideal-gas pressure at the configured number density'; state explicitly that no shipped consumer reads `"pressure"`; and either add a `with_density_field` accessor matching IonizationStage/VibrationalLagStage so it can use the evolved `"n_tot"`, or mark the stage as Tier-A-only.

**Adversarial check.** Sub-claims (1) and (2) are confirmed: p = n·k_B·T_tr is a single-temperature ideal-gas closure — the two-temperature partial-pressure form for weakly ionized air is p = (n_a + n_i)k_B T_tr + n_e k_B T_e — and the density used is the constructor's frozen `self.number_density`, not a per-cell field, unlike the sibling IonizationStage which reads a density field. Sub-claim (3) is where the auditor's evidence is selectively truncated. The doc block does not stop at 'reuses.'; the very next sentence reads 'On the incompressible Tier-A rollout the marcher does not read it, so the in-scope ambient effect is intentionally limited.' The unread-field fact the finding presents as an undisclosed trap is stated in the same paragraph. My grep confirms the field name is otherwise unreferenced and that the carrier publishes 'pressure_atm' (compressible_march_run.rs:448), which is what the lag stage consumes (world.rs:169, :349), so the naming mismatch is real — but the doc's 'the interface the Tier-B marcher reuses' is forward-looking and immediately qualified. Net: a mislabelled '2T' closure and a frozen configuration density, not a hidden dead interface.

> Evidence re-read: src/types/flow/blackout.rs:434-437 full doc block including the qualifying sentence the finding omits, :448-451 `pressure_field: "pressure"`, :465-471 `let n_tot = self.number_density; let pressure: Vec<R> = t_tr.iter().map(|&t| n_tot * kb * t).collect();`; grep '"pressure"' over deep_causality_cfd/src → only blackout.rs:20/435/437/450; src/types/flow/compressible_march_run.rs:448 `field.set_scalar("pressure_atm", p_atm);`; examples/avionics_examples/src/shared/world.rs:169 `.with_pressure_field("pressure_atm")`

---

### 10.12 [MINOR] The documented derivation of the Millikan–White A coefficient does not reproduce the shipped value

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_physics/src/constants/hypersonic.rs:55`
- **Auditor confidence:** confirmed

**Claim.** The comment states the natural-log constants are the base-10 originals scaled by ln 10: '5.0e-4·ln10 = 1.16e-3 and 8.00·ln10 = 18.42'. The second is right (8.00·2.302585 = 18.4207); the first is not (5.0e-4·2.302585 = 1.151e-3, not 1.16e-3, a 0.8% discrepancy). The shipped value 1.16e-3 is the canonical published natural-log coefficient, so the code is correct and the stated derivation is wrong.

**Code evidence.**

```
constants/hypersonic.rs:53-61
  `// The natural-log constants below are the base-10 originals (5.0e-4, 0.015, 8.00)`
  `// converted via ×ln(10): 5.0e-4·ln10 = 1.16e-3 and 8.00·ln10 = 18.42.`
  `pub const MILLIKAN_WHITE_A_COEFFICIENT: f64 = 1.16e-3;`
the same claim is repeated at thermochemistry.rs:43-44
  `///   (The `1.16e-3` / `18.42` natural-log constants are the base-10 `5.0e-4` /`
  `///   `8.00` originals scaled by `ln 10`.)`
```

**Reference form.** Millikan & White (1963) / Park (1990): τ_sr p = exp[A_sr(T^(−1/3) − 0.015 μ^(1/4)) − 18.42] with A_sr = 1.16e-3 μ^(1/2) θ_v^(4/3). The canonical natural-log pair is (1.16e-3, 18.42); the base-10 pre-factor consistent with 1.16e-3 is 5.038e-4, not 5.0e-4.

**Impact.** A_sr sits inside an exponential, so a reader who 'corrects' 1.16e-3 to the stated derivation's 1.151e-3 would shift τ_vt by roughly 1.2% at 8000 K. More importantly the traceability chain is broken: a certification reviewer following the stated derivation cannot reproduce the shipped constant, which undermines confidence in the rest of the constants block.

**Recommended fix.** Replace the derivation comment with a direct citation of the natural-log form as published (Park 1990, eq. for τ_sr; or Anderson, Hypersonic and High-Temperature Gas Dynamics), and drop the ×ln 10 claim for the A coefficient — or state 5.038e-4 as the exact base-10 equivalent.

**Adversarial check.** Both comment sites are verbatim as quoted and the arithmetic is as the auditor states: 5.0e-4 × ln 10 = 1.1513e-3, not 1.16e-3 (0.8% off), while 8.00 × ln 10 = 18.4207 rounds correctly to 18.42. The shipped 1.16e-3 is the canonical natural-log Millikan–White prefactor (the base-10 pre-factor consistent with it is 5.04e-4), so the constant is right and the stated derivation is what is wrong — the auditor's direction of blame is correct, which matters because a reader 'correcting' the constant would make the code worse. This is a traceability defect in a comment, with no effect on any shipped number; minor is the right grading.

> Evidence re-read: deep_causality_physics/src/constants/hypersonic.rs:53-61 (`// The natural-log constants below are the base-10 originals (5.0e-4, 0.015, 8.00) converted via ×ln(10): 5.0e-4·ln10 = 1.16e-3 and 8.00·ln10 = 18.42.` then MILLIKAN_WHITE_A_COEFFICIENT = 1.16e-3); kernels/hypersonic/thermochemistry.rs:43-44 (same claim repeated)

---

### 10.13 [MINOR] README's RegimeClass field list is three fields out of date and omits the published regime-transition counter

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/README.md:128`
- **Auditor confidence:** confirmed

**Claim.** The README documents RegimeClass as carrying four fields. The shipped struct carries seven, including the three powered-descent axes (mach_regime, thrust_state, touchdown) that participate in the regime-change key and therefore in the corridor's logged transitions. The README also never mentions REGIME_TRANSITIONS_FIELD, a public constant naming a monotone counter the classifier publishes for consumers.

**Code evidence.**

```
README.md:128
  `// `field.regime()` -> Option<&RegimeClass> { model, knudsen, plasma_frequency, gnss_denied }.`
versus regime.rs:102-118, which declares `model, knudsen, plasma_frequency, gnss_denied, mach_regime, thrust_state, touchdown`
and regime.rs:19
  `pub const REGIME_TRANSITIONS_FIELD: &str = "regime_transitions";`
with regime.rs:322-326 publishing it: `field.set_scalar(REGIME_TRANSITIONS_FIELD, alloc::vec::Vec::from([transitions]));`
```

**Reference form.** Public API documentation must enumerate the public surface it purports to enumerate (bidirectional docs-vs-code parity).

**Impact.** A consumer following the README will not know the three flight-phase axes exist or that they change the regime key, and will reimplement transition counting by scraping 'regime ->' substrings from the rendered log — the exact anti-pattern regime.rs:311-316 says the counter was added to eliminate.

**Recommended fix.** Update README.md:128 to list all seven fields, note that the last three are neutral unless RegimeClassify::with_flight_axes is called, and document REGIME_TRANSITIONS_FIELD in the Native Multi Regime section.

**Adversarial check.** The README comment is verbatim at the cited line and lists four fields; the shipped struct declares seven — model, knudsen, plasma_frequency, gnss_denied, mach_regime, thrust_state, touchdown. The three missing axes are not cosmetic: RegimeClass::key() includes them, so they participate in transition detection. REGIME_TRANSITIONS_FIELD is a public constant published by the stage on every genuine change, and a grep across both deep_causality_cfd/README.md and verification/README.md returns no mention of it. The auditor's impact note also checks out — the code comment at regime.rs:307-313 states the counter exists precisely to stop consumers counting 'regime ->' substrings in a rendered log.

> Evidence re-read: deep_causality_cfd/README.md:128 `// `field.regime()` -> Option<&RegimeClass> { model, knudsen, plasma_frequency, gnss_denied }.`; src/types/flow/corridor/regime.rs:102-118 (seven public fields), :19 `pub const REGIME_TRANSITIONS_FIELD: &str = "regime_transitions";`, :307-326 (rationale comment + set_scalar publish); grep for regime_transitions in both READMEs → no hits

---

### 10.14 [MINOR] Numeric-conversion failures fall back to values that silently change the physics rather than surfacing an error

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/corridor/branch.rs:155`
- **Auditor confidence:** confirmed

**Claim.** Several `R::from_f64(...).unwrap_or_else(...)` calls in physics paths substitute a numerically valid but physically wrong constant on conversion failure, with no diagnostic. The worst is the dynamic-pressure factor ½, which falls back to 1.0 — doubling q, the drag, the lift, the g-load and the heat-flux input. The same file's epsilon falls back to zero, and blackout.rs's frozen-chemistry timescale falls back to τ = dt (near-instant equilibration instead of frozen chemistry).

**Code evidence.**

```
branch.rs:155  `let half = R::from_f64(0.5).unwrap_or_else(R::one);`
branch.rs:157  `let q = half * self.rho_ref * u_max * u_max;`
branch.rs:168  `let eps = R::from_f64(1.0e-12).unwrap_or_else(R::zero);`
blackout.rs:356 `let huge = R::from_f64(1.0e30).unwrap_or_else(R::one);`
blackout.rs:357 `let frozen_tau = ctx.dt() * huge;`
regime.rs:179-181 `slip_threshold: R::from_f64(0.01).unwrap_or_else(R::zero),` / `free_molecular_threshold: R::from_f64(10.0).unwrap_or_else(R::one),`
contrast with the correct pattern used one file away, finite_rate_ionization.rs:117-121 and 151: `Self::lift(1.0e30)?` which propagates PhysicsError.
```

**Reference form.** For a certification-grade path, an unconvertible constant is a defect that must be surfaced (Result) or made impossible (compile-time), never replaced by a different physical value. The crate already has the correct idiom in FiniteRateIonizationStage::lift.

**Impact.** For the shipped f32/f64/Float106 scalar types these conversions do not fail, so the current numbers are unaffected — this is a latent hazard, not an active wrong number. But the failure mode is silent and severe: a Kn classifier whose slip threshold collapses to 0 reports Slip for every continuum flow, and a q that is 2× reports double the heat flux into the safety envelope. The fallbacks also make the 1e30 and 1e-12 literals load-bearing without justification of their magnitudes (1e-12 is used as an absolute threshold against three quantities with different units: m/s, m, and a dimensionless unit-vector norm).

**Recommended fix.** Convert every `unwrap_or_else` in a physics path to `?` with PhysicsError::NumericalInstability, following the FiniteRateIonizationStage::lift pattern. Replace the single 1e-12 in branch.rs with per-quantity relative tolerances (e.g. v_norm <= eps_rel * u_max, n_norm <= eps_rel since n_hat is normalized). Document why 1e30 is the frozen-chemistry marker, or replace it with an explicit Option<R> 'frozen' sentinel.

**Adversarial check.** Every cited line is verbatim: branch.rs:155 `let half = R::from_f64(0.5).unwrap_or_else(R::one);` feeding `let q = half * self.rho_ref * u_max * u_max;`, branch.rs:168 eps → zero, blackout.rs:356-357 huge → one making frozen_tau collapse to dt (near-instant equilibration where the comment explicitly demands a timescale ≫ dt, i.e. the fallback inverts the stated intent), and regime.rs:179-181 slip → zero / free_molecular → one (the latter would also invert the band order against transitional = 0). The contrasting correct idiom exists in the same subsystem (finite_rate.rs uses `lift::<R>(2.0, "2.0")?` propagating PhysicsError). The auditor is scrupulous that no shipped scalar type can trigger these, so this is a latent-hazard/style defect rather than a wrong number — minor is correct, and the sub-remark about 1e-12 being applied to quantities of different units is a secondary observation I did not fully trace.

> Evidence re-read: src/types/flow/corridor/branch.rs:155-157 and :168-172; src/types/flow/blackout.rs:355-357 (comment 'A frozen-chemistry timescale ≫ dt' immediately above `let huge = R::from_f64(1.0e30).unwrap_or_else(R::one); let frozen_tau = ctx.dt() * huge;`); src/types/flow/corridor/regime.rs:178-181; deep_causality_physics/src/kernels/hypersonic/finite_rate.rs:240-241 (`lift::<R>(2.0, "2.0")?`)

---

### 10.15 [MINOR] RegimeClassify::with_thresholds and with_flight_axes document ordering preconditions they do not validate

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/corridor/regime.rs:190`
- **Auditor confidence:** confirmed

**Claim.** with_thresholds' doc states the precondition '(slip ≤ transitional ≤ free_molecular)' but the setter accepts any values. If they are supplied out of order, model_for's cascading `if kn < ...` chain silently makes one or more bands unreachable — e.g. slip > transitional makes GoverningModel::Slip impossible. with_flight_axes has the same exposure: subsonic_ceiling > supersonic_floor makes MachRegime::Supersonic unreachable, since mach_regime_of tests the subsonic branch first.

**Code evidence.**

```
regime.rs:190-196
  `/// Override the Knudsen band thresholds (`slip ≤ transitional ≤ free_molecular`).`
  `pub fn with_thresholds(mut self, slip: R, transitional: R, free_molecular: R) -> Self {`
  `    self.slip_threshold = slip;`  ...  // no comparison performed
regime.rs:266-276 `if kn < self.slip_threshold { Continuum } else if kn < self.transitional_threshold { Slip } ...`
regime.rs:230-236 `if mach <= axes.subsonic_ceiling { Subsonic } else if mach >= axes.supersonic_floor { Supersonic } else { Transonic }`
```

**Reference form.** A documented precondition on a public builder either has to be checked (returning Result) or has to be enforced by construction. Silently producing a classifier with an unreachable band is a wrong-classification hazard, and the classification drives the governing-model selection and the GNSS-denial-adjacent regime key.

**Impact.** A misconfigured classifier produces a plausible-looking but wrong governing model for every step of a run, with no error and no log entry — the corridor's provenance log would simply never show a 'slip' transition. Given that the regime key gates the Kalman filter's measurement folding, a silently wrong band is a nav-integrity issue, not just a labelling one.

**Recommended fix.** Make with_thresholds and with_flight_axes return Result<Self, PhysicsError> and reject out-of-order edges, or clamp-and-log. Given the builder is used at construction time in example config, a Result is cheap and matches the crate's error idiom elsewhere.

**Adversarial check.** Verified verbatim. with_thresholds' doc line states the ordering `slip ≤ transitional ≤ free_molecular` and the body performs three unchecked assignments and returns Self — no comparison, no Result. model_for is a cascading if/else-if on those thresholds, so slip > transitional makes GoverningModel::Slip unreachable, exactly as claimed. mach_regime_of tests `mach <= axes.subsonic_ceiling` before `mach >= axes.supersonic_floor`, so subsonic_ceiling > supersonic_floor makes MachRegime::Supersonic unreachable. The misconfiguration is silent — no log entry, no error. The nav-integrity framing in the impact section is an inference I did not trace to the Kalman gating code, but the API defect itself is established from the source.

> Evidence re-read: src/types/flow/corridor/regime.rs:190-196 (doc `(slip ≤ transitional ≤ free_molecular)` + unchecked setter), :266-276 model_for cascade, :229-236 mach_regime_of ordering, :198-203 with_flight_axes doc

---
