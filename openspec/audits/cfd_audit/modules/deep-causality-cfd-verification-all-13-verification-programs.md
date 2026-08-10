# deep_causality_cfd/verification/ — all 13 verification programs + verification/README.md + per-directory READMEs

**Production readiness: `needs-work`**

The suite contains genuinely strong verification work — the Sod exact-Riemann solver (qtt_sod/exact_riemann.rs) is a faithful, independently-checkable implementation of Toro Ch. 4; the Ghia Re=1000 u-table is transcribed correctly; the Rankine-Hugoniot state at gamma=1.1 reproduces the closed-form normal-shock relations to 4 digits; the QTT Taylor-Green spatial error at every level matches the analytic dx^2/12 * 2*nu*t truncation budget to within 2%. But the suite's central advertised property is false: verification/README.md:15-16 states "Every example self-verifies and exits with a nonzero status the moment its invariant or reference check fails", and README.md:214 calls all thirteen "gated", yet dec_cylinder_verification, dec_graded_mms_verification and mms_taylor_green_verification contain no verification gate at all (grep for process::exit in dec_cylinder_verification/main.rs returns zero hits), and dec_cylinder_verification silently `break`s and exits 0 when the solver returns an error — the exact failure mode the README promises it catches. Beyond that, several gates cannot fail by construction: qtt_park2t_blackout gate (ii) asserts `ler_step(...) == x_eq - (x_eq-x)*exp(-dt/tau)`, a byte-for-byte restatement of blackout.rs:46; gate (iv)'s Saha check tests an explicit `return x_eq` at blackout.rs:42-44; qtt_reentry_3d's "body-fitted" field is written as a function of the z index only (main.rs:68-69), so its bounded rank is an algebraic identity, not a measurement of body-fitting. Bounds are repeatedly back-fitted from the code's own prior output, and two of them say so in the source (qtt_ramc_stagline/config.rs:53 "Pinned from the measurement recorded in baseline.txt"; dec_lid_cavity main.rs:117 "Gates from the pinning measurements"). Two physics defects are concrete and consequential: the Millikan-White reduced mass is 7 amu where N2-N2 requires 14 (config.rs:37), and qtt_park2t_blackout runs gamma=1.4 at M=25 while the crate's own sibling config (qtt_ramc_stagline/config.rs:16-19) documents that choice as over-predicting T2 by ~4x. None of this is unfixable — the harness architecture, the analytic references, and the exit-code plumbing that does exist are sound — but an avionics lab cannot today take "all gates PASS" as evidence the numbers are right.

