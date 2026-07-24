# deep_causality_cfd/studies/ — all 14 study programs, the studies README, and each study's own README

**Production readiness: `needs-work`**

The arithmetic in these studies is largely sound — I verified the relativistic clock split, the Strang convergence order, the blend-metric Jacobian, the A=A0+A1 operator split, the nozzle closed form, and the diffusion-CFL statements against reference forms, and all check out. The defect class is not wrong arithmetic; it is that several headline conclusions are drawn from measurements that do not measure the claimed quantity. Three are load-bearing for design decisions: qtt_repin_marcher's positive Part-2 result (which grounds the Stage-4 "Rankine-Hugoniot interface" mechanism) is measured on a field that is provably independent of the second coordinate, i.e. a 1-D linear advection with no shock, no curvature and no metric; qtt_rank_plume's "fork costs 1.00-1.04x an unforked trunk" never times anything derived from the fork object it creates; and the "body-fitted" references in qtt_rank_study and qtt_rank_3d are synthetic functions of the first index rather than the curved shock resampled in a fitted chart, with one of them bit-identical to the flat control it is gated against. Two gates cannot fail by construction (qtt_rank_dynamic G2, traj_fs1 G4), and qtt_acoustic_precond's AC-C bounds sit 6-9% below the observed values, making them pinned regression bounds presented as a finding. Several studies disclose their limits honestly in caveats sections — srp_momentum_jet and qtt_rank_plume are notably candid — but the studies README summarises them more strongly than either the study README or the code supports, and it carries an unreconciled contradiction between qtt_rank_fitted_dynamic and qtt_repin_marcher on whether re-pinning is the lever. For an avionics R&D consumer the corpus is usable as an engineering notebook but not as decision evidence until the degenerate measurements are replaced with controlled ones and the gates are re-derived.

