# deep_causality_cfd — QTT compressible marchers (solvers/qtt/compressible/*, solvers/qtt/mod.rs)

**Production readiness: `needs-work`**

The mathematical core is genuinely correct and I confirmed it line by line: the 1-D, 2-D and 3-D conservative Euler flux vectors, the ideal-gas EOS with gamma as a constructor parameter (never hardcoded), the sound speed, the exact Rankine-Hugoniot density/pressure/velocity ratios and the T2 kernel, the gradient/laplacian MPO stencils, the algebraic identity that makes the code's central-flux-plus-Laplacian form exactly equal to the Rusanov update, and the entire closed-form acoustic-core inverse (contracting root, binary doubling, finite-N gain correction, free-stream exactness). The Sod gate compares against a genuinely independent Toro exact Riemann solver. Above 1-D, however, the trustworthiness breaks down. The 2-D/3-D marchers compute the instantaneous max wave speed every step and discard it into `_s_max` while stabilizing with a fixed user-supplied `s_ref`, so an interior wave speed exceeding `s_ref` under-dissipates with no error, no warning and no diagnostic — and the shipped production configuration sets `S_REF = 1.8` with a comment saying it is "deliberately snug". `CompressibleMarcher2d` hardcodes the dissipation grid spacing as `1/nx`, ignoring the metric's spacing entirely and reusing hx for hy, so the explicit (physical) flux divergence and the implicit (computational) dissipation live in different coordinate systems on any non-unit-square chart. Conservation is unenforced above 1-D: `conservation_round` and `positivity_floor` exist, are publicly exported, and are called by no solver anywhere in the workspace, while the shipped example runs under `Truncation::by_bond(16)` — a pure rank cap with zero tolerance, i.e. unbounded per-step truncation error — and no test gates mass/momentum/energy for the 2-D or 3-D marcher. Several supporting tests are tautological (`peak >= 1`, a conservation test that applies the pinning fixup every step, an RH test that re-types the implementation's own algebra instead of the tabulated values). None of this is unfixable, and the honesty of the verification README ("the only quantitative-accuracy gate") is a genuine credit, but an avionics consumer cannot today take a 2-D or 3-D marched result as conservative or as stability-checked.

- Files read: **42**
- Findings raised: **20** — surviving adversarial verification: **20** (refuted: 0)
- Surviving by severity: major 5, minor 11, info 4
- Independently confirmed-correct items: **13**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| 1-D conservative Euler flux vector F = (rho*u, rho*u^2+p, (E+p)*u) | `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:113-115` | d/dt(rho, rho*u, E) + d/dx F = 0, F = (rho*u, rho*u^2+p, (E+p)u) — LeVeque, Finite Volume Methods for Hyperbolic Problems, eq. 2.38; Toro Ch.3 |
| Rusanov / local-LF update is algebraically identical to the implemented central-flux + Laplacian form | `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:124-137 with operators.rs:95-131` | F_{i+1/2} = 0.5(F_i+F_{i+1}) - 0.5*s*(U_{i+1}-U_i); dU/dt = -(F_{i+1/2}-F_{i-1/2})/dx |
| Ideal-gas EOS and gamma parameterization (1-D and 2-D) | `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:36-39; marcher_2d.rs:276-279` | p = (gamma-1)*(E - 0.5*rho*\|u\|^2) = (gamma-1)*(E - 0.5*\|m\|^2/rho) |
| Sound speed a = sqrt(gamma*p/rho) | `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:112` | a = sqrt(gamma*p/rho) for a calorically perfect gas |
| 2-D Euler flux tensors F and G | `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:137-144` | F = (rho*u, rho*u^2+p, rho*u*v, (E+p)u); G = (rho*v, rho*u*v, rho*v^2+p, (E+p)v) |
| 3-D Euler flux tensors F, G, H (Cartesian and body-fitted marchers) | `deep_causality_cfd/src/solvers/qtt/compressible/marcher_3d.rs:140-154; marcher_3d_fitted.rs:150-164` | F = (rho*u, rho*u^2+p, rho*u*v, rho*u*w, (E+p)u), and cyclic permutations for G, H |
| Rankine-Hugoniot density, pressure and velocity ratios | `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:118-120` | rho2/rho1 = (g+1)M^2/((g-1)M^2+2); p2/p1 = (2gM^2-(g-1))/(g+1); u2/u1 = rho1/rho2 (continuity) — Anderson, Modern Compressible Flow, eqs. 3.53/3.57 |
| Rankine-Hugoniot temperature jump T2/T1, and its consistency with p_ratio/rho_ratio | `deep_causality_physics/src/kernels/hypersonic/shock.rs:60-62 (called from fitting.rs:117)` | T2/T1 = (p2/p1)*(rho1/rho2) = (2gM^2-(g-1))((g-1)M^2+2)/((g+1)^2 M^2) |
| Closed-form acoustic-core inverse: factorization, contracting root, and free-stream exactness | `deep_causality_cfd/src/tensor_bridge/acoustic_inverse.rs:122-135` | A0 = (1+2s)I - s(S+ + S-) factors as (s/rho)(I - rho*S+)(I - rho*S-) when s*rho^2 - (1+2s)*rho + s = 0; A0^-1*const = const requires s*(1-rho)^2 = rho |
| IMEX operator split sums to the full operator | `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:13-16, 120-130` | A = I - dt*kappa*c^2(x)*d2 must equal A0 + A1 with A0 = I - dt*kappa*cbar2*d2, A1 = -dt*kappa*(c^2-cbar2)*d2 |
| Exact Riemann solver used as the Sod reference is genuinely independent of the solver under test | `deep_causality_cfd/verification/qtt_sod/exact_riemann.rs:24-139` | Toro, Riemann Solvers and Numerical Methods for Fluid Dynamics, Ch. 4 (pressure functions f_K, eq. 4.6-4.9; shock density eq. 4.53; rarefaction fan eq. 4.56) |
| 3-D re-pin index arithmetic (roll offset and inner-radius slide) | `deep_causality_cfd/src/solvers/qtt/compressible/marcher_3d_fitted.rs:343, 397-399` | To move a feature at index kstar to index target: shift = target - kstar, rolled[k] = src[k - shift]. To keep the feature's physical radius fixed under r(k) = r0 + (k/nz)*dr: r0_new = r0 + ((kstar-tar |
| QTT codec mode layouts are consistent across 1-D/2-D/3-D, so the 1-D helpers operate correctly on multi-D states | `deep_causality_cfd/src/tensor_bridge/codec.rs:36-39, 81-83, 137-141` | For a flat row-major power-of-two buffer, quantize/quantize_2d/quantize_3d must produce the same [2;L] mode ordering for the helpers to be interoperable |

## Findings

### 13.1 [MAJOR] 2-D/3-D marchers compute the instantaneous max wave speed and discard it, stabilizing instead with a fixed s_ref — under-dissipation is silent and undetectable

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:225`
- **Auditor confidence:** confirmed

**Claim.** The 2-D/3-D marchers' documented precondition — s_ref >= max(|u|+c) the flow will see — is enforced only against the inflow state at the carrier level, never against the marched interior, even though flux_and_speed computes the interior s_max on every step and discards it. A per-step comparison of s_max against s_ref (as a returned diagnostic or a hard error) is the cheap fix; recomputing the dissipation per step is not available without rebuilding the closed-form D10 inverse.

**Code evidence.**

```
marcher_2d.rs:225  `let (f, g, _s_max) = self.flux_and_speed(&dense)?;`\nmarcher_3d.rs:241  `let (f, g, h, _s_max) = self.flux_and_speed(&dense)?;`\nmarcher_3d_fitted.rs:190  `let (f, g, h, _s_max) = self.flux_and_speed(&dense)?;`\nmarcher_2d.rs:88-90  `let half = R::from_f64(0.5).unwrap_or_else(R::one);\n        let beta = dt * half * s_ref * h;\n        let acoustic_inv = AcousticCoreInverse2d::new(lx, ly, h, h, beta, trunc)?;`\nContrast the 1-D marcher, which does it correctly — euler_1d.rs:183: `let diss = half * s_max * self.dx;` recomputed each step from the current state.\nShipped production value, examples/avionics_examples/src/shared/constants.rs:48-50: `/// Reference wave speed of the implicit acoustic envelope. Deliberately snug: the peak-station\n/// inflow outgrows it once, so the rebuild-on-drift mechanism fires where the descent steepens.\npub const S_REF: f64 = 1.8;`\nThe only rebuild trigger keys on the inflow state, never the marched interior — compressible_march_run.rs:363: `let s_needed = u_hat + (self.gamma * t_hat).sqrt();`
```

**Reference form.** Rusanov / local Lax-Friedrichs: F_{i+1/2} = 0.5(F_L+F_R) - 0.5*s_max*(U_R-U_L), s_max = max(|u|+a) evaluated on the current state. Equivalent viscous form nu = 0.5*s_max*dx (LeVeque, Finite Volume Methods, sec. 12.5). The wave speed must track the solution; a frozen estimate voids the monotonicity argument.

**Impact.** An engineer running the 2-D or 3-D marcher on a flow whose interior wave speed grows past the configured s_ref (a hot spot, a compression, a plume interaction) gets an under-dissipated, oscillatory solution with no error, no warning, and no returned diagnostic. The failure only surfaces if density actually goes non-positive (marcher_2d.rs:125). Between clean and rho<=0 there is a wide band of quantitatively wrong results that look plausible. The quantity needed to detect this is computed on every single step and deliberately dropped on the floor.

**Recommended fix.** Return the per-step s_max from `step`/`run` (or accumulate its max into the returned tuple alongside `peak` bond), and error with `PhysicsError::NumericalInstability` when s_max exceeds s_ref by more than a documented margin. Better: make the dissipation state-dependent by rebuilding beta from the current s_max, or at minimum document loudly on `CompressibleMarcher2d::new` that s_ref is a hard correctness precondition the marcher does not check.

**Adversarial check.** Every cited line is real. marcher_2d.rs:225, marcher_3d.rs:241 and marcher_3d_fitted.rs:190 all bind `_s_max` and drop it; the dissipation is built once from s_ref (marcher_2d.rs:88-90). The 1-D contrast at euler_1d.rs:183 is accurate. The carrier's only rebuild trigger is inflow-keyed (compressible_march_run.rs:363 `let s_needed = u_hat + (self.gamma * t_hat).sqrt();`), and its own comment enumerates the trigger's blind spots (density anchor) without mentioning the marched interior. S_REF=1.8 is documented as 'Deliberately snug'. So the substantive defect stands: an interior s_max above s_ref produces an under-dissipated result with no error and no diagnostic, using a quantity computed on every step. The framing is overstated in two respects: (1) the 2-D/3-D marchers do not document themselves as per-step Rusanov — marcher_2d.rs:53 and :62-64 explicitly document a *reference* dissipation nu_bar = 1/2*s_ref*dx built once, and D10 requires a constant coefficient for the closed-form inverse to exist at all, so recomputing per step is not a drop-in fix; (2) the constructor already validates s_ref finite/positive, so this is an unenforced documented precondition rather than an unstated one.

> Evidence re-read: marcher_2d.rs:88-90, 121-152, 225; marcher_3d.rs:86-88, 241; marcher_3d_fitted.rs:76-77, 190; euler_1d.rs:172-183; types/flow/compressible_march_run.rs:355-391; examples/avionics_examples/src/shared/constants.rs:48-50

---

### 13.2 [MAJOR] CompressibleMarcher2d hardcodes the dissipation spacing as 1/nx, ignoring the metric's spacing entirely, and reuses hx as hy

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:78`
- **Auditor confidence:** confirmed

**Claim.** The implicit acoustic dissipation is built from a spacing h = 1/nx computed internally from the mode count, not from the coordinate chart the marcher is running over, and the same h is passed for both the x and y spacings of the 2-D ADI inverse. The explicit convective term meanwhile uses the metric's chain-rule *physical* gradient. The two halves of the IMEX split therefore live in different coordinate systems on any chart where the physical spacing is not 1/nx, and any grid with lx != ly gets the wrong y-axis stiffness.

**Code evidence.**

```
marcher_2d.rs:76-90:\n`let (lx, ly) = metric.dims();\n let nx = 1usize << lx;\n let h = R::one()\n     / R::from_usize(nx).ok_or_else(...)?;\n ...\n let beta = dt * half * s_ref * h;\n let acoustic_inv = AcousticCoreInverse2d::new(lx, ly, h, h, beta, trunc)?;`\nNote `ly` is used for the mode count but `ny` never appears — `h` is derived from `nx` alone and passed twice.\nAcousticCoreInverse2d then forms per-axis stiffnesses from those spacings, acoustic_inverse.rs:229-230: `let sx = beta / (dx * dx);  let sy = beta / (dy * dy);`\nThe explicit half uses the metric instead, marcher_2d.rs:244-246: `let (dfx, _) = self.metric.physical_gradient(&fq)?;  let (_, dgy) = self.metric.physical_gradient(&gq)?;  let div = dfx.add(&dgy)?;`\n`MetricProvider` exposes no spacing at all (traits/metric_provider.rs:31-53: `dims`, `sample`, `physical_gradient`, `jacobian`), so the marcher structurally cannot read the chart's dx/dy — yet `CartesianIdentity::new` takes them (coordinate/cartesian.rs:40-51, doc: \"physical spacing (dx, dy)\").\nThe doc comment claims a physical spacing: marcher_2d.rs:53 `/// (I - dt*nu_bar*grad^2)^-1 with reference dissipation nu_bar = 1/2*s_ref*dx, the implicit acoustic step (D10).`
```

**Reference form.** For an IMEX split of dU/dt = -div F + nu*lap U, both operators must be discretized on the same metric. The Rusanov-equivalent viscosity is nu = 0.5*s_max*dx_physical. Writing the implicit half as (I - beta*D2/h^2) with h != dx_physical yields effective nu = 0.5*s_ref*dx_physical^2/h, i.e. off by the factor dx_physical/h.

**Impact.** All shipped configurations happen to set dx = 1/nx (examples/avionics_examples/src/shared/world.rs:94-97 `let dx = utils::ft(1.0) / utils::ft(n as f64); ... .grid(L, L, dx, dx)`), so the bug is currently masked. It is not masked in verification/qtt_blunt_body_2d/main.rs:88-97, which runs `CompressibleMarcher2d` over a `BlendedMap` polar fan with R0=1.0, DR=1.0, DTHETA=PI/2: the radial physical spacing is DR/side = h, but the transverse physical spacing is about 1.5*(PI/2)/side ~ 2.4*h, so the transverse artificial viscosity is ~2.4x the Rusanov-consistent value and the dissipation is anisotropic in a way nothing documents. Any user who constructs a CartesianIdentity with a real physical spacing (metres) gets a dissipation coefficient wrong by the factor dx*nx — potentially orders of magnitude.

**Recommended fix.** Add a `spacing() -> (R, R)` method to `MetricProvider` and build beta and the ADI inverse from it, using hy = 1/(1<<ly) at minimum; or accept explicit (dx, dy) arguments in `CompressibleMarcher2d::new`. Until then, change the doc on marcher_2d.rs:53 and :62-64 to state explicitly that the dissipation is a computational-space regulariser at spacing 1/Nx and that the marcher assumes a unit-square chart with square cells — the way marcher_3d_fitted.rs:18-20 already honestly does.

**Adversarial check.** marcher_2d.rs:76-90 is quoted correctly: `h` is derived from `nx = 1 << lx` alone and passed for both dx and dy of AcousticCoreInverse2d; `ny` never appears. acoustic_inverse.rs:229-230 forms sx = beta/(dx*dx), sy = beta/(dy*dy) as claimed. MetricProvider (traits/metric_provider.rs:28-53) exposes dims/sample/physical_gradient/jacobian and no spacing, while CartesianIdentity::new takes real (dx, dy). The explicit half does use the metric's chain-rule physical_gradient (marcher_2d.rs:244-246). I re-derived the blunt-body numbers: R0=1.0, DR=1.0, DTHETA=PI/2 (verification/qtt_blunt_body_2d/main.rs:32-34) give radial spacing DR/side = h and mean transverse arc spacing 1.5*(PI/2)/side ~ 2.36h — the ~2.4x anisotropy is right. Two caveats worth recording, neither of which refutes it: (a) marcher_3d_fitted.rs:16-18 documents the acoustic dissipation as 'a computational-space regulariser', which makes h = 1/nx defensible as the *computational* lattice spacing — but marcher_2d.rs:53 states nu_bar = 1/2*s_ref*Dx with no such qualifier, so the doc/implementation mismatch is real; (b) the sharpest defect is arguably the one the title mentions last: passing h twice is wrong even in computational space whenever ly != lx, and CompressibleMarcher2d::new accepts any such metric today.

> Evidence re-read: marcher_2d.rs:53, 62-64, 76-90, 244-246; tensor_bridge/acoustic_inverse.rs:194-233; traits/metric_provider.rs:28-53; coordinate/cartesian.rs:23-53; verification/qtt_blunt_body_2d/main.rs:32-34, 88-97; examples/avionics_examples/src/shared/world.rs:94-97

---

### 13.3 [MINOR] conservation_round and positivity_floor are exported as the crate's conservation/positivity machinery but are called by no solver anywhere in the workspace

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:158`
- **Auditor confidence:** confirmed

**Claim.** Neither helper is called by any solver, carrier, example or verification program — they are public-API utilities only. No crate documentation claims otherwise, so this is an unused-API observation that duplicates the substantive gap already reported in Finding 5, not a separate doc overclaim.

**Code evidence.**

```
imex.rs:158-163 (the promise): `/// Conservation-preserving rounding (design D4): `round` minimizes Frobenius error, not the integral, and\n/// the implicit solve carries its own residual, so a marched conservative field drifts its total. Carry the\n/// conserved `target` total (the invariant from `t = 0`) and, after rounding, restore it with a **rank-1\n/// uniform fixup** ...`\nExhaustive grep over the whole workspace for both symbols returns only: imex.rs:167 and :192 (definitions), compressible/mod.rs:21, solvers/qtt/mod.rs:19-20, solvers/mod.rs:27-29, lib.rs:145 (re-export chain), and tests/solvers/qtt/compressible_imex_tests.rs:11,125,143,168. Zero call sites in any solver, carrier, study, verification program or example.\nThe marchers instead round unconditionally with no fixup — marcher_2d.rs:247-248: `let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;\n Ok(self.acoustic_inv.apply(&predictor)?.round(&self.trunc)?)`
```

**Reference form.** A scheme documented as conservative must either use a discretization whose per-step compression is conservation-preserving, or apply an explicit conservation fixup. The crate itself states the requirement at imex.rs:158-163 and then does not meet it in any marcher.

**Impact.** A reader of the module docs (or of the public API surface, which exports `conservation_round` at lib.rs:145) reasonably concludes that the QTT compressible path defends its conserved integrals. It does not. Combined with the truncation policy finding below, mass/momentum/energy drift in the 2-D and 3-D marchers is unbounded, unmeasured and unfixed. For a pre-certification consumer this is exactly the class of gap that invalidates a conservation claim.

**Recommended fix.** Either wire `conservation_round` into the marchers' `step_component` (carrying the t=0 integral per conserved component), or move both functions out of the compressible module and document them explicitly as unused reference implementations / building blocks not on any shipped path. Do not leave a documented cure that no patient receives.

**Adversarial check.** The grep result is exactly reproducible: a workspace-wide search returns only the definitions (imex.rs:167, :192), the re-export chain (compressible/mod.rs:21, solvers/qtt/mod.rs:19-20, solvers/mod.rs:27-29, lib.rs:145), the four test references, and two archived task-list entries. No solver, carrier, example or verification program calls either. The docstring at imex.rs:158-163 is quoted verbatim and correctly. But the doc-overclaim framing does not survive: no doc anywhere says the marchers *use* these helpers. They are public utilities documented against archived tasks 3.2/3.3, not dead code in the Rust sense (they are part of the public API). And the crate's only conservation claim — verification/qtt_sod/README.md:37, 'The companion unit tests gate conservation' — is about the 1-D marcher, which conserves by construction (telescoping gradient/Laplacian MPOs) and is genuinely gated by compressible_tests.rs:49-81. So the real content here is 'the 2-D/3-D marchers have no conservation fixup', which is Finding 5, not an independent overclaim.

> Evidence re-read: imex.rs:158-208; workspace grep for conservation_round|positivity_floor (16 hits, zero solver call sites); lib.rs:145; verification/qtt_sod/README.md:37-39; tests/solvers/qtt/compressible_tests.rs:49-81

---

### 13.4 [MINOR] positivity_floor breaks conservation and the doc does not say so; its floor value is entirely caller-supplied with no guidance

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:186`
- **Auditor confidence:** confirmed

**Claim.** `positivity_floor` replaces every cell below `floor` with `floor`, which strictly increases the field's integral. For a conserved variable (density, energy) this injects mass/energy. The docstring describes it as \"a pragmatic guard\" and mentions the deferred structural upgrade, but never states that applying it violates conservation, never bounds the injected amount, and gives no criterion for choosing `floor`.

**Code evidence.**

```
imex.rs:186-190 (the full doc): `/// Positivity limiter (task 3.3): clamp a field to a small positive `floor` (dequantize -> `max(., floor)`\n/// -> requantize). A pragmatic guard keeping `rho, p > 0` through a strong rarefaction; the structural\n/// upgrade is entropy / log-variable evolution (deferred).`\nimex.rs:200-207 (the implementation): `let dense = dequantize(u)?;\n let clamped: vec::Vec<R> = dense\n     .as_slice()\n     .iter()\n     .map(|&v| if v > floor { v } else { floor })\n     .collect();`\nNo compensating removal anywhere; the returned field's sum is >= the input's, strictly greater whenever any cell was clamped.\nThe only test, tests/solvers/qtt/compressible_imex_tests.rs:152-175, asserts `worst >= floor - 1e-9` — i.e. it verifies the clamp fires, and never measures how much mass the clamp added.
```

**Reference form.** Any positivity limiter that clamps rather than redistributes is non-conservative. Conservative positivity-preserving limiters (Zhang & Shu, J. Comput. Phys. 229 (2010) 3091) rescale the cell about its mean precisely so the integral is untouched. The crate's own doc at imex.rs:158-163 shows the authors know the distinction.

**Impact.** A user who reaches for `positivity_floor` to rescue a strong rarefaction silently trades a positivity violation for a conservation violation, with no bound on the exchange rate and no diagnostic. Because the function is exported at lib.rs:145 as public API, this is a live trap even though no internal solver calls it. The choice of `floor` directly determines the injected mass and there is no documented basis for picking it (the test uses 0.01 with no justification).

**Recommended fix.** State in the docstring that the operation is non-conservative and by how much (return the injected total, or pair it with `conservation_round`), and give a concrete criterion for `floor` (e.g. a fixed fraction of the initial minimum density). Alternatively implement the Zhang-Shu conservative rescaling, which the module already has the machinery for.

**Adversarial check.** The implementation at imex.rs:200-207 is quoted exactly and the analysis is right: `map(|&v| if v > floor { v } else { floor })` with no compensating removal strictly increases the integral whenever any cell is clamped. The docstring at imex.rs:186-188 is quoted verbatim and indeed never mentions conservation. The Zhang & Shu (2010) reference form is correctly stated — conservative positivity limiters rescale about the cell mean, leaving the integral untouched. The test at compressible_imex_tests.rs:152-175 does only assert `worst >= floor - 1e-9` and never measures the injected mass. Downgraded only on impact: no solver calls this function (confirmed by exhaustive grep), so no shipped result is affected; the exposure is limited to an external caller of a helper whose docstring already labels it 'a pragmatic guard' and names the structural upgrade as deferred.

> Evidence re-read: imex.rs:186-208; tests/solvers/qtt/compressible_imex_tests.rs:151-175; workspace grep confirming zero solver call sites

---

### 13.5 [MAJOR] The 2-D/3-D marchers round every step with no conservation fixup, and the shipped production configuration uses a pure bond cap with zero error tolerance

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:247`
- **Auditor confidence:** confirmed

**Claim.** Every step of the 2-D/3-D marchers applies `round(&self.trunc)` two or three times per conserved component. TT-SVD rounding minimizes the Frobenius error, not the integral, so each round perturbs the conserved totals. Under a `Truncation::by_bond(cap)` policy both tolerances are exactly zero, so the truncation error per round is whatever the (cap+1)-th singular value happens to be — unbounded, unmeasured, and with no conservation fixup applied. The shipped plasma-blackout production example runs exactly this configuration.

**Code evidence.**

```
marcher_2d.rs:247-248: `let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;\n Ok(self.acoustic_inv.apply(&predictor)?.round(&self.trunc)?)`\nmarcher_3d.rs:263-265 does the same three times per component.\ndeep_causality_tensor .../truncation/mod.rs:77-83: `/// Builds a pure bond-cap policy (both tolerances zero): keep the leading `max_bond` values.\npub fn by_bond(max_bond: usize) -> Result<Self, CausalTensorError> {\n    Self::new(max_bond, T::zero(), T::zero())\n}`\nShipped production config — examples/avionics_examples/src/shared/constants.rs:43-45: `pub const L: usize = 5;` / `/// Bond cap of the tensor-train round policy, i.e. the compression ceiling.\npub const CAP: usize = 16;` and utils.rs:37-39: `pub fn trunc() -> Truncation<FloatType> {\n    Truncation::<FloatType>::by_bond(CAP).expect(\"bond cap is valid\")\n}`\nAt L=5 the maximal mid-bond of a 2^5 x 2^5 field is 32, so a cap of 16 is an active, load-bearing truncation, not a slack ceiling.\nNo conservation test exists for the 2-D or 3-D marcher: grep for `conserv` across tests/solvers/qtt/compressible_marcher2d_tests.rs, compressible_marcher3d_tests.rs and compressible_marcher3d_fitted_tests.rs returns nothing. Only the 1-D marcher has one (compressible_tests.rs:50-81).
```

**Reference form.** A conservative finite-volume scheme guarantees sum_i U_i^{n+1} = sum_i U_i^n on a periodic domain to machine precision. Any lossy compression inserted into the update destroys that guarantee unless it is projected back (which is precisely what the unused `conservation_round` at imex.rs:167 was written to do).

**Impact.** The README describes the QTT family as \"compressible Euler marchers\" carrying a conservative state, and the qtt_sod README advertises that the unit tests gate conservation — but that gate exists only in 1-D and only under a `by_tol(1e-10)` policy. The shipped 2-D production run has neither a bounded truncation error nor a conservation fixup nor a conservation test. An engineer cannot state, from anything in this repository, what the mass or energy budget error of a plasma-blackout run is.

**Recommended fix.** Add a conservation gate for the 2-D and 3-D marchers mirroring compressible_tests.rs:50-81, run it under the shipped `by_bond(16)` policy, and report the measured drift in verification/README.md. If the drift is material, wire `conservation_round` into `step_component`. At minimum, document on `CompressibleMarcher2d`/`3d` that a bond-capped truncation carries no error bound and therefore no conservation bound.

**Adversarial check.** marcher_2d.rs:247-248 rounds twice per component per step; marcher_3d.rs:262-265 rounds three times (div, predictor, post-inverse) — verified verbatim. Truncation::by_bond is a pure cap with both tolerances zero (causal_tensor_network/truncation/mod.rs:77-83 — the auditor's elided path resolves to this file and the line numbers match). The shipped avionics config is CAP=16 at L=5 (constants.rs:43-45) wired through utils.rs:37-39, and the maximal mid-bond of a 32x32 QTT field is 32, so the cap is load-bearing. I re-grepped for 'conserv' across the three marcher test files and found nothing; the only conservation test is compressible_tests.rs:49-81 for the 1-D marcher under Truncation::by_tol(1e-10) (compressible_tests.rs:15-17). The reference form is correct: TT-SVD rounding minimizes Frobenius error, which does not preserve the sum, and a pure bond cap gives no a-priori error bound. Nothing in the repository states the mass/energy budget error of a 2-D marched run.

> Evidence re-read: marcher_2d.rs:247-248; marcher_3d.rs:251-266; deep_causality_tensor/src/types/causal_tensor_network/truncation/mod.rs:77-83; examples/avionics_examples/src/shared/constants.rs:41-45 and utils.rs:36-39; tests/solvers/qtt/compressible_tests.rs:15-17, 49-81; grep 'conserv' over the three marcher test files (no hits)

---

### 13.6 [MAJOR] Negative pressure is silently floored to 1e-300 in the wave-speed estimate, with no error path — the standard shock-capturing failure mode is masked

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:111`
- **Auditor confidence:** confirmed

**Claim.** All four marchers reject non-positive density but silently accept non-positive pressure (equivalently negative internal energy), substituting a tiny positive floor into the sound-speed formula only. The run completes and returns numbers for a state outside the hyperbolic admissible set. The defect is the missing p > 0 / e_internal > 0 invariant check, not the choice of the 1e-300 constant, and it does not measurably change the grid-wide s_max.

**Code evidence.**

```
euler_1d.rs:100  `let tiny = R::from_f64(1e-300).unwrap_or_else(R::zero);`\neuler_1d.rs:103-107 (density IS guarded): `if r <= R::zero() || !r.is_finite() {\n     return Err(PhysicsError::PhysicalInvariantBroken(\n         \"compressible Euler: density must stay positive\".into(),\n     ));\n }`\neuler_1d.rs:110-115 (pressure is NOT): `// Wave speed uses a positive pressure floor for robustness; the flux carries the true p.\n let p_floor = if p > tiny { p } else { tiny };\n let c = (self.gamma * p_floor / r).sqrt();\n f1.push(mom[i]);\n f2.push(mom[i] * u + p);\n f3.push((energy[i] + p) * u);`\nIdentical pattern at marcher_2d.rs:123/135-136, marcher_3d.rs:124/138-139, marcher_3d_fitted.rs:136/148-149.\nThe value 1e-300 appears in four files with no cited basis and no explanation of why that magnitude rather than, say, a fraction of the initial pressure.
```

**Reference form.** For the Euler equations the admissible set is {rho > 0, p > 0} (equivalently e_internal = E - 0.5*rho|u|^2 > 0). Both are physical invariants of equal standing; see Zhang & Shu (2010) on positivity-preserving schemes, where the pressure constraint is the harder of the two to maintain. A first-order central + LLF scheme loses pressure positivity before density positivity in strong rarefactions.

**Impact.** Negative internal energy is the canonical failure of a shock-capturing scheme in a strong expansion. Here it produces no error, no log entry, and no NaN — the run completes and returns numbers. The comment at euler_1d.rs:110 documents the mechanism but frames it as \"robustness\", which is precisely backwards: it converts a detectable failure into an undetectable one. The 1e-300 magnitude is a magic number with no traceable justification in any of the four files.

**Recommended fix.** Return `PhysicsError::PhysicalInvariantBroken(\"pressure must stay positive\")` when p <= 0, matching the density guard three lines above. If a floor is genuinely wanted for a documented robustness reason, make it a constructor parameter with a stated basis (e.g. 1e-12 * p_initial), log when it activates, and document that a run in which it activated is not a valid result.

**Adversarial check.** All four sites are real and identical (euler_1d.rs:100/103-115, marcher_2d.rs:123/125-136, marcher_3d.rs:124/138-139, marcher_3d_fitted.rs:136/148-149): density non-positive raises PhysicalInvariantBroken, pressure non-positive does not. The reference form is right — {rho>0, p>0} are invariants of equal standing and pressure positivity is the harder one. The real defect is the missing invariant check, and it is confirmed. Two corrections. First, the magic-number axis is misapplied: 1e-300 is a smallest-positive guard, and any tiny positive value gives the same behaviour (c ~ 0) — the magnitude is immaterial, so 'no traceable justification for that magnitude' is not the defect. Without the floor the site produces NaN, so the floor's stated purpose ('the flux carries the true p', euler_1d.rs:110) is accurate as written. Second, the impact claim that s_max is reduced 'exactly when more dissipation is needed' is overstated for these schemes: s_max is a single grid-wide maximum, so one cell contributing |u| instead of |u|+c generally does not move it. The consequence is not under-dissipation, it is that a non-hyperbolic state marches on undetected while the flux carries a negative p.

> Evidence re-read: euler_1d.rs:99-122; marcher_2d.rs:121-152; marcher_3d.rs:122-124, 138-139; marcher_3d_fitted.rs:134-136, 148-149

---

### 13.7 [MAJOR] The ionization-lag time constant uses total heavy-particle density in place of the bimolecular reactant densities, while the doc calls the rate "grounded - not a free fit"

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:184`
- **Auditor confidence:** likely

**Claim.** The associative-ionization channel N + O -> NO+ + e- is bimolecular: d[NO+]/dt = k_f*[N]*[O], so the characteristic time is tau = 1/(k_f * [O]) for an N atom (or symmetrically). The code uses tau_ion = 1/(k_si * n_tot2) with n_tot2 the *total* post-shock heavy-particle number density, which equals [N]*[O]/n only when the gas is fully dissociated into equal parts N and O. At RAM-C II conditions the atom mole fractions are well below unity, so tau_ion is under-estimated and the lag factor (1 - exp(-t_res/tau)) is over-estimated.

**Code evidence.**

```
fitting.rs:183-187: `let k_si = k_cgs * cm3_per_m3 / avogadro;\n let tau_ion = R::one() / (k_si * post.n_tot2);\n\n let frac = R::one() - (R::zero() - residence_time / tau_ion).exp();\n let alpha = alpha_eq * frac;`\nIdentical at fitting.rs:259-263 in `stagnation_line_blackout_2t`.\nThe doc claim, fitting.rs:154-159: `/// `tau_ion = 1 / (k_f . n_2)`, with `k_f` the **dominant associative-ionization rate** N + O -> NO+ + e-\n/// (Park / Gupta), grounded - not a free fit. The closed-form LER relaxation gives\n/// `alpha = alpha_eq.(1 - e^{-t_res/tau_ion})`, so the peak `n_e` sits well below the Saha equilibrium of`\n`post.n_tot2` is defined at fitting.rs:121 as the total heavy-particle density: `let n_tot2 = n_tot_inf * rho_ratio;` and documented at fitting.rs:35-36 as `/// Post-shock heavy-particle number density `n_2` (m^-3).`\nThe rate constant itself is properly traceable — deep_causality_physics/src/constants/hypersonic.rs:31-44 cites `NASA RP-1232 (Gupta et al. 1990), Table II, reaction 7` with Cf = 9.03e9 cm3/mol/s, eta = 0.5, theta_d = 32400 K.
```

**Reference form.** For A + B -> products with rate k_f, the pseudo-first-order relaxation time for species A is tau = 1/(k_f*[B]), where [B] is the number density of the *co-reactant*, not the total mixture density. Substituting n_tot for [O] over-counts by 1/x_O, the reciprocal O-atom mole fraction.

**Impact.** The rate constant is impeccably sourced but the density it multiplies is not the one the reference rate expression calls for. The error is one-sided (tau too short, so alpha driven too close to the Saha equilibrium the lag was introduced to suppress) and its magnitude is the reciprocal atom mole fraction — potentially an order of magnitude at partial dissociation. The phrase "grounded - not a free fit" invites the reader to treat the resulting n_e as first-principles when one of the two inputs to the rate is a substitution. Given the RAM-C peak n_e is the crate's headline flight-data comparison, this is material.

**Recommended fix.** Either (a) take the N and O number densities (or a dissociation fraction) as explicit arguments and form tau_ion = 1/(k_si * n_O), or (b) if the total-density substitution is a deliberate Tier-B simplification, say so in the docstring with the direction and approximate magnitude of the resulting bias, and soften "grounded - not a free fit" to describe the rate constant rather than the whole tau. UNCERTAIN on the exact magnitude: settling it requires the post-shock O-atom mole fraction at the RAM-C condition, which is not computed anywhere in this module.

**Adversarial check.** fitting.rs:183-184 (`let k_si = k_cgs * cm3_per_m3 / avogadro; let tau_ion = R::one() / (k_si * post.n_tot2);`) and the identical block at :259-260 are verbatim. post.n_tot2 is the total heavy-particle density (set at :121, documented at :34-35). The docstring at :154-158 does call the rate 'grounded - not a free fit'. I re-derived the reference form independently and the auditor has it right: for A + B -> products, the pseudo-first-order relaxation time for A is tau = 1/(k_f*[B]) with [B] the co-reactant number density. For N + O -> NO+ + e-, substituting n_tot for [O] over-counts by 1/x_O, so tau_ion is short by that factor and frac = 1 - exp(-t_res/tau_ion) is biased high — one-sided, toward the Saha equilibrium the lag exists to suppress. The rate constant's provenance is genuinely traceable (deep_causality_physics/src/constants/hypersonic.rs cites NASA RP-1232 Gupta et al. 1990), which sharpens rather than softens the point: the sourced k_f is multiplied by a density the reference rate expression does not call for.

> Evidence re-read: fitting.rs:34-35, 105-130, 153-196, 246-263; deep_causality_physics/src/constants/hypersonic.rs (Park/Gupta rate provenance)

---

### 13.8 [MINOR] CompressibleEuler1d::run silently returns a state at the wrong physical time when it hits an undocumented 1,000,000-step guard

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:166`
- **Auditor confidence:** confirmed

**Claim.** The march loop terminates on `t < t_final && guard < max_steps`. If the step guard trips first, `run` returns the partially-marched state with an `Ok`, indistinguishable from a completed march to `t_final`. The caller receives no time stamp, no step count, and no error. The literal 1_000_000 has no documented basis.

**Code evidence.**

```
euler_1d.rs:165-167: `let mut guard = 0usize;\n let max_steps = 1_000_000usize;\n while t < t_final && guard < max_steps {`\nand euler_1d.rs:192-196, the unconditional success return: `Ok((\n     dequantize(&rho)?.as_slice().to_vec(),\n     dequantize(&mom)?.as_slice().to_vec(),\n     dequantize(&energy)?.as_slice().to_vec(),\n ))`\nThe `# Errors` section of the doc (euler_1d.rs:142-144) lists DimensionMismatch, step errors and positivity failure — the step-budget exhaustion is not mentioned.
```

**Reference form.** A time-marching routine documented as marching \"to physical time t_final\" must either reach t_final or report that it did not. Silently returning an earlier state as if it were the requested one is a correctness contract violation.

**Impact.** A stiff or low-CFL configuration (small dx, large s_max, small cfl) can exhaust 1e6 steps before t_final. The caller then compares a t < t_final state against a t_final reference and sees an unexplained accuracy loss, with nothing in the API to diagnose it. The Sod example needs only ~250 steps so it is nowhere near the guard, but the guard exists precisely for configurations that are.

**Recommended fix.** Return `PhysicsError::NumericalInstability` when the guard trips before t >= t_final, and document the budget on `run`. Alternatively return the achieved time alongside the state so the caller can check.

**Adversarial check.** euler_1d.rs:165-167 is verbatim (`let mut guard = 0usize; let max_steps = 1_000_000usize; while t < t_final && guard < max_steps {`) and the return at :192-196 is unconditional Ok with no time stamp or step count. The `# Errors` block at :141-144 lists DimensionMismatch, step errors and positivity failure, and does not mention budget exhaustion. There is no other exit path — the only in-loop error is the non-physical-wave-speed guard at :174-178. The 1_000_000 literal carries no comment and no named constant. The contract violation is real: the doc at :139 says 'March ... to physical time t_final'.

> Evidence re-read: euler_1d.rs:139-197

---

### 13.9 [MINOR] The CFL number is documented as "<= 1" but never validated, so an unstable configuration runs to completion

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:63`
- **Auditor confidence:** confirmed

**Claim.** `CompressibleEuler1d::new` documents `cfl` as "CFL number `cfl` (<= 1)" but the constructor validates neither the bound nor positivity nor finiteness. Since dt = cfl*dx/s_max, a caller passing cfl = 2.0 runs the forward-Euler global-LF scheme at twice its stability limit and gets divergence rather than an error.

**Code evidence.**

```
euler_1d.rs:62-63 (the documented contract): `/// Build the marcher for a periodic `2^L`-point grid of spacing `dx`, ratio of specific heats\n/// `gamma`, and CFL number `cfl` (<= 1).`\neuler_1d.rs:67-85 (the constructor, which validates nothing about cfl, gamma or dx): `pub fn new(\n     l: usize,\n     dx: R,\n     gamma: R,\n     cfl: R,\n     trunc: Truncation<R>,\n ) -> Result<Self, PhysicsError> {\n     let grad = gradient::<R>(l, dx, &trunc)?;\n     let lap = laplacian::<R>(l, dx, &trunc)?;\n     Ok(Self { l, dx, gamma, cfl, grad, lap, trunc })\n }`\nCompare `FittedNormalShock::new` (fitting.rs:92-99), which does reject gamma <= 1, and `CompressibleMarcher2d::new` (marcher_2d.rs:82-87), which does reject non-positive s_ref — so the crate's own convention is to validate.
```

**Reference form.** For the global Lax-Friedrichs / Rusanov update with forward Euler time stepping, the stability condition is s_max*dt/dx <= 1, i.e. exactly the CFL number the constructor takes. This is the standard bound (LeVeque, Finite Volume Methods, sec. 4.4).

**Impact.** The documented precondition is unenforced, so a mis-specified CFL produces garbage rather than a rejection. gamma <= 1 and dx <= 0 are likewise unvalidated here while being validated elsewhere in the same crate, so the API is inconsistent about which preconditions it defends.

**Recommended fix.** Add constructor validation: `cfl` finite and in (0, 1], `gamma` > 1, `dx` finite and > 0, returning `PhysicsError::PhysicalInvariantBroken`. This matches the existing pattern at fitting.rs:92-99.

**Adversarial check.** euler_1d.rs:62-63 documents 'CFL number `cfl` (<= 1)' and the constructor at :67-85 validates nothing — not cfl, not gamma, not dx. The contrast cases are real: FittedNormalShock::new (fitting.rs:92-98) rejects gamma <= 1, and CompressibleMarcher2d::new (marcher_2d.rs:82-87) rejects non-finite/non-positive s_ref, so the crate's convention is to defend documented preconditions. I re-derived the reference bound independently: the update U^{n+1} = U^n - dt/(2dx)(F_{i+1}-F_{i-1}) + s*dt/(2dx)(U_{i+1}-2U_i+U_{i-1}) — which is exactly what update_component assembles with diss = 0.5*s_max*dx (euler_1d.rs:183) — is stable under s_max*dt/dx <= 1, i.e. the CFL number the constructor takes. The auditor's reference form is correct.

> Evidence re-read: euler_1d.rs:62-85, 124-137, 179-189; fitting.rs:92-98; marcher_2d.rs:82-87

---

### 13.10 [MINOR] conservation_round re-rounds after applying the fixup, so it does not exactly "pin" the total as documented

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:183`
- **Auditor confidence:** confirmed

**Claim.** The docstring states the rank-1 uniform fixup "pins the total to `target` with no secular drift". The implementation applies the fixup and then calls `round(trunc)` a second time on the result, which can perturb the integral again. The returned field's total is therefore target plus a second-round residual, not target.

**Code evidence.**

```
imex.rs:161-163 (the claim): `/// rounding error and the solver residual each step.` preceded by `... this pins the total to `target` with no secular drift, projecting out both the`\nimex.rs:175-183 (the implementation): `let rounded = u.round(trunc)?;\n ...\n let delta = (target - total_after) / n_r;\n let offset = quantize(&CausalTensor::new(vec![delta; n], vec![n])?, trunc)?;\n Ok(rounded.add(&offset)?.round(trunc)?)`\nThe final `.round(trunc)?` is applied after the integral has been corrected, so the correction is not the last operation.\nThe test at tests/solvers/qtt/compressible_imex_tests.rs:124-129 uses `Truncation::by_tol(1e-2)` — a 1% relative policy — and asserts `(sum(&rounded) - s0).abs() < 1e-9` on a field whose sum is ~128. It passes only because the rank-1 constant offset happens not to trigger truncation on this particular field; that is not guaranteed in general.
```

**Reference form.** An operation documented to pin a conserved integral to a target must have the correction as its final, exact step. Any lossy operation applied afterwards reintroduces the error the correction removed.

**Impact.** The guarantee is weaker than documented by an amount that depends on the truncation policy and the field's spectrum. Under an aggressive policy (the shipped `by_bond(16)`) the residual could be material, and the single test would not detect it because it exercises a smooth rank-2 sine on a tolerance policy. A caller relying on the documented "pins" would not think to re-measure.

**Recommended fix.** Drop the trailing `.round(trunc)` (the rank-1 offset raises the bond by at most 1, so an unconditional re-round is not needed), or re-measure and re-correct after the final round in a short fixed-point loop. Update the docstring to state the achieved bound rather than "pins". Strengthen the test to run under `by_bond` and to assert a relative bound.

**Adversarial check.** imex.rs:175-183 ends with `Ok(rounded.add(&offset)?.round(trunc)?)` — the lossy round is the final operation, applied after the integral has been corrected, exactly as claimed. The docstring at :160-163 says the fixup 'pins the total to `target` with no secular drift'. The reference form is sound: a correction documented as pinning an integral must be the terminal exact step. The test detail also checks out — compressible_imex_tests.rs:117-129 builds `coarse = Truncation::by_tol(1e-2)`, calls conservation_round with it, and asserts an absolute 1e-9 deviation on a field summing to ~128; it passes because a rank-1 constant offset added to a rank-2 field does not trip that tolerance, which is a property of this field, not a guarantee.

> Evidence re-read: imex.rs:158-184; tests/solvers/qtt/compressible_imex_tests.rs:116-130

---

### 13.11 [INFO] The IMEX module doc claims "the step is unconditionally robust", but the lagged variable remainder A1 is explicit and carries its own diffusion-number stability bound

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/imex.rs:22`
- **Auditor confidence:** likely

**Claim.** The module doc never states the diffusion-number stability bound that the explicitly-lagged remainder A1 imposes on the composite step, and the sentence 'so the step is unconditionally robust' is ambiguous enough to be read as covering it. The claim is true of the A0 core it directly follows.

**Code evidence.**

```
imex.rs:20-23: `... and its inverse is applied in `O(l)` shift-applies with no iterative\n/// solve, so the step is unconditionally robust and **free-stream-exact** (an AMEn-per-step solve loses\n/// free-stream to its residual tolerance).`\nimex.rs:14-15 shows A1 is the variable remainder: `//!   A_1 = -dt*kappa*(c^2-cbar^2)*d2  (variable remainder - a bounded perturbation, lagged explicitly)`\nimex.rs:124-128, the lagged application: `let rem = self\n     .dc2\n     .hadamard_rounded(&lap_u, &self.trunc)?\n     .scale(self.kappa * self.dt);\n let rhs = u.add(&conv)?.add(&rem)?.round(&self.trunc)?;`\nNothing bounds `dc2`; it is whatever c2 - cbar2 is, computed at imex.rs:96.\nThe gating test itself runs c = 1 + 0.3*sin (tests/solvers/qtt/compressible_imex_tests.rs:40-49), giving c^2 in [0.49, 1.69] against cbar2 ~ 1.045 — the remainder is ~62% of the mean, not a small perturbation.
```

**Reference form.** In an IMEX splitting, the explicitly-treated operator sets the stability constraint. For an explicit diffusion term with coefficient kappa*(c^2-cbar2), the constraint is dt*kappa*max|c^2-cbar2|/dx^2 <= 1/2. Unconditional stability holds only for the implicitly-treated part. (Ascher, Ruuth & Spiteri, Appl. Numer. Math. 25 (1997) 151.)

**Impact.** An engineer reading "the step is unconditionally robust" will omit a stability check on the variable sound-speed field. The claim is true of A0 and of the free-stream property, and false of the composite step whenever c^2(x) varies appreciably — which is exactly the regime the module says motivates the split ("the acoustic mode is stiff at micrometre cells"). Note the same doc correctly qualifies elsewhere; this one sentence over-reaches.

**Recommended fix.** Rewrite imex.rs:22 to scope the claim: "the implicit core is advanced unconditionally stably and free-stream-exactly; the lagged remainder A1 retains an explicit diffusion-number bound dt*kappa*max|c^2 - cbar^2|/dx^2 <= 1/2". Consider exposing that bound as a checkable method on `AcousticImex1d` so a caller can validate its dt.

**Adversarial check.** The quoted sentence exists verbatim at imex.rs:20-22, A1 is described as 'lagged explicitly' at imex.rs:15, and step() applies it explicitly at imex.rs:124-128 with `dc2` unbounded (computed at :96 as c2 - cbar2). The reference form is right: in an IMEX split the explicit operator sets the stability constraint, dt*kappa*max|c^2-cbar2|/dx^2 <= 1/2 here. Downgraded on reading: in context 'the step' most naturally names the antecedent of the sentence — 'That core is advanced by its closed-form low-rank inverse ... applied in O(l) shift-applies with no iterative solve, so the step is unconditionally robust' — i.e. the A0 core solve, whose robustness claim is true, and the word is 'robust', not 'unconditionally stable'. The same paragraph two lines earlier states plainly that A1 is lagged explicitly and calls it 'a bounded perturbation'. This is an ambiguous sentence that invites an overread, not a false statement; the genuinely missing item is that no doc or code states the explicit diffusion-number bound the composite step inherits.

> Evidence re-read: imex.rs:6-24, 88-97, 113-130; tests/solvers/qtt/compressible_imex_tests.rs (build helper with c = 1 + 0.3*sin)

---

### 13.12 [MINOR] Post-shock number density is set from the mass-density ratio, assuming an unchanged mean molecular mass across the shock

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:121`
- **Auditor confidence:** confirmed

**Claim.** `n_tot2 = n_tot_inf * rho_ratio` is exact only if the mean molecular mass is identical on both sides of the shock. Since rho2/rho1 = (n2*mbar2)/(n1*mbar1), the correct relation is n2/n1 = (rho2/rho1)*(mbar1/mbar2). At the RAM-C conditions this module targets, post-shock air dissociates substantially, so mbar2 < mbar1 and the true n2 exceeds the computed value. The `PostShockState.n_tot2` doc carries no caveat.

**Code evidence.**

```
fitting.rs:121: `let n_tot2 = n_tot_inf * rho_ratio;`\nfitting.rs:35-36 (the doc, uncaveated): `/// Post-shock heavy-particle number density `n_2` (m^-3).\n    pub n_tot2: R,`\nThe module is explicitly aware that the gas reacts — fitting.rs:88-89 documents gamma as `(> 1; an effective post-shock value for reacting air narrows the `T_2` over-prediction of the perfect-gas value)` — so the perfect-gas assumption is corrected for temperature but not for number density.\nThe test tests/solvers/qtt/compressible_fitting_tests.rs:52-53 encodes the same assumption as the expectation: `// Number density scales with the density ratio.\n assert!((post.n_tot2 - 1.0e22 * rho_ratio_exact).abs() / post.n_tot2 < 1e-12);`
```

**Reference form.** Continuity across a normal shock conserves mass flux, so rho2/rho1 = ((gamma+1)M^2)/((gamma-1)M^2+2). Converting to number density requires the mean molecular mass ratio: n2/n1 = (rho2/rho1)*(mbar1/mbar2). For dissociating air behind a strong shock mbar drops toward half its freestream value (Anderson, Hypersonic and High-Temperature Gas Dynamics, Ch. 11).

**Impact.** n_tot2 feeds directly into the Saha equilibrium (`park2t_ionization_surrogate_kernel(t2, post.n_tot2)`), the electron density (`n_e = alpha * n_tot2`), and the ionization lag (`tau_ion = 1/(k*n_tot2)`) — i.e. every downstream blackout number. The bias is systematic and in a single direction. Because the crate uses an effective reacting gamma to correct T2, a reader reasonably assumes the reacting correction has been applied consistently; it has not been applied to the number density.

**Recommended fix.** Either accept a mean-molecular-mass ratio (or dissociation fraction) argument and apply it, or document on `PostShockState.n_tot2` and on `post_shock` that the number-density ratio equals the mass-density ratio under a frozen-composition assumption, and state which direction that biases n_e.

**Adversarial check.** fitting.rs:121 is verbatim `let n_tot2 = n_tot_inf * rho_ratio;` and the field doc at :34-35 carries no caveat. The reference relation is correctly stated: rho2/rho1 = (n2*mbar2)/(n1*mbar1), so n2/n1 = (rho2/rho1)*(mbar1/mbar2); with post-shock dissociation mbar2 < mbar1 and the true n2 exceeds the computed value, a systematic one-directional bias. The internal-inconsistency argument holds: fitting.rs:87-88 documents gamma as 'an effective post-shock value for reacting air', so the reacting correction is applied to T2 but not to the number density. The propagation is as claimed — n_tot2 feeds park2t_ionization_surrogate_kernel (:142, :169, :244), n_e = alpha*n_tot2 (:188, :264) and tau_ion (:184, :260). The test at compressible_fitting_tests.rs:52-53 does encode the assumption as the expectation.

> Evidence re-read: fitting.rs:34-42, 87-88, 105-130, 142, 169, 184, 188, 244, 260, 264; tests/solvers/qtt/compressible_fitting_tests.rs:38-54

---

### 13.13 [INFO] The Sod verification and baseline name the flux "Rusanov (local Lax-Friedrichs)" but the implementation uses a single global wave speed

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/qtt_sod/README.md:14`
- **Auditor confidence:** confirmed

**Claim.** The verification README, the summary table, and the baseline all name the scheme local Lax-Friedrichs. `flux_and_speed` computes a single scalar s_max over the entire grid and `run` applies it uniformly, which is the *global* Lax-Friedrichs flux — strictly more dissipative than local-speed Rusanov, and a different scheme.

**Code evidence.**

```
verification/qtt_sod/README.md:14: `marches the Rusanov (local Lax-Friedrichs) update.`\nverification/README.md:40: `| `qtt_sod` | Sod shock tube vs exact Riemann (L1 of rho/u/p) | 0.018 / 0.027 / 0.015 | < 0.03 (1st-order Rusanov) | ...`\nverification/qtt_sod/baseline.txt: `Ideal-gas EOS + Rusanov (local Lax-Friedrichs) flux;`\nThe implementation, euler_1d.rs:99 and 116-119: `let mut s_max = R::zero();` ... `let speed = u.abs() + c;\n     if speed > s_max {\n         s_max = speed;\n     }` — one scalar for the whole grid, applied uniformly at euler_1d.rs:183 `let diss = half * s_max * self.dx;`\nThe module doc is more accurate but still mislabels: euler_1d.rs:20 `//! so it is assembled from ... `s_max = max(|u| + c)` is the state-derived global wave speed (the LLF estimate).` — calling a global maximum "the LLF estimate" is the same conflation.
```

**Reference form.** Rusanov / local Lax-Friedrichs uses an interface-local speed s_{i+1/2} = max(|u_L|+a_L, |u_R|+a_R). Global (Rusanov-)Lax-Friedrichs uses s = max over the whole grid. The two differ in dissipation by the ratio of local to global max speed; the global variant is the classical Lax-Friedrichs scheme. (Toro, Riemann Solvers, sec. 10.5.)

**Impact.** The naming understates the numerical dissipation of the shipped scheme. On the Sod problem the global s_max is set by the left state (a ~1.18, u 0) while much of the domain has far lower local speeds, so the contact smearing is worse than the label implies, and a reader benchmarking against a published local-Rusanov result will not reproduce these L1 errors. The physics is self-consistent and the docs do say "global" in one place; the mismatch is a traceability defect, not a math error.

**Recommended fix.** Rename consistently to "global Lax-Friedrichs (Rusanov with a global wave-speed estimate)" in verification/qtt_sod/README.md:14, verification/README.md:40, the baseline text, and euler_1d.rs:7 and :20. If local speeds are intended later, note it as the named upgrade the way the TT-cross upgrade is already noted.

**Adversarial check.** All three doc quotes are verbatim (verification/qtt_sod/README.md:14; verification/README.md qtt_sod row; baseline.txt Notes line 'Ideal-gas EOS + Rusanov (local Lax-Friedrichs) flux'), and euler_1d.rs:99/116-119/183 does compute one scalar s_max over the whole grid and apply it uniformly. The reference distinction is stated correctly: Rusanov/LLF uses an interface-local s_{i+1/2}, the grid-wide maximum is the global (classical) Lax-Friedrichs variant. Downgraded because the mislabel is accompanied everywhere by the unambiguous formula: qtt_sod/README.md:19 writes 's_max = max(|u| + c)' with no index, euler_1d.rs:20 calls it 'the state-derived global wave speed', and euler_1d.rs:87-88 says 'the global LLF wave speed'. A reader is told exactly what is computed; the parenthetical alias is loose, not the substance. This is a naming/traceability nit, and the auditor concedes the physics is self-consistent.

> Evidence re-read: verification/qtt_sod/README.md:11-24; verification/README.md qtt_sod row; verification/qtt_sod/baseline.txt Notes; euler_1d.rs:6-22, 87-122, 179-189

---

### 13.14 [MINOR] peak_bond_tracks_growth_over_the_march asserts a condition that cannot fail and does not test what its comment claims

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_marcher2d_tests.rs:275`
- **Auditor confidence:** confirmed

**Claim.** The test's comment states the tracked peak max_bond "must rise above its starting value - exercising the bond-growth branch in `run`". The assertion is `peak >= 1`, which holds for any valid tensor train (a rank-1 train has max_bond 1), independent of whether the bond ever grew. The stated property is never checked.

**Code evidence.**

```
compressible_marcher2d_tests.rs:251-256 (the claim): `// A localized perturbation on an otherwise-uniform field encodes at low rank, then develops structure\n// under the march, so the tracked peak `max_bond` must rise above its starting value - exercising the\n// bond-growth branch in `run`.`\ncompressible_marcher2d_tests.rs:275: `assert!(peak >= 1, \"peak bond must be recorded: {peak}\");`\nThe quantity under test is produced by marcher_2d.rs:191-198: `let mut peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);\n for _ in 0..steps {\n     u = self.step(&u)?;\n     let step_peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);\n     if step_peak > peak {\n         peak = step_peak;\n     }\n }` — the `if step_peak > peak` branch the comment refers to is never verified to have been taken.
```

**Reference form.** A test asserting a growth property must compare against the initial value. Here that would mean capturing the encoded state's max_bond before the march and asserting the returned peak strictly exceeds it.

**Impact.** The bond-growth tracking in `run` — which is the rank witness the Stage-5 gate depends on — has no test that would fail if the `if step_peak > peak` branch were deleted. The peak-bond number is reported in the qtt_blunt_body_2d and qtt_reentry_3d verification tables as the headline structural result, so its accumulation logic deserves a real gate.

**Recommended fix.** Capture `quantize_2d(initial)?.max_bond()` and assert `peak > initial_bond`, matching the comment. If the bond does not in fact grow for this initial condition, either strengthen the initial condition or correct the comment.

**Adversarial check.** The comment at compressible_marcher2d_tests.rs:251-256 and the assertion `assert!(peak >= 1, "peak bond must be recorded: {peak}");` at :275 are verbatim. I checked concretely whether an input can make the gate fail: `peak` comes from marcher_2d.rs:191-198, `u.iter().map(|t| t.max_bond()).max().unwrap_or(0)` over a fixed four-element array, so the array is never empty, and every valid tensor train has max_bond >= 1. No input makes peak < 1 — the assertion is unfalsifiable. Deleting the `if step_peak > peak { peak = step_peak; }` branch would leave the test green. The test does independently gate positivity/finiteness of the output (:271-274), so it is not wholly vacuous, but the growth property it names is untested. Correct fix as stated: capture the encoded state's max_bond before the march and assert the returned peak strictly exceeds it.

> Evidence re-read: tests/solvers/qtt/compressible_marcher2d_tests.rs:250-276; marcher_2d.rs:185-199

---

### 13.15 [MINOR] imex_run_conserves_mass cannot fail: conservation_round pins the sum to the asserted target by construction on every step

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_imex_tests.rs:133`
- **Auditor confidence:** confirmed

**Claim.** The test measures the corrector, not the integrator: conservation_round is applied every step, so the assertion reduces to a bound on that function's terminal-rounding residual and is independent of whether AcousticImex1d::step conserves anything. It is falsifiable in principle (a coarse truncation policy would break it), so it is near-circular rather than tautological. No test anywhere measures the uncorrected drift of AcousticImex1d::step.

**Code evidence.**

```
compressible_imex_tests.rs:132-149: `// Task 3.2: no secular drift of the integral of u over a long IMEX run with conservation rounding (periodic mass).\n ...\n let s0 = sum(&u0);\n let mut u = u0;\n for _ in 0..200 {\n     u = imex.step(&u).unwrap();\n     u = conservation_round(&u, s0, &tr()).unwrap();\n }\n assert!(\n     (sum(&u) - s0).abs() / s0.abs() < 1e-8,\n     \"mass must not drift secularly over the run\"\n );`\nThe fixup being tested is imex.rs:181-183: `let delta = (target - total_after) / n_r;\n let offset = quantize(&CausalTensor::new(vec![delta; n], vec![n])?, trunc)?;\n Ok(rounded.add(&offset)?.round(trunc)?)` — it subtracts exactly the deviation from `target` that the assertion then measures.\nThe immediately preceding test, compressible_imex_tests.rs:117-130, already gates the fixup in isolation, so this test adds no independent information.
```

**Reference form.** A conservation test must measure the integral drift of the scheme under test. Applying the corrective projection inside the measurement loop makes the measurement circular — the result is determined by the corrector, not the integrator.

**Impact.** There is no test anywhere in the repository that measures the uncorrected conservation drift of `AcousticImex1d::step`, so the magnitude of the drift the fixup is compensating is unknown. Since no shipped solver calls `conservation_round` at all (separate finding), the drift this test papers over is exactly the drift that production runs actually experience.

**Recommended fix.** Split into two tests: one that marches 200 steps *without* `conservation_round` and records/bounds the observed relative drift (the number an engineer actually needs), and one that marches with it and asserts the drift is reduced to the rounding floor. Rename the first to reflect that it measures rather than gates.

**Adversarial check.** The test body at compressible_imex_tests.rs:132-149 is quoted exactly, conservation_round is applied inside the loop after every imex.step, and the assertion measures the integral the corrector just set. The circularity charge is right in substance: the measurement says nothing about AcousticImex1d::step, and I confirmed no other test anywhere measures its uncorrected drift. But the strict tautology test fails: the assertion *can* fail. conservation_round's terminal `.round(trunc)` (imex.rs:183 — the residual of Finding 9) is applied after the fixup, so the final sum is target plus that residual; with tr() = by_tol(1e-10) it is far below the 1e-8 relative bound, but a coarser policy or a field whose constant offset trips truncation would break it. The test is therefore a (weak, single-step) gate on conservation_round's terminal residual mislabelled as a gate on the integrator's secular drift, not an assertion that no input can falsify.

> Evidence re-read: tests/solvers/qtt/compressible_imex_tests.rs:17-19 (tr() = by_tol(1e-10)), 116-150; imex.rs:167-184

---

### 13.16 [MINOR] post_shock_ratios_match_exact_rh re-types the implementation's own algebra instead of checking against tabulated normal-shock values

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_fitting_tests.rs:41`
- **Auditor confidence:** confirmed

**Claim.** The test that gates the crate's central claim ("the exact Rankine-Hugoniot state") computes its expected values with the same algebraic expressions the implementation uses. A shared error in either expression — a transposed sign, a wrong factor — would be reproduced identically in the expectation and the test would pass. The chosen case M=5, gamma=1.4 has clean tabulated values that were not used.

**Code evidence.**

```
compressible_fitting_tests.rs:40-42 (the expectation): `let m2 = mach * mach;\n let rho_ratio_exact = (gamma + 1.0) * m2 / ((gamma - 1.0) * m2 + 2.0);\n let p_ratio_exact = (2.0 * gamma * m2 - (gamma - 1.0)) / (gamma + 1.0);`\nfitting.rs:118-120 (the implementation): `let rho_ratio = (g + one) * m2 / ((g - one) * m2 + two);\n let u_ratio = one / rho_ratio;\n let p_ratio = (two * g * m2 - (g - one)) / (g + one);`\nThe two are character-for-character the same expression. The assertion at compressible_fitting_tests.rs:48-51 then compares them to 1e-10.\nNo hard-coded reference value appears anywhere in the file.
```

**Reference form.** At M1 = 5, gamma = 1.4 the tabulated normal-shock values are rho2/rho1 = 5.000, p2/p1 = 29.000, T2/T1 = 5.800, M2 = 0.4152 (Anderson, Modern Compressible Flow, Appendix B normal-shock tables). These are exact closed-form values, independent of any code in this repository.

**Impact.** The README states "the exact Rankine-Hugoniot state is the boundary of the marched layer" and calls it a central claim. Its only unit gate is self-referential. I independently verified the implementation against the textbook relations and it is correct, so no defect is being hidden today — but the gate would not catch a future regression in the shared expression, and an auditor cannot take the passing test as evidence.

**Recommended fix.** Replace the recomputed expectations with the tabulated constants: assert rho_ratio ~ 5.0, p_ratio ~ 29.0, and t2/t_inf ~ 5.8 at M=5, gamma=1.4, citing the normal-shock table. Add M2^2 = ((g-1)M1^2+2)/(2g*M1^2-(g-1)) = 0.1724 as a fourth check — the downstream Mach relation is currently not computed or tested at all.

**Adversarial check.** compressible_fitting_tests.rs:40-42 and fitting.rs:118-120 are character-for-character the same expressions for rho_ratio and p_ratio, compared at 1e-10 (:48-51), and I confirmed no hard-coded reference value appears anywhere in the file. A shared algebraic error would be reproduced in the expectation and the test would pass. I independently evaluated the reference at M=5, gamma=1.4: rho2/rho1 = 2.4*25/(0.4*25+2) = 60/12 = 5.000 and p2/p1 = (2*1.4*25-0.4)/2.4 = 69.6/2.4 = 29.000, matching the auditor's tabulated values — so the implementation is correct today and no defect is hidden, exactly as the auditor states. The gate is nonetheless self-referential and would not catch a regression in the shared expression. Note in mitigation: post.t2 is *not* self-referential (it comes from rankine_hugoniot_temperature_kernel) and is band-checked separately at :57-64.

> Evidence re-read: tests/solvers/qtt/compressible_fitting_tests.rs:37-54, 56-64; fitting.rs:110-130

---

### 13.17 [INFO] roll_zeta is documented as "a rank-preserving relabel" but fully densifies the state and re-runs TT-SVD

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_3d_fitted.rs:323`
- **Auditor confidence:** confirmed

**Claim.** The docstring's 'rank-preserving relabel' describes an operation implemented as dequantize -> dense permute -> re-quantize, so nothing structurally preserves the rank and the output is whatever TT-SVD yields under the policy. Contrary to the finding's reference claim, a cyclic shift by an arbitrary integer IS a bond-2 QTT operator (binary constant-addition with a carry bond, as the crate's own shift_plus shows), so the correct reading is that the doc describes the intended low-rank operation while the code takes a dense shortcut that could increase the bond.

**Code evidence.**

```
marcher_3d_fitted.rs:323-324 (the claim): `/// Cyclically roll a train by `shift` cells along `zeta` (a rank-preserving relabel) and re-encode - the\n/// move that keeps a tracked front coordinate-stationary.`\nmarcher_3d_fitted.rs:336-348 (the implementation): `let dense = dequantize_3d(u, lx, ly, lz)?;\n let s = dense.as_slice();\n let mut rolled = vec![R::zero(); nx * ny * nz];\n for i in 0..nx { for j in 0..ny { for k in 0..nz {\n     let src = ((k as isize - shift).rem_euclid(nz as isize)) as usize;\n     rolled[(i * ny + j) * nz + k] = s[(i * ny + j) * nz + src];\n }}}\n quantize_3d(&CausalTensor::new(rolled, vec![nx, ny, nz])?, trunc)`\nThis is called inside the per-step re-pin loop at marcher_3d_fitted.rs:394-396: `for t in u.iter_mut() {\n     *t = roll_zeta(t, lx, ly, lz, shift, &self.trunc)?;\n }`\nThe method's own doc at marcher_3d_fitted.rs:355-357 repeats the claim: `roll the state back to it (a rank-preserving relabel that keeps the feature coordinate-stationary)`.
```

**Reference form.** A rank-preserving operation on a tensor train acts on the cores without recompression (e.g. an MPO with bond 1, or a core permutation). Densify-permute-recompress preserves nothing structurally; the output rank is determined by the SVD. In QTT, a cyclic shift by 2^j is a bond-2 MPO (the crate itself builds these — tensor_bridge/operators.rs:49-78), but a shift by a general integer is not.

**Impact.** `run_repinned` is the mechanism the README's rank story leans on ("bounding the marched chi needs re-pinning", verification/README.md). Describing its central operation as rank-preserving asserts precisely the property the whole exercise is trying to demonstrate. A reader would not expect the re-pin to be capable of increasing the bond it exists to bound. Separately, the O(2^L) densify makes the re-pin path dense-cost, which the module does not flag (though the per-step flux evaluation is already documented as densifying).

**Recommended fix.** Change the wording to "a cyclic relabel, applied densely and recompressed" and, if the rank claim matters, add a test that asserts the post-roll max_bond does not exceed the pre-roll max_bond. Alternatively implement the roll as a composition of the existing bond-2 `shift_plus` powers (the binary decomposition of `shift`), which would make the claim true and stay in compressed form.

**Adversarial check.** Both doc sites (marcher_3d_fitted.rs:323-324 and :355-357) and the implementation (:336-348: dequantize_3d -> dense permute -> quantize_3d) are quoted correctly, and it is called per-step inside run_repinned at :393-396. So the wording does describe an operation implemented by densify-plus-TT-SVD, whose output rank is whatever the SVD yields under the truncation policy. That part stands. The auditor's reference form is wrong, however, and it is load-bearing for the claimed severity: 'a shift by 2^j is a bond-2 MPO ... but a shift by a general integer is not' is incorrect. The crate's own shift_plus (tensor_bridge/operators.rs:39-78) is a binary ripple-carry increment with a rank-2 carry bond; a cyclic shift by an arbitrary integer is binary addition of a constant, which is likewise a bond-2 carry MPO. So a general-integer cyclic shift *is* low-rank in QTT and bounds the shifted field's rank by ~2x the input's — the operation is nearly rank-preserving in the sense the doc intends, and could be implemented without densifying at all.

> Evidence re-read: marcher_3d_fitted.rs:6-20, 322-348, 351-400; tensor_bridge/operators.rs:39-86

---

### 13.18 [INFO] R::from_f64(0.5).unwrap_or_else(R::one) silently doubles the dissipation coefficient rather than erroring, in eight places on the physics path

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/euler_1d.rs:163`
- **Auditor confidence:** confirmed

**Claim.** Nine sites (not eight) lift 0.5 with unwrap_or_else(R::one). The claim is otherwise accurate, including the same-crate error-returning contrast at fitting.rs:113-114. Unreachable for every scalar type the workspace currently instantiates.

**Code evidence.**

```
euler_1d.rs:37: `let half = R::from_f64(0.5).unwrap_or_else(R::one);` (inside `ideal_gas_pressure`)\neuler_1d.rs:163: `let half = R::from_f64(0.5).unwrap_or_else(R::one);` (feeds `diss = half * s_max * self.dx` at line 183)\nmarcher_2d.rs:88: `let half = R::from_f64(0.5).unwrap_or_else(R::one);` (feeds `beta = dt * half * s_ref * h`)\nmarcher_2d.rs:122, marcher_2d.rs:277, marcher_3d.rs:86, marcher_3d.rs:123, marcher_3d_fitted.rs:76, marcher_3d_fitted.rs:135 — same pattern.\nContrast deep_causality_physics/src/kernels/hypersonic/shock.rs:87-88, which errors on the identical conversion: `let half = R::from_f64(0.5)\n     .ok_or_else(|| PhysicsError::NumericalInstability(\"R::from_f64(0.5) failed\".into()))?;`\nAnd fitting.rs:113-114 in this very module, which also errors: `let two = R::from_f64(2.0)\n     .ok_or_else(|| PhysicsError::NumericalInstability(\"from_f64(2.0)\".into()))?;`
```

**Reference form.** The Rusanov dissipation coefficient is exactly 0.5*s_max*dx and the EOS kinetic term is exactly 0.5*rho*|u|^2. Substituting 1.0 for 0.5 gives a different scheme and a different equation of state; a numeric-lift failure is an error condition, not a case with a sensible default.

**Impact.** Unreachable for f64 and f32, so no shipped configuration is affected today. It becomes reachable for any exotic `CfdScalar` (the crate is generic over R and the workspace ships Float106 and other scalar types) whose `from_f64` can fail. The consequence would be a silently different scheme rather than an error — the worst failure mode for a certification consumer. The inconsistency with the same crate's own error-returning convention two files away makes this a defect rather than a style choice.

**Recommended fix.** Replace all eight `unwrap_or_else(R::one)` with `ok_or_else(|| PhysicsError::NumericalInstability(...))?`, matching fitting.rs:113-114 and shock.rs:87-88. For `ideal_gas_pressure`/`ideal_gas_pressure_2d`, which currently return R rather than Result, either change the signature or derive the half as `R::one()/(R::one()+R::one())`, which cannot fail.

**Adversarial check.** A grep for from_f64(0.5) over the compressible module returns nine sites, not eight — every line the auditor lists (euler_1d.rs:37, 163; marcher_2d.rs:88, 122, 277; marcher_3d.rs:86, 123; marcher_3d_fitted.rs:76, 135) exists verbatim. The contrast is real: deep_causality_physics/src/kernels/hypersonic/shock.rs:87-88 errors on the identical conversion, and fitting.rs:113-114 in this very module errors on from_f64(2.0). The reference point is trivially correct — substituting 1.0 for 0.5 changes both the EOS and the dissipation coefficient, so a lift failure is an error condition and not a case with a sensible default. Severity is bounded exactly as the auditor concedes: unreachable for f64/f32, so no shipped configuration is affected; the exposure is a future exotic CfdScalar. Recording it as info rather than minor on that basis.

> Evidence re-read: grep from_f64(0.5) over src/solvers/qtt/compressible/ (9 hits); euler_1d.rs:36-39, 163; marcher_2d.rs:88, 122, 277; deep_causality_physics/src/kernels/hypersonic/shock.rs:80-92; fitting.rs:112-114

---

### 13.19 [MINOR] relax_length is in units of the normalized sampled extent, not a physical length as the docstring implies

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:274`
- **Auditor confidence:** confirmed

**Claim.** `relaxation_profile_bond` documents the profile as `n_e(s) = alpha_eq*n_2*(1 - e^{-s/L})` "along the streamline" with `relax_length` as L, implying a physical distance. The implementation forms s = i/N, a dimensionless index fraction in [0,1), so `relax_length` must be supplied as a fraction of the sampled extent. Neither the parameter doc nor the `# Errors` section states this.

**Code evidence.**

```
fitting.rs:274-278 (the doc): `/// The post-shock ionization relaxation profile `n_e(s) = alpha_eq.n_2.(1 - e^{-s/L})` along the streamline\n/// as a 1-D QTT field (the smooth post-shock zone). Returns `(max_bond, peak n_e)` - the bond witnesses\n/// the \"each side `O(1)` rank\" of task 4.1.`\nfitting.rs:294-300 (the implementation): `for (i, d) in data.iter_mut().enumerate() {\n     let s = R::from_usize(i)\n         .ok_or_else(|| PhysicsError::NumericalInstability(\"from_usize(i)\".into()))?\n         / n_r;\n     let frac = R::one() - (R::zero() - s / relax_length).exp();\n     *d = peak * frac;\n }`\n`s` is i/N in [0,1). The only caller that gets this right does so by convention encoded at the call site, not in the API — verification/qtt_ramc_stagline/config.rs:44-45: `/// Relaxation length as a fraction of the sampled streamwise extent.\npub const RELAX_LENGTH: f64 = 0.2;`\nThe unit test passes 0.2 with no comment at all (tests/solvers/qtt/compressible_fitting_tests.rs:185).
```

**Reference form.** A parameter named `relax_length` in a physics API describing a profile "along the streamline" is read as a physical length in metres. The function's actual contract is a dimensionless fraction of the sampled domain.

**Impact.** A caller passing a physical relaxation length in metres (e.g. 0.02 m) would get a profile that saturates in the first few cells, and the returned bond and peak would look plausible. Because the returned peak is `alpha_eq * n_tot2` regardless of the profile shape (fitting.rs:289), the error would show up only in the reported bond — the exact quantity the function exists to witness.

**Recommended fix.** Add a parameter section to the docstring: `relax_length` is the e-folding length expressed as a fraction of the sampled streamwise extent (s runs 0..1 over the 2^l samples). Consider renaming to `relax_fraction`, or accepting a physical length plus the physical extent so the function can non-dimensionalize internally.

**Adversarial check.** fitting.rs:274-276 documents 'n_e(s) = alpha_eq.n_2.(1 - e^{-s/L}) along the streamline'. The loop at :294-300 computes s = R::from_usize(i)/n_r, i.e. i/N in [0,1), and then frac = 1 - exp(-s/relax_length). relax_length is therefore necessarily a fraction of the sampled extent, and neither the parameter doc nor the # Errors block (:278-279) says so. I confirmed the mitigation the auditor names: verification/qtt_ramc_stagline/config.rs documents RELAX_LENGTH as 'Relaxation length as a fraction of the sampled streamwise extent' at the call site, and the unit test at compressible_fitting_tests.rs:181-184 passes 0.2 with no comment. The impact analysis is also right: peak is alpha_eq*n_tot2 at :289, independent of relax_length, so a metres-valued argument would corrupt only the returned bond — the quantity the function exists to witness.

> Evidence re-read: fitting.rs:274-303; tests/solvers/qtt/compressible_fitting_tests.rs:179-192; verification/qtt_ramc_stagline/config.rs (RELAX_LENGTH doc)

---

### 13.20 [MINOR] The body-fitted 3-D marcher's radial (zeta) axis is treated as periodic, so no wall or outflow boundary exists — not stated in the marcher's own docs

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_3d_fitted.rs:6`
- **Auditor confidence:** confirmed

**Claim.** All gradient operators underlying `BodyFittedCoordinate3d` are cyclic on every axis, including zeta, which the coordinate maps to the radial/stand-off direction r(zeta) = r0 + zeta*dr. The body surface at zeta=0 and the outer freestream boundary at zeta=1 are therefore wrapped into each other: there is no solid-wall condition and no outflow. `run_repinned`'s `roll_zeta` further relies on that periodicity to cyclically wrap state across the body. The marcher's module docs do not mention it.

**Code evidence.**

```
marcher_3d_fitted.rs:6-20 (the module doc) describes the fitted marcher, the metric seam and the acoustic regulariser but never mentions boundary conditions on zeta.\nThe underlying operators are cyclic by construction — tensor_bridge/operators.rs:88-91: `/// Centered first-difference operator `d_x ~ (u[k+1] - u[k-1])/(2dx)` on a periodic `2^L` grid.` built from `shift_plus`, documented at operators.rs:44-45 as `The MSB mode drops the overflow carry\n/// (cyclic, mod `2^L`)`.\n`BodyFittedCoordinate3d::physical_gradient` uses exactly these — coordinate/body_fitted_3d.rs:31-32 imports `gradient_x_3d, gradient_y_3d, gradient_z_3d` and applies them at body_fitted_3d.rs:232-234.\nThe limitation IS documented, but one module away — coordinate/body_fitted_3d.rs:25-26: `Interior gradients are correct to scheme order;\n//! the periodic radial/polar boundary stencils are the same Stage-2 refinement noted for the 2-D chart.`\nAnd `roll_zeta` wraps across it unconditionally — marcher_3d_fitted.rs:343: `let src = ((k as isize - shift).rem_euclid(nz as isize)) as usize;`
```

**Reference form.** A body-fitted forebody solver requires a wall condition at the body surface (no-penetration for Euler: u.n = 0) and a non-reflecting or supersonic-outflow condition at the outer boundary. A cyclic stencil in the wall-normal direction imposes neither and instead couples the two boundaries.

**Impact.** A reader of `CompressibleMarcher3dFitted` — whose doc says "Identical conservative physics to the Cartesian CompressibleMarcher3d ... over BodyFittedCoordinate3d a bow shock standing off the nose is a zeta = const surface" — would reasonably assume a nose-region simulation with a body. There is no body: freestream cells at the outer edge feed directly into the innermost radial cells. This is fine for the rank-lever measurement the marcher is actually gated on (verification/README.md is explicit that qtt_reentry_3d gates rank, not physical accuracy), but the marcher's own docs do not carry that scope limit, and `run_repinned` is presented as a physics-meaningful re-pin.

**Recommended fix.** Repeat the boundary caveat in marcher_3d_fitted.rs's module doc and on `run_repinned`, mirroring the honest note already present at body_fitted_3d.rs:25-26 and the one at marcher_3d_fitted.rs:18-20 about the computational-space regulariser. State plainly that the zeta axis is periodic, that no wall or outflow condition is enforced, and that results are therefore rank/structure witnesses rather than forebody flowfield predictions.

**Adversarial check.** I read marcher_3d_fitted.rs:6-20 in full: it covers the five-train IMEX state, the MetricProvider3d seam, the acoustic regulariser and the named Stage-2 refinements, and says nothing about boundary conditions. The underlying operators are cyclic by construction — operators.rs:39-45 documents shift_plus as dropping the overflow carry 'cyclic, mod 2^L', and gradient is built from it — and BodyFittedCoordinate3d::physical_gradient uses gradient_x/y/z_3d. r(zeta) = r0 + zeta*dr is confirmed at body_fitted_3d.rs:12. roll_zeta wraps unconditionally via rem_euclid (marcher_3d_fitted.rs:343). The limitation is documented, but one module away, at body_fitted_3d.rs:25-26: 'the periodic radial/polar boundary stencils are the same Stage-2 refinement noted for the 2-D chart'. The reference form is right — an Euler forebody solver needs u.n = 0 at the wall and a non-reflecting/supersonic outflow at the outer edge, and a cyclic wall-normal stencil supplies neither. The auditor correctly scopes the impact: verification/README.md labels qtt_reentry_3d as gating rank, not physical accuracy; the gap is that the marcher's own docs carry no such scope limit while run_repinned is presented as a physics-meaningful re-pin.

> Evidence re-read: marcher_3d_fitted.rs:6-20, 322-348, 351-400; coordinate/body_fitted_3d.rs:6-26, 231-234; tensor_bridge/operators.rs:39-96; verification/README.md validation-scope labels

---