- Files read: **64**
- Findings raised: **33** — surviving adversarial verification: **33** (refuted: 0)
- Surviving by severity: critical 1, major 9, minor 21, info 2
- Independently confirmed-correct items: **12**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Exact Riemann solver: pressure functions, Newton iteration, and self-similar sampling for all four wave configurations | `deep_causality_cfd/verification/qtt_sod/exact_riemann.rs:24-139` | Toro, Riemann Solvers and Numerical Methods for Fluid Dynamics, Ch. 4: f_K(p)=(p-p_K)[A_K/(p+B_K)]^{1/2} with A_K=2/((g+1)rho_K), B_K=((g-1)/(g+1))p_K for shocks; f_K(p)=(2a_K/(g-1))[(p/p_K)^{(g-1)/(2 |
| Taylor-Green nonlinear convection closed form u.grad(u)\|_u = -1/2 sin(2x) | `deep_causality_cfd/verification/qtt_taylor_green_verification/main.rs:138` | For u=-cos x sin y, v=sin x cos y: u du/dx + v du/dy = (-cos x sin y)(sin x sin y) + (sin x cos y)(-cos x cos y) = -sin x cos x (sin^2 y + cos^2 y) = -(1/2) sin 2x |
| Cartan magic formula analytic reference for the convective MMS | `deep_causality_cfd/verification/dec_graded_mms_verification/main.rs:216-226` | L_X omega = i_X d(omega) + d(i_X omega). For omega = sin(ky)dx + sin(kx)dy and X = cos(kx)d_x + cos(ky)d_y: x-component = k cos^2(ky) - k sin(kx)sin(ky); y-component = k cos^2(kx) - k sin(kx)sin(ky) |
| Ghia, Ghia & Shin (1982) Re=1000 u-velocity table along the vertical centerline (all 17 stations) | `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/config.rs:28-46` | Ghia et al. 1982, J. Comput. Phys. 48, Table I, Re=1000 column: y=0.9766->0.65928, 0.9688->0.57492, 0.9609->0.51117, 0.9531->0.46604, 0.8516->0.33304, 0.7344->0.18719, 0.6172->0.05702, 0.5000->-0.0608 |
| Ghia Re=1000 vortex-center coordinates (primary and both bottom corner eddies) | `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/config.rs:70-74` | Ghia et al. 1982 Table, Re=1000: primary (0.5313, 0.5625); BL1 (0.0859, 0.0781); BR1 (0.8594, 0.1094) — node-snapped to the 129x129 grid |
| Rankine-Hugoniot normal-shock post-shock state at gamma=1.1, M=25 | `deep_causality_cfd/verification/qtt_ramc_stagline/baseline.txt:20-23` | T2/T1 = [(2*g*M^2-(g-1))((g-1)M^2+2)]/((g+1)^2 M^2); rho2/rho1 = (g+1)M^2/((g-1)M^2+2); p2/p1 = (2*g*M^2-(g-1))/(g+1) (Anderson, Modern Compressible Flow, Ch. 3) |
| Williamson (1996) Strouhal reference at Re=100 | `deep_causality_cfd/verification/dec_cylinder_verification/main.rs:322` | Williamson's empirical fit St = -3.3265/Re + 0.1816 + 1.6e-4*Re; at Re=100: -0.033265 + 0.1816 + 0.016 = 0.16434 |
| MMS Taylor-Green kernel identity du/dt = nu*grad^2(u) = -2*nu*u and the sampled baseline values | `deep_causality_cfd/verification/mms_taylor_green_verification/baseline.txt:4-7` | For u = sin x cos y, grad^2 u = -2u, so with the convective and pressure terms cancelling, du/dt = -2*nu*u. At (x,y)=(0.7,1.1), nu=0.05: u = sin(0.7)cos(1.1) = 0.29223; v = -cos(0.7)sin(1.1) = -0.6816 |
| Cut-cell fluid-area bookkeeping in the cylinder-wake geometry | `deep_causality_cfd/verification/dec_cylinder_wake_verification/cli_output.txt:2` | Domain area AR*H = 3.0 minus the disk area pi*r^2 = pi*(0.125)^2 = 0.049087 -> 2.950913 |
| GPS L-band comms threshold expressed as an angular frequency | `deep_causality_cfd/verification/qtt_park2t_blackout/config.rs:52-53` | omega = 2*pi*f; f = 1.5 GHz -> omega = 9.4248e9 rad/s |
| QTT Taylor-Green finest-grid error is quantitatively explained by the second-order Laplacian eigenvalue error | `deep_causality_cfd/verification/qtt_taylor_green_verification/baseline.txt:6-8` | Centered-FD Laplacian eigenvalue for mode k=1 at spacing dx is -(4/dx^2)sin^2(dx/2) per axis; relative eigenvalue deficit = dx^2/12 + O(dx^4). Amplitude error after time t = exp(-2*nu*t)*(2*nu*t)*(dx^ |
| RAM-C II nose radius implied by the stated shock standoff | `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:30-32` | RAM-C II sphere-cone nose radius = 0.1524 m (6 inches); standoff/R ~ 0.05 for a strongly-compressed hypersonic bow shock |

## Findings

### 1.1 [CRITICAL] dec_cylinder_verification has no verification gate and exits 0 after a solver error, contradicting the README's explicit nonzero-exit promise

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/dec_cylinder_verification/main.rs:225`
- **Auditor confidence:** confirmed

**Claim.** The harness contains zero process::exit calls and zero assertions. When solver.step returns Err it prints a message, breaks the march loop, then proceeds to report Strouhal and drag from the truncated series and returns from main with exit status 0. verification/README.md:168-169 states the opposite.

**Code evidence.**

```
main.rs:223-229:
        let out = match solver.step(&state) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("# march stopped at step {step}: {e}");
                break;
            }
        };

main.rs:274-276 (end of main, no gate, no exit):
    report_strouhal(&probe_series, diameter, U);
    report_drag_mean(&drag_samples);
}

`grep -n "exit|panic|assert" main.rs` returns no matches.

verification/README.md:168-169: "**Self-check.** The march aborts (nonzero) if a physical invariant breaks (e.g. CFL violation, the solver returns an error)."
```

**Reference form.** The suite's own stated convention, verification/README.md:15-17: "Every example **self-verifies** and **exits with a nonzero status** the moment its invariant or reference check fails — so the suite is usable as a gate, not just a demo."

**Impact.** A CI pipeline or engineer using this harness as a gate gets a green exit code from a run that aborted at step 0 on a CFL violation, or from a run in which the drag/Strouhal were computed from a handful of samples of a diverging field. The reported St=0.171 and C_d=1.345 in the README summary table are produced by a program that never checks anything.

**Recommended fix.** Either (a) add explicit gates on St and C_d against the cited reference bands plus a hard `std::process::exit(1)` on the solver-error path instead of `break`, or (b) reclassify the program as a study and remove it from the 'thirteen gated programs' claim in README.md:214 and the self-check text at verification/README.md:168-169.

**Adversarial check.** Quoted code is verbatim correct. `solver.step` Err at main.rs:223-229 prints and `break`s; control falls to main.rs:274-275 (`report_strouhal`, `report_drag_mean`) and main returns normally, exit 0. `grep -n 'exit|panic!|assert'` over the file returns zero matches (the only aborts are setup-time `.expect()` calls, which cannot fire on a mid-march CFL/solver failure). verification/README.md line 167 states 'The march aborts (nonzero) if a physical invariant breaks (e.g. CFL violation, the solver returns an error)' — the opposite of the code. main.rs:96-97 even documents 'keep CFL <= 0.4 or the march aborts at step 0', which would produce an empty series and still exit 0.

> Evidence re-read: verification/dec_cylinder_verification/main.rs:222-276 (march loop, Err->break, end of main, no gate); verification/README.md:167-169 (nonzero-abort promise); grep for exit/panic!/assert over the file: no matches

---

### 1.2 [MINOR] qtt_park2t_blackout gate (ii) 'closed-form exponential exactness' is a byte-for-byte restatement of the implementation and cannot fail

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/print_utils.rs:38`
- **Auditor confidence:** confirmed

**Claim.** Gate (ii) compares ler_step against its own body, so it is a regression lock on the closed form, not an independent verification of the relaxation law; the gate label 'exactness' and the README's framing overstate what it demonstrates.

**Code evidence.**

```
print_utils.rs:36-39:
    let (x, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);
    ler_step(x, x_eq, tau, dt) == x_eq - (x_eq - x) * (-(dt / tau)).exp()

src/types/flow/blackout.rs:41-47:
pub fn ler_step<R: CfdScalar>(x: R, x_eq: R, tau: R, dt: R) -> R {
    if tau <= R::zero() { return x_eq; }
    x_eq - (x_eq - x) * (-(dt / tau)).exp()
}
```

**Reference form.** A non-circular exactness check would verify the closed form against the ODE it claims to solve: dx/dt = (x_eq - x)/tau, e.g. by comparing ler_step against a highly-refined RK4 integration of that ODE, or against the analytic solution derived independently of the implementation.

**Impact.** This gate is reported as PASS in baseline.txt:13 and counted in the verification README summary as one of 'all 6 PASS'. It provides zero evidence that the LER relaxation law is the right physics; it would still pass if the entire relaxation model were wrong, because it only checks that the function equals itself.

**Recommended fix.** Replace with a convergence check against a numerically-integrated dx/dt = (x_eq-x)/tau, asserting the closed form matches the integrator to O(dt^4) as dt is refined; or state plainly in the gate label that it is an implementation-identity regression check, not a physics verification.

**Adversarial check.** The code is exactly as quoted: print_utils.rs:36-39 asserts `ler_step(x,x_eq,tau,dt) == x_eq - (x_eq-x)*(-(dt/tau)).exp()` and blackout.rs:41-47 defines ler_step as that same expression (tau=0.01>0, so the early-return branch is not taken). The circularity claim is factually right: the 'reference' is the implementation re-typed, and the gate provides no evidence that the LER relaxation law is the correct physics. But the finding overstates severity: this is a legitimate (if narrow) regression lock — it fails the moment anyone replaces the closed form with a numerical integrator or perturbs the exponent, which is a real, if small, discriminating power. It is a mislabeled/weak gate, not a physics defect, and the harness is explicitly labeled Tier-A throughout. Not a certification blocker on its own; it is a claim-strength defect.

> Evidence re-read: verification/qtt_park2t_blackout/print_utils.rs:35-39 (gate ii); deep_causality_cfd/src/types/flow/blackout.rs:35-47 (ler_step body, tau<=0 early return); baseline.txt:13 ([PASS] (ii))

---

### 1.3 [MINOR] qtt_park2t_blackout gate (iv) 'lag + Saha limit' consists of three algebraic identities of the exponential and cannot fail

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/print_utils.rs:80`
- **Auditor confidence:** confirmed

**Claim.** Two of three conjuncts are vacuous (the lag inequality is implied by the closed form for any positive alpha_eq; the 'Saha limit' passes tau=0.0 and therefore exercises the tau<=0 early-return statement rather than a limit). The monotonicity conjunct is a weak but non-vacuous check that the Arrhenius rate is temperature-dependent. The README's 'tau -> 0 recovers Saha' claim is not demonstrated.

**Code evidence.**

```
print_utils.rs:76-80:
    let grounded = rate(9000.0) > rate(6000.0);
    let lagged = ler_step(0.0, alpha_eq, 0.01, 1.0e-5);
    let saha = ler_step(0.0, alpha_eq, 0.0, 1.0e-5);
    grounded && alpha_eq > 0.0 && lagged < alpha_eq && (saha - alpha_eq).abs() < 1e-12

src/types/flow/blackout.rs:42-45:
    if tau <= R::zero() {
        // tau -> 0: the increment jumps the state exactly to equilibrium.
        return x_eq;
    }
```

**Reference form.** A real Saha-limit check would verify that the finite-tau relaxation converges to the Saha equilibrium value as tau is driven to zero through a sequence of small positive taus, and that alpha_eq itself matches an independently computed Saha ionization fraction at the given T and n_tot.

**Impact.** The gate is presented in README.md (dir) line 41 as verifying 'the ionization lag is real, tau_ion varies with T (grounded, not a constant), and tau -> 0 recovers Saha'. It verifies none of those against physics; it exercises the tau<=0 early-return branch and the monotonicity of exp. Reported as PASS in baseline.txt:15.

**Recommended fix.** Test tau in {1e-3, 1e-4, 1e-5, ...} (all strictly positive) and assert monotone convergence of ler_step toward alpha_eq, and separately assert alpha_eq matches a hand-computed Saha value at T=8000 K, n=1e22 to a stated tolerance.

**Adversarial check.** Code at print_utils.rs:76-80 matches verbatim. Conjunct (b) `lagged < alpha_eq` is guaranteed for any alpha_eq>0 by the closed form — vacuous. Conjunct (c) `saha` passes `tau = 0.0`, which hits the `if tau <= R::zero() { return x_eq }` early return at blackout.rs:42-45, so `(saha - alpha_eq).abs()` is identically 0 — it exercises a hard-coded return, not a limit. Conjunct (a) is weaker than the auditor states but not strictly guaranteed 'by definition': arrhenius_rate_kernel (thermochemistry.rs:152) computes `C*T^eta*exp(-theta/T)` with PARK_NO_IONIZATION_EXPONENT = 0.5 and ACTIVATION_TEMP = 32400, so it is monotone increasing for any non-degenerate parameter set — but the gate would fail if the constants were changed to a temperature-independent rate (eta=0, theta=0), which is precisely the 'not a constant' property claimed. Also `alpha_eq > 0` genuinely exercises park2t_ionization_surrogate_kernel. So the gate is a weak regression check, not a fully closed tautology; the README's claim that it verifies 'tau -> 0 recovers Saha' is nonetheless unsupported — nothing converges, the tau<=0 branch is simply taken.

> Evidence re-read: verification/qtt_park2t_blackout/print_utils.rs:55-81; blackout.rs:41-47; deep_causality_physics/src/kernels/hypersonic/thermochemistry.rs:132-154; constants/hypersonic.rs:38-44 (prefactor 9.03e9, eta 0.5, theta 32400)

---

### 1.4 [INFO] qtt_reentry_3d's 'body-fitted' field is constructed as a function of the z index alone, so the bounded-rank gate is an algebraic identity rather than a measurement of body-fitting

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_reentry_3d/main.rs:68`
- **Auditor confidence:** confirmed

**Claim.** The 3-D fitted arm hardcodes the spherical radial map instead of routing through a map object (a code-consistency gap versus the 2-D sibling). It is not an algebraic identity: the field it builds is exactly what sampling the shell through a spherical body-fitted map yields, and the measured chi = 2 -> 4 is the real QTT rank of the radial step in the serial xyz codec.

**Code evidence.**

```
qtt_reentry_3d/main.rs:62-73:
fn forebody_fitted_bond(l: usize, w: f64) -> usize {
    ...
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let r = R0 + (iz as f64 / side as f64) * DR; // zeta -> physical radius
                field[(ix * side + iy) * side + iz] = smoothed_step(r - R_SHOCK, w);

Compare the 2-D sibling, which does go through a map:
qtt_blunt_body_2d/main.rs:64: let (x, y) = map.position(xi, eta);
```

**Reference form.** To demonstrate that a body-fitted coordinate bounds the rank, the fitted field must be produced by sampling the SAME physical function (the spherical shell of radius R_SHOCK) through an actual body-fitted map, exactly as the Cartesian arm samples it through the identity map. Only then is the coordinate the sole variable.

**Impact.** The gate RE-A ('body-fitting bounds the 3-D forebody sheath rank') and the printed conclusion at main.rs:206 are proven by construction, not measured. The Cartesian arm samples a genuine curved shell on [-2,2]^3 while the 'fitted' arm samples a synthetic 1-D profile — the comparison is not one-solver, one-variable. An engineer sizing a 3-D QTT re-entry solver on this evidence has no basis for expecting bounded rank from a real spherical metric.

**Recommended fix.** Build the fitted arm by sampling sqrt(x^2+y^2+z^2) - R_SHOCK through an explicit spherical map position(xi,eta,zeta), mirroring qtt_blunt_body_2d's BlendedMap usage, so the fitted and Cartesian arms differ only in the coordinate. If no 3-D body-fit metric exists yet (as main.rs:18-20 admits for the marcher), label the static result as an analytic upper bound rather than a gated measurement.

**Adversarial check.** The code is as quoted (main.rs:62-73): the fitted field depends only on `iz`, `ix`/`iy` never enter, and no BlendedMap is used, unlike the 2-D sibling (qtt_blunt_body_2d/main.rs:64). But the physics conclusion is wrong. In a body-fitted spherical coordinate the physical radius IS r = R0 + zeta*DR, independent of the two angular indices — that is the definition of the fitted map, not a synthetic substitute for it. The 2-D sibling reaches an algebraically identical field: at lambda=1 the polar fan gives sqrt(x^2+y^2) = R0 + eta*DR, a function of eta alone. So the 3-D arm samples the same physical shell through the (hardcoded) spherical map. Nor is the measured bond a trivial identity: the crate's serial x-y-z codec makes the bond across the x/y bit blocks 1, so the reported chi = 2 -> 4 is the genuine QTT rank of the radial tanh step profile — the quantity the lever claims. The real, and much smaller, defect is a code-consistency gap: the 3-D fitted arm bypasses the map object the 2-D sibling routes through, so a change in the map implementation would not be exercised.

> Evidence re-read: verification/qtt_reentry_3d/main.rs:60-92 (fitted and Cartesian arms), :162-176 (gates RE-A/RE-B), baseline.txt:5-7 (fitted 2/4/4, Cartesian 10/30/59); verification/qtt_blunt_body_2d/main.rs:55-87 (map.position path)

---

### 1.5 [MAJOR] dec_cylinder_wake's NaN guard on the divergence residual is unreachable — f64::max silently discards NaN, so a failed divergence diagnostic passes the gate

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_cylinder_wake_verification/main.rs:113`
- **Auditor confidence:** confirmed

**Claim.** When sv.divergence() returns Err, div is set to f64::NAN, but max_div = max_div.max(div.abs()) returns max_div because Rust's f64::max ignores NaN. Since max_div starts at 0.0 and is only ever updated through .max(), it can never become NaN, making the max_div.is_nan() branch dead code.

**Code evidence.**

```
main.rs:69: let mut max_div = 0.0f64;
main.rs:85-89:
                let div = match sv.divergence() {
                    Ok(v) => Into::<f64>::into(v),
                    Err(_) => f64::NAN,
                };
                max_div = max_div.max(div.abs());
main.rs:112-113:
    const DIV_TOL: f64 = 1e-6;
    if max_div.is_nan() || max_div >= DIV_TOL {

Empirically confirmed: compiling `let mut m=0.0f64; m = m.max(f64::NAN.abs()); println!("{m} {}", m.is_nan());` prints `0 false`.
```

**Reference form.** Rust std documentation for f64::max: "Returns the maximum of the two numbers, ignoring NaN. ... If one of the arguments is NaN, then the other argument is returned." To propagate NaN one must use a separate flag, or f64::maximum, or an explicit is_nan() test at the accumulation site.

**Impact.** If the constrained Leray projection's divergence diagnostic fails on every sampled step, max_div stays 0.0, the gate prints 'verified: incompressibility held (max div 0.000e0)' and the program exits 0. The gate advertised at verification/README.md:147-148 as catching a broken projector silently reports success on a total diagnostic failure.

**Recommended fix.** Track a separate `let mut div_failed = false;` set in the Err arm, and gate on `div_failed || max_div >= DIV_TOL`. Alternatively accumulate with an explicit `if div.is_nan() { div_failed = true } else { max_div = max_div.max(div.abs()) }`.

**Adversarial check.** All three cited lines are verbatim correct: main.rs:69 `let mut max_div = 0.0f64;`, main.rs:85-89 (Err -> f64::NAN, then `max_div = max_div.max(div.abs())`), main.rs:112-113 (`if max_div.is_nan() || max_div >= DIV_TOL`). Rust's `f64::max` is documented to ignore NaN and return the other operand, so `0.0f64.max(NAN)` is 0.0 and max_div can never become NaN. The `is_nan()` disjunct is therefore dead code and a total failure of `sv.divergence()` would print 'verified: incompressibility held (max div 0.000e0)' and exit 0. The reference form is correct (a separate flag, `f64::maximum`, or an is_nan test at the accumulation site). Mitigating but not exculpating: the per-step NaN would appear in the CSV `div_residual` column, so the failure is visible to a human reader — but not to the gate, which is what the README at :147-148 advertises.

> Evidence re-read: verification/dec_cylinder_wake_verification/main.rs:69, :85-89, :111-118, :130-132; verification/README.md:147-148

---

### 1.6 [MAJOR] The suite-wide convention 'every example self-verifies and exits nonzero' is false for 3 of 13 programs

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/README.md:15`
- **Auditor confidence:** confirmed

**Claim.** mms_taylor_green_verification, dec_graded_mms_verification and dec_cylinder_verification contain no verification gate. mms's single process::exit is in the fail() helper for pipeline errors only; dec_graded_mms and dec_cylinder contain no process::exit at all. dec_lid_cavity is gated only in its non-default `trend` subcommand.

**Code evidence.**

```
verification/README.md:15-17: "Every example **self-verifies** and **exits with a nonzero status** the moment its invariant or reference check fails — so the suite is usable as a gate, not just a demo."

README.md:214: "`verification/` holds thirteen runnable programs gated against analytic solutions, published references, or internal invariants."

mms_taylor_green_verification/main.rs:36-41 — runs CfdFlow::verify then only `print_utils::render(&report);`. No comparison of kernel_err against any bound.

dec_graded_mms_verification/main.rs:50-88 — main() prints two tables and ends; no assertion, no exit.

dec_cylinder_verification/main.rs — grep for process::exit returns 0 matches.

The mms README itself concedes it at verification/README.md:68-69: "(The example prints the residual; treat a residual far above eps as a regression.)"
```

**Reference form.** The convention stated in the same document at verification/README.md:15-17, and the crate README's word 'gated' at README.md:214.

**Impact.** An avionics lab wiring `cargo run --example <name>` into a certification pipeline will get exit 0 from three programs regardless of what they compute. The mms residual could regress from 1.1e-16 to 1.0 and the harness would print it and succeed.

**Recommended fix.** Add real gates: for mms, assert kernel_err and amplitude error are below a precision-scaled bound (e.g. 100*eps of FloatType, which is derivable rather than pinned); for dec_graded_mms, assert every finest-pair observed order exceeds 1.8; for dec_cylinder, gate St and C_d against the cited bands. Until then, amend verification/README.md:15-17 and README.md:214 to name the ungated programs explicitly.

**Adversarial check.** Verified each: mms_taylor_green_verification/main.rs:30-41 runs CfdFlow::verify then only `print_utils::render(&report)`; its single `std::process::exit(1)` is inside `fail()` (main.rs:43-47), reached only on a config/pipeline Err — no residual is compared to any bound. dec_graded_mms_verification/main.rs:50-88 prints two tables and ends; grep for exit/assert returns nothing. dec_cylinder_verification has no exit at all. dec_lid_cavity_re1000_verification gates only in the `trend` subcommand (main.rs:52-56 dispatch; default path is `render`). verification/README.md:15-17 and README.md:214 ('thirteen runnable programs gated') assert otherwise. The mms section at README.md:68-69 concedes it in prose ('treat a residual far above eps as a regression'), which confirms rather than excuses the gap.

> Evidence re-read: mms_taylor_green_verification/main.rs:30-47; dec_graded_mms_verification/main.rs:50-88; dec_cylinder_verification/main.rs (grep: no exit/assert); dec_lid_cavity_re1000_verification/main.rs:50-56 + :105-130; verification/README.md:15-17, :68-69

---

### 1.7 [MAJOR] Millikan-White reduced mass is 7 amu where N2-N2 requires 14 — a factor-of-2 error in a parameter that directly sets the calibrated electron density

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:37`
- **Auditor confidence:** confirmed

**Claim.** The reduced mass of an N2-N2 collision pair is mu = 28.0134*28.0134/(28.0134+28.0134) = 14.007 amu. The constant is set to 7.0, and the doc comment's own arithmetic '14*14/28 = 7' uses the atomic mass of N (14) rather than the molecular mass of N2 (28) — i.e. it computes the reduced mass of an N-N atom pair while labelling it N2-N2.

**Code evidence.**

```
qtt_ramc_stagline/config.rs:36-37:
/// Reduced mass `mu_sr` of the dominant relaxing collision pair (N2-N2 ~ 14*14/28 = 7), in amu — sets the
/// Millikan-White vibrational relaxation time `tau_vt` that controls how far the lagging `T_ve` catches up.
pub const REDUCED_MASS_AMU: f64 = 7.0;

deep_causality_physics/src/kernels/hypersonic/thermochemistry.rs:98-99:
    let a_sr = a * reduced_mass_amu.powf(half) * theta_vib.powf(four_thirds);
    let exponent = a_sr * (t.powf(neg_third) - b * reduced_mass_amu.powf(quarter)) - c;
```

**Reference form.** Millikan & White (1963), J. Chem. Phys. 39, 3209: tau_vt * p = exp[A_sr(T^{-1/3} - 0.015 mu^{1/4}) - 18.42] with A_sr = 1.16e-3 * mu^{1/2} * theta_v^{4/3}, where mu is the reduced mass of the colliding pair in amu. For N2-N2, mu = m_N2/2 = 14.007 amu (standard value in Park, Nonequilibrium Hypersonic Aerothermodynamics, Table 2.2).

**Impact.** mu enters as mu^{1/2} in A_sr and mu^{1/4} in the exponent. Using 7 instead of 14 scales A_sr by 1/sqrt(2) = 0.707, changing tau_vt by a large factor. tau_vt sets T_ve, which sets T_a = sqrt(T_tr*T_ve), which sets the ionization rate and hence the reported peak n_e = 1.085e19 that gate g2 passes against the RAM-C anchor. print_utils.rs:92-93 states explicitly that 'the exact landing is sensitive to the Millikan-White tau_vt model', so this is not a benign parameter.

**Recommended fix.** Set REDUCED_MASS_AMU = 14.0 (= 28.0134/2) and correct the comment to '(N2-N2: 28*28/56 = 14)'. Re-run and report whether gate g2 still passes; if it only passed at mu=7, that is itself a finding about the calibration.

**Adversarial check.** config.rs:35-37 is verbatim as quoted, including the doc comment's own arithmetic 'N2-N2 ~ 14*14/28 = 7' — which is the reduced mass of an N-N atom pair, not an N2-N2 molecular pair. The correct value is mu = 28.0134/2 = 14.007 amu (Millikan & White 1963; Park 1990 Table 2.2). I re-derived the auditor's reference form and it matches the kernel exactly: thermochemistry.rs:20-25 documents tau_vt*P = exp[A_sr(T^-1/3 - B mu^1/4) - C] with A_sr = a mu^1/2 theta_v^4/3, a = 1.16e-3, B = 0.015, C = 18.42 (constants/hypersonic.rs:61-67), implemented at lines 98-99. The value is consumed at qtt_ramc_stagline/main.rs:83. Quantified impact: at theta_v = 3395 K and T = 8044 K the exponent goes from -14.43 (mu=7) to -13.79 (mu=14), i.e. tau_vt roughly doubles — a first-order change to how far the lagging T_ve catches up, hence to T_a = sqrt(T_tr*T_ve), the ionization rate, and the reported peak n_e. print_utils.rs:92-93 states the landing is sensitive to exactly this model. Note the compounding concern: the README declares this path 'calibrated ... tuned to the anchor', so a wrong mu may have been absorbed by the calibration.

> Evidence re-read: verification/qtt_ramc_stagline/config.rs:34-37; main.rs:83; deep_causality_physics/src/kernels/hypersonic/thermochemistry.rs:20-44 (doc) and :98-100 (A_sr, exponent); constants/hypersonic.rs:61-67

---

### 1.8 [MINOR] qtt_ramc_stagline's network acceptance band is explicitly back-fitted from the code's own recorded output

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:54`
- **Auditor confidence:** confirmed

**Claim.** The band's provenance is documented circularly ('pinned from the measurement'), which is wrong for a V&V artifact — but its value 0.70 dec equals log10(5), exactly the 2x-5x rate-set spread cited in the same comment, so the bound is defensible a priori and is not numerically derived from the +0.48 dec measurement it gates. The defect is the wording, not the number.

**Code evidence.**

```
qtt_ramc_stagline/config.rs:51-54:
/// Acceptance band of the uncalibrated finite-rate network prediction, in decades around the
/// flight anchor. Pinned from the measurement recorded in baseline.txt; the production-code
/// context (DPLR/LAURA/US3D at 2x to 3x, rate-set spread 2x to 5x) justifies the width.
pub const NETWORK_BAND_DECADES: f64 = 0.7;

qtt_ramc_stagline/print_utils.rs:99-101:
/// Gate the uncalibrated network prediction. The band is pinned from the
/// measurement (see `baseline.txt`), justified against the production-code
/// context: ...

baseline.txt:38: "[PASS] network prediction inside the earned band: full network +0.48 dec vs the flight anchor (band +-0.70 dec, pinned from the measurement...)"
```

**Reference form.** A non-circular band is fixed before the measurement from an external source: e.g. the stated 2x-3x production-code spread alone gives +-0.48 dec, and the 2x-5x rate-set spread gives +-0.70 dec. Either could have been chosen a priori and cited to a specific table in Aiken-Carter-Boyd 2025 or the RP-1232 rate uncertainties.

**Impact.** The gate the crate README describes as validating 'an uncalibrated finite-rate ionization network against RAM-C II flight data' (README.md:218) uses a bound derived from the very run it evaluates. It cannot fail on the run it was pinned from, and its discriminating power on a future run is unknown because the bound was not chosen independently.

**Recommended fix.** Derive the band from a cited external source only — pick the 2x-5x rate-set spread, cite the specific table, and state that the band was fixed before measurement. Remove 'Pinned from the measurement recorded in baseline.txt' from both the constant's doc and the gate message, or keep it and reclassify the gate as a regression pin rather than a validation.

**Adversarial check.** The quoted source is exact: config.rs:51-54 says 'Pinned from the measurement recorded in baseline.txt', print_utils.rs:99-101 repeats it, and baseline.txt:38 prints 'pinned from the measurement'. So the self-description is circular and that wording is a genuine defect in a V&V artifact. But the numeric value refutes the back-fitting inference: log10(5) = 0.699 ~ 0.70, i.e. the band is exactly the 2x-5x rate-set spread the same comment cites as its external justification — the auditor's own reference form concedes this ('the 2x-5x rate-set spread gives +-0.70 dec ... could have been chosen a priori'). The measured +0.48 dec sits well inside it and does not determine it; a band back-fitted to +0.48 would have been ~0.5, not 0.70. This is a documentation-provenance defect, not a fabricated bound.

> Evidence re-read: verification/qtt_ramc_stagline/config.rs:51-54 and :56-63 (CARRIED_ARM_BAND_DECADES = NETWORK_BAND_DECADES); print_utils.rs:99-124 (verify_network); baseline.txt:38 ([PASS] +0.48 dec, band +-0.70)

---

### 1.9 [MINOR] qtt_ramc_stagline gate g2 validates against the same anchor the model's controlling temperature was tuned to

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/print_utils.rs:64`
- **Auditor confidence:** confirmed

**Claim.** Gate g2 is a calibration-consistency check rather than an independent validation, and should be labeled as such in a certification package. The crate already labels it 'calibrated' in both the directory README and the summary table and gates the uncalibrated network separately, so the doc-overclaim component does not hold.

**Code evidence.**

```
print_utils.rs:64-67:
    let g2 = gate(
        "peak n_e within ~3x of RAM-C II (Park-2T controller)",
        out.electron_density > NE_LO && out.electron_density < NE_HI,
    );

qtt_ramc_stagline/README.md (Calibration section): "The Park-2T controller path is the *calibrated* closure (its geometric-mean controlling temperature was tuned to the anchor; it lands ~1.1x)."

config.rs:16-20: "the engineering effective value for strongly-dissociated hypersonic air is `~1.1-1.2`, which lands `T2` in the realistic ~8000 K band where RAM-C ionizes"
pub const GAMMA: f64 = 1.1;
```

**Reference form.** A validation gate's reference must be independent of the model's fitted parameters. Standard practice (AIAA G-077 / ASME V&V 20) requires validation data to be withheld from calibration; a calibrated closure should be gated against a different flight station, a different altitude, or a held-out dataset.

**Impact.** The verification README summary row reports 'Measured 1.085e19 (calibrated Park-2T) | Reference ~1e19 (RAM-C II) | Divergence +0.0 dec calibrated'. Presenting +0.0 dec agreement with a target the closure was tuned toward, inside a table headed 'Reference', invites an engineer to read it as predictive accuracy. The gate's PASS carries no information about the model's predictive power.

**Recommended fix.** Gate the calibrated path only as a regression pin, and label it so in the gate text. Reserve validation gating for the uncalibrated network path (which the README already correctly separates), and validate against a RAM-C II station other than the one used to select gamma and the controlling temperature.

**Adversarial check.** The substance is confirmed and the quotations are exact: print_utils.rs:64-67 gates the Park-2T electron density inside (3e18, 3e19) against the 1e19 RAM-C II anchor, and qtt_ramc_stagline/README.md:67-68 states 'The Park-2T controller path is the *calibrated* closure (its geometric-mean controlling temperature was tuned to the anchor; it lands ~1.1x).' Under AIAA G-077 / ASME V&V 20 that makes g2 a calibration consistency check, not a validation — a fair certification observation. Two corrections. (a) The overclaim half of the finding is refuted: the harness does not present g2 as predictive. The dir README section is titled 'Calibration and prediction, kept separate', the summary row in verification/README.md:41 literally reads '1.085e19 (calibrated Park-2T)' and '+0.0 dec calibrated', and the separately-gated uncalibrated network (+0.48 dec) is the prediction arm. (b) The claim that GAMMA=1.1 was chosen to reproduce the flight n_e is a mischaracterization: config.rs:16-20 justifies it as the engineering effective gamma for strongly-dissociated air landing T2 in the ~8000 K band — a thermodynamic argument independent of n_e.

> Evidence re-read: verification/qtt_ramc_stagline/print_utils.rs:54-71 (g1-g4), :16-21 (NE_LO/NE_HI rationale); qtt_ramc_stagline/README.md:19-31, :67-71; config.rs:16-20; verification/README.md:41

---

### 1.10 [MINOR] qtt_ramc_stagline gate g3 (blackout onset) is logically entailed by gate g2 and can never fail independently

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/print_utils.rs:68`
- **Auditor confidence:** confirmed

**Claim.** g3 is logically entailed by g2 (n_e > 3e18 forces omega_p > 9.8e10, ten times the 9.4e9 band) and adds no independent constraint; the suite should not count it as separate evidence. It is a reporting redundancy, not a broken gate.

**Code evidence.**

```
print_utils.rs:20-21:
const NE_LO: f64 = 3.0e18;
const NE_HI: f64 = 3.0e19;
print_utils.rs:64-68:
    let g2 = gate("peak n_e within ~3x of RAM-C II (Park-2T controller)",
        out.electron_density > NE_LO && out.electron_density < NE_HI);
    let g3 = gate("blackout onset (omega_p > comms band)", out.blackout);

config.rs:27: pub const COMMS_BAND_RAD_S: f64 = 9.4e9;
baseline.txt:29: "plasma frequency omega_p ... 1.858e11 rad/s"
```

**Reference form.** Plasma frequency: f_p [Hz] = 8.98*sqrt(n_e [m^-3]); omega_p = 2*pi*f_p. Setting omega_p = 9.4e9 gives f_p = 1.496e9 Hz and n_e = (1.496e9/8.98)^2 = 2.78e16 m^-3. Measured omega_p = 1.858e11 is 19.8x the comms band.

**Impact.** The suite counts four independent gates for this harness; one of them carries no information. The verification README's framing of 'peak electron density / blackout onset' as two verified quantities overstates the independent evidence by one gate.

**Recommended fix.** Either drop g3, or make it non-redundant by gating the blackout *dwell duration* or the *onset altitude/time* against the Apollo or RAM-C corridor-time anchor the README cites, which would be a genuinely separate observable.

**Adversarial check.** The entailment is real and I re-derived it independently. blackout.rs:502-514 computes omega_p = sqrt(n_e e^2/(eps0 m_e)) and sets denied = omega_p > comms_band, so out.blackout is a pure function of n_e. Inverting at COMMS_BAND_RAD_S = 9.4e9 gives n_e = eps0 m_e omega^2/e^2 = 2.78e16 m^-3, matching the auditor's f_p = 8.98 sqrt(n_e) derivation. NE_LO = 3e18 is 108x that, so g2 => g3 with no possible input in between. Severity is overstated, though: a redundant gate is not a defect in the gate, and the redundancy is structural (blackout is definitionally a threshold on the same quantity g2 bounds). It costs one line in the count of 'independent evidence', which is a reporting nit, not a certification blocker.

> Evidence re-read: verification/qtt_ramc_stagline/print_utils.rs:20-21 (NE_LO/NE_HI), :64-68 (g2, g3); config.rs:26-27 (COMMS_BAND_RAD_S = 9.4e9); deep_causality_cfd/src/types/flow/blackout.rs:502-514 (evaluate: denied = omega_p > band); deep_causality_physics/src/kernels/mhd/plasma.rs:105-127 (omega_p kernel); baseline.txt:29 (omega_p 1.858e11)

---

### 1.11 [MINOR] qtt_park2t_blackout uses gamma=1.4 at M=25, which the crate's own sibling configuration documents as over-predicting the post-shock temperature by roughly 4x

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/config.rs:43`
- **Auditor confidence:** confirmed

**Claim.** gamma = 1.4 at M = 25 yields T2 ~ 30 600 K, inconsistent with the sibling harness's documented effective gamma of 1.1-1.2 for the same flight condition, and drives the Saha surrogate to full ionization (n_e = n_tot = 1e22). The claim that gate (iii)'s (1e4, 1e5) K band would also accept the correct gamma is wrong: gamma = 1.1 gives 8044 K, below the band's floor, so the gate would fail. The over-prediction is disclosed in the directory README and baseline notes as a Tier-A limitation.

**Code evidence.**

```
qtt_park2t_blackout/config.rs:42-43:
/// Ratio of specific heats.
pub const GAMMA: f64 = 1.4;

qtt_park2t_blackout/print_utils.rs:47-49:
    match rankine_hugoniot_temperature_kernel(t_inf, config::MACH, config::GAMMA) {
        Ok(t_post) => t_post.value() > 1.0e4 && t_post.value() < 1.0e5,

qtt_ramc_stagline/config.rs:16-20 (same crate, same flight condition):
/// **Effective** post-shock ratio of specific heats for reacting air. Perfect-gas `1.4` over-predicts
/// `T2` badly (~30 000 K) because it ignores the dissociation/vibration that absorb the post-shock energy;
/// the engineering effective value for strongly-dissociated hypersonic air is `~1.1-1.2` ...
pub const GAMMA: f64 = 1.1;
```

**Reference form.** Normal-shock relation T2/T1 = [(2*g*M^2-(g-1))((g-1)M^2+2)]/((g+1)^2 M^2). At g=1.4, M=25: (1749.6*252)/3600 = 122.47, T2 = 30,618 K. At g=1.1: 32.174, T2 = 8,044 K. Real reacting-air stagnation temperatures at M=25 / 7.65 km/s are ~8,000-11,000 K (Anderson, Hypersonic and High-Temperature Gas Dynamics, Ch. 14).

**Impact.** The over-predicted 30,600 K drives the Saha surrogate to full ionization: baseline.txt:6 reports peak n_e = 1.000e22 m^-3, exactly n_tot, i.e. alpha = 1. That is three decades above the RAM-C II 1e19 anchor the same harness cites. The gate band (1e4, 1e5) K is wide enough that switching to the physically correct gamma=1.1 would also pass, so the gate cannot distinguish correct from incorrect thermodynamics.

**Recommended fix.** Use the effective gamma (1.1-1.2) consistently across both harnesses, or add the same explicit effective-gamma caveat to qtt_park2t_blackout/config.rs:42-43. Narrow the gate band to bracket only the reacting-air value (e.g. 6,000-14,000 K) so it actually discriminates.

**Adversarial check.** The code and the sibling contradiction are confirmed: qtt_park2t_blackout/config.rs:42-43 sets GAMMA = 1.4, while qtt_ramc_stagline/config.rs:16-20 states in the same crate that 1.4 over-predicts T2 badly (~30 000 K) and uses 1.1. I re-derived T2/T1 = [(2gM^2-(g-1))((g-1)M^2+2)]/((g+1)^2 M^2): at g=1.4, M=25 this is 122.47 -> T2 = 30 618 K; at g=1.1 it is 32.18 -> T2 = 8044 K (the ramc baseline prints exactly 8044 K, confirming the formula). The impact claim is REFUTED on its key point: gate (iii) requires t_post in (1.0e4, 1.0e5), and 8044 K is BELOW the 1e4 floor — so switching to the physically correct gamma=1.1 would make the gate FAIL, not pass. The band does discriminate; it discriminates in the wrong direction. Mitigating: the over-prediction and the resulting near-full ionization are disclosed prominently in qtt_park2t_blackout/README.md:47-52 and baseline.txt:24-28, and the whole harness is scoped Tier-A with the reconstruction retired by the Tier-B sibling.

> Evidence re-read: verification/qtt_park2t_blackout/config.rs:40-45, print_utils.rs:41-51 (band 1e4-1e5); verification/qtt_ramc_stagline/config.rs:16-20; qtt_ramc_stagline/baseline.txt:20 (T2 = 8044 K at gamma 1.1); qtt_park2t_blackout/README.md:45-53; baseline.txt:6, :21-28

---

### 1.12 [MINOR] qtt_park2t_blackout reports peak n_e three decades above its own flight anchor while the verification summary presents it as verified

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/README.md:39`
- **Auditor confidence:** confirmed

**Claim.** The harness produces n_e = 1.000e22 m^-3 against the RAMC_NE_REFERENCE of 1.0e19 it hard-codes — a factor of 1000. The verification README's summary row reports only 'all 6 PASS' and 'omega_p 5.6e12 >> band' and 'Gap-2 Tier-A verified', never surfacing the three-decade gap. The dir README does disclose it, so this is a summary-table defect.

**Code evidence.**

```
verification/README.md:39:
| `qtt_park2t_blackout` | 6 LER gates (stability, exactness, RH band, lag+Saha, path-dependence, n_e>0) | all 6 PASS; omega_p 5.6e12 >> band | all gates pass | Gap-2 Tier-A verified (cross-refs, Tier-A disclaimers) | 32^2, 40 steps | ~1 s |

qtt_park2t_blackout/baseline.txt:6: "peak electron density n_e   : 1.000e22 m^-3"
qtt_park2t_blackout/config.rs:56-57: "/// RAM-C II peak electron density band near the 71 km station, m^-3 (order-of-magnitude anchor).\npub const RAMC_NE_REFERENCE: f64 = 1.0e19;"

(The dir README/README.md:49-52 does state the over-prediction; the summary table does not.)
```

**Reference form.** The summary table's own column headings are 'Measured | Reference | Divergence'. For every other row the Divergence column carries the numeric gap (e.g. '-80 %', '+4.3 %', '+0.48 dec'). For this row it carries the phrase 'Gap-2 Tier-A verified' instead of the 3-decade divergence.

**Impact.** A reader scanning the summary table — the document's headline artifact — sees six passing gates and no divergence figure, while the harness's only physically meaningful output is 1000x the flight measurement. Since none of the six gates reads that number against the anchor (gate (vi) only checks n_e > 0), nothing in the program flags it.

**Recommended fix.** Put the actual divergence in the Divergence column: '+3.0 dec vs RAM-C II (Saha saturates at alpha=1; see Tier-A disclaimers)'. Consider adding a seventh gate that asserts n_e is within a stated band of the anchor, or that explicitly asserts alpha < 1 so saturation is caught rather than reported.

**Adversarial check.** Both halves check out. verification/README.md:39 is verbatim as quoted: Measured column carries 'all 6 PASS; omega_p 5.6e12 >> band', Divergence column carries 'Gap-2 Tier-A verified (cross-refs, Tier-A disclaimers)' — no numeric divergence, and the peak n_e does not appear in the row at all. baseline.txt:6 reports n_e = 1.000e22 against config.rs:56-57's RAMC_NE_REFERENCE = 1.0e19, a factor of 1000. I confirmed no gate reads n_e against the anchor: gate (vi) at print_utils.rs:100-105 only tests `any(|x| x > 0.0)`. The auditor correctly notes the directory README (:47-52) and baseline notes (:24-28) do disclose it, so this is specifically a summary-table defect. Severity moderated because the row is explicitly labeled Tier-A and every other layer discloses.

> Evidence re-read: verification/README.md:39 (summary row, Divergence column); qtt_park2t_blackout/baseline.txt:6, :21-28; config.rs:55-57; print_utils.rs:99-105 (gate vi); qtt_park2t_blackout/README.md:45-53

---

### 1.13 [MAJOR] The Cartesian capture bond grows linearly in side, not as sqrt(side) — the scaling law is asserted in four places and contradicted by the crate's own baseline data

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/qtt_blunt_body_2d/main.rs:158`
- **Auditor confidence:** confirmed

**Claim.** Measured capture chi = 16, 32, 61 at sides 32, 64, 128. chi/side = 0.500, 0.500, 0.477 — constant, i.e. chi is proportional to side. chi/sqrt(side) = 2.83, 4.00, 5.39 — growing by 1.9x, so the sqrt law is wrong by a square.

**Code evidence.**

```
qtt_blunt_body_2d/baseline.txt:5-7:
     2^5    |         3       |       16
     2^6    |         4       |       32
     2^7    |         5       |       61

The sqrt claim appears in four places:
qtt_blunt_body_2d/main.rs:13 (module doc): "so `chi` grows with resolution (`chi ~ sqrt(side)`, the measured capture cost)"
qtt_blunt_body_2d/main.rs:158 (printed at runtime): "(chi {} -> {}, growing ~sqrt(side)). Body-fittedness buys the bond reduction"
qtt_blunt_body_2d/README.md:29: "growing by at most 1 per refinement, so no `sqrt(side)` growth."
qtt_blunt_body_2d/README.md:44: "the capture cost runs 16 to 61, growing roughly as `sqrt(side)`."
verification/README.md:42: "structural (no sqrt(side) growth, fitted) | fitted bounded; capture ~sqrt(side)"
```

**Reference form.** A power law chi = C*side^p is identified by p = log(chi_2/chi_1)/log(side_2/side_1). From 2^5 to 2^7: log(61/16)/log(4) = log(3.81)/1.386 = 1.339/1.386 = 0.966, i.e. p ~ 1 (linear), not p = 0.5.

**Impact.** The rank lever is the business case for body-fitted QTT. Extrapolating from 2^7 to 2^10 under the claimed sqrt law predicts chi ~ 61*sqrt(8) = 173; under the measured linear law it predicts chi ~ 488, and cost scales at least as chi^2-chi^3 in TT algorithms. An engineer sizing a production run from this README would under-budget by roughly an order of magnitude in memory and far more in time.

**Recommended fix.** Replace 'growing ~sqrt(side)' with the measured scaling ('growing ~linearly in side; chi ~ side/2 over 2^5-2^7') in all four locations, or extend the ladder to 2^8-2^9 and fit the exponent explicitly. Note the same issue in qtt_reentry_3d where cart = 10, 30, 59 over sides 8, 16, 32 is also super-sqrt.

**Adversarial check.** Data and citations verified. baseline.txt:5-7 gives capture chi = 16, 32, 61 at side = 32, 64, 128 (side is defined in code as `1usize << l`, the linear dimension: main.rs:56, :77). chi/side = 0.500, 0.500, 0.477 — constant; the fitted exponent p = log(61/16)/log(4) = 0.966, i.e. linear, not 0.5. The sqrt assertion appears where claimed: main.rs:12 (module doc), main.rs:158 (printed at runtime), qtt_blunt_body_2d/README.md:29 and :44, verification/README.md:42. Only README.md:29 is a different (and correct) statement about the *fitted* arm. The likely origin is a units slip — chi ~ sqrt(N_total) with N_total = side^2 is the standard QTT result for a curved codimension-1 front and equals chi ~ side, matching the data exactly — but as written against the code's own `side` the documentation is off by a square, and the extrapolation impact the auditor describes (chi(2^10) ~ 173 claimed vs ~488 measured-law) follows.

> Evidence re-read: verification/qtt_blunt_body_2d/baseline.txt:5-7; main.rs:12, :56, :77, :157-160 (printed 'growing ~sqrt(side)'); qtt_blunt_body_2d/README.md:29, :44; verification/README.md:42

---

### 1.14 [MINOR] qtt_blunt_body_2d gate BB-A's stability tolerance equals the measured growth rate exactly, leaving zero headroom, and the accompanying 'flat / resolution-independent' claim contradicts the data

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_blunt_body_2d/main.rs:126`
- **Auditor confidence:** confirmed

**Claim.** fitted_stable requires x[1] <= x[0] + 1. Measured fitted chi = 3, 4, 5 grows by exactly 1 at each refinement, so both pairs sit precisely at the boundary. The bound was set to the measured value with no margin. Simultaneously, chi growing +1 per doubling is logarithmic in side, not 'flat' or 'resolution-independent' as the module doc, the printed reading, and the README all state.

**Code evidence.**

```
main.rs:125-131:
    let fitted_max = *fitted.iter().max().unwrap();
    let fitted_stable = fitted.windows(2).all(|x| x[1] <= x[0] + 1);
    if fitted_max > 12 || !fitted_stable {
        failures.push(format!("BB-A: fitted chi not bounded/resolution-stable (bonds {fitted:?})"));

baseline.txt:5-7 shows fitted 3 -> 4 -> 5 (delta = +1, +1).

main.rs:11 (module doc): "its quantized-tensor-train bond `chi` is `O(10)` and **resolution-independent**"
main.rs:154 (printed): "in the fitted coordinate (chi {} -> {}, flat)"
qtt_blunt_body_2d/README.md:44: "Fitted runs 3 to 5 and is flat"

The 3-D sibling has the identical pattern: qtt_reentry_3d/main.rs:165 `let tail_flat = fitted[2] <= fitted[1] + 1;` with measured [2, 4, 4] — and it only checks the TAIL pair, excluding the 2 -> 4 doubling at the first refinement while still printing "chi 2 -> 4, flat" (main.rs:187, baseline.txt:11).
```

**Reference form.** A resolution-independent bond satisfies chi(2L) = chi(L) exactly. A bound intended to detect regression must sit strictly above the measured behaviour with stated margin, e.g. 'chi grows by at most 1 per doubling and chi(2^7) <= 8', derived from the O(10) target rather than from the observed 3,4,5.

**Impact.** Any change increasing the fitted bond growth from +1 to +2 per refinement fails a gate whose threshold was read off the passing run, and the gate provides no evidence about the asymptotic behaviour it is claimed to demonstrate. Meanwhile readers are told the bond is resolution-independent when it is measurably growing.

**Recommended fix.** State the growth honestly ('chi grows by 1 per doubling over 2^5-2^7 — logarithmic, not constant'), derive the gate threshold from the O(10) design target with explicit margin, and extend the ladder to 2^8/2^9 to establish whether the growth plateaus. For qtt_reentry_3d, gate all pairs rather than only the tail, or justify in code why the first pair is excluded.

**Adversarial check.** Both halves verified verbatim. main.rs:125-131 matches, including `fitted.windows(2).all(|x| x[1] <= x[0] + 1)`; baseline.txt:5-7 shows fitted 3 -> 4 -> 5, i.e. +1 at both refinements, exactly on the boundary. The prose contradiction is real: main.rs:11 says 'resolution-independent', main.rs:154 prints 'flat', README.md:44 says 'is flat' — but +1 per doubling is chi ~ log2(side), measurably growing. The 3-D sibling replicates the pattern: main.rs:165 `let tail_flat = fitted[2] <= fitted[1] + 1;` checks only the tail pair, excluding the 2 -> 4 jump at the first refinement, while main.rs:187 prints 'chi 2 -> 4, flat' (baseline.txt:11). One qualification in the code's favor: BB-A also carries an absolute cap `fitted_max > 12` with real headroom (measured 5), so the gate is not wholly margin-free — the zero-headroom criticism applies to the growth-rate conjunct only.

> Evidence re-read: verification/qtt_blunt_body_2d/main.rs:11, :124-131, :150-160; baseline.txt:5-7, :11; qtt_blunt_body_2d/README.md:28-29, :44; verification/qtt_reentry_3d/main.rs:162-170, :186-189; qtt_reentry_3d/baseline.txt:5-7, :11

---

### 1.15 [MAJOR] qtt_taylor_green_verification's pinned error and convection bounds are tied to the finest grid, so the documented CLI argument max_level=4 makes a correct run fail

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/print_utils.rs:18`
- **Auditor confidence:** confirmed

**Claim.** FINEST_ERR_BOUND = 2.0e-4 sits between the N=16 error (2.411e-4) and the N=32 error (5.316e-5). The ladder's finest level is user-selectable via `max_level`, documented at main.rs:32 as extending or truncating. Passing 3 or 4 yields levels [3,4] with finest N=16 and error 2.411e-4 > 2.0e-4, so the gate fails on correct code. CONVECTION_BOUND = 1.0e-2 fails too, since the O(dx^2) convection error quadruples from 3.207e-3 at N=32 to ~1.3e-2 at N=16.

**Code evidence.**

```
print_utils.rs:17-24:
/// Pinned finest-grid bound on the max-norm field error vs. the analytic decay.
const FINEST_ERR_BOUND: f64 = 2.0e-4;
/// Pinned minimum observed spatial-convergence order ...
const MIN_ORDER: f64 = 1.8;
/// Pinned bound on the convection-operator error vs. the closed form `-1/2 sin(2x)`.
const CONVECTION_BOUND: f64 = 1.0e-2;

main.rs:56: let levels: Vec<usize> = (3..=max_level.max(4)).collect();
main.rs:32: "`max_level` (default 5) extends the ladder to a `2^max_level` grid."

baseline.txt:6-8:
  N =   8   max_err = 9.789e-4 ...
  N =  16   max_err = 2.411e-4 ...
  N =  32   max_err = 5.316e-5 ...
```

**Reference form.** A grid-dependent quantity must be gated by a grid-dependent bound. Since the scheme is second order, the correct form is err(N) <= C/N^2 with C fixed once (here C ~ 0.055 from 5.316e-5*32^2), which holds at every level of the ladder. A single absolute constant is only valid if the grid is fixed.

**Impact.** A documented, supported invocation of the harness reports FAIL on unmodified, correct code — the worst outcome for a gate, since it trains users to distrust or ignore failures. It also means the bounds encode the default grid rather than the physics.

**Recommended fix.** Replace the two absolute constants with grid-scaled bounds: `finest.max_err > C_ERR / (finest.n * finest.n)` and `conv_err > C_CONV * dx * dx`, with C_ERR and C_CONV derived from the analytic truncation coefficients (see the dx^2/12 * 2*nu*t budget, which predicts the measured errors to within 2%) rather than pinned from a run.

**Adversarial check.** Verified end to end. print_utils.rs:17-24 holds the three pinned constants exactly as quoted. main.rs:56 is `let levels: Vec<usize> = (3..=max_level.max(4)).collect();` and main.rs:32 documents `max_level` as extending the ladder, with the in-code comment at :54-55 explicitly saying 'extends or truncates it'. Passing 3 or 4 yields levels [3,4], finest N=16, whose recorded max_err is 2.411e-4 (baseline.txt:7) — above FINEST_ERR_BOUND = 2.0e-4, so print_utils.rs:162-168 fires FAIL and main.rs:94 exits 1 on correct code. The convection check compounds it: main.rs:84-86 evaluates it at `finest_l`, and the operator is centered (gradient_x), so the recorded 3.207e-3 at N=32 becomes ~1.3e-2 at N=16, over CONVECTION_BOUND = 1.0e-2. The reference form is right: a grid-dependent quantity needs a grid-dependent bound (err(N) <= C/N^2 with C fixed once).

> Evidence re-read: verification/qtt_taylor_green_verification/print_utils.rs:17-24, :147-204 (verify); main.rs:32, :48-56, :83-95, :106-145 (convection at finest level); baseline.txt:6-9

---

### 1.16 [MAJOR] qtt_taylor_green's reported 'clean 2nd-order convergence' order of 2.18 is produced by cancellation between the spatial error and a fixed first-order-in-time error, not by the spatial scheme

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/print_utils.rs:133`
- **Auditor confidence:** confirmed

**Claim.** Same claim, with one correction: the error budget reproduces the measured ladder to roughly 3 percent, not exactly. The conclusion — that the >2 observed order is a fixed-dt cancellation artifact and that the one-sided MIN_ORDER gate cannot catch it — stands.

**Code evidence.**

```
config.rs:26-30:
pub const NU: f64 = 0.05;
/// Explicit-Euler time step (exact specification).
pub const DT: f64 = 0.01;
/// Number of marched steps (horizon `t = DT*STEPS = 0.2`).
pub const STEPS: usize = 20;

config.rs:70-74 — build_config(l) varies only the grid; `.solver(ft(DT), ft(NU), trunc())` uses the same DT at every level.

print_utils.rs:132-135 (printed on PASS):
        "\nTaylor-Green vortex reproduced: 2nd-order convergence to the analytic e^(-2 nu t) decay,"

verification/README.md:197-198: "observed order **2.02 -> 2.18** — clean 2nd-order convergence to the analytic decay"
baseline.txt:9: "observed order = 2.18 (centered FD + spectral projection -> 2)"
```

**Reference form.** A space-time convergence study refines dt with dx (dt ~ dx for a first-order scheme, dt ~ dx^2 for balanced second-order). Spatial-only error for the TG mode is exp(-2*nu*t)*(2*nu*t)*(dx^2/12); forward-Euler temporal error over 20 steps at 2*nu*dt = 1e-3 is 0.98*[e^{-0.02} - (1-0.001)^20] = +9.8e-6 of opposite sign.

**Impact.** MIN_ORDER = 1.8 is a one-sided gate, so an order of 2.18 (or 3, or 5) passes. The harness therefore certifies 'clean 2nd-order convergence' for a run whose apparent super-convergence is a numerical accident, and the first-order temporal scheme is never verified at all. At N=64 the fixed temporal error would dominate and the observed order would collapse, with no gate to catch it.

**Recommended fix.** Either refine dt alongside dx (e.g. dt proportional to dx^2) so the study measures true space-time order, or state explicitly in the README and the summary line that this is a spatial-only convergence study at fixed dt and that the temporal error is a separate, ungated 1e-5 floor. Add a two-sided order gate (1.8 <= p <= 2.2) so cancellation-driven super-convergence is flagged rather than rewarded.

**Adversarial check.** The mechanism is real and I re-derived it independently. config.rs:26-30 fixes NU=0.05, DT=0.01, STEPS=20, and build_config(l) at :69-85 varies only the grid — DT is identical at every level, so the forward-Euler temporal error is level-independent. Spatial: centered FD under-resolves the Laplacian by (1-(k dx)^2/12), giving a positive error exp(-2 nu t)*2 nu t*dx^2/12 = 1.01e-3 / 2.52e-4 / 6.3e-5 at N=8/16/32. Temporal: (1-2 nu dt)^20 - exp(-2 nu t) = 0.9801887 - 0.9801987 = -1.0e-5, constant and opposite in sign. Sum: 1.00e-3 / 2.42e-4 / 5.3e-5 -> observed orders 2.04 and 2.19, against the printed 2.02 and 2.18. That reproduces the ladder to about 3 percent (not 'exactly', as the finding says) and accounts for the above-theoretical order. MIN_ORDER = 1.8 is one-sided (print_utils.rs:173), so an anomalously high order cannot fail, and the first-order temporal scheme is never gated. The reference form (refine dt with dx) is correct.

> Evidence re-read: verification/qtt_taylor_green_verification/config.rs:25-30, :69-85; print_utils.rs:20 (MIN_ORDER), :170-179, :131-136 (summary prose); baseline.txt:6-9; verification/README.md:197-198

---

### 1.17 [MINOR] dec_lid_cavity's refinement-trend gates are pinned from the code's own prior measurements with post-hoc headroom

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/main.rs:117`
- **Auditor confidence:** confirmed

**Claim.** The two absolute RMSE bounds are pinned from the harness's own measurements with unequal and partly mislabeled margins (27 % coarse, 50 % fine, both described as '~25 %'). The third gate, TREND_MARGIN's strict decrease under refinement, is a genuine non-circular convergence check, so the gated mode is not wholly self-referential.

**Code evidence.**

```
main.rs:117-118:
    // Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom) plus the
    // strict refinement-trend margin. Compared in native `FloatType` (the `f64` gates lift via `ft`).

config.rs:82-87:
/// Pinned RMSE gate for the coarse (17^2) grid.
pub const TREND_COARSE_GATE: f64 = 0.32;
/// Pinned RMSE gate for the fine (33^2) grid.
pub const TREND_FINE_GATE: f64 = 0.20;
/// Required strict-decrease margin between the coarse and fine RMSE.
pub const TREND_MARGIN: f64 = 0.04;

dec_lid_cavity_re1000_verification/README.md (closing paragraph): "the time-converged values 0.252 -> 0.133 belong here"
```

**Reference form.** A non-circular RMSE bound for a second-order wall-bounded solver is derived from the discretization error at the given grid, e.g. C*h^2 with C fixed by a single reference-grid calibration and applied at all levels, or from a published grid-convergence study of the same scheme against Ghia.

**Impact.** The gate's PASS on the pinned run is guaranteed. Its ability to catch a regression is bounded by an arbitrarily chosen 27-50% margin, and the two grids carry different margins with no stated reason. The verification README presents this as the gated mode of a Ghia-referenced benchmark.

**Recommended fix.** Derive the bounds from the second-order scaling: if RMSE(17^2) <= B then RMSE(33^2) <= B/4 * safety. Fix one anchor from a reference-grid run and let the other follow. Document the safety factor and correct the '~25 % headroom' comment to state the two actual margins.

**Adversarial check.** Locations and arithmetic verified: main.rs:117-118 carries the comment 'Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom)' verbatim, config.rs:82-87 holds TREND_COARSE_GATE = 0.32, TREND_FINE_GATE = 0.20, TREND_MARGIN = 0.04, and the dir README's closing paragraph says 'the time-converged values 0.252 -> 0.133 belong here'. The headroom arithmetic is as the auditor computes: 0.32/0.252 = 1.27 and 0.20/0.133 = 1.50, so the '~25 %' label is right for the coarse grid and wrong (50 %) for the fine one. But 'the gate's PASS is guaranteed' overstates: the gate set also enforces TREND_MARGIN, a strict-decrease-under-refinement requirement between the two grids, which is a genuine non-circular convergence property no pinning can manufacture — a solver that stopped converging would fail it regardless of the absolute bounds.

> Evidence re-read: verification/dec_lid_cavity_re1000_verification/main.rs:117-130 (trend gates); config.rs:76-87; dec_lid_cavity_re1000_verification/README.md closing paragraph

---

### 1.18 [MAJOR] dec_cylinder_wake's module doc claims the case sheds a von-Karman street; the recorded run reports no shedding and the probe signal is monotone

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/dec_cylinder_wake_verification/main.rs:26`
- **Auditor confidence:** confirmed

**Claim.** The module doc (main.rs:26) asserts a von-Karman street the recorded run does not produce, and verification/README.md:151-152 describes a disclaimed Strouhal that is never printed. The probe signal is not monotone and does cross its mean, but its peak-to-peak amplitude is 1.3e-3 (~0.1 % of U) with no sign change — no shedding. The kinetic energy has plateaued in the recorded run, so the 'never reached a developed state' inference does not follow.

**Code evidence.**

```
main.rs:26-28 (module doc):
//! sensor drives a **prescribed moving wall** (a Dirichlet boundary), confined in a **periodic-x channel**. This
//! sheds a von-Karman street and is a faithful exercise of the cut-cell + uncertain-zone machinery.

cli_output.txt:5: "# Strouhal: no clear shedding detected in the developed signal"

cylinder_wake.csv second half, min/max of v_probe: 0.026174 / 0.027459 (no mean crossings)
cylinder_wake.csv last rows: monotonically decreasing 0.026958 -> 0.026954

verification/README.md:152-153: "Strouhal is reported but disclaimed (confined/periodic, not the isolated-cylinder reference -- see `dec_cylinder_verification` for that)."

The dir README is correct: dec_cylinder_wake_verification/README.md states "the run reports **no clear shedding**".
```

**Reference form.** A von-Karman street at Re_D = 100 produces a transverse-velocity probe signal oscillating about its mean with multiple sign-relative crossings per shedding period. The recorded series has zero mean crossings over 1000 samples.

**Impact.** Two of the three documentation layers assert a physical phenomenon the artifact shows did not occur. The kinetic energy in baseline.txt also climbs monotonically from 5.17e1 to 2.42e2 over the full 2000 steps, indicating the run never reached a developed state — so the 'developed (second-half) window' the Strouhal estimator assumes does not exist.

**Recommended fix.** Correct main.rs:26-28 to state that the confined periodic channel at 25% blockage damps the street at this configuration, matching the dir README. Correct verification/README.md:152 to say Strouhal is *attempted* and reports no shedding. Consider adding a gate that the run reached a statistically stationary state, since the current energy trace shows it did not.

**Adversarial check.** The core contradiction is confirmed. main.rs:26-27 states 'This sheds a von-Karman street and is a faithful exercise of the cut-cell + uncertain-zone machinery', while cli_output.txt:5 records '# Strouhal: no clear shedding detected in the developed signal'. verification/README.md:151-152 compounds it by saying 'Strouhal is reported but disclaimed' when no Strouhal value is reported at all. The directory README is correct. Two sub-claims are wrong and should not carry into the finding. (a) I computed the second-half probe statistics directly: min 0.026168, max 0.027459, mean 0.027162 — the mean lies strictly inside the range, so the series is not monotone and does have mean crossings (just fewer than the estimator's minimum of two). What is true is that the signal never changes sign and its peak-to-peak amplitude is 1.3e-3, ~0.1 % of U — no oscillation worth calling a street. (b) The kinetic energy has plateaued by the end, not climbed monotonically throughout: the last five recorded samples run 2.4137e2 -> 2.4155e2, a change of under 0.01 % per 10 steps, so 'the run never reached a developed state' is not supported by the artifact.

> Evidence re-read: verification/dec_cylinder_wake_verification/main.rs:23-28; cli_output.txt:1-7; cylinder_wake.csv second half computed with awk (min 0.0261681, max 0.0274593, mean 0.0271621, n=1001); baseline.txt first and last rows; verification/README.md:151-153; dec_cylinder_wake_verification/README.md

---

### 1.19 [MAJOR] dec_cylinder_verification's Strouhal estimator reports a number derived from 7th-decimal numerical noise with no amplitude guard

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/verification/dec_cylinder_verification/main.rs:320`
- **Auditor confidence:** confirmed

**Claim.** report_strouhal detects mean crossings with no check on the signal amplitude or signal-to-noise ratio. On a steady (non-shedding) wake it counts crossings of round-off fluctuation and prints a plausible-looking Strouhal next to the Williamson reference. The crate's own README documents this having happened: the staircase run printed St = 0.2444 from a wake that never shed.

**Code evidence.**

```
main.rs:308-322:
    let mut crossings: Vec<f64> = Vec::new();
    for w in tail.windows(2) {
        let (t0, v0) = w[0]; let (t1, v1) = w[1];
        if v0 - mean <= 0.0 && v1 - mean > 0.0 {
            crossings.push(t0 + (mean - v0) / (v1 - v0) * (t1 - t0));
        }
    }
    if crossings.len() < 2 { ... return; }
    let period = (crossings.last().unwrap() - crossings[0]) / (crossings.len() - 1) as f64;
    let st = (1.0 / period) * diameter / u_ref;
    eprintln!("# shedding: period {period:.4}, St = f*D/U ~ {st:.4}  (Williamson Re=100 ~ 0.164)");

re100_16_staircase.txt: "# shedding: period 4.0912, St = f*D/U ~ 0.2444  (Williamson Re=100 ~ 0.164)"

dec_cylinder_verification/README.md:120 admits it:
| staircase | **none** -- wake decays to a steady residual `v_probe ~ -0.0069` (flat from t~20 to t=100) | n/a (printed `0.244` is the crossing-detector on 7th-decimal noise) | ...
```

**Reference form.** A Strouhal estimator must qualify the signal before reporting a frequency: require the peak-to-peak amplitude of the developed window to exceed a stated fraction of U (e.g. 1%), or require the dominant spectral peak to carry a stated fraction of the total variance. Standard practice in cylinder-wake post-processing.

**Impact.** The harness writes 'St = f*D/U ~ 0.2444 (Williamson Re=100 ~ 0.164)' to stderr with no qualifier. Read without the README, that is a measurement 49% above the reference; it is in fact noise. Since the program has no gate and exits 0, nothing distinguishes the two cases programmatically.

**Recommended fix.** Add an amplitude guard before reporting: compute peak-to-peak of the tail window and refuse to report St (printing 'steady wake, no shedding') when it falls below e.g. 0.01*U. This is exactly the guard the README's own gate-result table applies by hand.

**Adversarial check.** Code verified verbatim at main.rs:299-323: the estimator takes the second-half tail, computes the mean, counts upward mean crossings, and requires only `crossings.len() >= 2` before printing a Strouhal next to the Williamson reference — no amplitude, variance, or SNR qualification anywhere. The failure is documented as having actually occurred: re100_16_staircase.txt:10 prints '# shedding: period 4.0912, St = f*D/U ~ 0.2444  (Williamson Re=100 ~ 0.164)', and dec_cylinder_verification/README.md line 121 records that same run as 'none -- wake decays to a steady residual v_probe ~ -0.0069 (flat from t~20 to t=100)' with 'printed 0.244 is the crossing-detector on 7th-decimal noise'. The reference form (require peak-to-peak amplitude above a stated fraction of U, or a dominant spectral peak carrying a stated share of variance) is standard cylinder-wake practice. Compounded by finding 1: the program has no gate and exits 0, so nothing distinguishes the noise case from the real one programmatically.

> Evidence re-read: verification/dec_cylinder_verification/main.rs:299-323 (report_strouhal, no amplitude guard); re100_16_staircase.txt:10; re100_16_resolved.txt:9; dec_cylinder_verification/README.md:114-122

---

### 1.20 [MAJOR] dec_cylinder_verification cites two mutually inconsistent C_d reference bands for Re=100, and the printed band is the one the crate's own README calls a low-side outlier

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/dec_cylinder_verification/main.rs:294`
- **Auditor confidence:** confirmed

**Claim.** The runtime output, the module doc and verification/README.md all use 'ref C_d ~ 1.24-1.33'. The same directory's README states the reference window is 1.32-1.36 (citing Qu et al. 2013, Posdziech & Grundmann 2007 via arXiv:2303.09262) and describes 1.24 as matching 'only the low-side cut-cell value of Droge-Verstappen'. Using the wider low band makes the measured value appear inside or barely above the reference.

**Code evidence.**

```
main.rs:292-296 (printed at runtime):
    eprintln!(
        "# drag (cycle mean over {} samples): C_d ~ {cd:.3} (pressure {cd_p:.3} + friction {cd_f:.3}), \
         C_l ~ {cl:.3}, C_d swing [{cd_min:.3}, {cd_max:.3}]  (ref C_d ~ 1.24-1.33, friction ~ 25%)",

dec_cylinder_verification/README.md:113-116:
"Reference window (2-D laminar, unconfined; Qu et al. 2013, Posdziech & Grundmann 2007, Williamson, as compiled in arXiv:2303.09262): `St ~ 0.164-0.165`, mean `C_d ~ 1.32-1.36` ..."

dec_cylinder_verification/README.md:125-127:
"cycle-mean `C_d ~ 1.246` is **~6 % below** the `1.32-1.36` consensus (it matches only the low-side cut-cell value of Droge-Verstappen 1.24)"

verification/README.md:36 and :173 both use the 1.24-1.33 band.
```

**Reference form.** Published 2-D laminar cylinder C_d at Re=100 clusters at 1.32-1.38 (Braza et al. 1986 ~1.36; Liu et al. 1998 ~1.35; the arXiv:2303.09262 compilation the crate's own README cites gives 1.32-1.36). A verification reference band should be a single, source-identified interval used consistently.

**Impact.** The verification README reports 'C_d 1.345 vs reference band 1.24-1.33 -> +1.1 % above the top of the band'. Against the consensus band the same crate cites elsewhere (1.32-1.36), 1.345 would be *inside* the band, and the archived 16-cells/D run's 1.246 would be 6% *below* it. The choice of band changes the verdict's sign. Neither citation identifies a table or figure number.

**Recommended fix.** Adopt one band throughout — the 1.32-1.36 consensus with its arXiv:2303.09262 compilation citation — and update main.rs:294, main.rs:34-36, and verification/README.md:36/:173. Add the specific table or figure for the Droge-Verstappen 1.24 = 0.93 + 0.31 split if it is retained as a secondary reference.

**Adversarial check.** Both bands verified in place. The runtime string at main.rs:292-296 ends '(ref C_d ~ 1.24-1.33, friction ~ 25%)', the module doc at main.rs:34-36 repeats it, and verification/README.md:36 and :173 use the same band. The directory README:115-117 states a different reference window — 'St ~ 0.164-0.165, mean C_d ~ 1.32-1.36' citing Qu et al. 2013, Posdziech & Grundmann 2007 and Williamson via arXiv:2303.09262 — and at :127-128 explicitly calls 1.246 '~6 % below the 1.32-1.36 consensus (it matches only the low-side cut-cell value of Droge-Verstappen 1.24)'. The auditor's point about the verdict's sign flipping with the band choice is correct: 1.345 is +1.1 % above 1.33 but inside 1.32-1.36. One correction: the module doc does identify the sources for the printed band (Droge-Verstappen 1.24 = 0.93 pressure + 0.31 friction; Lehmkuhl lineage ~1.33), so the band is source-attributed, just not to a table or figure number and not consistent with the other band in the same crate.

> Evidence re-read: verification/dec_cylinder_verification/main.rs:29-36 (module doc), :292-296 (runtime string); dec_cylinder_verification/README.md:114-117, :125-129; verification/README.md:36, :173; re100_16_resolved.txt:10

---

### 1.21 [MINOR] qtt_cylinder's accuracy-vs-bond gate has eleven orders of magnitude of headroom and is vacuous as written

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:18`
- **Auditor confidence:** confirmed

**Claim.** The gate's tolerance sits ~11 orders above the measured convergence residual, so it certifies far less than the 'converges to machine-eps' the summary table reports. It is not vacuous — a gross loss of bond convergence would still fail it — but it cannot support the headline claim.

**Code evidence.**

```
print_utils.rs:17-18:
/// Pinned drag-convergence bound: the relative change between the two finest bond caps.
const CONVERGENCE_BOUND: f64 = 0.10;

print_utils.rs:94-103:
        let rel = (finest.drag - prev.drag).abs() / finest.drag.abs().max(1e-12);
        if rel > CONVERGENCE_BOUND { ... }

baseline.txt:8-9:
  bond <=  16   C_d = 23.7577   |dC_d| = 7.22e-3 ...
  bond <=  24   C_d = 23.7577   |dC_d| = 1.89e-11 ...

verification/README.md:38 reports it as the headline: "ΔC_d 1.9e-11 | 0 (converged) | converges to machine-ε"
```

**Reference form.** A convergence gate's tolerance should sit within one or two orders of the expected converged residual, e.g. rel < 1e-6 given a measured 8e-13, so that a genuine loss of convergence (say to 1e-4) is caught. A 0.10 threshold against an 8e-13 measurement provides no discrimination.

**Impact.** The harness's stated headline result — 'the convergence trend is the verification result' (main.rs:22, README.md:39-40) — rests on a gate that cannot distinguish machine-precision convergence from 9% disagreement. The suite's Divergence column reports 'converges to machine-eps' while the gate certifies only 'within 10%'.

**Recommended fix.** Tighten CONVERGENCE_BOUND to a value that reflects the claim, e.g. 1e-6, and document the derivation. Note also that the same run shows divergence changing from 3.01e-7 at bond 16 to 5.47e-14 at bond 24 — an ungated quantity moving by seven orders across the same pair, which the drag gate is blind to.

**Adversarial check.** The arithmetic checks out: print_utils.rs:17-18 sets CONVERGENCE_BOUND = 0.10, applied at :94-103 to the relative change between the two finest bond caps; baseline.txt:9 gives |dC_d| = 1.89e-11 against C_d = 23.7577, i.e. 7.96e-13, so the tolerance is ~1.3e11x the measured value. verification/README.md:38 does report 'converges to machine-eps' as the headline while the gate certifies only 10 %. But 'vacuous as written' is too strong: the gate does discriminate — a solver regression that broke bond convergence outright (say 30 % drift between bond 16 and 24) would fail it, and the intermediate rows in the same table (2.89e-1 at bond 8) show the quantity genuinely spans that range. The defect is a mismatch between the certified bound and the advertised property, the same class as finding 23.

> Evidence re-read: verification/qtt_cylinder_verification/print_utils.rs:17-18, :93-104; baseline.txt:6-9 (|dC_d| 2.89e-1 / 7.22e-3 / 1.89e-11); verification/README.md:38; qtt_cylinder_verification/main.rs:20-22

---

### 1.22 [MINOR] The qtt_sod L1 tolerance is undocumented in origin and the velocity error sits at 91% of it, with no convergence study to justify either

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_sod/print_utils.rs:22`
- **Auditor confidence:** confirmed

**Claim.** TOL = 0.03 is a single absolute bound applied to all three primitives at one resolution with a qualitative rationale and no convergence ladder, so it cannot be checked against the scheme's expected order and its effective strictness differs 3x between channels. There is no positive evidence it was back-fitted to the measured 0.0274.

**Code evidence.**

```
qtt_sod/print_utils.rs:21-22:
/// L1 tolerance -- first-order Rusanov smears the contact, so the bound reflects mean accuracy.
const TOL: f64 = 0.03;

print_utils.rs:88-101 — `let pass = v < TOL;` applied identically to rho, u, p.

baseline.txt:14-16:
  [PASS] density  L1 error = 0.0175
  [PASS] velocity L1 error = 0.0274
  [PASS] pressure L1 error = 0.0151

main.rs:31: const L: usize = 9;  // single grid, 512 cells; no ladder anywhere in the harness.
```

**Reference form.** For a first-order monotone scheme on a solution with discontinuities, the L1 error converges as O(dx^{1/2}) to O(dx). A defensible gate runs at least two resolutions and asserts both the observed rate and an extrapolated bound, e.g. L1(N) <= C/N^p with C and p fixed from the theory. A single absolute number at one grid cannot be checked against anything.

**Impact.** This is the suite's designated 'only quantitative-accuracy gate' (verification/README.md:46-47). A regression degrading velocity accuracy by 10% — from 0.0274 to 0.0301 — would fail; one degrading it by 9% would pass. A regression degrading density accuracy by 70% would pass. The gate's sensitivity differs by more than 3x between channels for no stated reason.

**Recommended fix.** Add a two- or three-level refinement ladder (2^8, 2^9, 2^10) and gate on the observed L1 convergence rate as well as the finest-grid error, with the bound written as C/N^p and C derived once from theory. At minimum, document where 0.03 came from and use per-channel bounds scaled to each variable's dynamic range.

**Adversarial check.** Facts verified: print_utils.rs:21-22 sets TOL = 0.03 with only the rationale 'first-order Rusanov smears the contact, so the bound reflects mean accuracy' and no derivation; :88-101 applies the identical bound to rho, u and p; main.rs:31 fixes L = 9 (512 cells) with no ladder anywhere; baseline.txt:14-16 gives 0.0175 / 0.0274 / 0.0151. The observation that a single absolute number at one grid cannot be checked against anything, and that per-channel sensitivity therefore varies 3x for no stated reason, is a fair criticism of the suite's designated quantitative gate. Two qualifications. The bound does have a stated (if unquantified) rationale, which distinguishes it from a bare magic number. And the 'indistinguishable from back-fitting' inference is weak: 0.03 is a round engineering figure, and the auditor's own framing ('a regression degrading velocity by 10 % would fail, 9 % would pass') is a property of every threshold, not evidence of pinning. The constructive half — run at least two resolutions and assert an observed rate — is the right remedy.

> Evidence re-read: verification/qtt_sod/print_utils.rs:19-22, :88-102; main.rs:29-36 (L = 9, single grid); baseline.txt:13-16; verification/README.md:45-47

---

### 1.23 [MINOR] Divergence gates across three harnesses are 7-9 orders of magnitude looser than the measured values they certify as 'machine precision'

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/dec_cylinder_wake_verification/main.rs:112`
- **Auditor confidence:** confirmed

**Claim.** DIV_TOL = 1e-6 against a measured 3.33e-15 (headroom 3e8); qtt_taylor_green's DIVERGENCE_BOUND = 1e-6 against a measured 4.14e-14 (headroom 2.4e7). Both READMEs describe the property as holding 'at machine precision' or 'exact to machine precision', but the gates only enforce 1e-6.

**Code evidence.**

```
dec_cylinder_wake_verification/main.rs:112: const DIV_TOL: f64 = 1e-6;
cli_output.txt:7: "verified: incompressibility held (max div 3.334e-15)"
verification/README.md:147-148: "Gates max divergence `< 1e-6` ... the constrained Leray projector keeps the divergence residual at machine precision"

qtt_taylor_green_verification/print_utils.rs:23-24:
/// Pinned bound on the post-projection divergence residual.
const DIVERGENCE_BOUND: f64 = 1.0e-6;
baseline.txt:8: "divergence = 4.14e-14"
verification/README.md:200-201: "Divergence **~1e-14** (the spectral Leray projection is exact to machine precision)"
```

**Reference form.** A gate should encode the property being claimed. If the claim is machine precision for an O(1) field, the bound belongs near 1e-12, giving two orders of operational headroom over a 1e-14 measurement while still catching a projector that degrades to 1e-8.

**Impact.** A regression that degraded the Leray projection by eight orders of magnitude — from 1e-14 to 1e-7 — would pass both gates while the READMEs continue to assert machine precision. Bounded impact, since divergence is also printed, but the gate provides far less assurance than the prose implies.

**Recommended fix.** Tighten both bounds to ~1e-12 (or a precision-scaled multiple of FloatType::EPSILON) so the gate matches the documented claim, and note the operational headroom in the constant's doc comment.

**Adversarial check.** All four citations verified verbatim: dec_cylinder_wake_verification/main.rs:112 `const DIV_TOL: f64 = 1e-6;` against cli_output.txt:7 'max div 3.334e-15'; qtt_taylor_green_verification/print_utils.rs:23-24 `const DIVERGENCE_BOUND: f64 = 1.0e-6;` against baseline.txt:8 'divergence = 4.14e-14'. The prose in verification/README.md:147-148 ('at machine precision') and :200-201 ('exact to machine precision') asserts a property eight orders tighter than either gate enforces. The reference form is sound and the impact assessment is correctly bounded: the residual is also printed at every sample, so a degradation would be visible, but not to the gate. Note the finding's title says 'three harnesses' while only two are substantiated in the evidence.

> Evidence re-read: verification/dec_cylinder_wake_verification/main.rs:111-118; cli_output.txt:7; verification/qtt_taylor_green_verification/print_utils.rs:23-24, :194-201; baseline.txt:8; verification/README.md:147-148, :200-201

---

### 1.24 [MINOR] qtt_cylinder's no-slip floor of 0.15*U is unjustified, does not scale with the penalization parameter, and the module doc calls the drag 'O(1)' while gating it at 100

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:16`
- **Auditor confidence:** confirmed

**Claim.** NO_SLIP_FLOOR = 0.15 is a fixed fraction of the free-stream with no derivation and no coupling to ETA, so it cannot detect a mis-set penalization — the substantive defect. DRAG_SANITY_MAX = 100 is documented in-source as a sanity bound and the directory README explains at length why the absolute C_d ~ 23.8 is not an O(1) value; the 'O(1)' wording at main.rs:19 is imprecise but the gate and its documentation do not disagree.

**Code evidence.**

```
print_utils.rs:15-20:
/// Pinned no-slip floor: interior speed must fall below this fraction of the free-stream.
const NO_SLIP_FLOOR: f64 = 0.15;
/// Pinned drag-convergence bound: ...
const CONVERGENCE_BOUND: f64 = 0.10;
/// Pinned upper bound on a physical O(1) drag coefficient (sanity).
const DRAG_SANITY_MAX: f64 = 100.0;

qtt_cylinder_verification/main.rs:19: "3. **Physical drag** -- the streamwise drag is positive and `O(1)`."
baseline.txt:9: "bond <=  24   C_d = 23.7577 ... interior_max|u| = 4.22e-2"
config.rs:28: pub const ETA: f64 = 0.016;
```

**Reference form.** Angot, Bruneau & Fabrie (1999), Numer. Math. 81, 497-520 establish that the Brinkman penalization error scales with eta; the interior residual velocity should be gated as a function of eta (note eta^{3/4} = 0.045, close to the measured 4.22e-2), not as a fixed fraction of the free stream. 'O(1)' conventionally means order unity, i.e. within a small factor of 1.

**Impact.** Changing ETA to make the wall harder or softer leaves the no-slip gate unchanged, so it cannot detect a mis-set penalization. And DRAG_SANITY_MAX = 100 permits a C_d 74x the measured value while the doc describes the check as verifying an O(1) result — the label and the bound disagree by two orders.

**Recommended fix.** Express the no-slip floor as a multiple of the theoretical penalization residual (e.g. 3*eta^{3/4}) so it tracks ETA, citing Angot et al. for the scaling. Replace 'positive and O(1)' at main.rs:19 with the actual claim ('positive and finite, magnitude inflated by blockage and the smoothing skirt'), which print_utils.rs:106-107 already states correctly.

**Adversarial check.** Constants verified exactly: print_utils.rs:15-20 (NO_SLIP_FLOOR 0.15, CONVERGENCE_BOUND 0.10, DRAG_SANITY_MAX 100.0), config.rs:28 (ETA = 0.016), main.rs:19 ('the streamwise drag is positive and O(1)'), baseline.txt:9 (interior_max|u| = 4.22e-2, C_d = 23.7577). The substantive half holds: the no-slip floor is a fixed fraction of the free-stream with no stated derivation and no dependence on ETA, so changing the penalization strength cannot move the gate — and Angot-Bruneau-Fabrie (1999) is correctly cited as the basis for an eta-dependent bound (the eta^{3/4} = 0.045 coincidence with the measured 4.22e-2 is suggestive, though it is the auditor's numerology, not a derivation). The DRAG_SANITY_MAX half is weaker than presented: the constant's own doc calls it a 'sanity' bound, print_utils.rs:106-107 states the absolute magnitude is inflated by the smoothing skirt and blockage, and the dir README devotes a section ('Honest reading of the absolute C_d') to explaining why 23.8 is not an O(1) isolated-cylinder value. The mismatch is a loose word in one module-doc line, not a gate that contradicts its documentation.

> Evidence re-read: verification/qtt_cylinder_verification/print_utils.rs:15-20, :83-111; config.rs:20-34 (ETA, U_INF); main.rs:16-23; baseline.txt:9; qtt_cylinder_verification/README.md:41-56

---

### 1.25 [MINOR] qtt_cylinder's DEC cross-reference C_d = 1.345 is a self-generated constant not reproducible from any archived artifact, and the Reynolds mismatch between the two cases is undisclosed

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/config.rs:38`
- **Auditor confidence:** confirmed

**Claim.** DEC_CD_REF = 1.345 is hard-coded as 'the committed DEC isolated-cylinder drag'. The only archived DEC cylinder output in the repository reports C_d = 1.246, not 1.345. Separately, the QTT case runs at Re = U*D/nu = 1*1.885/0.05 = 37.7, while the cross-reference is at Re = 100 — a difference the README's list of disclaimers does not include.

**Code evidence.**

```
qtt_cylinder_verification/config.rs:36-38:
/// Committed DEC isolated-cylinder drag at Re 100 (`dec_cylinder_verification`) -- the **cross-reference**
/// (disclaimed: the periodic penalized box is not the DEC inflow/outflow/far-field configuration).
pub const DEC_CD_REF: f64 = 1.345;

dec_cylinder_verification/re100_16_resolved.txt (the only archived DEC drag artifact):
"# drag (cycle mean over 40 samples): C_d ~ 1.246 (pressure 1.078 + friction 0.167) ..."

QTT case Reynolds number, from config.rs:21, :30, :52:
pub const NU: f64 = 0.05;  pub const U_INF: f64 = 1.0;
pub fn diameter() -> FloatType { ft(2.0 * RADIUS_FRAC * 2.0 * std::f64::consts::PI) }  // = 1.885
=> Re = 1.0 * 1.885 / 0.05 = 37.7

qtt_cylinder_verification/README.md lists three disclaimers (blockage, penalization-integral force, transient); Reynolds number is not among them.
```

**Reference form.** A cross-reference constant should be traceable to a stored artifact or a published source. The archived DEC run in the repository gives 1.246; the 1.345 in the verification README summary (with its 1.173 + 0.172 split) corresponds to no committed output.

**Impact.** The number is printed at runtime as 'DEC isolated-cylinder cross-reference (Re 100): C_d ~ 1.345' and appears in the verification README summary as the dec_cylinder measured value, but no run in the repository produces it. The Re 38 vs Re 100 mismatch is a fourth source of disagreement beyond the three disclosed, and at Re 38 the physical C_d would be roughly 1.6-1.8 rather than 1.33.

**Recommended fix.** Commit the DEC run that produced 1.345 (the 96^2 / 1500-step default the summary table cites) as an artifact alongside re100_16_resolved.txt, or change DEC_CD_REF to the archived 1.246. Add the Reynolds-number difference to the disclaimer list in qtt_cylinder_verification/README.md.

**Adversarial check.** Both halves verified. config.rs:36-38 holds DEC_CD_REF = 1.345 described as the 'Committed DEC isolated-cylinder drag at Re 100'; the only archived DEC drag artifacts are re100_16_resolved.txt:10 (C_d ~ 1.246, at 16 cells/D, 4000 steps) and re100_16_staircase.txt:11 — neither yields 1.345. The number does correspond to a documented measurement (verification/README.md:173 records C_d ~ 1.345 with the 1.173 + 0.172 split at the 8 cells/D, 1500-step default), so it is traceable to prose but to no committed run artifact, exactly as the title states. The Reynolds arithmetic checks out from config.rs:21, :30, :51-53: diameter = 2*0.15*2*pi = 1.885, Re = 1.0*1.885/0.05 = 37.7, against a cross-reference at Re 100. qtt_cylinder_verification/README.md:41-56 lists exactly three disclaimers (blockage, penalization-integral force, transient) and never mentions Reynolds number, while :43 and :81 both assert 'Re 100' for the cross-reference without noting the QTT case is not at Re 100.

> Evidence re-read: verification/qtt_cylinder_verification/config.rs:20-38, :50-53 (diameter); dec_cylinder_verification/re100_16_resolved.txt:10 (C_d 1.246); verification/README.md:36, :173; qtt_cylinder_verification/README.md:41-56, :81

---

### 1.26 [MINOR] Ghia Re=1000 v-velocity table value at x=0.9063 is -0.51500 where the published value is -0.51550

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/config.rs:55`
- **Auditor confidence:** likely

**Claim.** The GHIA_V table entry (0.9063, -0.51500) appears to be a transcription error. This station is the table's extremum and the value -0.5155 is the widely quoted v_min for Ghia Re=1000. All 16 other entries in GHIA_V and all 17 in GHIA_U match the published tables exactly, which makes this single entry stand out.

**Code evidence.**

```
config.rs:49-56:
/// Ghia et al. (1982), Re = 1000: v along the horizontal centerline, (x, v).
pub const GHIA_V: [(f64, f64); 17] = [
    (1.0000, 0.00000),
    (0.9688, -0.21388),
    (0.9609, -0.27669),
    (0.9531, -0.33714),
    (0.9453, -0.39188),
    (0.9063, -0.51500),
    (0.8594, -0.42665),
```

**Reference form.** Ghia, Ghia & Shin (1982), J. Comput. Phys. 48, Table II, Re=1000 column: x=0.9063 -> v = -0.51550. This is the table minimum and is quoted as v_min = -0.5155 throughout the cavity-benchmark literature.

**Impact.** Bounded. The RMSE pools 34 stations, so a 5.0e-4 error at one station shifts the pooled RMSE by well under 0.1% — it will not change any verdict. But it is a defect in a published reference table that the harness's entire accuracy claim rests on, and it should be corrected on principle for a pre-certification artifact.

**Recommended fix.** Change the entry to (0.9063, -0.51550) after confirming against the original Table II. Consider adding a unit test that checksums the two Ghia tables against an independently-typed copy so future edits are caught.

**Adversarial check.** config.rs:55 reads `(0.9063, -0.51500)` exactly as quoted, inside GHIA_V (config.rs:48-67). Ghia, Ghia & Shin (1982) Table II, Re = 1000 column gives v = -0.51550 at x = 0.9063; this is the table extremum and the value quoted as v_min throughout the cavity-benchmark literature. I checked the surrounding entries against the published table and every other GHIA_V value (-0.21388, -0.27669, -0.33714, -0.39188, -0.42665, -0.31966, 0.02526, 0.32235, 0.33075, 0.37095, 0.32627, 0.30353, 0.29012, 0.27485) and all 17 GHIA_U values match exactly, which isolates this single entry as a transcription slip. The impact assessment is correctly bounded — a 5.0e-4 error at one of 34 pooled stations cannot move any verdict — but it is a defect in the reference table the harness's accuracy claim rests on.

> Evidence re-read: verification/dec_lid_cavity_re1000_verification/config.rs:27-67 (GHIA_U and GHIA_V, all entries compared against Ghia et al. 1982 Table I/II Re=1000 columns)

---

### 1.27 [MINOR] The verification README's config column for dec_lid_cavity ('33², t=40') matches neither the program's default nor its gated trend mode

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/README.md:34`
- **Auditor confidence:** confirmed

**Claim.** The summary row's config '33^2, t=40' matches neither the 65^2/t=100 default nor the 17^2->33^2/t=60 trend mode, and the RMSE 0.137 and vortex coordinates it reports appear in no committed artifact (the pinned trend value is 0.133). The '6 % of span' figure is ambiguous rather than wrong — it matches the offset relative to the half-domain.

**Code evidence.**

```
verification/README.md:34:
| `dec_lid_cavity_re1000_verification` | primary vortex (x, y); centerline RMSE | (0.563, 0.594); RMSE 0.137 | Ghia (0.531, 0.563) | Δ ≈ (0.031, 0.031) ≈ **6 % of span** | 33², t=40 | ~28 s |

main.rs:56-57:
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(65);
    let t_end: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100.0);

config.rs:79-81:
pub const TREND_T_END: f64 = 60.0;
pub const TREND_GRIDS: [usize; 2] = [17, 33];

config.rs:117 (trend pinning, per main.rs:117): 33² time-converged RMSE is 0.133, not 0.137.

baseline.txt:2 is a 65²/t=100 run, truncated at t=45, that never reached the vortex/RMSE output.
```

**Reference form.** The other twelve rows were checked and match: mms (default), graded MMS (8²-64², main.rs:51), TGV (16³/t*=10, main.rs:44-48), wake (2000 steps/93×32, config.rs:29/:43), cylinder (96²/Re100/1500, main.rs:89-93), qtt TG (8²-32²/t=0.2, config.rs:28-30 + main.rs:56), qtt cylinder (32²/4 caps, main.rs:41/:57), park2t (32²/40 steps, main.rs:35 + config.rs:29), sod (512 cells/t=0.2, main.rs:31/:34), blunt body (2^5-2^7, main.rs:116), reentry (2^3-2^5, main.rs:154).

**Impact.** The headline RMSE 0.137 and the vortex coordinates (0.563, 0.594) cannot be reproduced by running the program as documented, and no stored artifact contains them. A reviewer attempting to reproduce the table gets 0.133 from the trend mode or a different number from the 65² default. Also, the stated '6 % of span' does not match either component of the offset (0.031 each, i.e. 3.1%) or its magnitude (4.4%).

**Recommended fix.** Either add an explicit invocation for 33²/t=40 to the row (the table has no invocation column, so a footnote), or restate the row using the trend mode's committed numbers (33², t=60, RMSE 0.133) and commit that artifact. Correct '6 % of span' to '3.1 % per axis (one cell at 33²)'.

**Adversarial check.** The central claim is confirmed. verification/README.md:34 lists Config '33^2, t=40' (and :130 repeats '33^2 grid, t=40, ~28 s'), while main.rs:56-57 defaults to `unwrap_or(65)` and `unwrap_or(100.0)` and config.rs:79-81 sets TREND_T_END = 60.0 over TREND_GRIDS [17, 33]. No path produces t=40. baseline.txt:2 confirms a 65^2 / t_end 100 run. The pinned fine-grid RMSE is 0.133 (main.rs:117 comment), not the 0.137 in the table, and no committed artifact contains 0.137 or the vortex pair (0.563, 0.594). One sub-claim is weaker than stated: '6 % of span' plausibly means 6 % of the half-domain (0.031/0.5 = 6.2 %) rather than of the unit span, so it is ambiguous rather than simply wrong. I did not independently re-verify all twelve other rows; I spot-checked dec_cylinder ('96^2' = 12 D / (1/8) = 96, matching main.rs:90-93 defaults) and it holds.

> Evidence re-read: verification/README.md:34, :130-133; dec_lid_cavity_re1000_verification/main.rs:50-60, :117; config.rs:78-87; baseline.txt:1-6

---

### 1.28 [MINOR] Committed baseline.txt files contain hand-written prose the programs do not print, and one omits a block the program does print

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/baseline.txt:21`
- **Auditor confidence:** confirmed

**Claim.** Both READMEs describe baseline.txt as 'the recorded reference output'. qtt_sod/baseline.txt lines 20-27 carry a 'Notes:' block that appears in no source file (grep for 'Notes:' across qtt_sod/*.rs returns nothing). qtt_park2t_blackout/baseline.txt lines 21-28 carry a 'Notes (Tier-A disclaimers):' block that the code does not print, and simultaneously omits the entire '--- Published reference cross-references (Tier-A disclaimers) ---' section that print_utils::render does print at lines 176-191.

**Code evidence.**

```
qtt_park2t_blackout/baseline.txt:21-28 begins:
Notes (Tier-A disclaimers):
  - Rides the INCOMPRESSIBLE QTT rollout; T_tr is a recovery-temperature reconstruction ...

qtt_park2t_blackout/print_utils.rs:176-180 (which render() always executes, called from main.rs:66 before verify()):
    println!("\n--- Published reference cross-references (Tier-A disclaimers) ---");
    println!(
        "  RAM-C II peak n_e (~71 km)  : ~{:.1e} m^-3 [order-of-magnitude anchor]",
        config::RAMC_NE_REFERENCE
    );

That section appears nowhere in baseline.txt.

qtt_sod/README.md:41: "See `baseline.txt` for the recorded reference output."
qtt_park2t_blackout/README.md:60: "See `baseline.txt` for the recorded reference output."
```

**Reference form.** A recorded reference output should be byte-reproducible by running the documented command, so that a diff against a fresh run is a valid regression check.

**Impact.** The baselines cannot be used as regression fixtures — a diff against a real run would report both spurious deletions (the hand-written Notes) and spurious additions (the cross-reference block). It also means no committed artifact demonstrates that the RAM-C cross-reference and its disclaimers actually reach the user's terminal.

**Recommended fix.** Regenerate every baseline.txt by redirecting the program's actual stdout+stderr, and move the hand-written Notes prose into the directory README where it belongs. Consider a CI check that re-runs each harness and diffs against its baseline for the deterministic ones.

**Adversarial check.** Verified by grep and by diffing content against the print paths. `grep -rn 'Notes' qtt_sod/*.rs` and `grep -rn 'Notes' qtt_park2t_blackout/*.rs` both return zero matches, yet qtt_sod/baseline.txt:20-27 carries a 'Notes:' block and qtt_park2t_blackout/baseline.txt:21-28 a 'Notes (Tier-A disclaimers):' block. Conversely qtt_park2t_blackout/print_utils.rs:176-191 unconditionally prints '--- Published reference cross-references (Tier-A disclaimers) ---' plus the RAM-C anchor line and five disclaimer lines from `render()`, and that entire section is absent from baseline.txt. Both READMEs (qtt_sod:41, qtt_park2t_blackout:60) call baseline.txt 'the recorded reference output'. Corroborating evidence the auditor did not cite: the park2t baseline transliterates the program's Unicode (prints 'w_p', 'tau = dt/1000', '10^4' where the code emits omega_p, tau, 10 superscript-4), so these files are hand-edited transcriptions, not captures, and cannot serve as diff fixtures.

> Evidence re-read: grep 'Notes' over qtt_sod/*.rs and qtt_park2t_blackout/*.rs (no matches); qtt_sod/baseline.txt:20-27; qtt_park2t_blackout/baseline.txt:11-28 vs print_utils.rs:135-142 (verify) and :167-192 (render); qtt_sod/README.md:41; qtt_park2t_blackout/README.md:60

---

### 1.29 [MINOR] dec_graded_mms's grading-amplitude claim of a 3:1 spacing ratio exceeds the amplitudes the code actually runs

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/dec_graded_mms_verification/main.rs:25`
- **Auditor confidence:** confirmed

**Claim.** The module doc and README claim second order holds 'even at strong grading (a 3:1 spacing ratio)' and 'to amp 0.5 = 3:1 ratio'. The code runs amplitudes [0.0, 0.1, 0.2, 0.3]. At amplitude a the spacing ratio is (1+a)/(1-a); at a=0.3 that is 1.86:1, not 3:1. A 3:1 ratio requires a=0.5, which the harness does not run.

**Code evidence.**

```
main.rs:52: let amplitudes = [0.0, 0.1, 0.2, 0.3];

main.rs:24-26 (module doc):
//! norms at every grading amplitude**, even at strong grading (a 3:1 spacing ratio). The
//! error *constant* grows mildly with grading; the *order* holds at ≈ 2.

dec_graded_mms_verification/README.md:32:
| **Convective** `i_X omega` | ≈ 2 (to amp 0.5 = 3:1 ratio) | **≈ 2** | only the error *constant* grows mildly |

main.rs:116: let len = move |pos: usize| 1.0 + amp * (2.0 * PI * pos as f64 / n as f64).cos();
=> max/min = (1+a)/(1-a); at a=0.3 this is 1.3/0.7 = 1.857.
```

**Reference form.** For the cosine modulation l(pos) = 1 + a*cos(2*pi*pos/N), the extreme spacing ratio is (1+a)/(1-a). Solving (1+a)/(1-a) = 3 gives a = 0.5.

**Impact.** The measured evidence covers grading up to 1.86:1; the claim advertises 3:1. Since the coarse-pair order already dips to 1.72 at a=0.3 (baseline.txt:8), the behaviour at a=0.5 is not obviously safe to extrapolate. An engineer choosing a wall-grading ratio for a boundary-layer mesh on this guidance could exceed the verified envelope.

**Recommended fix.** Either extend the amplitude sweep to include 0.4 and 0.5 and re-record the baseline, or restate both claims as 'up to a 1.86:1 spacing ratio (amplitude 0.3)'. Additionally, add the gate this study lacks: assert every finest-pair observed order exceeds 1.8.

**Adversarial check.** main.rs:52 is `let amplitudes = [0.0, 0.1, 0.2, 0.3];` and main.rs:116 is `let len = move |pos: usize| 1.0 + amp * (2.0 * PI * pos as f64 / n as f64).cos();`. I re-derived the reference form: for l(pos) = 1 + a cos(2 pi pos/N) the extreme spacing ratio is (1+a)/(1-a), which is 1.857:1 at a = 0.3 and requires a = 0.5 for 3:1. The claim appears at main.rs:24 (module doc, 'even at strong grading (a 3:1 spacing ratio)') and dec_graded_mms_verification/README.md:32 ('to amp 0.5 = 3:1 ratio') and :37-38 ('Even at strong grading (a 3:1 spacing ratio) the order holds'), while a = 0.5 is never run. The extrapolation caution is supported by the data: baseline.txt:8 shows the a=0.3 convective coarse-pair max-norm order dipping to 1.72 (from 1.87 at a=0), so the trend with grading is downward at the coarse end.

> Evidence re-read: verification/dec_graded_mms_verification/main.rs:24-26, :52, :116; dec_graded_mms_verification/README.md:32, :36-40; baseline.txt:5-8 (convective orders 1.87/1.88/1.76/1.72 at a = 0.0/0.1/0.2/0.3)

---

### 1.30 [MINOR] dec_graded_mms and dec_taylor_green_re1600 directory READMEs document a model.rs file that does not exist

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/dec_taylor_green_re1600_verification/README.md:92`
- **Auditor confidence:** confirmed

**Claim.** The affected harnesses are dec_taylor_green_re1600_verification and mms_taylor_green_verification (not dec_graded_mms_verification, which has no such README table). Both document a model.rs that does not exist, and the TGV README additionally cites a `flt!` macro where the code uses a plain `ft` function in config.rs.

**Code evidence.**

```
dec_taylor_green_re1600_verification/README.md:89-93:
| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the `FloatType` alias, argument parsing, and the `CausalFlow` chain ... |
| `model.rs` | The precision-generic model: the lattice manifold, the solver configuration (`nu` from Re at `R`), the two flow stages, and the `Sample<R>`/`Report<R>` carriers. |
| `print_utils.rs` | Presentation only ... |

README.md:104-107: "Exact `f64` specifications (`Re`, the CFL step, pi) lift once into `R` through the `flt!` macro in `model.rs`"

Actual directory listing: README.md, baseline.txt, config.rs, main.rs, print_utils.rs

Same defect at mms_taylor_green_verification/README.md:65: "| `model.rs` | The Taylor-Green field equations, the tangent-functor plumbing ..."

Actual lift is config.rs:32-34: pub fn ft(x: f64) -> FloatType { FromPrimitive::from_f64(x)... }
```

**Reference form.** The crate's own stated example file layout convention is main/model_config/model/model_types/utils_print/constants; these two harnesses use main/config/print_utils and the READMEs were not updated when model.rs was folded into config.rs.

**Impact.** Documentation directs a reader to a nonexistent file for the physics content. Low functional impact but it is the first thing a reviewer looks for when auditing where the constants and the model live, and it signals the READMEs have drifted from the code.

**Recommended fix.** Update both File layout tables to list config.rs with its actual responsibility, and replace the `flt!` macro reference at dec_taylor_green_re1600_verification/README.md:104 with the `ft` function in config.rs.

**Adversarial check.** The defect is real but the title names the wrong second harness. Verified: dec_taylor_green_re1600_verification/README.md:92 lists `model.rs` in its File-layout table and :104-105 refers to 'the `flt!` macro in `model.rs`'; mms_taylor_green_verification/README.md:65 lists `model.rs` with a described responsibility. Directory listings show neither has model.rs — dec_taylor_green_re1600_verification contains README.md, baseline.txt, config.rs, main.rs, print_utils.rs, and mms_taylor_green_verification the same set. The lift is a plain `ft` function in config.rs, not a `flt!` macro. dec_graded_mms_verification, named in the finding's title, has only README.md, baseline.txt and main.rs and its README does not mention model.rs — the finding's own code-evidence block correctly cites mms_taylor_green_verification, so this is a title slip.

> Evidence re-read: grep 'model.rs|flt!' over both READMEs (dec_taylor_green_re1600_verification/README.md:92, :104-105; mms_taylor_green_verification/README.md:65); directory listings of all three harnesses; qtt_taylor_green/config.rs-style `pub fn ft` lift

---

### 1.31 [MINOR] dec_taylor_green_re1600's reported 'peak dissipation' occurs at the final step, so the dissipation curve has not peaked and the value is a run-truncation artifact

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/dec_taylor_green_re1600_verification/print_utils.rs:39`
- **Auditor confidence:** confirmed

**Claim.** The peak-tracking logic reports the maximum dissipation seen so far. In the recorded run the maximum occurs at the last emitted step (t* = 10.05) and the dissipation column is still rising monotonically through the final five rows. The quantity is therefore the endpoint value, not a peak, and reporting it as 'peak dissipation ... at t* = 10.05' next to a DNS peak at t* ~ 9 invites a false comparison.

**Code evidence.**

```
print_utils.rs:39-42:
        if dissipation > peak.1 {
            peak = (t_star, dissipation);
        }

baseline.txt (final rows, dissipation still increasing):
9.7389,0.10812840,0.00241123
9.8175,0.10793787,0.00242587
9.8960,0.10774622,0.00244021
9.9746,0.10755346,0.00245426
10.0531,0.10735962,0.00246801

baseline.txt (summary): "marched to t* = 10.05: E*/E0 = 0.8929, peak dissipation 0.002468 at t* = 10.05"

verification/README.md:111-112: "Peak dissipation **≈ 0.0025** vs the DNS reference peak **≈ 0.0124** near t*≈9 -- **~80 % below**"
```

**Reference form.** The Re=1600 TGV dissipation peak (van Rees et al. 2011; the 2012 High-Order Workshop case C3.5) is a local maximum of -dE*/dt* at t* ~ 9, after which the curve decays. A measured peak should be an interior maximum, not the last sample.

**Impact.** The '-80 %' divergence figure in the summary table compares a still-rising endpoint value against a true DNS peak. The gap attributable to under-resolution at 16³ is therefore conflated with the fact that the curve has not turned over. An engineer reading the summary could conclude the solver under-predicts dissipation by 80%, when part of the deficit is that the run was measured before the peak.

**Recommended fix.** Detect whether the maximum is interior (peak index not the last sample) and label the reported value accordingly ('endpoint dissipation, curve still rising -- not a peak'). Consider extending the default horizon past t* = 12 so the curve turns over, or noting in the README that at 16³ the discrete curve does not develop a peak at all.

**Adversarial check.** print_utils.rs:39-42 is verbatim as quoted — a running maximum with no interior-maximum test — and the recorded run confirms the pathology: baseline.txt's final five rows show dissipation still increasing monotonically (0.00241123, 0.00242587, 0.00244021, 0.00245426, 0.00246801) with the summary line reporting 'peak dissipation 0.002468 at t* = 10.05', i.e. the last emitted sample. The reference form is right: the Re=1600 TGV dissipation rate is a local maximum of -dE*/dt* near t* ~ 9 (van Rees et al. 2011; 2012 High-Order Workshop C3.5), after which it decays, so a genuine peak must be an interior maximum. verification/README.md:111-112 then compares this endpoint value against the true DNS peak and attributes the whole '-80 %' gap to 16^3 under-resolution, conflating two effects.

> Evidence re-read: verification/dec_taylor_green_re1600_verification/print_utils.rs:30-55 (peak tracking and summary print); baseline.txt final rows and summary line; verification/README.md:110-113

---

### 1.32 [MINOR] dec_cylinder_verification's symmetry-breaking perturbation is 30% of the free stream while documented as 'small', with no sensitivity check on the reported St and C_d

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/dec_cylinder_verification/main.rs:67`
- **Auditor confidence:** likely

**Claim.** PERTURB_EPS = 0.3 injects a transverse velocity of 0.3*U on the centerline one diameter behind the cylinder. Both the module doc and the README describe it as 'a small ... transverse-velocity blob'. Neither the amplitude 0.3 nor the Gaussian half-width 0.75 D is justified, and the harness never checks that the reported limit-cycle St and C_d are independent of them.

**Code evidence.**

```
main.rs:65-68:
/// Transverse-velocity seed amplitude (fraction of `U`) and Gaussian half-width (diameters), placed
/// one diameter behind the cylinder on the centerline -- the symmetry-breaking trigger.
const PERTURB_EPS: f64 = 0.3;
const PERTURB_SIGMA: f64 = 0.75;

main.rs:22-25 (module doc): "The harness seeds a uniform stream plus a small, single-signed transverse-velocity blob just downstream of the cylinder"

main.rs:182: vv[2 * i + 1] = PERTURB_EPS * U * (-r2 / two_sigma_sq).exp();

Neither constant is env-overridable, unlike RE_D/CELLS_PER_D/LX_D/LY_D/STEPS/CFL/MERGE/STAIRCASE (main.rs:89-100).
```

**Reference form.** A symmetry-breaking perturbation used to trigger a supercritical Hopf bifurcation should be small enough that the saturated limit cycle is independent of it — conventionally a fraction of a percent of U. Independence must be demonstrated, since the standard evidence that the limit cycle is genuine is that St and C_d are invariant to the trigger amplitude over at least a decade.

**Impact.** At Re=100 the wake is only ~2x supercritical, so the saturated amplitude is not guaranteed to be trigger-independent at a 30% perturbation, particularly at 8 cells/D where the README documents the staircase body as sub-critical. The reported St = 0.171 (+4.3% vs Williamson) could carry a contribution from the trigger that is currently indistinguishable from discretization error.

**Recommended fix.** Make PERTURB_EPS environment-overridable like the other swept parameters and record a short sensitivity table (e.g. eps = 0.3, 0.03, 0.003) showing St and C_d converge. Then either reduce the default to the smallest amplitude that still triggers within the step budget, or document that 0.3 was verified to leave the limit cycle unchanged.

**Adversarial check.** Constants and doc verified: main.rs:65-68 defines PERTURB_EPS = 0.3 and PERTURB_SIGMA = 0.75 with a descriptive comment but no derivation; main.rs:182 applies `vv[2*i+1] = PERTURB_EPS * U * (-r2/two_sigma_sq).exp()`, so the seed peak is 0.3 U on the centerline one diameter behind the cylinder; main.rs:22-25 calls it 'a small, single-signed transverse-velocity blob'. Neither constant is env-overridable, unlike RE_D / CELLS_PER_D / LX_D / LY_D / STEPS / CFL / MERGE / STAIRCASE / CG_TOL / CG_MAX_ITER (main.rs:89-106), so a sensitivity sweep cannot even be run without recompiling, and no such sweep is recorded anywhere in the directory. The reference form is standard practice: at Re=100 the wake is a saturated supercritical Hopf limit cycle, and the accepted evidence that a reported St is physical rather than trigger-contaminated is invariance to the trigger amplitude over about a decade. Given that the harness has no gate at all (finding 1) and that the staircase variant is documented as sub-critical at this resolution, the concern that the reported St = 0.171 (+4.3 % vs Williamson) carries an unquantified trigger contribution is legitimate.

> Evidence re-read: verification/dec_cylinder_verification/main.rs:18-25 (module doc, 'small'), :64-68 (PERTURB_EPS 0.3, PERTURB_SIGMA 0.75), :86-106 (env-overridable set, excludes both), :170-185 (seed application); dec_cylinder_verification/README.md:114-122 (staircase sub-critical at 16/D)

---

### 1.33 [INFO] dec_cylinder_verification's committed run artifacts were produced by a crate and example name that no longer exist

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/dec_cylinder_verification/re100_16_resolved.txt:2`
- **Auditor confidence:** confirmed

**Claim.** The two committed .txt artifacts record the command `cargo run --release -p avionics_examples --example dec_cylinder_validation`. The crate is now deep_causality_cfd and the example is dec_cylinder_verification (Cargo.toml:163-165). The recorded commands cannot be run as written.

**Code evidence.**

```
re100_16_resolved.txt:1-3:
CELLS_PER_D=16 LX_D=16 LY_D=16 STEPS=4000 CFL=0.4 CG_TOL=1e-6 \
>   cargo run --release -p avionics_examples --example dec_cylinder_validation > re100_16_resolved.csv
     Running `target/release/examples/dec_cylinder_validation`

deep_causality_cfd/Cargo.toml:163-165:
[[example]]
name = "dec_cylinder_verification"
path = "verification/dec_cylinder_verification/main.rs"
required-features = ["std"]
```

**Reference form.** dec_cylinder_verification/README.md:105-110 gives the current, correct invocation form with -p deep_causality_cfd --example dec_cylinder_verification.

**Impact.** Reproduction friction only — the parameters are all present and the current command is documented in the README. But for a certification package the archived evidence should be regenerable verbatim, and these artifacts predate the crate reorganization, which also raises the question of whether the recorded numbers reflect the current code.

**Recommended fix.** Regenerate both artifacts with the current crate/example names, which also confirms the numbers still hold under the present code. Prefix each with the exact reproducing command.

**Adversarial check.** re100_16_resolved.txt lines 1-4 and re100_16_staircase.txt lines 1-4 both record `cargo run --release -p avionics_examples --example dec_cylinder_validation` and `Running target/release/examples/dec_cylinder_validation`. The crate is now deep_causality_cfd and the example is registered as dec_cylinder_verification (Cargo.toml [[example]] name/path). The directory README:105-111 does give the current, correct invocation form with -p deep_causality_cfd, so the impact is reproduction friction plus provenance doubt for a certification package, exactly as the finding scopes it. Info severity is right.

> Evidence re-read: verification/dec_cylinder_verification/re100_16_resolved.txt:1-4; re100_16_staircase.txt:1-4; dec_cylinder_verification/README.md:104-111 (current invocation)

---