- Files read: **44**
- Findings raised: **30** — surviving adversarial verification: **29** (refuted: 1)
- Surviving by severity: major 12, minor 13, info 4
- Independently confirmed-correct items: **11**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| traj_fs3 relativistic clock split: formula, sign convention, and all four reported numbers | `studies/traj_fs3_clock/main.rs:52-59, 78-81` | dtau/dt - 1 = Phi/c^2 - v^2/(2c^2), Phi = -GM/r; offset vs a reference clock = [Phi_c - Phi_ref]/c^2 - [v_c^2 - v_ref^2]/(2c^2). Source: Ashby, Living Reviews in Relativity 6:1 (2003), cited in deep_c |
| relativistic_clock_drift_rate_kernel / relativistic_clock_offset_kernel implementation | `deep_causality_physics/src/kernels/chronometric/forward_clock.rs:76-78, 107-114` | 1PN weak-field monopole: dtau/dt - 1 = -GM/(rc^2) - v^2/(2c^2); two-clock offset is the difference of rates. Ashby (2003); IERS Conventions (2010) TN36. |
| traj_fs2 second-order convergence is measured from a genuine refinement, not asserted | `studies/traj_fs2_coupling/main.rs:166-182; output.txt:6-11` | Strang splitting of two non-commuting exact sub-flows is globally O(H^2); observed order = log2(err_H / err_{H/2}) must approach 2. Standard operator-splitting result (e.g. Hairer-Lubich-Wanner, Geome |
| qtt_blend_metric Jacobian of the blended chart | `studies/qtt_blend_metric/main.rs:142-146` | For x = r cos(theta), y = r sin(theta) with r = R0 + eta*DR and theta = -DTHETA/2 + xi*DTHETA: dx/dxi = -r sin(theta) DTHETA, dx/deta = cos(theta) DR, dy/dxi = r cos(theta) DTHETA, dy/deta = sin(theta |
| The dissipation floor formula the srp study attributes to the marcher | `studies/srp_momentum_jet/README.md:25 vs deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:89` | The study claims the marcher applies nu_bar = 0.5 * s_ref * dx. |
| srp_momentum_jet closed-form fixed-nozzle throttle solve | `studies/srp_momentum_jet/config.rs:164-171` | For a fixed exit Mach, rho_e*u_e^2 = gamma_jet * M_e^2 * p_e; thrust per depth T' = h*(rho_e u_e^2 + p_e - p_inf) set equal to C_T*q_inf*D. |
| srp_momentum_jet total-axial-force convention matches the cited J-A kernel | `studies/srp_momentum_jet/main.rs:291 vs deep_causality_physics/src/kernels/propulsion/srp.rs:187,210` | C_A,total = C_T + f(C_T)*C_A0, per srp_total_axial_force_coefficient_kernel's own docstring formula and Jarvinen & Adams (1970) Fig. 56. |
| qtt_acoustic_precond operator split is algebraically exact | `studies/qtt_acoustic_precond/main.rs:191-197` | A = I - dt^2*c(x)^2*d2; A0 = I - dt^2*cbar^2*d2; A1 = -dt^2*(c^2 - cbar^2)*d2. Requires A0 + A1 = A identically. |
| qtt_rank_nonlinear and qtt_rank_3d explicit diffusion-CFL statements | `studies/qtt_rank_nonlinear/main.rs:55,100-101; studies/qtt_rank_3d/main.rs:63` | Explicit-Euler stability for the d-dimensional heat operator requires nu*dt*sum_i(1/h_i^2) <= 1/2, i.e. nu*dt/h^2 <= 1/2 (1-D), <= 1/4 (2-D equal spacing), <= 1/6 (3-D equal spacing). |
| qtt_rank_plume chamber-pressure sizing is exact, not an approximation | `studies/qtt_rank_plume/main.rs:365-367` | For a choked fixed-geometry nozzle, mdot ∝ p_c and p_e ∝ p_c at fixed area ratio, so thrust + p_inf*A_e is exactly linear in p_c through the origin. |
| traj_fs1 orbit parameters printed in the README | `studies/traj_fs1_generator/README.md:23 vs main.rs:236-240` | Vis-viva a = -mu/(2*E) with E = v^2/2 - mu/r; e = \|((v^2-mu/r)r - (r.v)v)/mu\|; T = 2*pi*sqrt(a^3/mu). |

## Findings

### 2.1 [MAJOR] qtt_repin_marcher Part 2 measures a 1-D linear advection with no curvature, no shock and no metric, yet grounds the Stage-4 Rankine-Hugoniot design decision

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_repin_marcher/main.rs:127`
- **Auditor confidence:** confirmed

**Claim.** The field in march_radial_fitted is independent of the xi coordinate at t=0 and every operator applied to it preserves that independence, so the 2-D field is exactly a rank-1 outer product across the xi-block/eta-block cut for all time. The measured 'bond 8, flat in resolution' is therefore the bond of a 1-D tanh front under linear advection-diffusion, not evidence about body-fitted coordinates, re-pinning, curved fronts, or Rankine-Hugoniot interfaces.

**Code evidence.**

```
L137: `.sample(|_xi, eta| 0.5 * (1.0 - ((eta - 0.5) / delta).tanh()))` — the initial condition ignores xi.
L141-142: `let gy = gradient_y::<f64>(l, l, h, &trunc)` and `let lap = laplacian_2d::<f64>(l, l, h, h, &trunc)` — computational-space operators only.
L151: `let adv = gy.apply(&u, &trunc).expect("adv").scale(-speed);` — pure LINEAR advection at constant speed = 1.0 (L145); there is no u^2 flux, so no shock forms.
L125 (docstring, author's own words): "The field stays a function of `eta` (an aligned interface), so its bond stays `O(1)`."
L124 (docstring): "The metric is not needed for the radial update, so the coordinate is used only to lay down the initial condition."
Conclusion drawn at L104-108: "The Stage-4 mechanism is therefore re-pin AND treat the front as an exact Rankine-Hugoniot interface (smooth each side), not march Cartesian fluxes across it."
```

**Reference form.** To support 'aligning the transport with the coordinate bounds the rank', the measurement must retain the two-dimensional structure that makes the rank question non-trivial: a front with curvature in physical space, transported by a flux that couples both coordinates, with the metric entering the update. A field that is exactly f(eta) for all time has bond 1 across the xi/eta cut by the definition of a tensor-train rank (Oseledets, 'Tensor-Train Decomposition', SIAM J. Sci. Comput. 33(5), 2011), so its low bond is a property of the construction, not a result.

**Impact.** The single positive result in the Round-2 batch — the one the studies README elevates to 'THAT is the Res-5 lever working dynamically' and that fixes the Stage-4 mechanism — carries no information about the mechanism it is cited for. An engineer reading the studies README would conclude that re-pinning plus interface tracking has been demonstrated to bound the marched rank of a curved front; nothing in this code demonstrates that. Note that the only case in the whole corpus that does combine curvature with a fitted coordinate (march_polar) grows 25 -> 35.

**Recommended fix.** Replace march_radial_fitted with a case that retains xi-dependence: sample a front that is curved in physical space, advect it with the metric-aware physical flux (coord.physical_gradient, as march_polar does) restricted to the coordinate-normal direction, and re-pin. Report the bond of that. If the intent is only to illustrate that an exactly-aligned interface is rank-1 by construction, say so and drop the Stage-4 mechanism conclusion, which the measurement cannot carry.

**Adversarial check.** Every quoted line is present verbatim. L137 samples `|_xi, eta| 0.5*(1.0 - ((eta-0.5)/delta).tanh())` — no xi dependence. The update at L151-153 uses only gradient_y (d/d-eta) and laplacian_2d; applied to a xi-constant field the xi-derivative terms vanish identically, and roll_eta is a pure eta relabel, so the field is exactly f(eta) for all 200 steps. With block bit-ordering the xi cores are then bond 1 and max_bond equals the 1-D bond of the tanh front — the reported 8, which matches qtt_rank_nonlinear's 1-D Burgers peak of 8. speed=1.0 with no u^2 flux means no shock forms, and L133-134 states the metric is unused. The auditor's reference form is correct: a rank-1 factorization across the xi/eta cut is a definitional property of a separable field (Oseledets 2011), so Part 2 cannot bear on curvature, re-pinning, or Rankine-Hugoniot interface treatment. Downgraded from critical to major because the code docstring (L123-125) openly states the field stays a function of eta; the overclaim lives in the conclusion text (L104-108) and studies/README.md:42, not in a concealed defect.

> Evidence re-read: qtt_repin_marcher/main.rs:123-168 (march_radial_fitted in full), L80-84 (gate RP-D), L97-108 (printed conclusion), output.txt:6-7 (FITTED+tracked 8/8); studies/README.md:42

---

### 2.2 [MAJOR] qtt_rank_plume fork-cost table never times anything derived from the fork; the 1.00-1.04x number measures throttle workload, not forking

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_rank_plume/main.rs:178`
- **Auditor confidence:** confirmed

**Claim.** Both the 'trunk' and 'branch' timings are forked continuations (continue_with forks internally via Arc::clone), so the fork appears identically in both arms and cancels: the 1.00-1.04x ratio measures throttle workload, not fork cost, and the README's 'unforked trunk' is factually wrong.

**Code evidence.**

```
L168: `let fork = pause.fork();`  L170: `let shares = fork.shares_fluid_with(&pause) && fork.shares_field_with(&pause);`  — last use of `fork`.
L178-181: `let _trunk_cont = pause.continue_with(&trunk_world, CONT)...; let t_trunk = t0.elapsed().as_secs_f64();`
L207-211: `let report = pause.continue_with(world, CONT)...; let dt_branch = t0.elapsed().as_secs_f64();`
L231: `let ratio = dt_branch / t_trunk.max(1e-12);`
README.md:47: "Continuation cost against an unforked trunk is a ratio of **1.00 to 1.04** for every powered branch".
studies/README.md:64: "every powered continuation costs 1.00-1.04x an unforked trunk".
output.txt:28 shows the C_T = 1.00 branch — the configuration identical to the trunk — measuring 1.01, establishing the single-sample timing noise floor at ~1%.
```

**Reference form.** A fork-overhead measurement requires timing a continuation launched from the forked handle against a continuation launched from a handle that was never forked, with everything else held equal. As written, the two arms of the comparison are the same call on the same object.

**Impact.** Roadmap M1 risk 3 is recorded green on the strength of this number. The 1.00-1.04 spread is barely above the 1% noise floor the C_T=1.00 row itself exposes, and it is a single sample per row with no warmup. The genuine fork cost that IS measured — 42 ns of setup plus structural by-reference sharing — is a much weaker claim than 'continuation cost indistinguishable from an unforked march', because copy-on-write costs are paid on first write during the continuation, which this design cannot observe.

**Recommended fix.** Time `fork.continue_with(world, CONT)` against `pause.continue_with(world, CONT)` for the same world, repeat with warmup and report a median over several samples, and restate the finding as fork overhead. Separately report the throttle-workload ratio under its own name, since the coast case at 0.67 is a real and interesting workload result.

**Adversarial check.** The line citations are exact and the conclusion is right, but the stated mechanism is wrong in a way that matters. `pause.fork()` at L168 is indeed used only for the L170 structural assertion, yet `continue_with` is NOT an unforked call: carrier.rs:591-646 shows it takes a ForkSample and Arc::clones state and field, and its own comment says it 'Mirror[s] fork().alternate_context(world).continue_march(steps) exactly'. So both the trunk arm (L178) and every branch arm (L208) are forked continuations. The consequence the auditor draws still holds and is stronger for it: the fork is present identically in both arms, so it cancels in the ratio, and the number measures branch-world workload (C_T=1.0 trunk config vs C_T = 0/0.5/1.0/1.5/4.0 branch configs), never fork overhead. Copy-on-write first-write cost is paid in both arms and is therefore unobservable. README.md:47 and studies/README.md:64 both say 'unforked trunk', which is false. The C_T=1.00 row at 1.01 does expose a ~1% single-sample noise floor. Severity reduced to major: the O(1) sharing claim itself is genuinely gated (shares_fluid_with is a hard failure at L173, and ForkEconomics records Arc::ptr_eq rather than asserting it), so M1 risk 3 is not resting solely on the mislabeled ratio.

> Evidence re-read: qtt_rank_plume/main.rs:166-181, 206-237; src/types/flow/carrier.rs:506-533 (fork_sample/fork), 591-646 (continue_with / continue_with_sampled, Arc::clone + ForkEconomics); qtt_rank_plume/output.txt:22-30; README.md:46-50

---

### 2.3 [MAJOR] qtt_rank_dynamic gate G2 is unfalsifiable: peak is initialised to init and only ever increased, so `b_peak < b0` cannot be true

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_rank_dynamic/main.rs:108`
- **Auditor confidence:** confirmed

**Claim.** march_ranks initialises `peak = init` and updates it only via `peak.max(b)`, so peak >= init holds identically. Gate G2 tests `b_peak < b0`, which is therefore false for every possible input. The gate cannot fail and tests nothing.

**Code evidence.**

```
L163-165: `let init = state.max_bond();  let mut peak = init;  let mut samples = vec![(0usize, init)];`
L169: `peak = peak.max(b);`
L108-112: `if b_peak < b0 { failures.push(format!("G2: steep low-nu peak fell below its own start (init={b0}, peak={b_peak})")); }`
L106-107 (the stated intent): "Gate G2: a near-grid-scale steep feature does settle ABOVE its smooth-encode floor (some dynamic activity is real), i.e. it is not trivially constant."
```

**Reference form.** A gate must have at least one reachable input under which it reports failure. The stated intent — 'some dynamic activity is real, it is not trivially constant' — is testable as `b_peak == b0` (rank never moved) or against a separately measured static reference, neither of which is what the code checks.

**Impact.** The studies README presents the gates as encoding the findings and exiting nonzero on regression. G2 is one of three gates in this study and provides zero regression coverage while appearing in the pass list. Compare the sibling study qtt_rank_nonlinear:137, which uses `p2 <= init_2d` for the analogous check and is therefore (barely) falsifiable — the inconsistency suggests G2's `<` is a typo for `<=`.

**Recommended fix.** Change to `b_peak <= b0` at minimum. Better, compute the static rank of the advected profile at a representative later time and gate that the marched peak lands near it, which is what the README's 'settles at the static rank' claim actually asserts.

**Adversarial check.** Airtight. march_ranks L163-165 sets `let init = state.max_bond(); let mut peak = init;` and L169 is `peak = peak.max(b)`, so peak >= init identically for every input. main.rs:108 tests `b_peak < b0` where b0 is that same init, which is unreachable. No input exists that makes G2 fire; it contributes zero regression coverage while appearing in the pass list. The sibling comparison is also accurate: qtt_rank_nonlinear/main.rs:137 uses `p2 <= init_2d`, which is (barely) reachable. Severity reduced to major rather than critical: the study's conclusions rest on G1 (runaway) and G3 (diffusion does not raise rank), neither of which is affected, and the dead gate does not falsify any reported number.

> Evidence re-read: qtt_rank_dynamic/main.rs:106-112 (G2), 157-175 (march_ranks, peak init and update); qtt_rank_nonlinear/main.rs:135-141 (the falsifiable sibling)

---

### 2.4 [MAJOR] The 'body-fitted' and 'shock-aligned' references in qtt_rank_study and qtt_rank_3d are synthetic one-index profiles, not the curved shock resampled in a fitted chart; one is bit-identical to the flat control it is gated against

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_rank_study/main.rs:100`
- **Auditor confidence:** confirmed

**Claim.** obl_aligned is character-identical to flat, so the 'shock-aligned oblique' row carries no new information and G3's second disjunct is unfalsifiable. bow_polar and shell_fitted are, by contrast, the exact analytic polar/spherical resamples of a circular/spherical front and are legitimate (if idealized) measurements; the 151 -> 5 and 3-D fitted numbers stand.

**Code evidence.**

```
qtt_rank_study/main.rs:81 (flat): `let flat = build_2d(side, &|x, _| 0.5 * (1.0 + ((x - 0.5) / d2).tanh()));`
qtt_rank_study/main.rs:100 (obl_aligned): `let obl_aligned = build_2d(side, &|s, _t| 0.5 * (1.0 + ((s - 0.5) / d2).tanh()));`  — identical body, renamed parameters.
qtt_rank_study/main.rs:97-99 (bow_polar): `build_2d(side, &|r_axis, _theta| 0.5 * (1.0 + ((r_axis * 0.7 - 0.35) / d2).tanh()))` — ignores the second argument.
qtt_rank_study/main.rs:113: `if r_polar > 2 * r_flat || r_obl_al > 2 * r_flat` — with r_obl_al == r_flat == 5 by construction, the second disjunct is `5 > 10`.
qtt_rank_3d/main.rs:164-165: `let flat = build_3d(side, &|x, _, _| smooth_step(x, 0.5, d)); let shell_fitted = build_3d(side, &|r_axis, _, _| smooth_step(r_axis * 0.7, 0.3, d));` — the 'curved shell' is a plane.
qtt_rank_3d/output.txt:18 labels it: "curved shell, body-fitted (fn of r) ...... 6   (alignment fix in 3-D)".
```

**Reference form.** A body-fitted-coordinate rank claim requires sampling the SAME physical field on a transformed lattice, so that the metric distortion, grid stretching and imperfect front alignment of a real chart are all present in the measurement. qtt_blend_metric/main.rs:117-127 does exactly this (it computes a genuine blended position map and resamples one fixed physical shock on it) and is the correct pattern; qtt_rank_study and qtt_rank_3d do not follow it.

**Impact.** The headline of qtt_rank_study — 'the fix collapses it, chi 151 -> 5, roughly a 290x win' — and the Tier-B-deciding conclusion of qtt_rank_3d — 'a body-fitted coordinate is mandatory for 3-D tractability' — both rest on comparing a curved-shock measurement against a separately constructed flat profile. The 'fitted' number restates the already-reported flat number and carries no additional information. G3 in qtt_rank_study and the r_fit gate at qtt_rank_3d:277 provide no regression coverage.

**Recommended fix.** Build an explicit polar map (as qtt_blend_metric does), sample the identical bow-shock indicator function on it, and report that bond. Expect it to exceed 5 because of metric stretching; the difference is the real result. Until then, relabel these rows as 'a one-dimensional profile, for reference' and remove the 'was 151' / 'alignment fix' framing.

**Adversarial check.** Half the finding is exactly right and half is a misread of what a polar resample is. CONFIRMED: qtt_rank_study L81 `flat = build_2d(side, &|x, _| 0.5*(1.0 + ((x-0.5)/d2).tanh()))` and L100 `obl_aligned = build_2d(side, &|s, _t| 0.5*(1.0 + ((s-0.5)/d2).tanh()))` are the same function body, so r_obl_al == r_flat == 5 by construction (output.txt:8 and :12 both read 5), the 'shock-aligned, oblique 5 (was 394)' row restates the flat number, and G3's second disjunct `r_obl_al > 2*r_flat` reduces to `5 > 10` and can never fire. REFUTED: bow_polar (L97-98) is not 'constructed from scratch'. The bow field is 0.5*(1+((r-0.35)/d2).tanh()) with r = |(x,y)-(0.2,0.5)|; its exact polar resample about that center IS a function of the radial index alone, which is precisely what L97-98 encodes (with r = 0.7*r_axis covering the domain). The measured 5 agrees with qtt_blend_metric's genuine lambda=1 resample of a physical shock, which also gives 5 — so the 151 -> 5 headline is not undermined. Likewise qtt_rank_3d's shell_fitted (L165, `smooth_step(r_axis*0.7, 0.3, d)`) is the idealized spherical resample and is NOT bit-identical to flat (different offset and radial scaling), so the r_fit gate at L277 (`r_fit > 2*r_flat`, 6 vs 10) is reachable and not tautological. The legitimate residual for the fitted references is that they are perfectly-centered idealizations with no chart imperfection or grid stretching — a caveat, not a fabricated measurement.

> Evidence re-read: qtt_rank_study/main.rs:81, 85-88, 97-104, 112-117; output.txt:8-12 (flat 5, oblique 394, bow 151, polar 5, obl_aligned 5); qtt_rank_3d/main.rs:163-167, 269-286; qtt_rank_3d/output.txt:17-18; qtt_blend_metric/output.txt:11 (lambda=1 -> 5)

---

### 2.5 [MAJOR] traj_fs1 G3's 'independent element propagation' shares the element extraction and the perifocal closed form; only a Kepler-equation round trip differs, so it cannot detect an error in either

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/traj_fs1_generator/main.rs:292`
- **Auditor confidence:** confirmed

**Claim.** The candidate computes x_pf = a*cos(E) - a*e, y_pf = b*sin(E) with E = ea0 + s. The reference calls elements_from_state(r0,v0,mu) with the identical arguments used at line 238 (so el is bit-identical), then computes m = m0 + n*dt where dt was itself derived as (E - e sin E - m0)/n, so m = E - e sin E exactly, Newton-inverts it back to E, and evaluates the SAME expression a*(cos E - e), b*sin E. The two branches differ only by a forward-then-inverse pass through Kepler's equation. The reported 2.3e-15*a is that round trip's round-off.

**Code evidence.**

```
Candidate, L287-288: `let (x_pf, y_pf) = (q1 - el.a * el.e, q2);` where q1,q2 come from `omega_flow_closed(s)` applied to psi0 = (a cos E0, b sin E0, ...).
Reference, L101-111: `fn propagate_kepler(...) { let el = elements_from_state(r0, v0, mu); ... let m = m0 + el.n * dt; let ea = solve_kepler(m, el.e); let x_pf = el.a * (ea.cos() - el.e); let y_pf = b * ea.sin(); ... }`
Matched time, L290-292: `let ea = el.ea0 + s; let dt = (ea - el.e * ea.sin() - m0) / el.n; let pos_ref = propagate_kepler(r0, v0, mu, dt);`
Nothing in either branch integrates rdotdot = -mu*r/|r|^3, and the loop never compares against r0 or v0 themselves.
```

**Reference form.** An independent check of a Kepler propagator is either (a) numerical integration of the inverse-square ODE rdotdot = -mu r/r^3 from (r0,v0) to the same epoch, or (b) an external ephemeris. Comparing two evaluations of the same perifocal parametrisation with the same extracted elements is an internal-consistency check of the Kepler-equation inversion only.

**Impact.** A systematic error in elements_from_state — a wrong factor in the eccentricity vector, a wrong semi-minor axis, a wrong argument of periapsis — cancels identically on both sides and G3 still reports 2.3e-15*a. The test also never verifies that the reconstructed orbit passes through r0 with velocity v0. The study README (line 51-52) does disclose 'internal consistency of two formulations rather than agreement with an external ephemeris', but understates it: the two 'formulations' share the closed form as well as mu and the initial conditions. The studies README (line 75) drops the caveat entirely and states 'reproduces a Kepler orbit to 2.3e-15*a against independent element propagation'.

**Recommended fix.** Add an RK4 or Dormand-Prince integration of rdotdot = -mu r/r^3 from (r0,v0) as the reference (traj_fs2 already contains a suitable rk4_reference), gate G3 against that at a tolerance consistent with the integrator, and add an assertion that the reconstructed position and velocity at s=0 equal r0 and v0. Correct the studies README wording.

**Adversarial check.** Re-derived and confirmed. psi = omega_flow_closed(s)*psi0 with psi0 = (a cos E0, b sin E0, -a sin E0, b cos E0) gives psi[0] = a cos(E0+s), psi[1] = b sin(E0+s); L287 then forms x_pf = a cos(E0+s) - a e, y_pf = b sin(E0+s). The reference at L292 is called with dt = (E - e sin E - m0)/n (L290-291), so propagate_kepler computes m = m0 + n*dt = E - e sin E exactly, Newton-inverts it back to E (L106), and evaluates x_pf = a(cos E - e), y_pf = b sin E (L108-109) — the identical expression, rotated by the identical omega_peri. Both branches call elements_from_state(r0, v0, mu) on the same arguments, so el is bit-identical; a wrong eccentricity vector, semi-minor axis, or argument of periapsis cancels on both sides. Nothing integrates rddot = -mu r/r^3 and the loop never compares against r0/v0. The auditor's reference form (numerical integration of the inverse-square ODE, or an external ephemeris) is correct. The study README:52-53 does disclose 'internal consistency of two formulations', but studies/README.md:75 drops the caveat and says 'against independent element propagation'.

> Evidence re-read: traj_fs1_generator/main.rs:68-112 (elements_from_state, propagate_kepler), 272-295 (G3 loop); README.md:39 and 50-53; studies/README.md:75; output.txt (2.251e-15*a)

---

### 2.6 [MAJOR] traj_fs1 G4 'constant generator, semigroup law' never touches the generator or the matrix exponential; it verifies the cosine addition formula in f64

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/traj_fs1_generator/main.rs:266`
- **Auditor confidence:** confirmed

**Claim.** Both sides of the G4 comparison are evaluations of omega_flow_closed, a hardcoded cos/sin block matrix. The generator omega and the mat_exp routine appear nowhere in the computation. The identity being checked is cos(s1+s2) = cos s1 cos s2 - sin s1 sin s2, which holds for any rotation family by construction and cannot fail for any reason connected to Omega, Kepler, or s-independence.

**Code evidence.**

```
L266-267: `let compose = mat_mul(&omega_flow_closed(s1), &omega_flow_closed(s2)); let g4_err = mat_max_abs_diff(&compose, &omega_flow_closed(s1 + s2));`
L200-212 defines omega_flow_closed as a literal cos/sin block, with no dependence on generator_omega().
L311-314 (the gate label): `gate("constant generator (semigroup law to round-off)", g4_err < 1e-12)`.
README.md:40-41: "**One constant generator drives the whole flow.** The semigroup law holds to round-off, so `G` is genuinely `s`-independent."
```

**Reference form.** To test that a single constant generator drives the flow, compose the numerically computed exponentials: mat_exp(Omega*s1) * mat_exp(Omega*s2) compared against mat_exp(Omega*(s1+s2)). That composition exercises the generator and the exponential routine and can fail if either is s-dependent or wrong.

**Impact.** One of the four gates that produce the study's PASS verdict, and the one the README cites for the s-independence claim, has no connection to the object under test. Reported at 1.110e-16, which is simply f64 round-off in a trig identity. Related: G1 and G2 do validate mat_exp, but the trajectory in G3 is built with omega_flow_closed (L284), not mat_exp, so the generic exponential never enters the physics path either.

**Recommended fix.** Rewrite G4 as `mat_mul(&mat_exp(&mat_scale(&omega, s1)), &mat_exp(&mat_scale(&omega, s2)))` versus `mat_exp(&mat_scale(&omega, s1 + s2))`. Consider also driving G3's trajectory from mat_exp so the 'the matrix exponential is literal' claim covers the trajectory result, not just a spot check at s = 1.234.

**Adversarial check.** L266-267 is verbatim as quoted: both operands and the target are omega_flow_closed, defined at L200-212 as a literal cos/sin block with no reference to generator_omega() or mat_exp(). The identity reduces to cos(s1+s2) = cos s1 cos s2 - sin s1 sin s2 and sin(s1+s2) = ..., which hold to f64 round-off for every s1, s2; no input can push g4_err above 1e-12, so the gate at L311-314 cannot fail. The reported 1.110e-16 is trig round-off. The auditor's reference form is correct: composing mat_exp(Omega*s1)*mat_exp(Omega*s2) against mat_exp(Omega*(s1+s2)) would exercise the object under test. The related observation is also correct — G3's trajectory is built from omega_flow_closed at L284, not mat_exp, so the generic exponential enters only through G2's single-point check at s = 1.234. README.md:40-41 cites G4 for the s-independence claim.

> Evidence re-read: traj_fs1_generator/main.rs:189-212 (generator_omega, omega_flow_closed), 249-268 (G1/G2/G4), 284 (G3 uses omega_flow_closed), 311-314 (gate label); README.md (semigroup bullet); output.txt (1.110e-16)

---

### 2.7 [MAJOR] qtt_rank_plume's 'mirrored, bit-identical, deterministic' continuation is marched without the plume the carrier trunk carries

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_rank_plume/main.rs:263`
- **Auditor confidence:** confirmed

**Claim.** The mirror's pre-pause state is produced by mirror_march(None, K_PAUSE, seed) — plume argument None. The carrier trunk it claims to mirror is branch_world("trunk", CT_TRUNK = 1.0), which does install a forcing region because ct > 0. The two paths therefore differ in whether a plume was imprinted during the 20 pause steps, so the post-fork bond numbers are read off a different trajectory from the one that was forked.

**Code evidence.**

```
L263: `let trunk_state = mirror_march(None, K_PAUSE, &mirror_seed()).unwrap_or_else(|e| fail("mirror trunk", e));`
L421-444 (branch_world): `... .publish_constant("commanded_throttle", ct); if ct > 0.0 { let (_, geometry, jet) = throttle_point(ct)?; builder = builder.forcing_region(plume_region(&geometry, jet, 0.55)?); }` — with CT_TRUNK = 1.0 (L87), the trunk world has a region.
L258 (the claim): "// (c) Post-fork bond growth, on the mirrored (bit-identical, deterministic) continuation."
L26-28 (module docstring): "the mirror re-marches the identical deterministic path on the bare marcher — same seed, same steps, same round policy — and reads the bond there".
Additionally, mirror_march applies sponge and body every step (L526-528) while the CompressibleMarchConfig branch worlds are built with only a forcing_region, a second structural divergence.
```

**Reference form.** A mirror intended to recover a hidden quantity from a run must reproduce that run's operator sequence exactly. Here that means marching the trunk's plume region for the K_PAUSE pre-pause steps and matching the carrier's sponge/body treatment.

**Impact.** The reported 'mirrored post-fork bond 16, flat across the roster' is presented as the post-fork rank of the forked carrier worlds and is one of the three quantities recorded green for M1 risk 2. It is instead the bond of a plume-free 20-step march continued under each throttle. The word 'bit-identical' in the code comment is not achievable given the difference and should not appear.

**Recommended fix.** Pass the trunk's own plume region into mirror_march for the pre-pause segment, and verify the mirror against the carrier by comparing a cheap observable (e.g. the dequantised field L2) at the pause point before trusting the bond read. If exact mirroring is not achievable, drop 'bit-identical' and state what the mirror approximates.

**Adversarial check.** L263 is verbatim `mirror_march(None, K_PAUSE, &mirror_seed())`, and the carrier trunk it claims to mirror is branch_world("trunk", CT_TRUNK) with CT_TRUNK = 1.0 (L87), which takes the `if ct > 0.0` branch at L440-443 and installs plume_region(..., 0.55). So the mirror's 20 pre-pause steps carry no plume where the carrier's do. The second divergence is real and worse than claimed: mirror_march applies sponge (L525-527) and body (L528) every step, while the carrier config carries only a single optional forcing region — compressible_march_run.rs has one `forcing: Option<ForcingRegion<R>>` field and no sponge or body at all, and branch_world builds nothing else. So the mirror runs a strictly different operator sequence in both directions. 'bit-identical' (L258) and 'identical deterministic path' (L26-28) are not achievable. The reported flat-16 post-fork bond is the bond of a plume-free, sponge-and-body-damped 20-step march continued under each throttle.

> Evidence re-read: qtt_rank_plume/main.rs:26-28, 258-280, 421-445 (branch_world), 479-537 (mirror_march); src/types/flow/compressible_march_run.rs:74-76, 239-264, 399-401 (single optional forcing region, no sponge/body)

---

### 2.8 [MAJOR] srp_momentum_jet's 'compression is innocent' claim is absent from the code, the gates, and the committed output

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/srp_momentum_jet/README.md:57`
- **Auditor confidence:** confirmed

**Claim.** No code path runs both bond cap 24 and bond cap 32 and compares observables. cap is a single env-overridable value used for one run; output.txt records only the default cap-24 sweep. The cap-24 run has peak bond 24 in every row, i.e. the cap is binding everywhere, so truncation is active in the recorded artifact and the claim that removing it changes nothing is untested in-repo.

**Code evidence.**

```
main.rs:103: `cap: env_usize("SRP_MJ_CAP", 24),` — one cap per process.
main.rs:375: `let trunc = Truncation::<f64>::by_bond(cap)` — a pure bond cap, both tolerances zero (see truncation/mod.rs:81-83).
output.txt:17-35: every sweep row ends `|  24`, and line 4 records `bond cap 24`. There is no cap-32 row anywhere in the committed output.
README.md:57-58: "**Compression is innocent.** Raising the bond cap 24 to 32, exact at 2^5, leaves every observable unchanged at displayed precision. The discretization is the limit, not the tensor-train truncation."
studies/README.md:65 repeats it: "Raising the bond cap 24 -> 32, exact at 2^5, leaves every observable unchanged at displayed precision, so **compression is innocent**".
```

**Reference form.** A claim that truncation does not affect the result requires the two runs to be executed and their observables compared, with the comparison committed as an artifact or enforced by a gate. The supporting sub-claim — that cap 32 is exact at L=5 because no bond can exceed 2^5 = 32 on a 10-core 2-D QTT — is correct, which makes the comparison a genuinely decisive one and worth actually running.

**Impact.** This is the sentence that discharges the tensor-train-truncation hypothesis and lets the study attribute the missing Jarvinen-Adams collapse to the discretisation. It is the load-bearing step in the risk-1 attribution, and it is unreproducible from the repository. Since the recorded run truncates at every point of the sweep, the possibility that truncation contributes to the monotone augmentation is not excluded by anything committed.

**Recommended fix.** Run both caps inside the binary, print both columns, and gate that the annulus fractions agree to the displayed precision. If runtime forbids it by default, commit a second output file (output_cap32.txt) and cite it from the README claim.

**Adversarial check.** cap is a single per-process value (main.rs:103, `env_usize("SRP_MJ_CAP", 24)`) threaded into one Truncation::by_bond(cap) at L350 and L375; there is no loop, no second run, and no comparison of observables across caps anywhere in the file. output.txt:4 records 'bond cap 24' and every sweep row (17-35) plus the baseline (11) ends in peak bond 24, so the cap is binding at every point of the recorded artifact — by_bond sets both tolerances to zero (truncation/mod.rs:81-83 confirmed), so reaching the cap means singular values were discarded. README.md:57-58 and studies/README.md:65 both assert the cap-24 -> 32 comparison as a finding. The supporting sub-claim is correct (32 = 2^5 is the exact central-cut bound for a 2^5 x 2^5 QTT), which makes the untaken comparison a decisive one. The truncation hypothesis is therefore not excluded by anything committed, and this is the step that lets the study attribute the missing Jarvinen-Adams collapse to the discretization.

> Evidence re-read: srp_momentum_jet/main.rs:95-115, 350, 371-378; output.txt:4, 11, 17-35; README.md (Compression is innocent bullet); studies/README.md:65; deep_causality_tensor/.../truncation/mod.rs:81-83

---

### 2.9 [MINOR] qtt_rank_3d's storage table reports QTT parameter counts larger than the dense tensor itself, which is impossible for the actual tensor train

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/studies/qtt_rank_3d/main.rs:199`
- **Auditor confidence:** confirmed

**Claim.** The proxy params(m, chi) = 2*m*chi^2 assumes every one of the 3L binary cores carries the maximum bond simultaneously, ignoring that boundary bonds are capped by min(2^k, 2^(3L-k)). At 16^3 it reports 48,600 parameters for a tensor that has only 4,096 entries, and at 32^3 it reports 94,080 for 32,768 entries. A TT-SVD of those tensors cannot exceed the dense size. The derived 'dense/QTT' ratios of 0.08x and 0.35x are artifacts of the formula, not measurements.

**Code evidence.**

```
L199: `let params = |m: usize, chi: usize| (2 * m * chi * chi) as f64;`
L208: `let pp = params(3 * ll, chi);`
output.txt:30-31: `   16 |   45 |   48600 |  0.08x` and `   32 |   56 |   94080 |  0.35x` against dense sizes 16^3 = 4096 and 32^3 = 32768.
README.md:36-39 reports these as findings: "The `dense/QTT` storage ratio runs 0.08x at 16^3, then 0.35x, then 0.92x at the 64^3 break-even, then 2.74x at 128^3."
The object whose bond is measured is available at L47-49 (`CausalTensorTrain::from_dense(&q, &trunc)`), so the true core shapes could be summed directly.
```

**Reference form.** The parameter count of a tensor train with cores G_k of shape r_{k-1} x n_k x r_k is sum_k r_{k-1} * n_k * r_k, with r_k <= min(prod_{i<=k} n_i, prod_{i>k} n_i). The uniform-max-bond form 2*(3L)*chi^2 is an upper bound only, and a loose one whenever chi approaches the boundary limit.

**Impact.** Two of the four rows of the storage table are not just imprecise but numerically impossible, and the README uses the shape of that table to argue 'the 64^3 break-even is a small-grid artifact'. It is an artifact — of the proxy, not of the physics — so the argument is circular. The same uniform-bond proxy appears at qtt_rank_study/main.rs:121 (`2*2*l2*chi*chi`), where it produces the '3.1x LARGER' and '21.3x LARGER' verdicts; those are also upper bounds presented as measurements.

**Recommended fix.** Sum the actual core shapes of the CausalTensorTrain returned at line 47 and report that. Keep the analytic 6*L*chi^2 as a separately labelled asymptotic estimate if desired, and restate the storage findings from the measured counts.

**Adversarial check.** L199 is verbatim `let params = |m: usize, chi: usize| (2 * m * chi * chi) as f64;` and L208 calls it with 3*ll cores, giving 6L*chi^2. output.txt:30-31 record 48600 params for a 16^3 tensor (4096 entries) and 94080 for 32^3 (32768 entries) — both strictly impossible for any TT of those tensors, since a TT-SVD's parameter count is bounded by the dense size. The auditor's reference form is correct: sum_k r_{k-1}*n_k*r_k with r_k <= min(prod_{i<=k} n_i, prod_{i>k} n_i), and the uniform-max-bond form is a loose upper bound whenever chi approaches the boundary limit (at 16^3, chi=45 against a 12-core binary train whose outer bonds cap at 2, 4, 8, ...). The true core shapes are available at L47-49. Severity reduced to minor: the header at L200 and qtt_rank_study:121 both label it a proxy ('QTT ~ 6L*chi^2', '~ rough TT param count'), and the direction of every decision the study makes survives — the auditor's own finding 28 concedes the asymptotic argument holds. The residual defect is that README.md:36-39 presents 0.08x and 0.35x as measured ratios and then explains the 64^3 break-even as a small-grid artifact, when it is an artifact of the proxy.

> Evidence re-read: qtt_rank_3d/main.rs:37-50 (the real TT is in hand), 198-214; output.txt:28-33; README.md:36-39; qtt_rank_study/main.rs:119-139

---

### 2.10 [MINOR] compressible_carrier_timing issues a GO at bond cap 32 while the cap is binding on a smooth test field, with no accuracy justification anywhere

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/studies/compressible_carrier_timing/main.rs:148`
- **Auditor confidence:** confirmed

**Claim.** The GO configuration's bond cap is a timing parameter only, with the cap binding (peak bond 32 = cap 32) even on the smooth low-Mach test state and no accuracy measurement anywhere in the study; the studies README does pre-label the whole study as a wall-clock budget measurement rather than a physics probe, which blunts but does not close the gap.

**Code evidence.**

```
main.rs:167: `let d = 1.0 + 0.2 * (TAU * x).sin() * (TAU * y).sin();` with `let (u, v) = (0.1, -0.05);` at L163 — a smooth perturbation at |u| = 0.11 against c = sqrt(1.4) = 1.18, i.e. M ~ 0.1.
main.rs:145: `let tr = Truncation::<f64>::by_bond(cap)?;` — a pure bond cap.
output.txt:19: `=== GO: corridor carrier 2d at 64^2, bond cap 32 (0.175 s/step, peak bond 32). ===` — peak_bond == cap.
No accuracy, residual, or convergence quantity is computed anywhere in the file; Measurement (L136-140) carries only assembly_s, per_step_s, peak_bond.
For scale, the sibling studies measure a curved shock at bond 114 (qtt_blend_metric/output.txt:7, 256^2, lambda=0) and a marched curved Burgers shock at 25 (qtt_rank_fitted_dynamic/output.txt:4, 128^2).
```

**Reference form.** A carrier go/no-go that names a compression parameter must pair the cost measurement with an accuracy measurement at that parameter — e.g. the relative error of a target observable at cap 32 versus an uncapped or higher-cap reference on a representative shocked state.

**Impact.** An engineer reading 'GO: the corridor carrier is 2-D at 64^2, bond cap 32' in the studies README (line 92) would reasonably take cap 32 as a validated configuration. It is a timing datum only. Because the cap binds even on a smooth field, a shocked corridor field will be truncated harder, and nothing in the study bounds the resulting error. The README's caveats section covers hardware, sample size and coupling overhead but not this.

**Recommended fix.** Add a cap sweep against an uncapped (by_tol) reference on a shocked state and report the observable error at each cap alongside the per-step cost, so the GO names a configuration that is both affordable and adequate. At minimum, state explicitly in the README and the studies README that cap 32 is a cost parameter carried forward untested for accuracy.

**Adversarial check.** The facts are all confirmed. L167 is verbatim `let d = 1.0 + 0.2*(TAU*x).sin()*(TAU*y).sin();` with (u,v) = (0.1,-0.05) at L163, i.e. M ~ 0.09 against c = sqrt(1.4); L145 is a pure by_bond cap (zero tolerances, verified in truncation/mod.rs:81-83); the Measurement struct (L136-140) carries only assembly_s, per_step_s, peak_bond, and no residual, error or convergence quantity is computed anywhere in the file; output.txt:19 reads 'bond cap 32 ... peak bond 32', so the cap binds even on this benign state. The auditor's reference form is right in principle. Downgraded to minor because the framing objection is partly answered in-repo: studies/README.md:87-88 explicitly labels this study 'a wall-clock budget measurement rather than a physics probe' and 'a different workstream from the batches above'. The gap is that neither README pairs the named cap with an accuracy bound, and the caveats section covers hardware, sample size and coupling overhead but not this.

> Evidence re-read: compressible_carrier_timing/main.rs:136-197 (measure_2d in full), 34-39, 82-99; output.txt:5-19; studies/README.md:85-92; truncation/mod.rs:81-83

---

### 2.11 [MAJOR] qtt_rank_study README reports a tolerance sweep from 1e-4 to 1e-12 that the code does not perform

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_rank_study/README.md:28`
- **Auditor confidence:** confirmed

**Claim.** The study runs at a single hardcoded tolerance of 1e-8. There is no loop, array, or parameter sweep over tolerance anywhere in main.rs, so the claim that the aligned bond 'stays near 5 across tolerances 1e-4 to 1e-12' is not produced by this code and is not present in the committed output.

**Code evidence.**

```
main.rs:24: `let tol = 1e-8;` — the only tolerance in the file, threaded into rank_1d (L156) and rank_2d (L164) for every measurement.
output.txt:1 and :7 both record `tol=1e-8`; no other tolerance appears in the artifact.
README.md:27-28: "The oblique in an aligned coordinate is chi ~ 5, down from 394, and **stays near 5 across tolerances 1e-4 to 1e-12**."
```

**Reference form.** Documented numbers in a self-verifying study must be reproducible by running the committed program and reading the committed output.

**Impact.** Tolerance sensitivity is exactly the right question for a rank claim, since a rank measured under an SVD truncation threshold is a function of that threshold. Asserting insensitivity without the sweep removes the reader's ability to check the one dependency that most threatens the headline numbers 5 / 151 / 394. It also compounds the separate defect that the 'aligned' case is not a coordinate transform at all.

**Recommended fix.** Either add the sweep (a loop over [1e-4, 1e-6, 1e-8, 1e-10, 1e-12] calling rank_2d) and print it, or delete the sentence. Given that rank depends on the truncation threshold by construction, the sweep is worth adding for the curved and oblique cases too, not only the aligned one.

**Adversarial check.** main.rs:24 `let tol = 1e-8;` is the only tolerance in the file; it is threaded into rank_1d (L156-161) and rank_2d (L164-168) for every measurement. There is no loop, array, or parameter over tolerance anywhere in the 179-line file (grep-verified by reading it in full). output.txt:1 and :7 record tol=1e-8 and nothing else. README.md:27-28 nevertheless states the aligned oblique 'stays near 5 across tolerances 1e-4 to 1e-12'. That number is not produced by the committed program and not present in the committed artifact. The point is well taken that a rank measured under an SVD truncation threshold is a function of that threshold, so this is exactly the sensitivity a reader would want to check.

> Evidence re-read: qtt_rank_study/main.rs:22-24, 89-104, 155-168 (whole file read); output.txt:1,7; README.md:26-28

---

### 2.12 [MAJOR] qtt_rank_3d claims the fitted reference is constant in resolution while encoding it at a single grid

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_rank_3d/main.rs:166`
- **Auditor confidence:** confirmed

**Claim.** r_flat and r_fit are computed once, at side = 64 only. The resolution sweep at lines 180-191 covers the forming-shock case exclusively. No flat or fitted reference is ever encoded at 16^3, 32^3 or 128^3, so 'constant', 'flat in resolution' and 'at ANY resolution' are unmeasured extrapolations.

**Code evidence.**

```
L152-153: `let l = 6usize; let side = 1usize << l;` then L166-167: `let r_flat = rank_of_field(&flat, side, l, tol); let r_fit = rank_of_field(&shell_fitted, side, l, tol);` — both inside the fixed side = 64 scope.
L180-191: `for ll in 4..=7 { ... let (_, pk, _) = burgers_3d_rank(s, ll, tol); ... }` — only the forming shock is swept.
L253-254 (printed): "The body-fitted shell holds chi ~ O(10) at ANY resolution."
README.md:33-35: "while the flat and body-fitted references stay **constant at 5 to 6**"; README.md:26-31's table leaves the flat/fitted column blank at 16^3, 32^3 and 128^3, correctly reflecting that they were not measured, while the prose asserts constancy.
studies/README.md:28: "flat and body-fitted stay **chi ~ 6, constant**".
```

**Reference form.** A resolution-independence claim requires measurements at two or more resolutions. The study already has the sweep machinery in place at lines 180-191; extending it to the two references costs four extra from_dense calls.

**Impact.** The study's central comparison is chi ~ sqrt(side) (measured over four points) against chi ~ O(10) constant (measured at one point). Half of the comparison that decides 'a body-fitted coordinate is mandatory for 3-D tractability' has no resolution evidence. This compounds with the separate finding that shell_fitted is a plane rather than a resampled curved shell.

**Recommended fix.** Move the flat and fitted reference encodings inside the ll loop and print them as columns of the sweep table. Fill in the README table's blank cells with the measured values.

**Adversarial check.** L152-153 fix l = 6 / side = 64, and L163-167 build and encode flat and shell_fitted inside that scope only. The resolution sweep at L180-191 iterates `for ll in 4..=7` calling burgers_3d_rank alone — the forming shock only. No flat or fitted field is encoded at 16^3, 32^3 or 128^3. Yet L253-254 prints 'The body-fitted shell holds chi ~ O(10) at ANY resolution', output.txt:38 says the references 'stay ~6 (constant)', README.md:33-35 says they 'stay constant at 5 to 6', and studies/README.md:28 says 'flat and body-fitted stay chi ~ 6, constant'. The README table (lines 26-31) correctly leaves the flat/fitted column blank at 16^3, 32^3, 128^3, which makes the prose's constancy claim self-contradicting within the same document. The study's central comparison is a four-point growth law against a one-point 'constant', and the sweep machinery to fix it is already present.

> Evidence re-read: qtt_rank_3d/main.rs:151-167, 172-196, 244-255; output.txt:16-18, 20-26, 36-38; README.md:26-35; studies/README.md:28

---

### 2.13 [MINOR] qtt_acoustic_precond gate AC-C uses bounds set 6-9% below the observed values, making it a pinned regression test presented as a finding gate

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_acoustic_precond/main.rs:70`
- **Auditor confidence:** confirmed

**Claim.** The gate thresholds 0.82 and 1.35 sit just under the observed rho_jump = 0.872 and the observed ratio 0.872/0.590 = 1.478. Neither number is derivable from the Resolution-6 claim, from the sound-speed contrast, or from any cited source; both are consistent only with having been chosen after seeing the output. The gate encodes the recorded numbers, not the finding 'a jump degrades the bound toward 1'.

**Code evidence.**

```
L70-74: `if rho_jump < 0.82 || rho_jump < 1.35 * rho_smooth { failures.push(format!("AC-C: captured jump did not degrade the preconditioner toward 1 (jump={rho_jump:.3}, smooth={rho_smooth:.3})")); }`
output.txt:6: `AC-C  captured c-jump  : rho(A0^-1 A1) = 0.872` — 6.0% above the 0.82 threshold.
output.txt:5: `AC-B  smooth interior  : rho(A0^-1 A1) = 0.590` — the ratio 1.478 is 9.5% above the 1.35 threshold.
Contrast AC-B at L62: `if rho_smooth >= 0.8` against an observed 0.590, a 35% margin, which reads as a genuine finding gate.
Related unjustified literals in the same file: `const STIFF: f64 = 8.0;` (L31) and the jump magnitude `if x < 0.5 { 1.0 } else { 25.0 }` (L128).
```

**Reference form.** A gate encoding a finding states the finding's threshold: for AC-C the finding is 'rho degrades toward the divergence threshold 1', which is testable as rho_jump > rho_smooth by a stated factor derived from the sound-speed contrast, or simply rho_jump > 0.8 with the 0.8 justified. A gate pinned within 6% of the observed value is a regression test and should be labelled one.

**Impact.** The studies README (line 43) and the study README present AC-C as a measured finding with an honest counterpart reading. Both are legitimate framings, but the gate's tightness means any change to STIFF, the grid level l = 7, or the jump magnitude will trip it as a 'regression' even when the qualitative finding is unchanged. Separately, the analytic limit of rho as STIFF grows is max|c^2 - cbar^2|/cbar^2, which evaluates to 0.923 for the jump case and 0.617 for the smooth case — very close to the measured 0.872 and 0.590. This means both headline numbers are essentially determined by the two unjustified coefficient literals (the +/-0.3 sine amplitude and the x25 jump), neither of which is traced to a reentry sound-speed profile.

**Recommended fix.** Restate AC-C as a directional gate (rho_jump > rho_smooth, and rho_jump > 0.8 with 0.8 justified as 'materially closer to 1 than the smooth case'), and label the current numeric bounds as pinned regression values if they are kept. Separately, tie the jump magnitude to a computed post-shock sound-speed ratio for the reentry condition rather than a bare 25.0, since a Mach-20 normal shock implies a far larger c^2 contrast and would push rho much closer to 1.

**Adversarial check.** L70 is verbatim `if rho_jump < 0.82 || rho_jump < 1.35 * rho_smooth`. Observed rho_jump = 0.872 (6.3% above 0.82) and rho_jump/rho_smooth = 0.872/0.590 = 1.478 (9.5% above 1.35). AC-B at L62 uses `rho_smooth >= 0.8` against 0.590, a 35% margin, so the contrast the auditor draws is real. Neither 0.82 nor 1.35 is derivable from anything in the file, the papers folder, or any comment; the only nearby documented constants are STIFF = 8.0 (L30-31, which justifies s > 1 but not 8) and the x25 jump (L111, L127, documented as a 5x sound-speed contrast but not sourced to a reentry profile). I independently re-derived the auditor's large-stiffness limit and it checks out: max|c^2 - cbar^2|/cbar^2 gives 12/13 = 0.923 for the jump and 0.645/1.045 = 0.617 for the smooth case, close to the measured 0.872 and 0.590 — so both headline numbers are essentially set by the +/-0.3 sine amplitude and the x25 jump. Severity reduced to minor: the qualitative finding (a captured jump degrades rho toward 1) is genuinely measured and correct, and the defect is that a regression pin is presented under the studies README's blanket 'gates encode the finding' claim.

> Evidence re-read: qtt_acoustic_precond/main.rs:29-34, 55-74, 106-133 (c2_profile), 178-217 (spectral_radius); output.txt:5-6; studies/README.md:11-12, 43

---

### 2.14 [MAJOR] srp_momentum_jet config documents a 0.25 m nozzle that is grid-independent; the code realises 0.375 m at L=5 and 0.3125 m at L=6

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/srp_momentum_jet/config.rs:84`
- **Auditor confidence:** confirmed

**Claim.** The docstring states the patch is 0.25 m tall with r_jet/R_body = 0.25 and that fixing it in physical units keeps the injected momentum flux identical across grid levels so an L=6 run varies only resolution. The node-lattice predicate |j*dx - 0.5| <= 1/32 selects 3 rows at L=5 (h_eff = 3/32 -> 0.375 m) and 5 rows at L=6 (h_eff = 5/64 -> 0.3125 m), so the realised nozzle geometry differs from the documented one and changes by 17% between the two grid levels.

**Code evidence.**

```
config.rs:83-85: "Jet exit patch half-height in **cells of the L = 5 grid** ... Fixing the patch in physical units (0.25 m tall, r_jet/R_body = 0.25) keeps the injected momentum flux identical across grid levels, so an L = 6 companion run varies only the resolution, not the nozzle."
config.rs:86-88: `pub const JET_HALF_HEIGHT_L5_CELLS: f64 = 1.0;` and `pub const DX_L5: f64 = 1.0 / 32.0;` giving h = 1/32.
main.rs:129-138: `let jet_rows = (0..cfg.n).filter(|&j| ((j as f64) * cfg.dx - BODY_CY).abs() <= jet_half_height_hat()).count(); ... let h_hat_eff = jet_rows as f64 * cfg.dx;`
output.txt:5: `jet patch: 1 node-column(s) x 3 node-rows at the body face -> realized exit height 0.375 m, r_jet/R_body = 0.375` — the code's own output contradicts the docstring.
main.rs:346-347 repeats the wrong figure: "one L5-cell-wide column flush against the body face, 0.25 m tall (grid-independent physical geometry)".
```

**Reference form.** At L=6 (dx = 1/64) the rows satisfying |j/64 - 0.5| <= 1/32 are j = 30..34, i.e. 5 rows, h_eff = 5/64 = 0.078125 -> 0.3125 m. The mechanism the code uses to compensate (sizing p_e by h_hat_eff in jet_exit_state) preserves the thrust coefficient but not the nozzle geometry.

**Impact.** The L=6 run is the study's resolution-robustness companion, and its stated purpose is to halve the dissipation while holding the nozzle fixed. Because the realised exit height also changes, an L=6 comparison confounds dissipation with nozzle geometry — which matters, since the study's whole conclusion is an attribution between the dissipation floor and the model class. The code does correctly account for the fencepost when sizing the throttle (config.rs:155-158 explains it); it is the two docstrings and the grid-independence claim that are wrong.

**Recommended fix.** Correct both docstrings to 0.375 m / 0.375 at L=5. Either snap the predicate to a physical half-height that yields the same realised rows at both levels, or state explicitly that the L=6 companion varies nozzle height as well as resolution and bound the effect.

**Adversarial check.** config.rs:82-85 states the patch is '0.25 m tall, r_jet/R_body = 0.25' and that fixing it in physical units 'keeps the injected momentum flux identical across grid levels, so an L = 6 companion run varies only the resolution, not the nozzle'. jet_half_height_hat() = 1.0 * (1/32) (L86-88, L128-130). main.rs:129-138 counts node rows with |j*dx - BODY_CY| <= 1/32: at L=5 (dx=1/32) that is j in 15..=17, 3 rows, h_eff = 3/32 -> 0.375 m; at L=6 (dx=1/64) it is j in 30..=34, 5 rows, h_eff = 5/64 -> 0.3125 m. I re-derived both. output.txt:5 prints 'realized exit height 0.375 m, r_jet/R_body = 0.375', contradicting the docstring in the study's own artifact, and main.rs:345-347 repeats the wrong '0.25 m tall (grid-independent physical geometry)'. The auditor's nuance is right: config.rs:151-166 does size p_e by h_hat_eff, so the injected momentum flux (and hence C_T) IS held across levels — it is the geometry claim and the 'varies only the resolution, not the nozzle' inference that are wrong, and an L=6 comparison therefore confounds halved dissipation with a 17% smaller nozzle.

> Evidence re-read: srp_momentum_jet/config.rs:82-88, 127-130, 151-171; main.rs:126-143, 345-366; output.txt:5

---

### 2.15 [MINOR] qtt_rank_plume README states the A2 Cartesian bond 'pins at the ceiling'; A2 runs on a 64x64 grid with no bond cap, where the ceiling is 64

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_rank_plume/README.md:42`
- **Auditor confidence:** confirmed

**Claim.** proxy_bond builds a 2^6 x 2^6 field, whose QTT bond ceiling is 2^6 = 64, and encodes it with Truncation::by_tol, which sets max_bond = usize::MAX. The reported 32 is therefore neither a ceiling nor a cap for that grid. The RANK_CEILING = 32 constant printed in the output header applies to the L=5 A1 grid only and is used solely in A1's gate comparison.

**Code evidence.**

```
main.rs:579-582 (proxy_bond): `let l = 6usize; let side = 1usize << l; ... let trunc = Truncation::<f64>::by_tol(TOL)`.
truncation/mod.rs:89-91: `pub fn by_tol(rel_tol: T) -> Result<Self, CausalTensorError> { Self::new(usize::MAX, rel_tol, T::zero()) }` — explicitly 'no bond cap'.
main.rs:81-82: `/// The rank ceiling of a 2^5 x 2^5 quantized field: bond saturation = no compression.  const RANK_CEILING: usize = 32;` — scoped to the L=5 field.
main.rs:118 uses it only against A1: `if a1_max >= RANK_CEILING`.
README.md:42: "The Cartesian capture pins at the ceiling."  README.md:60-61 makes the complementary error about A1: "A1's bond 16 is read under a rank ceiling of 32, so it measures what the tolerance policy chose, not an unconstrained rank" — but A1 also uses by_tol (main.rs:487), so no ceiling is applied there either.
```

**Reference form.** For a d-level binary QTT of a 2^l x 2^l field with block bit ordering, the maximum attainable bond at the central cut is 2^l. At l = 6 that is 64.

**Impact.** Two adjacent README sentences misdescribe how both A2 and A1 numbers were produced. 'Pins at the ceiling' invites the reader to treat 32 as a saturation floor on the true rank (i.e. the real rank is unknown and >= 32); in fact 32 is an unconstrained measurement and the fitted-versus-Cartesian ratio of roughly 3x is the honest reading. The identical value 32 across all four C_T rows deserves comment in its own right but is not explained by saturation.

**Recommended fix.** Correct both sentences: state that A1 and A2 both use an uncapped relative-tolerance policy at 1e-6, that A2's grid is 64x64 with a theoretical ceiling of 64, and that RANK_CEILING = 32 is a gate threshold for A1's 32x32 grid only. Explain or investigate why all four A2 lambda=0 rows land on exactly 32.

**Adversarial check.** proxy_bond L579-582 sets l = 6, side = 64 and uses Truncation::by_tol(TOL); by_tol is confirmed at truncation/mod.rs:89-91 to set max_bond = usize::MAX ('no bond cap'). For block bit-ordering on a 2^6 x 2^6 field the central-cut maximum is 2^6 = 64, so the reported 32 is neither a cap nor a ceiling — it is an unconstrained measurement. RANK_CEILING = 32 is scoped in its own docstring to 'a 2^5 x 2^5 quantized field' (L81-82) and is used only against the L=5 A1 peak (L118) and the L=5 mirror branches (L275). The complementary README error is also confirmed: README.md:60-61 says A1's bond 16 'is read under a rank ceiling of 32', but A1 goes through mirror_march, whose truncation is by_tol at L487, so no cap applies there either. Severity reduced to minor: both are README wording errors about provenance; the underlying numbers (32 vs 10-12, roughly 3x) are honest measurements and the conclusion drawn from them is unaffected.

> Evidence re-read: qtt_rank_plume/main.rs:81-82, 118, 275, 487, 575-611; README.md:42, 60-61; deep_causality_tensor/.../truncation/mod.rs:81-91

---

### 2.16 [MAJOR] The studies README carries qtt_rank_fitted_dynamic's superseded conclusion alongside the result that overturned it

- **Verification verdict:** REFUTED — not a defect
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/README.md:41`
- **Auditor confidence:** confirmed

**Claim.** Line 41 states that Res-5 feedback re-pinning is 'necessary, not optional' on the strength of qtt_rank_fitted_dynamic. Line 42 then reports that qtt_repin_marcher measured re-pinning and found it does not curb the growth. Both are presented as standing findings in the same table with no note that the first was superseded; a reader taking the table at face value gets contradictory guidance on the Res-5 mechanism.

**Code evidence.**

```
studies/README.md:41: "Res-5 **feedback re-pinning (D9)** is therefore necessary, not optional. Alignment is the lever; maintaining it is the mechanism."
studies/README.md:42: "re-pinning the coordinate to the live front, 18 re-pins at 128^2, does **not** curb it: the driver is the angular structure a flux-through-front march injects, not the front's drift."
qtt_repin_marcher/main.rs:73-77 encodes the reversal as a gate that fires if re-pinning helps: `if grow_repin < grow_static { failures.push(format!("RP-C: re-pin-alone unexpectedly curbed the Cartesian-flux growth ...")); }`
output.txt:4-5 shows the static and re-pinned peaks are identical at both resolutions (25/35 and 25/35, both +10).
The individual study README does reconcile them (qtt_rank_fitted_dynamic/README.md:38-39: "`qtt_repin_marcher` carries the finding further and shows that re-pinning alone is still not sufficient"), so only the studies-level summary is inconsistent.
```

**Reference form.** A summary table of findings should mark superseded conclusions as superseded, in the same way srp_momentum_jet is marked as superseding the reverted verification harness at studies/README.md:58-60.

**Impact.** The Round-2 result paragraph (lines 46-52) does eventually give the corrected reading, but the table rows a reader scans first assert the opposite of each other. For a spec-facing document that feeds Resolution-5 design decisions, this is the kind of ambiguity that propagates.

**Recommended fix.** Annotate the qtt_rank_fitted_dynamic row to say the 'necessary, not optional' inference was tested and partly overturned by qtt_repin_marcher, and let the repin row carry the current conclusion. Note separately that gate RP-C will fail if a future change makes re-pinning effective, which is an unusual property for a regression gate and should be documented as intentional.

**Adversarial check.** The two rows are not contradictory. Line 41 asserts that re-pinning is 'necessary, not optional' — a necessity claim. Line 42 asserts that re-pinning alone 'does not curb' the growth and that the working lever is 're-pin AND treat the front as an exact Rankine-Hugoniot interface' — an insufficiency claim that explicitly retains re-pinning. Necessary-but-not-sufficient is the consistent reading, and it is the reading both rows state. Line 42 does not report that re-pinning is unnecessary, and line 41 nowhere claims sufficiency. The reconciliation is then made explicit in the same section at lines 49-52 ('re-pinning the coordinate alone does not fix it either. The lever that works is re-pinning plus treating the front as an exact Rankine-Hugoniot interface'). The gate at qtt_repin_marcher/main.rs:73-77 and output.txt:4-5 (25/35 both cases, +10) are quoted correctly, but they support insufficiency, not supersession. There is no superseded conclusion here to mark.

> Evidence re-read: studies/README.md:41, 42, 46-52; qtt_repin_marcher/main.rs:70-77; qtt_repin_marcher/output.txt:4-6; qtt_rank_fitted_dynamic/README.md:36-40

---

### 2.17 [MINOR] qtt_rank_nonlinear concludes that curvature rather than thickening sets the rank without measuring any stable viscosity variation

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_rank_nonlinear/main.rs:165`
- **Auditor confidence:** likely

**Claim.** This study runs one stable viscosity and one CFL-violating one, so it does not itself measure thickening as a lever; the 'curvature sets the rank' half of its conclusion is imported from qtt_rank_study, which does hold thickness fixed at 2 cells while varying alignment/curvature (flat 5 vs bow 151 vs oblique 394) and does support it.

**Code evidence.**

```
L109 and L118: the only two nu values are `nu: 1.0 * dx2` and `nu: 6.0 * dx2`.
L166-170 (printed conclusion): "Thickening is NOT the curved-shock lever: the rank is set by CURVATURE/mis-alignment, which thickening cannot remove — and naive over-thickening is diffusion-CFL-unstable".
studies/README.md:27 restates it: "Thickening is **not** the lever, because curvature sets the rank and naive over-thickening is diffusion-CFL-unstable, going to full rank."
No flat-versus-curved comparison at fixed nu exists in this file; the flat/curved contrast lives in qtt_rank_study, which is static and whose 'aligned' case is a separate defect.
The explicit 2-D limit nu*dt/dx^2 <= 0.25 with dt = 0.2*dx confines stable nu to below 1.25*dx, so the scheme admits almost no room to sweep — which is itself the finding.
```

**Reference form.** To attribute rank to curvature rather than front thickness, hold curvature fixed and vary thickness (several stable nu), or hold thickness fixed and vary curvature (a curved and a flat front at the same nu and same front width). Neither controlled comparison is present.

**Impact.** This study's conclusion feeds the Tier-B design choice to pursue coordinate alignment plus an implicit step instead of artificial thickening. The recommendation may well be right, but the study supports only the narrower statement that explicit-Euler thickening is boxed in by the diffusion CFL. Related observation on the same run: the reported peak bond of 20 occurs at step 31 (t = 0.097), while Burgers steepening for this initial bump implies t* ~ 1/max|u_x| ~ 0.13, and the bond then declines monotonically 20 -> 15 through step 248 (output.txt:21-28). The peak therefore precedes shock formation and the post-formation trend is downward, which does not match the narrative 'a forming CURVED shock drives bond 7 -> 20'.

**Recommended fix.** State the measured claim precisely: within the stable window of the explicit scheme there is no room to thicken, so thickening cannot be tested as a lever here. To support the curvature attribution, add a flat forming shock at the identical nu, dt and front width and compare peak bonds. Also report the bond time series alongside the peak, and note that the peak precedes shock formation.

**Adversarial check.** The within-file part is confirmed: L109 and L118 are the only two 2-D viscosities (1.0*dx2 and 6.0*dx2), the second violates the explicit 2-D diffusion limit (0.2*6 = 1.2 >> 0.25, acknowledged at L100-101) and saturates at 64, so there is exactly one stable point and no thickening sweep. The printed conclusion at L165-170 and studies/README.md:27 nevertheless assert that curvature rather than thickening sets the rank. But the auditor's claim that neither controlled comparison exists in the corpus is wrong: qtt_rank_study holds front thickness fixed at d2 = 2 cells (main.rs:76) across all five 2-D constructions and varies only the front's orientation/curvature, giving flat 5 vs bow 151 vs oblique 394 — that is precisely the 'hold thickness fixed, vary curvature' comparison, and it is unaffected by the separate obl_aligned defect, which concerns only the aligned row. So the imported half of the conclusion is supported elsewhere in the corpus; what this study alone supports is the narrower statement that explicit-Euler thickening is boxed in by the diffusion CFL. The supplementary observation about peak timing is also correct on the numbers (peak 20 at step 31, t = 0.097, declining monotonically to 15 by step 248 per output.txt:20-28, against a steepening estimate t* ~ 0.13), and the 7 -> 20 narrative does not distinguish pre- from post-formation.

> Evidence re-read: qtt_rank_nonlinear/main.rs:83-133, 135-155 (gates), 157-177 (reading), 232-264 (burgers_2d); output.txt:16-31; qtt_rank_study/main.rs:76, 81-94; studies/README.md:27

---

### 2.18 [MAJOR] qtt_rank_fitted_dynamic infers 'alignment bounds the bond' from a case whose field is exactly y-independent, so the marched problem is one-dimensional

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_rank_fitted_dynamic/main.rs:162`
- **Auditor confidence:** confirmed

**Claim.** The Ic::Planar initial condition 0.5 + 0.4*sin(TAU*x) has no y dependence, and the 2-D Burgers update preserves that: gy applied to a y-constant field is zero and the y-Laplacian contributions vanish. The field is therefore a rank-1 outer product across the x-block/y-block cut for the whole march, and the reported bond 7 at both L=6 and L=7 is the 1-D Burgers bond, already measured as 8 in qtt_rank_nonlinear. The case contains no curved feature, so it cannot show that keeping a curved feature aligned bounds the rank.

**Code evidence.**

```
L162: `Ic::Planar => 0.5 + 0.4 * (TAU * x).sin(),` — no y term. Compare L158-161 (Ic::Curved), which uses a 2-D Gaussian.
L174-179 applies gx, gy and lap to that field each step; none introduces y structure from a y-constant input.
L57-59 (printed): "Axis-ALIGNED planar            :  {align6}    {align7}   <- alignment: low & flat (the lever)".
README.md:29-31: "**Alignment bounds the bond under marching.** The axis-aligned front holds 7 / 7, flat in resolution. ... That is the Res-5 lever, confirmed dynamically rather than statically."
The two cases also differ in initial condition family (sine wave versus Gaussian bump), so alignment is not the only variable between them.
```

**Reference form.** A controlled test of alignment holds the feature fixed and changes only the coordinate in which it is represented. Here the misaligned control is a curved Gaussian bump and the aligned case is a planar sine wave — different fields, different dimensionality of the resulting dynamics.

**Impact.** The middle row of the study's three-row table, and the claim the studies README leads with ('Alignment bounds the bond dynamically'), rests on a degenerate case. The row that does combine curvature with a fitted coordinate (case 3, march_polar) grows 25 -> 35, i.e. it does not support the claim. Note also that case 3 stabilises with a computational-space Laplacian at uniform spacing while the physical cells are strongly anisotropic in the annulus, so its growth is partly attributable to metrically inconsistent dissipation rather than to the front drifting off the chart — the code acknowledges the fidelity compromise at L191-193 but the conclusion drawn from it is a physical causal claim.

**Recommended fix.** Add a case that marches the same curved Gaussian bump in a coordinate where it is axis-aligned, so the comparison isolates alignment. Restate the planar row as what it is: confirmation that a y-invariant problem stays at its 1-D rank. For case 3, use the metric-consistent Laplacian before attributing its growth to coordinate drift.

**Adversarial check.** L162 is verbatim `Ic::Planar => 0.5 + 0.4 * (TAU * x).sin(),` with no y term, in contrast to Ic::Curved's 2-D Gaussian at L158-161. The step at L174-179 applies gx, gy and lap2d; on a y-constant field gy returns zero and the y-Laplacian contribution vanishes, and hadamard_rounded preserves y-independence, so the field remains f(x) for the whole march. Across the x-block/y-block cut the train is therefore rank 1 and max_bond is the 1-D Burgers bond — the reported 7, against the 8 measured for 1-D Burgers in qtt_rank_nonlinear. There is no curved feature in this case, so it cannot show that keeping a curved feature aligned bounds the rank. The auditor's control critique is also correct: the misaligned control is a Gaussian bump and the 'aligned' case is a sine wave, so IC family and effective dimensionality both change. The row that does combine curvature with a fitted chart (burgers_fitted, L194-235) grows 25 -> 35 and does not support the claim; its L190-193 docstring does disclose the computational-space Laplacian on a metrically anisotropic annulus, so the growth is not cleanly attributable to front drift. This row is the one the study README (29-31) and studies/README.md:41 lead with.

> Evidence re-read: qtt_rank_fitted_dynamic/main.rs:140-188 (burgers_cart in full), 190-235 (burgers_fitted), 53-62, 72-78; output.txt:3-6; README.md:23-31; qtt_rank_nonlinear/output.txt:14

---

### 2.19 [INFO] qtt_blend_metric computes a fold marker it never checks, so a per-lambda fold would not be reported by the gate written to catch it

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_blend_metric/main.rs:154`
- **Auditor confidence:** confirmed

**Claim.** The sign == 2 fold marker is never tested and is effectively dead, but folds are still caught in this sweep: lambda = 0 is a non-foldable Cartesian chart that fixes first_sign = -1, so any later lambda returning 2 trips the sign_consistent check. The defect is latent (dead marker, dependent on lambda = 0 staying first) rather than an active gap in BM-A.

**Code evidence.**

```
L150-155: `let s = if det >= 0.0 { 1 } else { -1 }; if sign == 0 { sign = s; } else if s != sign { sign = 2; // mixed-sign marker (a fold) }`
L58-62 (the only consumer): `if first_sign == 0 { first_sign = sign; } else if sign != first_sign { sign_consistent = false; }`
L73: `if !sign_consistent || min_det_overall <= 1e-6 {` — the value 2 is never distinguished from 1 or -1.
Separately, jacobian_scan evaluates the analytic derivatives of the continuous chart (L142-145), so 'no cell folds' is a statement about the continuous map, not about discrete lattice cell areas.
```

**Reference form.** A fold is a sign change of det J anywhere in the domain. The marker exists to detect it and should gate directly: `if sign == 2 { fail }`.

**Impact.** Bounded in practice, because a fold implies det J passes through zero and the min|det J| test would likely catch it at 256^2 resolution. But the gate does not detect the condition it was written to detect, and BM-A is the gate that closes 'the one open residual Res 4 flagged'. Also worth noting: the two charts share the same eta -> r map, so the blend differs only transversally, which makes non-folding close to automatic for this pair; the study README does disclose this at line 44 ('Two charts over one patch is also the easy case for non-folding').

**Recommended fix.** Add an explicit `if sign == 2` failure branch. Optionally also check discrete cell areas (signed quadrilateral areas of adjacent lattice nodes) rather than only the analytic det J, since it is the discrete map the codec sees.

**Adversarial check.** The code reading is exact: L150-155 sets sign = 2 as an explicit 'mixed-sign marker (a fold)', and the only consumer (L58-62) compares sign against first_sign and never tests sign == 2, with L73 gating on !sign_consistent || min_det <= 1e-6. But the failure scenario the auditor constructs is unreachable for this sweep. lambda = 0 is the first entry of `lambdas` (L46) and is the pure Cartesian chart, whose Jacobian is constant (dxdxi = 0, dxdeta = DR, dydxi = span_y, dydeta = 0, det = -DR*span_y) and cannot fold, so first_sign is always -1. Any later lambda that folds returns 2, and 2 != -1 trips sign_consistent = false. So a per-lambda fold IS reported, via the cross-lambda comparison rather than the marker. The residual defects are latent robustness (the marker is dead code, and the check would break if lambda = 0 were dropped from the sweep) and the auditor's correct secondary point that jacobian_scan evaluates the analytic derivatives of the continuous chart (L142-145), so 'no cell folds' is a statement about the continuous map rather than discrete lattice cell areas.

> Evidence re-read: qtt_blend_metric/main.rs:46, 55-67, 72-77, 129-159 (jacobian_scan in full); output.txt:5-11 (sign '-' at every lambda); README.md:29-32, 44

---

### 2.20 [INFO] srp_momentum_jet's R-C' interface band has an upper bound the measurement geometry cannot reach, making the gate one-sided

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/srp_momentum_jet/config.rs:80`
- **Auditor confidence:** confirmed

**Claim.** The R-C' band's upper bound (0.56) sits between the two highest reachable node positions (0.53125 and 0.5625), so the downstream direction is gated by exactly one reachable failing value rather than being unfalsifiable; the containment evidence is one node wide at the top, not absent.

**Code evidence.**

```
config.rs:80: `pub const INTERFACE_X_BAND: (f64, f64) = (0.44, 0.56);`
main.rs:569-574: `let face_col = ((face_x / dx) as usize).saturating_sub(1); let interface_x = (1..=face_col).filter(...).max_by(...).map(node).unwrap_or(f64::NAN);` with face_x = BODY_CX - x_hat(R_BODY) = 0.72 - 0.125 = 0.595 and dx = 1/32, giving face_col = 19 - 1 = 18 and a search ceiling of 0.5625.
main.rs:271-279 applies the band two-sidedly: `if !(ix >= ilo && ix <= ihi)`.
Also note main.rs:571's `.filter(|i| !patch_cols.contains(i))` is dead at L=5: the patch column is index 19, which lies outside the 1..=18 range.
```

**Reference form.** A two-sided containment band should have both bounds inside the reachable range of the measurement.

**Impact.** Limited, because upstream interface motion is the physically interesting direction (jet penetration pushes the stagnation interface away from the body) and that direction is genuinely gated. But the band as written implies containment evidence it does not provide, and the study's headline 'freezes the stagnation interface at the face, x = 0.469-0.531 across a 32x thrust range' reads as a two-sided result. Note also that config.rs:66-80 is commendably explicit that these bands are 'pinned from the FIRST measured v3 run' and 'not a physics truth claim' — that disclosure is the right practice and should be mirrored in the study README, which presents the same numbers without it.

**Recommended fix.** Either extend the search range past the body face or drop the upper bound and gate only the upstream side, documenting why. Carry config.rs's honest 'pinned from the first run, regression not physics' framing into README.md, where the bands currently appear as findings.

**Adversarial check.** The geometry is exactly as stated and I re-derived it: face_x = 0.72 - 0.125 = 0.595, dx = 1/32, face_col = (0.595/0.03125) as usize - 1 = 19 - 1 = 18, so the search at L570-574 ranges over nodes 1..=18 and the largest reportable interface_x is 18/32 = 0.5625. The dead-filter observation is also right: the patch column is index 19 (the only node with 0.56375 < x < 0.595), which lies outside 1..=18, so `.filter(|i| !patch_cols.contains(i))` never removes anything at L=5. But the band is NOT unfalsifiable downstream. The band's upper limit is 0.56 and node 18 evaluates to 0.5625 > 0.56, so there is exactly one reachable value that trips R-C' in the downstream direction. The gate is one node wide at the top, not one-sided. The auditor's own framing is otherwise sound, including the observation that config.rs:66-80 is commendably explicit that these bands are pinned from the first measured run and are not a physics truth claim, while the README presents 0.469-0.531 without that disclosure.

> Evidence re-read: srp_momentum_jet/config.rs:66-80; main.rs:565-576, 271-279, 427-464 (patch_cols construction); output.txt:5, 17-35 (iface x values 0.469-0.531)

---

### 2.21 [MINOR] traj_fs3 gate G4 does not encode the reported -57.2 ns figure; a 3x regression would still pass

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/traj_fs3_clock/main.rs:106`
- **Auditor confidence:** confirmed

**Claim.** The gate accepts any accumulated offset with magnitude in [10, 200] ns, against a reported value of -57.2 ns. A change that moved the result to 150 ns (45 m of ranging drift rather than 17 m) would pass while making the README's headline number and its 17.2 m consequence wrong.

**Code evidence.**

```
L104-107: `let g4 = gate("reentry-blackout clock carry is finite and bounded (10-200 ns)", offset_ns.abs() > 10.0 && offset_ns.abs() < 200.0);`
README.md:35-37: "the accumulated `tau - t` offset is **-57.2 ns**, which is **17.2 m** of ranging drift if uncorrected."
studies/README.md:77: "A 180 s reentry blackout drifts **-57 ns, i.e. 17 m,** uncorrected."
The same pattern appears in qtt_rank_dynamic/main.rs:100-105, where `let cap = 32;` gates observed peaks of 8/8/7, and in traj_fs2_coupling/main.rs:224-226, where `rel_moderate < 1e-4` gates an observed 3.913e-7.
```

**Reference form.** The studies README states at line 11-12 that gates 'encode the finding and exit nonzero on regression'. A gate encoding -57.2 ns would be a tolerance band around that value; a 10-200 ns window is a sanity check.

**Impact.** Minor in isolation — the underlying physics is correct and I verified the -57.2 ns and 17.2 m figures independently. The issue is the corpus-wide pattern: gates whose thresholds sit several multiples away from the observed values are sanity checks, and presenting them under a blanket 'gates encode the finding' claim overstates the regression coverage. G1-G3 in this study are the good counter-example: they are tied to external textbook values with the observed results comfortably inside.

**Recommended fix.** Either tighten G4 to a band around -57.2 ns (e.g. +/-5%) or relabel it in the output as a sanity bound and adjust the studies README's blanket statement to distinguish finding-encoding gates from sanity gates. The same distinction should be applied to qtt_rank_dynamic G1 and traj_fs2 G2/G3.

**Adversarial check.** L104-107 is verbatim: `offset_ns.abs() > 10.0 && offset_ns.abs() < 200.0` against a reported -57.2 ns (output.txt:16). A drift to 150 ns — 45 m of ranging error rather than the 17.2 m the README headlines — would pass. The corpus pattern is confirmed at both cited siblings: qtt_rank_dynamic/main.rs:100-105 gates observed peaks of 8/8/7 against `let cap = 32;`, and traj_fs2_coupling gates `rel_moderate < 1e-4` against an observed 3.913e-7. The counter-example the auditor offers is also fair: G1-G3 in this study are tied to textbook GPS values (+45.7 / -7.2 / +38.5) with tight windows and the observed values comfortably inside. The issue is the studies README's blanket claim at lines 11-12 that gates 'encode the finding and exit nonzero on regression', which overstates the coverage of the sanity-band gates.

> Evidence re-read: traj_fs3_clock/main.rs:91-107; output.txt:16, 22; README.md (57.2 ns / 17.2 m); studies/README.md:11-12, 77; qtt_rank_dynamic/main.rs:99-105; traj_fs2_coupling/main.rs:223-226

---

### 2.22 [MINOR] srp_momentum_jet's W-T momentum-flux witness is near-tautological because the forcing region overwrites the patch exactly each step

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/srp_momentum_jet/main.rs:222`
- **Auditor confidence:** confirmed

**Claim.** ForcingRegion::apply uses w = min(dt/eta, 1); the study constructs the jet region with eta = cfg.dt, so w = 1 and the update becomes U <- U - chi*(U - T), which sets U = T exactly wherever chi = 1. t_realized is then read from the terminal state after the jet apply, summed over the same patch cells whose target was solved to satisfy the C_T relation. The gate therefore chiefly measures whether the truncated TT mask equals 1 on those nodes, not a property of the flow.

**Code evidence.**

```
main.rs:365: `ForcingRegion::new(mask, target, cfg.dt)` — eta = dt.
forcing.rs:100-101: `let ratio = dt / self.eta; let w = if ratio < R::one() { ratio } else { R::one() };` — w = 1.
forcing.rs:90 (docstring): "`U_k <- U_k - w*chi (*) (U_k - T_k)`, `w = min(dt/eta, 1)`"; forcing.rs:33: "`eta <= dt` enforces hard".
main.rs:581-587: `let t_realized: f64 = patch.iter().map(|&k| { let rho = d[0][k]; (d[1][k] * d[1][k] / rho + pressure(&d, k) - p_amb) * dx }).sum();`
main.rs:221-227: `let t_expect = ct * q_hat_inf() * d_hat_body(); if (out.t_realized - t_expect).abs() > 0.05 * t_expect { ... }` — and config.rs:164-166 solved p_e so that h_hat_eff*(rho_e u_e^2 + p_e - p_inf) equals exactly that product.
```

**Reference form.** A mechanism witness should constrain something the construction does not already determine. Here the only slack is the smoothed mask's deviation from unity after truncation at bond cap 24, and the tolerance is 5%.

**Impact.** The study docstring (main.rs:31-32) lists this among the 'mechanism witnesses' pre-registered by the adversarial review, alongside the interface location and the freestream probe, which are genuine measurements. W-T's actual content is much narrower — mask fidelity — and it should be described as such so the evidence bar is not overcounted. This does not affect the study's conclusions, which do not lean on W-T.

**Recommended fix.** Relabel W-T as a mask-fidelity check. If an independent flux witness is wanted, integrate the momentum flux across a plane one or two cells downstream of the pinned column, where the field is free to evolve.

**Adversarial check.** Verified end to end. main.rs:365 constructs the jet region with eta = cfg.dt; forcing.rs:76-77 computes `let ratio = dt / self.eta; let w = if ratio < R::one() { ratio } else { R::one() }`, so w = 1 and the update U <- U - w*chi*(U - T) sets U = T exactly wherever chi = 1 (docstring at forcing.rs:65-66; forcing.rs:33 notes eta <= dt enforces hard). t_realized (L581-587) is read from the terminal state after the jet apply, summing (rho u^2 + p - p_amb)*dx over the exit column's jet_rows nodes, which equals h_hat_eff*(rho_e u_e^2 + p_e - p_inf); config.rs:164-166 solved p_e so that this equals exactly ct*q_hat_inf()*d_hat_body(), the very quantity L221 forms as t_expect. The only slack is the smoothed mask's deviation from unity after by_bond(24) truncation, tested at 5%. So W-T measures mask fidelity, not a property of the flow. It is listed among the pre-registered 'mechanism witnesses' at main.rs:30-32 alongside the interface location and freestream probe, which are genuine measurements; it should be described narrowly. The study's conclusions do not lean on it.

> Evidence re-read: srp_momentum_jet/main.rs:30-32, 220-227, 348-366, 578-587; src/solvers/qtt/compressible/forcing.rs:31-48, 65-86; config.rs:151-171

---

### 2.23 [INFO] qtt_blend_metric's 'clean rank dial, intermediate lambda buys intermediate rank' is contradicted by its own measured sweep

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/qtt_blend_metric/README.md:34`
- **Auditor confidence:** confirmed

**Claim.** The lambda response is strongly nonlinear and concentrated near lambda = 1 (114/107/92/54/5), which the README does not say; but 'intermediate lambda buys intermediate rank' is not contradicted — lambda = 0.75 does deliver a 2.1x reduction — so this is an emphasis gap rather than a false claim.

**Code evidence.**

```
output.txt:7-11: `0.00 | 114`, `0.25 | 107`, `0.50 | 92`, `0.75 | 54`, `1.00 | 5`.
README.md:33-35: "**BM-B: lambda is a clean rank dial.** A fixed *physical* curved shock, sampled on the blended lattice, runs **114 -> 5** monotonically as lambda goes 0 to 1. Intermediate `lambda` buys intermediate rank."
main.rs:15-17 (docstring) makes the stronger version: "intermediate lambda gives intermediate rank at zero asymptotic cost".
studies/README.md:44: "lambda is a clean rank dial."
The gate at main.rs:81-82 only checks monotonicity with a +1 slack and the endpoint ratio, so it does not constrain the shape of the response.
```

**Reference form.** The measurement supports 'lambda is monotone and the endpoint gain is large'. It does not support proportionality or a useful intermediate regime.

**Impact.** This is the study that closes the Res-4 residual and licenses treating body-fittedness as a tunable free parameter. If a design leans on partial blending to trade geometric generality against rank, the measured curve says a partial blend buys almost nothing until lambda is near 1 — the opposite of the operational reading 'not an all-or-nothing commitment' (README.md:35). Note the underlying map and Jacobian in this study are correct; only the characterisation of the response is overstated.

**Recommended fix.** Restate as: monotone, with the reduction concentrated above lambda ~ 0.75; a partial blend delivers little rank benefit. Add intermediate points (0.85, 0.95) to locate the knee, since that is the operationally useful number.

**Adversarial check.** The measured numbers are exactly as quoted (output.txt:7-11: 114, 107, 92, 54, 5) and the shape observation is correct and worth recording — lambda 0 -> 0.5 buys 19% while 0.75 -> 1.0 buys a factor of 11, so the response is strongly nonlinear and concentrated at the fitted end. The gate at L81-82 does only check monotonicity with +1 slack plus the endpoint ratio, so it does not constrain the shape. But the README sentence is not contradicted by the sweep: 107, 92 and 54 ARE intermediate ranks, and lambda = 0.75 delivers 114 -> 54, a 2.1x reduction, so 'intermediate lambda buys intermediate rank' and 'not an all-or-nothing commitment' are both literally borne out. The overstatement is one of proportionality/emphasis, not of fact. Downgraded to info: the honest fix is one added sentence noting the response is convex toward lambda = 1, not a retraction.

> Evidence re-read: qtt_blend_metric/main.rs:14-15, 46-67, 79-86, 161-180; output.txt:5-11; README.md:33-35; studies/README.md:44

---

### 2.24 [MINOR] qtt_acoustic_precond describes I + A0^-1 A1 as contracting and extends a stationary-iteration bound to AMEn convergence

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/studies/qtt_acoustic_precond/README.md:17`
- **Auditor confidence:** confirmed

**Claim.** Two distinct imprecisions. First, I + A0^-1 A1 does not contract: with rho(A0^-1 A1) = 0.59 its spectrum lies in roughly [0.41, 1.59], so its spectral radius exceeds 1. What contracts is the preconditioned Richardson error propagator -A0^-1 A1. Second, rho(A0^-1 A1) < 1 governs a stationary preconditioned iteration; AMEn is an alternating-least-squares optimisation over the TT manifold, and its convergence is not characterised by that spectral radius.

**Code evidence.**

```
README.md:16-18: "on a **smooth** sound-speed field `||A0^-1 A1|| < 1`, so the preconditioned solve `A0^-1 A = I + A0^-1 A1` contracts. Together they convert an unbounded question, 'does AMEn converge?', into a measurable perturbation bound."
main.rs:11-13 (docstring) repeats it verbatim.
main.rs:80 (printed): "so A0^-1 is a cheap preconditioner — no AMEn-convergence gamble on the core."
main.rs:178-217: spectral_radius builds dense n x n matrices at l = 7 (n = 128) and power-iterates — the measurement is entirely dense and one-dimensional, with no TT or AMEn object involved.
The README does disclose the model-field limitation at line 42-45 but not the iteration-class substitution.
```

**Reference form.** For the splitting A = A0 + A1 preconditioned by A0, the stationary iteration error propagator is I - A0^-1 A = -A0^-1 A1, and convergence requires rho(A0^-1 A1) < 1 (Saad, Iterative Methods for Sparse Linear Systems, 2nd ed., Ch. 4). AMEn convergence (Dolgov & Savostyanov, SIAM J. Sci. Comput. 36(5), 2014) is governed by the ALS local-problem conditioning and the enrichment strategy, not by this spectral radius.

**Impact.** The arithmetic of the split and the spectral-radius computation are correct — I verified A0 + A1 = A and the power iteration. The defect is in what the number licenses. The study is titled as de-risking the implicit step and its stated purpose is to retire the AMEn convergence question; the measurement retires a different question, on a dense 1-D model problem. An engineer could reasonably take AC-B as evidence that the QTT implicit solve will converge, which it is not.

**Recommended fix.** State the correct object ('the preconditioned Richardson iteration contracts at rate 0.59', or 'the preconditioned spectrum lies in [1-rho, 1+rho]'). Scope the AMEn claim explicitly: rho < 1 shows the splitting is a sound preconditioner in the classical sense and bounds the operator's conditioning; it is not an AMEn convergence proof. AC-A does exercise the real solver (main.rs:169) and is the part that speaks to AMEn.

**Adversarial check.** Both imprecisions are real and both texts are as quoted. README.md:16-18 and main.rs:11-13 say 'the preconditioned solve A0^-1 A = I + A0^-1 A1 contracts'. With rho(A0^-1 A1) = 0.59 the spectrum of I + A0^-1 A1 lies roughly in [0.41, 1.59], so its spectral radius exceeds 1 and it does not contract; the auditor's reference form is the correct one — for the splitting A = A0 + A1 preconditioned by A0 the stationary error propagator is I - A0^-1 A = -A0^-1 A1, and convergence requires rho(A0^-1 A1) < 1 (Saad, Iterative Methods for Sparse Linear Systems, 2nd ed., Ch. 4). Second, main.rs:80 prints 'no AMEn-convergence gamble on the core' and the study is framed as converting 'does AMEn converge?' into a perturbation bound, but AMEn is an ALS optimisation over the TT manifold whose convergence is governed by local-problem conditioning and enrichment (Dolgov & Savostyanov 2014), not by this spectral radius — and the measurement itself is entirely dense and 1-D (L178-217 builds n x n matrices at l = 7 and power-iterates, with no TT or AMEn object involved). The arithmetic of the split and the power iteration are correct; the defect is in what the number licenses. README.md:42-45 discloses the model-field limitation but not the iteration-class substitution.

> Evidence re-read: qtt_acoustic_precond/main.rs:4-21, 76-91, 146-217; README.md:12-19, 29-35, 42-45; output.txt:11-12

---

### 2.25 [MINOR] traj_fs3 README states the study consumes the shipped kernel, but the two reported component offsets are computed inline

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/studies/traj_fs3_clock/README.md:23`
- **Auditor confidence:** confirmed

**Claim.** Only the net rate goes through relativistic_clock_offset_kernel. The +45.65 and -7.21 us/day figures that G1 and G2 gate are computed by inline arithmetic in the study, so two of the three reported GPS numbers do not exercise the shipped kernel.

**Code evidence.**

```
main.rs:53-54: `let grav_rate = (mu / r_e - mu / GPS_SEMI_MAJOR_M) / (c * c); let vel_rate = -(v_gps * v_gps) / (2.0 * c * c);`
main.rs:58: `let net_rate = relativistic_clock_offset_kernel(GPS_SEMI_MAJOR_M, v_gps, r_e, 0.0, mu).unwrap();` — the only kernel call for the GPS split.
main.rs:92-99: G1 gates grav_us_day and G2 gates vel_us_day, both derived from the inline rates.
README.md:22-24: "The forward clock rate offset has since been **promoted out of this study into a reusable physics kernel** (`relativistic_clock_offset_kernel`, capability 5). The study now consumes the shipped kernel rather than defining its own."
The code is honest about it at main.rs:51: "split the rate into its two physical pieces for reporting" and main.rs:57: "Sanity: the combined kernel equals the sum of the two pieces".
```

**Reference form.** If a study is cited as validating a shipped kernel, the gated quantities should be produced by that kernel.

**Impact.** Small, because I verified the inline expressions match the kernel's formula exactly and the net value cross-checks against the sum. But the kernel's own docstring (forward_clock.rs:11-12) cites this study as its validation — "the Gap-3 trajectory-axis feasibility study ... which validated it against the textbook GPS relativistic split" — so the circular citation should rest on calls that actually go through the kernel.

**Recommended fix.** Compute grav_rate and vel_rate via relativistic_clock_drift_rate_kernel with the appropriate arguments (setting speed or the potential difference to isolate each term), or state in the README that the components are reported inline and only the net is kernel-produced.

**Adversarial check.** main.rs:53-54 computes grav_rate and vel_rate by inline arithmetic; L58 is the only kernel call for the GPS split, producing net_rate; G1 (L92-95) and G2 (L96-99) gate grav_us_day and vel_us_day, both derived from the inline rates. README.md:22-24 states the study 'now consumes the shipped kernel rather than defining its own'. The code is honest at L51 ('split the rate into its two physical pieces for reporting') and L57 ('Sanity: the combined kernel equals the sum of the two pieces'), so the defect is confined to the README. The circular-citation concern is confirmed: forward_clock.rs:10-12 cites this study as having 'validated it against the textbook GPS relativistic split (+45.7 / -7.2 / +38.5 us/day)', two of whose three numbers are gated on expressions that do not go through the kernel. The inline expressions do match the kernel's formula, so nothing is numerically wrong.

> Evidence re-read: traj_fs3_clock/main.rs:22-26, 51-59, 91-107; README.md:22-24; deep_causality_physics/src/kernels/chronometric/forward_clock.rs:6-14; output.txt:8-12

---

### 2.26 [MINOR] compressible_carrier_timing's 200-step corridor budget is an undocumented constant that decides the 3-D no-go

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/studies/compressible_carrier_timing/main.rs:35`
- **Auditor confidence:** confirmed

**Claim.** CORRIDOR_STEPS = 200 is described only as 'three leg spans plus the counterfactual branch study' with no derivation from a physical corridor duration and the solver's dt. The entire 3-D verdict is 10.805 s/step x 200 = 2161 s against a 600 s budget; the family is declared out by a factor of 3.6, so any factor-of-3 error in the step count reverses the conclusion.

**Code evidence.**

```
L34-35: `/// The corridor's step budget: three leg spans plus the counterfactual branch study.  const CORRIDOR_STEPS: usize = 200;`
L114: `let projected = m.per_step_s * CORRIDOR_STEPS as f64;`
Output line 6: `3d-fitted 16^3 16 0.004 s 10.805 s 2161.0 s (36.0 min) over budget`.
Related unexplained literals in the same measurement path: L148 and L204 both `let (dt, s_ref) = (0.002, 1.3);` — dt is not tied to any stated corridor time span, and s_ref sets the dissipation floor nu = 0.5*s_ref*dx (verified in marcher_2d.rs:89) with no justification for 1.3.
README.md:51-54's caveats cover hardware, the exclusion of coupling overhead, the 5-sample size and the single-grid 3-D extrapolation, but not the step count or dt.
```

**Reference form.** A step budget should follow from the corridor's physical duration divided by the stable dt, so that a reader can check both the numerator and the denominator. dt = 0.002 over 200 steps is t_end = 0.4 in nondimensional time; the corridor's physical span in those units is never stated.

**Impact.** The methodology is otherwise sound for a timing study — there is a real untimed warmup step (L179, L224), wall-clock via Instant, and the go/no-go selection logic at L82-85 correctly picks the largest in-budget configuration. The 2-D margin is 17x so the GO is robust to this constant; the 3-D NO-GO at 3.6x is not comfortably so, and the study explicitly uses that single 16^3 point to rule out the whole 3-D family ('larger 3-D grids are a foregone conclusion', L56-57).

**Recommended fix.** Derive CORRIDOR_STEPS from a stated corridor duration and the marcher's dt, show the arithmetic in the output header, and note the sensitivity of the 3-D verdict to it. Justify or cite dt = 0.002 and s_ref = 1.3.

**Adversarial check.** L34-35 is verbatim, with the whole justification being the comment 'three leg spans plus the counterfactual branch study' — no corridor duration, no dt relation, no derivation. L114 multiplies per_step_s by it, and the 3-D verdict is 10.805 * 200 = 2161 s against BUDGET_S = 600 (output.txt:6), i.e. out by 3.6x, so a factor-of-3 error in the step count reverses a decision the study explicitly generalises ('larger 3-D grids are a foregone conclusion', L56-57). The related literals are confirmed: L148 and L204 both `let (dt, s_ref) = (0.002, 1.3);` with dt tied to no stated corridor span and s_ref = 1.3 unjustified. README.md's caveats cover hardware, coupling overhead, sample size and the single-grid 3-D extrapolation but not the step count or dt. The auditor's positive observations are also correct: there is a real untimed warmup (L179, L224), Instant-based wall-clock, and the selection logic at L82-85 correctly picks the largest in-budget configuration; the 2-D GO has a 17x margin and is robust to this constant.

> Evidence re-read: compressible_carrier_timing/main.rs:34-39, 56-63, 82-99, 114, 148, 179, 204, 224; output.txt:3-19; README.md caveats section

---

### 2.27 [MINOR] The synthetic shock thickness delta = 2 cells sets the headline rank numbers in three studies and is never varied

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/studies/qtt_rank_study/main.rs:76`
- **Auditor confidence:** confirmed

**Claim.** The QTT rank of a tanh front scales with how few cells the front spans, so the front-thickness literal is the dominant control on every static rank reported. It is fixed at 2 cells in qtt_rank_study and qtt_blend_metric and at 3 cells in qtt_repin_marcher, with no sensitivity analysis in any of them. The headline values 151, 394, 114 and 5 are therefore quoted without their principal dependency stated.

**Code evidence.**

```
qtt_rank_study/main.rs:76: `let d2 = 2.0 / side as f64; // ~2-cell thickened shock` — used in all five 2-D constructions (L81-100), producing the 5 / 394 / 151 headline.
qtt_blend_metric/main.rs:47: `let delta = 2.0 * DR / side as f64; // ~2 radial cells: a sharp front` — produces the 114 -> 5 dial.
qtt_repin_marcher/main.rs:131: `let delta = 3.0 / n as f64; // ~3-cell interface`.
qtt_rank_3d/main.rs:163: `let d = 2.0 / side as f64;`.
Other unjustified profile literals in the same family: qtt_rank_study/main.rs:48 `let (peak, eq, tau) = (1.0, 0.3, 0.05);` labelled 'Captured reentry stagnation line' with no source; qtt_rank_study/main.rs:98 `r_axis * 0.7 - 0.35`; qtt_rank_plume/main.rs:588,600 `r_shock = 1.3 + standoff` and `(x - (1.15 + half_length))`; qtt_rank_plume/main.rs:442 `plume_region(..., 0.55)` versus A1's computed placement at main.rs:454.
```

**Reference form.** Any numeric literal in a physics path that determines a reported result should be traceable to a source, a resolution argument, or a documented sensitivity. A 2-cell front is a plausible convention for a captured shock, but the convention should be stated and the result's sensitivity to it shown, since it is the leading term.

**Impact.** An engineer cannot tell whether 'a curved Cartesian shock costs chi = 151' is a property of curved shocks or of 2-cell-thick tanh profiles. This matters most for qtt_rank_study, whose numbers propagate into the Tier-B cost argument, and for the comparison against a real solver, where the numerically realised front thickness follows from the artificial viscosity rather than from a chosen constant. The 'captured reentry stagnation line' constants (peak 1.0, equilibrium 0.3, relaxation length 0.05) are additionally presented with a physical label but no reentry-profile provenance.

**Recommended fix.** Sweep the front thickness over 1, 2, 4 and 8 cells for the curved and oblique cases and report the rank as a function of it, so readers see the scaling rather than one point. Either cite a source for the stagnation-line constants or relabel that profile as a synthetic three-region test function.

**Adversarial check.** Every cited literal is present verbatim: qtt_rank_study/main.rs:76 `let d2 = 2.0 / side as f64; // ~2-cell thickened shock` feeding all five 2-D constructions at L81-100 (the 5/151/394 headline); qtt_blend_metric/main.rs:47 `let delta = 2.0 * DR / side as f64; // ~2 radial cells: a sharp front` (the 114 -> 5 dial); qtt_repin_marcher/main.rs:131 `let delta = 3.0 / n as f64;`; qtt_rank_3d/main.rs:163 `let d = 2.0 / side as f64;`. No study varies it, and the QTT rank of a tanh front is strongly controlled by how few cells it spans, so this is the leading dependency of every static rank reported and it is quoted without being stated. The secondary literals check out too: qtt_rank_study/main.rs:47-48 labels `(1.0, 0.3, 0.05)` a 'Captured reentry stagnation line' with no provenance; L98's `r_axis * 0.7 - 0.35`; qtt_rank_plume/main.rs:588 `let r_shock = 1.3 + standoff;` and L600 `(x - (1.15 + half_length))`; and plume_region(..., 0.55) at L442 against the computed placement at L451-455. The auditor's framing is fair — a 2-cell captured front is a plausible convention, but it should be stated with a sensitivity, not left implicit.

> Evidence re-read: qtt_rank_study/main.rs:47-48, 74-104; qtt_blend_metric/main.rs:47; qtt_repin_marcher/main.rs:131; qtt_rank_3d/main.rs:163; qtt_rank_plume/main.rs:440-443, 449-457, 586-604

---

### 2.28 [MINOR] qtt_rank_3d's growth exponent is a two-point endpoint fit that is below the local slope at the fine end, and the extrapolated flight-grid rank is understated by roughly an order of magnitude

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/studies/qtt_rank_3d/main.rs:195`
- **Auditor confidence:** confirmed

**Claim.** The endpoint-only fit is confirmed and understates the fine-end slope (local exponents 0.31/0.67/0.60 vs the reported 0.53); the flight-grid extrapolation evaluates to ~1.6e4 against the verbal 'chi ~ thousands', which is understated by a factor of a few rather than an order of magnitude.

**Code evidence.**

```
L192-195: `// Fit chi ~ side^p from first..last: p = ln(chi_last/chi_first) / ln(side_last/side_first). let (s0, c0) = sweep[0]; let (s1, c1) = sweep[sweep.len() - 1]; let exponent = (c1 as f64 / c0 as f64).ln() / (s1 as f64 / s0 as f64).ln();`
output.txt:22-26: 45, 56, 89, 135 at sides 16, 32, 64, 128 with printed ratios 1.24x, 1.59x, 1.52x; `=> fitted growth: chi ~ side^0.53`.
L246-249 (printed): "chi ~ sqrt(side) means at a flight-relevant micrometre grid (side ~ 1e6) a captured curved shock implies chi ~ thousands".
README.md:42: "a captured curved shock implies `chi ~ thousands`".
The smallest point is also close to its representational limit: at side = 16 with block ordering the central-cut bond ceiling is 2^6 = 64, and 45 is 70% of it, so the low anchor may not be in the asymptotic regime.
```

**Reference form.** A power-law exponent from four points should be a least-squares fit in log-log space with the residuals reported, and the extrapolation should be evaluated rather than characterised in words. log2 of the successive ratios gives 0.31, 0.67, 0.60.

**Impact.** The exponent drives the whole cost argument, including the claim that QTT storage 'always wins asymptotically' because chi^2 ~ side^1.1 is outrun by side^3. That conclusion survives a steeper exponent (side^1.3 still loses to side^3), so the qualitative reading holds. The solve-cost argument is more sensitive: at exponent 0.65 the flight-grid chi is roughly 4x larger than at 0.53. The README does disclose that 'the sqrt(side) law is read from four points' (line 57-58), which is the right caveat; the endpoint-only fitting method and the verbal extrapolation are what should be tightened.

**Recommended fix.** Use a least-squares log-log fit over all four points, print the residuals and the local slopes, and print the numeric extrapolated chi at side = 1e6 rather than describing it. Note that the 16^3 point sits near its bond ceiling and consider excluding it or adding a 256^3 point.

**Adversarial check.** The methodological half is confirmed exactly. L192-195 is verbatim an endpoint fit, discarding the two interior points; from output.txt:22-26 (45, 56, 89, 135 at sides 16-128) I re-derived ln(135/45)/ln(128/16) = 1.0986/2.0794 = 0.528, and the local log2 slopes of the printed ratios are log2(1.24) = 0.31, log2(1.59) = 0.67, log2(1.52) = 0.60 — so the fine-end slope is materially steeper than the reported 0.53 and a log-log least-squares fit with residuals would be the right form. The observation that the coarse anchor may not be asymptotic is also fair (at side 16 the central-cut ceiling is 2^6 = 64 and 45 is 70% of it). The extrapolation half is overstated: 135*(1e6/128)^0.53 = 1.56e4, which is high relative to 'chi ~ thousands' by a factor of roughly 2-5, not an order of magnitude — and 'thousands' is loose enough to be arguable. The auditor correctly concedes that the storage conclusion survives a steeper exponent and that README.md:56-58 already discloses the four-point basis.

> Evidence re-read: qtt_rank_3d/main.rs:172-196, 244-249; output.txt:20-26, 46-49; README.md:40-44, 53-58

---

### 2.29 [MINOR] qtt_rank_plume's A1 rank measurement returns an identical bond of 16 for all five configurations, with no positive control establishing sensitivity

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_rank_plume/main.rs:114`
- **Auditor confidence:** likely

**Claim.** Peak bond is 16 for C_T = 0.5, 1.0, 2.0, 4.0 and for the no-plume baseline. The gate at line 118 fires only above 32 and the finding at line 122 fires only if the plume case exceeds the baseline, so with all five equal neither branch executes and A1 produces no gate output at all. Nothing in the study demonstrates that this measurement responds to anything, so 'the plume imprint costs nothing in rank' is equally consistent with the measurement being insensitive at this grid and tolerance.

**Code evidence.**

```
L106-115: the sweep and `let unforced_peak = marched_peak_bond(None)...` all report 16 (output.txt:8-12).
L118-126: `if a1_max >= RANK_CEILING { failures.push(...) } else if a1_max > unforced_peak { findings.push(...) }` — with a1_max = unforced_peak = 16, neither arm runs.
README.md:30-31: "peak bond **16 at every C_T** tested (0.5, 1, 2, 4) and **16** for the no-plume body/shock baseline. The plume imprint costs nothing in rank over the baseline."
studies/README.md:64: "The marched imprint holds peak bond 16 at every C_T, matching the no-plume baseline".
The truncation is by_tol (main.rs:487), so 16 is not a cap; 16 = 2^4 on a 32x32 grid whose ceiling is 32.
```

**Reference form.** A null result ('X adds no cost') requires a positive control demonstrating that the measurement would have detected a cost had one been present.

**Impact.** M1 risk 2 is recorded green partly on this row. Without a control, the invariance across a 8x thrust range and across the presence or absence of the plume entirely is at least as suggestive of a measurement floor set by the sponge or body masks as of a genuine physical null. Worth noting that the study's own caveats section is otherwise unusually candid, and that A2 does provide independent rank evidence.

**Recommended fix.** Add a positive control to A1: march an imprint deliberately constructed to be high-rank (an oblique or strongly curved mask) and confirm the peak bond rises above 16. If it does not, the measurement is floor-limited and the null result should be withdrawn. Also report which of the four state components and which core carries the peak, to identify what sets the 16.

**Adversarial check.** output.txt:8-12 shows peak bond 16 at C_T = 0.5, 1, 2, 4 and 16 for the no-plume baseline. The gate structure at L118-126 is `if a1_max >= RANK_CEILING { failures.push } else if a1_max > unforced_peak { findings.push }`; with a1_max = unforced_peak = 16 and RANK_CEILING = 32, neither arm executes, so A1 emits no gate or finding output at all — and indeed no A1 finding appears in the committed artifact. The truncation is by_tol (L487), so 16 is not a cap, and on a 2^5 x 2^5 field with block ordering 16 is the saturation limit of the bonds adjacent to the central cut, which makes a measurement floor a live alternative explanation for the invariance across an 8x thrust range and across the presence or absence of the plume entirely. A null result of this kind does need a positive control. Correctly rated minor: A2 provides independent rank evidence (32 vs 10-12) and the study's caveats section is otherwise candid.

> Evidence re-read: qtt_rank_plume/main.rs:81-82, 101-126, 449-462, 479-537; output.txt:5-12; README.md:30-31, 57-62; studies/README.md:64

---

### 2.30 [INFO] srp_momentum_jet's baseline identity witness is a hardcoded prior output of a harness that has since been reverted

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/srp_momentum_jet/config.rs:93`
- **Auditor confidence:** confirmed

**Claim.** VERIFICATION_BASELINE is a literal copy of this codebase's own earlier output, and the harness it came from has been removed from the tree. The continuity check that consumes it is also disabled in the default configuration, so the committed run prints an assertion about a delta it did not compute.

**Code evidence.**

```
config.rs:90-93: "/// The committed unpowered baseline of the verification harness (first run, 2026-07-17, L = 5 / cap 24 / 500 steps / terminal snapshot): the identity witness this study's matching-configuration baseline must land on.  pub const VERIFICATION_BASELINE: f64 = 1.584393e-2;"
main.rs:173-182: `if cfg.l == 5 && cfg.cap == 24 && cfg.steps == 500 { println!("  continuity vs committed verification baseline: delta {:.1e}", ...) } else { println!("  (committed-baseline identity holds at L=5/cap24/500 steps: measured delta 4.1e-9 on the first run)") }` — the default steps is 2000 (main.rs:104), so the else branch runs.
output.txt:13: `  (committed-baseline identity holds at L=5/cap24/500 steps: measured delta 4.1e-9 on the first run)` — a claim, not a measurement, in the committed artifact.
README.md:67-68: "This study supersedes the reverted `verification/srp_drag_decrement/` pinned-envelope harness".
```

**Reference form.** Cross-harness continuity is a legitimate check, but the reference must remain executable so the comparison can be re-derived rather than asserted from a literal.

**Impact.** Low, and the provenance is documented clearly, which is good practice. The residual issue is reproducibility: with the source harness reverted, no one can regenerate 1.584393e-2, and the default run reports the identity as an assertion. This is the one place in the corpus where a 'reference' value is unambiguously this code's own previous output.

**Recommended fix.** Either commit the 500-step configuration as a second short run so the delta is computed in the default artifact, or record the reverted harness's own output file alongside the constant so the number has a retrievable source.

**Adversarial check.** config.rs:90-93 is verbatim as quoted, declaring VERIFICATION_BASELINE = 1.584393e-2 as 'the committed unpowered baseline of the verification harness (first run, 2026-07-17...)'. main.rs:173-182 computes the delta only when l == 5 && cap == 24 && steps == 500; the default steps is 2000 (main.rs:104), so the else branch runs, and output.txt:13 records the printed assertion '(committed-baseline identity holds at L=5/cap24/500 steps: measured delta 4.1e-9 on the first run)' — a claim, not a measurement, in the committed artifact. README.md (Conclusion) confirms the source harness was reverted. The auditor's assessment is fair on both sides: the provenance is documented clearly and this is good practice, but with the reference harness out of the tree 1.584393e-2 cannot be regenerated, so the continuity check is asserted rather than re-derivable. Correctly rated info.

> Evidence re-read: srp_momentum_jet/config.rs:6-23, 90-93; main.rs:95-106, 161-183; output.txt:11-13; README.md (Conclusion, supersession note)

---
