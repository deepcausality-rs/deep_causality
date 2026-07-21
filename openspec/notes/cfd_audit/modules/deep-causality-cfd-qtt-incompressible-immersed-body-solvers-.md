# deep_causality_cfd — QTT incompressible / immersed-body solvers and tensor-train observables (src/solvers/qtt/{incompressible_2d,immersed_2d,observe}.rs) with their supporting bridge (tensor_bridge/{projection,mask,operators,codec}.rs) and their two verification harnesses

**Production readiness: `not-ready`**

The core numerics are sound and I confirmed several non-trivial pieces against reference forms: the convection term is genuinely nonlinear and second-order (measured error 3.2e-3 -> 8.0e-4 -> 2.0e-4 at N=32/64/128, ratio 4), the discrete Leray projection is exactly consistent with its own divergence operator, and the spectral Poisson null-space mask is exactly the set where the eigenvalue vanishes. The defects are in the verification layer and in the immersed-body observable, and they are the kind an avionics lab cannot accept. I ran the shipped Taylor-Green harness at the documented `max_level 7` and it FAILS (observed order 0.02 vs the pinned 1.8 bound) because dt is held fixed across the refinement ladder; the committed "2nd-order" result exists only because the ladder stops one level before the explicit-Euler temporal floor dominates, and the N=64 order of 3.16 already exceeds the scheme's formal order. The convection gate's "non-zero amplitude" check is computed entirely from the analytic reference and is arithmetically incapable of failing. Most seriously, I measured the drag integral's provenance directly: 39.6% of the reported C_d = 23.76 comes from cells with chi <= 0.1 where |u| reaches 1.28 (free stream), and C_d scales from 7.70 to 47.27 as the mask smoothing width goes from 0.5 to 4 cells while the "no-slip" diagnostic stays pinned at ~4.2e-2 — so the "accuracy-vs-bond convergence" gate converges a quantity that is set by two untested numerical parameters and is not comparable to a drag coefficient. The eta sweep is non-monotone (17.39 -> 26.25 -> 21.40 over 16x), so the penalization limit the cited Angot/Bruneau/Fabrie theorem guarantees has not been reached or demonstrated.

- Files read: **25**
- Findings raised: **20** — surviving adversarial verification: **20** (refuted: 0)
- Surviving by severity: critical 5, major 10, minor 5
- Independently confirmed-correct items: **8**

> **⚠ NOT ADVERSARIALLY VERIFIED.** The verifier for this module terminated on a
> session limit before returning. Every finding below is a **single-auditor claim that
> has not been independently re-checked**. Treat as unconfirmed leads and verify at
> source before acting.

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Convection term u.grad(u) is a genuine nonlinear term and is second-order accurate | `src/solvers/qtt/incompressible_2d.rs:143-148; verification/qtt_taylor_green_verification/main.rs:126-131` | For 2-D Taylor-Green u = -cos(x)sin(y), v = sin(x)cos(y): (u.grad)u = u d_x u + v d_y u = -cos x sin y (sin x sin y) + sin x cos y (-cos x cos y) = -sin x cos x (sin^2 y + cos^2 y) = -(1/2) sin(2x). D |
| Spectral Poisson null-space handling is exactly the singular set, not an approximation | `src/tensor_bridge/projection.rs:158-173` | The centered-difference operator (u_{k+1}-u_{k-1})/(2*dx) has Fourier symbol i*sin(2*pi*k/N)/dx, so grad-of-grad has eigenvalue -sin^2(2*pi*k/N)/dx^2. The 2-D sum vanishes iff sin(2*pi*kx/Nx)=0 AND si |
| Brinkman penalization sign drives velocity toward the body velocity | `src/solvers/qtt/immersed_2d.rs:107-114, 127-128` | Angot, Bruneau & Fabrie (1999), Numer. Math. 81, 497-520: the penalized momentum equation is du/dt + ... = ... - (chi/eta)(u - u_body). |
| Penalization drag force direction and C_d denominator | `src/solvers/qtt/observe.rs:41, 67-72` | Kevlahan & Ghidaglia (2001) / Angot et al. (1999) volume-penalization force on the obstacle: F = (1/eta) * integral_{Omega_s} chi (u - u_s) dx. 2-D per-span nondimensionalization C_d = F' / (0.5 * rho |
| max_bond covers every internal bond of both trains | `src/solvers/qtt/observe.rs:205-215` | For a TT with d cores of shape [r_{i-1}, n_i, r_i] and r_0 = r_d = 1, the set of internal bonds is {r_1,...,r_{d-1}}. |
| ideal_gas_pressure_2d closure used by strip_pressure_force | `src/solvers/qtt/compressible/marcher_2d.rs:276-279` | 2-D Euler: E = rho*e + 0.5*rho*\|u\|^2 and p = (gamma-1)*rho*e, hence p = (gamma-1)*(E - 0.5*\|m\|^2/rho) with m = rho*u. |
| Gradient and Laplacian MPO assembly match the intended stencils | `src/tensor_bridge/operators.rs:105-107, 126-129` | Periodic centered first difference (S_- - S_+)/(2*dx); periodic second difference (S_+ + S_- - 2I)/dx^2. |
| preserved_drag_fraction guards its denominator | `src/solvers/qtt/observe.rs:159-165` | A ratio powered/unpowered is undefined when the denominator is zero or non-finite. |

## Findings

### 4.1 [CRITICAL] The convection "non-zero amplitude" gate is computed from the analytic reference alone and cannot fail

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/main.rs:141`
- **Auditor confidence:** confirmed

**Claim.** Gate 3's `conv_amp <= 0.0` check, whose failure message is "the nonlinear term is a no-op", is computed exclusively from the closed-form reference `-0.5*sin(2x)` and never touches the solver output. It is identically 0.5 for every possible solver behaviour, including a solver that returns zero.

**Code evidence.**

```
main.rs:135-143:
```
let mut max_err = 0.0f64;
let mut amp = 0.0f64;
for i in 0..n {
    let analytic = -0.5 * (2.0 * (i as f64 * dx)).sin();
    for j in 0..n {
        max_err = max_err.max((Into::<f64>::into(cs[i * n + j]) - analytic).abs());
        amp = amp.max(analytic.abs());
    }
}
Ok((max_err, amp))
```
`amp` depends only on `analytic`. print_utils.rs:183-186:
```
if conv_amp <= 0.0 {
    eprintln!("FAIL: convection signal amplitude is zero — the nonlinear term is a no-op");
    ok = false;
}
```
```

**Reference form.** A signal-amplitude check intended to prove the computed nonlinear term is non-trivial must be max|computed| (i.e. over `cs`), not max|reference|. Standard code-verification practice (Roache, Verification and Validation in Computational Science and Engineering): the discriminating quantity must be a function of the code under test.

**Impact.** The README (line 35) and print_utils render line 115 both present "signal amplitude 0.500" as evidence that "the nonlinear term is real and correct". It is evidence of nothing about the code. An engineer auditing the gate list sees four gates and counts this as one of them; in fact it is a constant comparison. (The sibling `conv_err` check does catch a zero convection, so the practical exposure is bounded — but a gate that cannot fail is exactly what a pre-certification audit must reject.)

**Recommended fix.** Compute `amp = amp.max(Into::<f64>::into(cs[i * n + j]).abs())` over the solver output, and rename the reported field to make clear which quantity it is. Keep the reference amplitude as a separate printed value if desired.

---

### 4.2 [CRITICAL] The Taylor-Green convergence-order gate fails at the documented max_level 7 because dt is held fixed across the refinement ladder

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/config.rs:28`
- **Auditor confidence:** confirmed

**Claim.** The ladder refines dx but holds dt = 0.01 constant. With explicit Euler (first order in time) the temporal error is an N-independent floor of ~6e-6, which the spatial error crosses just past the committed ladder's last rung. The reported order of 2.02/2.18 is a coincidence of where the ladder stops, and the N=64 order of 3.16 already exceeds the scheme's formal order — the signature of error cancellation, not convergence.

**Code evidence.**

```
config.rs:27-30:
```
pub const DT: f64 = 0.01;
/// Number of marched steps (horizon `t = DT·STEPS = 0.2`).
pub const STEPS: usize = 20;
```
No per-level dt refinement exists; build_config(l) passes `ft(DT)` unchanged at config.rs:74. Time integration is explicit Euler, incompressible_2d.rs:163-164:
```
let ustar = u.add(&ru.scale(self.dt))?.round(t)?;
```
I ran the shipped harness. `cargo run --release -p deep_causality_cfd --example qtt_taylor_green_verification 7`:
```
  N =  32   max_err = 5.316e-5   order = 2.18
  N =  64   max_err = 5.948e-6   order = 3.16
  N = 128   max_err = 5.868e-6   order = 0.02
FAIL: finest-pair observed order 0.019 below 1.8 (expected ~2)
```
```

**Reference form.** A spatial-order verification must either (a) refine dt with dx to keep the temporal error below the spatial error at every rung, or (b) use a temporal scheme of order >= the spatial order. For a scheme that is O(dt) + O(dx^2), the total error is E = C_t*dt + C_s*dx^2; with dt fixed the observed order log2(E_coarse/E_fine) tends to 0 as dx -> 0. Reference: Roache, "Code Verification by the Method of Manufactured Solutions", J. Fluids Eng. 124 (2002).

**Impact.** The README headline ("clean 2nd-order convergence to the analytic decay") and the MIN_ORDER = 1.8 gate are both artifacts of the ladder length, not properties of the solver. main.rs:32 and the README explicitly advertise `max_level` as a user-supplied extension of the ladder; using that documented option breaks the verification. An engineer who extends the ladder to confirm the claim gets a FAIL and no explanation, and an engineer who trusts the claim believes the marcher is second-order overall when it is first order in time.

**Recommended fix.** Refine dt with dx (e.g. dt = dt0 * (dx/dx0)^2 with STEPS scaled to hold t_final) so the temporal error stays subdominant, or state the temporal order explicitly and cap `max_level` at the level where the spatial error still dominates, with the reason recorded. Also report the temporal error floor so the observed order can be read correctly.

---

### 4.3 [CRITICAL] 39.6% of the reported penalization drag comes from cells outside the body, where the velocity is the free stream

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/observe.rs:25`
- **Auditor confidence:** confirmed

**Claim.** `penalization_integral` contracts the mask against the velocity over the entire periodic domain. The tanh mask has global support, so the far-field tail (chi ~ 1e-3 to 1e-1) multiplied by 1/eta = 62.5 and by the undisturbed free stream contributes a large fraction of the reported force. F is therefore not the penalization force on the body; it is that force plus a domain-wide free-stream bias.

**Code evidence.**

```
observe.rs:36-41:
```
let deficit = if a_body == R::zero() {
    mask.inner(a)?
} else {
    mask.inner(&a.add_scalar(R::zero() - a_body)?)?
};
Ok(deficit * cell_volume / eta)
```
`inner` is the full-domain train contraction — there is no restriction to the solid support. I decomposed the committed case (N=32, eta=0.016, bond<=24, 2-cell smoothing; my probe reproduces the baseline C_d = 23.7577 exactly):
```
  chi>0.9  (the 'no-slip' gate region)   cells=  21  C_d share = 1.9719 ( 8.3%)  max|u| = 0.0421
  0.5<chi<=0.9                           cells=  48  C_d share = 4.2608 (17.9%)  max|u| = 0.0757
  0.1<chi<=0.5                           cells=  76  C_d share = 8.1277 (34.2%)  max|u| = 0.5091
  chi<=0.1 (outside / far skirt)         cells= 879  C_d share = 9.3973 (39.6%)  max|u| = 1.2779
```
```

**Reference form.** Kevlahan & Ghidaglia (2001) and Angot, Bruneau & Fabrie (1999) define the penalization force as F = (1/eta) * integral over Omega_s (the SOLID domain) of chi*(u - u_s) dx. The integral is over the obstacle support, not over the whole domain against a globally-supported smooth kernel. The crate's own README (verification/qtt_cylinder_verification/README.md:78-79) cites Angot et al. as the method reference.

**Impact.** The quantity the crate labels C_d is dominated (74% by these bands, 40% from chi <= 0.1 alone) by regions that are not the body. The verification README's "Honest reading" section attributes the 23.8-vs-1.345 gap to blockage, smoothing skirt and transient; the free-stream tail contribution is not among them. An engineer told the number is "drag, inflated by known effects" will assume a bounded correction factor; in fact the majority of the number is a different quantity.

**Recommended fix.** Restrict the contraction to the solid support — either use a compactly-supported mask (clamp chi to 0 below a threshold at construction), or subtract the free-stream bias (1/eta)*u_inf*integral(chi) explicitly, or integrate only over cells where chi exceeds a documented cut. Then re-derive the C_d table and re-state which of the three README attributions survive. Report the band decomposition alongside the total so the provenance is visible.

---

### 4.4 [CRITICAL] The reported C_d scales linearly with the mask smoothing width, a purely numerical parameter, while the no-slip gate is insensitive to it

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/config.rs:34`
- **Auditor confidence:** confirmed

**Claim.** SMOOTH_CELLS = 2.0 is documented only as "Mask smoothing width in cells" with no justification, yet it sets the headline result: varying it from 0.5 to 4 cells with everything else fixed moves C_d from 7.70 to 47.27 (6.1x), essentially linearly in the width above 2 cells. The physical body radius is unchanged throughout. The no-slip diagnostic that gate 1 checks is invariant across the whole sweep.

**Code evidence.**

```
config.rs:33-34:
```
/// Mask smoothing width in cells.
pub const SMOOTH_CELLS: f64 = 2.0;
```
Used at config.rs:71: `let smoothing = ft(SMOOTH_CELLS) * dx;`. Measured sweep (eta = 0.016, bond <= 24, all else at committed values):
```
     cells        C_d    core max|u|
       0.5     7.7032      4.5057e-2
       1.0    12.3260      4.1021e-2
       2.0    23.7577      4.2192e-2
       3.0    35.8140      4.4475e-2
       4.0    47.2697      4.1934e-2
```
The smoothing is also large relative to the body: delta/radius = 0.417, and the mask area integral is 1.142x the nominal disc area.
```

**Reference form.** A drag coefficient is a property of the flow and the body geometry. In volume penalization the mask smoothing is a numerical regularization whose effect on the reported force must vanish as it is refined (delta -> 0 with the layer resolved), not grow linearly. Angot/Bruneau/Fabrie establish convergence in eta for a sharp characteristic function; a smoothed chi is a further approximation whose sensitivity must be quantified separately.

**Impact.** The published C_d = 23.7577 is a function of an undocumented tuning constant, and the smoothing sweep is never run. Gate 1 (no-slip, interior max|u| < 0.15) passes identically at every smoothing width in the table above, so it provides zero discrimination on the parameter that dominates the number gate 2 reports as "converged". The "accuracy-vs-bond" table therefore demonstrates convergence in the one parameter that does not matter.

**Recommended fix.** Add a smoothing-refinement study alongside the bond ladder and report whether C_d has a delta -> 0 limit; state the physical or rank-driven justification for delta = 2*dx in config.rs (currently the docstring gives none); and if no limit exists, say so in the README rather than listing the skirt as one of three bounded corrections.

---

### 4.5 [CRITICAL] The penalization parameter eta is set by the explicit-stability ratio, not by physics, and the reported C_d is non-monotone in it

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/config.rs:28`
- **Auditor confidence:** confirmed

**Claim.** ETA = 0.016 is chosen so that dt/eta = 0.25 (stated at config.rs:22-23), i.e. it is a time-step artifact. Sweeping eta over 16x with everything else fixed gives a non-monotone C_d with no limit: 17.39 (eta=0.128), 24.02, 26.25, 23.76 (the committed value), 21.40 (eta=0.008). The committed eta is a point on a curve, not a converged limit.

**Code evidence.**

```
config.rs:22-28:
```
/// Explicit-Euler time step (`dt/η = 0.25`, explicit-stable).
pub const DT: f64 = 0.004;
...
/// Brinkman penalization parameter (small → hard wall).
pub const ETA: f64 = 0.016;
```
Measured sweep at bond <= 24, 2-cell smoothing, dt fixed at 0.004:
```
     eta  dt/eta        C_d sqrt(eta*nu)     layer/dx  core max|u|
  0.1280   0.031    17.3875      0.08000        0.407    5.5860e-1
  0.0640   0.062    24.0175      0.05657        0.288    3.2832e-1
  0.0320   0.125    26.2464      0.04000        0.204    1.3472e-1
  0.0160   0.250    23.7577      0.02828        0.144    4.2192e-2
  0.0080   0.500    21.4031      0.02000        0.102    1.4532e-2
```
```

**Reference form.** Angot, Bruneau & Fabrie (1999), Numer. Math. 81, 497-520 prove the penalized solution converges to the no-slip solution as eta -> 0 with rate O(eta^{3/4}); the derived boundary-layer thickness is sqrt(eta*nu). Convergence of the derived force in eta is the property that licenses calling the penalization integral a drag. A non-monotone response over 16x in eta means that limit has not been demonstrated.

**Impact.** The audit brief's hypothesis is confirmed: eta's value directly sets both the reported interior max|u| (which swings from 5.59e-1 to 1.45e-2 across the sweep — note the committed NO_SLIP_FLOOR = 0.15 gate would FAIL at eta = 0.128 and eta = 0.064) and the reported C_d. The verification ladder varies only the bond cap and never eta, so it is structurally blind to the dominant error term. The number 23.7577 is not reproducible as a physical quantity — only as this exact parameter tuple.

**Recommended fix.** Run an eta ladder as a first-class gate and report whether C_d converges. Justify ETA in config.rs by the physics constraint (resolve the layer) rather than by dt/eta, or document explicitly that the value is stability-driven and that no eta-limit has been established.

---

### 4.6 [MAJOR] The Brinkman boundary layer is unresolved by a factor of seven, violating the precondition of the cited convergence theorem

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/immersed_2d.rs:46`
- **Auditor confidence:** confirmed

**Claim.** With the committed eta = 0.016 and nu = 0.05, the penalization layer thickness sqrt(eta*nu) = 0.0283 is 0.144 grid cells — the layer is seven times thinner than one cell and is not represented on the grid at all. The doc comment states only the explicit-stability constraint dt <~ eta and never mentions the resolution constraint.

**Code evidence.**

```
immersed_2d.rs:45-47:
```
/// `(ubx, uby)` (zero for a static wall), and the penalization parameter `eta` (small → hard wall;
/// explicit stepping needs `Δt ≲ η`).
```
Computed for the committed case (N=32, dx = 0.196350, nu = 0.05, eta = 0.016):
```
brinkman layer sqrt(eta*nu) = 0.028284 = 0.144 dx
```
The resolution requirement eta >~ dx^2/nu gives eta >~ 0.0385/0.05 = 0.77, versus the committed 0.016 — violated by ~48x.
```

**Reference form.** In volume penalization the numerical error is controlled only when the penalization layer is resolved: sqrt(eta*nu) >~ dx (equivalently eta >~ dx^2/nu). See Kevlahan & Ghidaglia (2001), Eur. J. Mech. B/Fluids 20, and Schneider (2005). Below this the discrete solution does not represent the analytic penalized solution, and the force integral does not converge to the drag.

**Impact.** This is the physical mechanism behind the non-monotone eta response above, and it is the missing fourth item in the README's three-item attribution of C_d = 23.8 vs DEC 1.345. Neither the solver doc, config.rs, nor the verification README states the constraint or reports that it is violated, so an engineer choosing eta for a new case has no guidance and will reproduce the same failure mode.

**Recommended fix.** Document the resolution constraint sqrt(eta*nu) >~ dx in the `QttImmersed2d::new` rustdoc next to the existing dt <~ eta note, and either enforce it as a warning/error or report sqrt(eta*nu)/dx as a run diagnostic so the regime is visible in every report.

---

### 4.7 [MAJOR] The QTT cylinder case runs at Re = 37.7 but is cross-referenced against a DEC result at Re = 100

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/README.md:81`
- **Auditor confidence:** confirmed

**Claim.** The committed parameters give Re = U*D/nu = 1.0 * 1.885 / 0.05 = 37.70. The README, config.rs and the stderr summary all present the DEC isolated-cylinder value at Re = 100 as the cross-reference, and the README's list of reasons for the discrepancy does not include the Reynolds-number mismatch.

**Code evidence.**

```
config.rs:20-38:
```
pub const NU: f64 = 0.05;
...
pub const U_INF: f64 = 1.0;
/// Cylinder radius as a fraction of the box length `2π`.
pub const RADIUS_FRAC: f64 = 0.15;
...
pub const DEC_CD_REF: f64 = 1.345;
```
diameter() at config.rs:51-53 = 2*0.15*2*pi = 1.8850. Computed: Re = 1.0*1.8850/0.05 = 37.70.
README.md:81: "DEC isolated-cylinder cross-reference: `verification/dec_cylinder_verification` (`C_d ≈ 1.345`, Re 100)."
README.md:46-51 lists exactly three causes: blockage, penalization-integral force, transient. Reynolds number is not among them. main.rs:45-54 prints nu, dt, steps, eta, U and grid — Re is never printed.
```

**Reference form.** A drag coefficient is only comparable between two computations at the same Reynolds number. For a 2-D circular cylinder, C_d(Re=37.7) is approximately 1.6-1.7 versus C_d(Re=100) approximately 1.35 (standard low-Re cylinder correlations, e.g. Sucker & Brauer 1975; Henderson 1995).

**Impact.** The one external anchor in the whole immersed-body verification is for a different flow. Even after the definitional problems above were corrected, the target number would be wrong by roughly 20%. The Reynolds number is nowhere computed, printed or asserted, so the mismatch is invisible to anyone reading the harness output or the baseline.

**Recommended fix.** Compute and print Re in the case banner and in the README; either retune nu (or U, or the radius) so the case is at Re = 100, or cite a DEC/literature C_d at Re = 37.7 instead. Add the Reynolds mismatch to the "Honest reading" list if the parameters are kept.

---

### 4.8 [MAJOR] The convection gate verifies a re-implementation of the convection term, not the shipped solver code path

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/main.rs:128`
- **Auditor confidence:** confirmed

**Claim.** The harness rebuilds gradient_x/gradient_y and open-codes `u ⊙ ∂ₓu + v ⊙ ∂ᵧu` inside the verification binary rather than calling the solver. A sign error, an axis swap, or a wrong operator inside `QttIncompressible2d::rate` would not be detected, because `rate` is never executed by this gate.

**Code evidence.**

```
main.rs:124-131 (the harness copy):
```
let gx = gradient_x::<FloatType>(l, l, dxf, &t)?;
let gy = gradient_y::<FloatType>(l, l, dxf, &t)?;
let dux = gx.apply(&u, &t)?;
let duy = gy.apply(&u, &t)?;
let conv_u = u
    .hadamard_rounded(&dux, &t)?
    .add(&v.hadamard_rounded(&duy, &t)?)?
    .round(&t)?;
```
incompressible_2d.rs:143-148 (the shipped code, a separate copy):
```
let dax = self.gx.apply(a, t)?;
let day = self.gy.apply(a, t)?;
let conv = u
    .hadamard_rounded(&dax, t)?
    .add(&v.hadamard_rounded(&day, t)?)?
    .round(t)?;
```
The public `rate_pair` (incompressible_2d.rs:100-106) exists and could have been called with nu = 0.
```

**Reference form.** Code verification requires that the artefact under test be the artefact executed. Duplicating the algorithm in the test breaks the link between the passing gate and the shipped implementation (Roache 2002; standard V&V practice).

**Impact.** The README states the gate checks "the solver's `u·∇u` (`u⊙∂ₓu + v⊙∂ᵧu`, the fused Hadamard the marcher uses)". It checks a copy. Since single-mode Taylor-Green's convective term is a pure gradient the projection removes (as the README itself argues), gate 1 also cannot detect a broken `rate` — so with this gate testing a copy, no gate in the harness exercises the shipped convection code.

**Recommended fix.** Drive the gate through the public API: call `rate_pair` on a solver built with nu = 0 and negate (rate = -(u.grad)u when nu = 0), or expose a dedicated `convection_pair` accessor. Delete the duplicated operator assembly from main.rs.

---

### 4.9 [MAJOR] The divergence gate measures an algebraic identity of the projector, not a solver property

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/solvers/qtt/observe.rs:191`
- **Auditor confidence:** confirmed

**Claim.** `divergence_residual` applies the same gx/gy operators the projector uses to build and invert its Laplacian, so div(project(u)) = 0 identically up to round-off and TT truncation. The measured 1e-14 is a construction property, but the README presents it as a verification result at machine precision.

**Code evidence.**

```
observe.rs:199-200:
```
let div = projector.divergence(u, v)?;
Ok(div.norm()?)
```
projection.rs:114-122 (project) removes grad-of-p where p solves the grad-of-grad Poisson symbol, using the SAME gx/gy:
```
let div = self.divergence(u, v)?;
let p = self.solve_poisson(&div)?;
...
let un = u.add(&self.gx.apply(&p, &self.trunc)?.scale(neg))?...
```
The author states the identity at projection.rs:153-155:
```
// The projection applies grad-of-grad (centered difference squared), eigenvalue -sin^2(2pik/N)/dx^2
// (the *consistent* operator, not the compact 5-point Laplacian) so div(project(u)) = 0 exactly.
```
Gate: print_utils.rs:24 `const DIVERGENCE_BOUND: f64 = 1.0e-6;` versus measured 1.06e-14 to 7.17e-13 — eight orders of margin.
```

**Reference form.** A meaningful incompressibility check must use a divergence operator independent of the one inverted by the projection (e.g. a staggered/MAC divergence, a compact 5-point form, or a spectral divergence), otherwise the result is an identity. The crate's own tests do this correctly elsewhere: incompressible_2d_tests.rs:57-68 uses an independently-coded dense `max_divergence`.

**Impact.** The Taylor-Green README states "the spectral Leray projection is exact to machine precision, not merely to an iterative tolerance" as a measured finding. It is a property of using the consistent operator, known a priori and documented in the source. Presenting it as one of four verification gates inflates the apparent evidence base. Note this also means the divergence column in the cylinder table (3.82e-1 at bond 4 down to 5.47e-14) is measuring only TT truncation, not projection quality.

**Recommended fix.** Either state in the README that the divergence residual is an identity check on truncation (which is a legitimate, useful thing to report) rather than a projection-correctness gate, or add a second residual computed with an independent divergence discretization and gate on that. Keep the note at projection.rs:153-155 as the citation for why the first one is exact.

---

### 4.10 [MAJOR] kinetic_energy omits the cell volume and is inconsistent with dec_kinetic_energy under the same Report series name

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/observe.rs:173`
- **Auditor confidence:** confirmed

**Claim.** The QTT kinetic energy is a bare Frobenius sum with no dx*dy weight, while the DEC sibling published under the identical `"kinetic_energy"` series name applies the Hodge star and therefore carries the metric. The two differ by a factor of dx*dy = 0.0385 at N=32 and 0.0024 at N=128, and the ratio changes with resolution.

**Code evidence.**

```
observe.rs:180-183:
```
let nu = u.norm()?;
let nv = v.norm()?;
let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
Ok(half * (nu * nu + nv * nv))
```
dec/diagnostics.rs:53-68 (the sibling):
```
let star_u = manifold.hodge_star_of(edge_form.as_slice(), 1);
...
let sum = edge_form.as_slice().iter().zip(star_u.as_slice().iter())
    .fold(R::zero(), |acc, (u, su)| acc + *u * *su);
Ok(sum * half)
```
Both are published as the same series: qtt_march_run.rs:754 `report.add_series("kinetic_energy", self.energy);` and march_run.rs:450 `report.add_series("kinetic_energy", self.energy);`
```

**Reference form.** Discrete kinetic energy on a uniform grid: E = (1/2) * integral (u^2 + v^2) dV = (1/2) * sum_ij (u_ij^2 + v_ij^2) * dx * dy. The cell volume is required for the quantity to be a physical energy and to be resolution-independent.

**Impact.** Two problems. (1) The crate README (line 59-61) explicitly invites picking between solver families behind one CfdFlow language and one Report type; an engineer comparing DEC and QTT energy series gets numbers differing by 26x at N=32 and 400x at N=128 with no signal that the definitions differ. (2) `MarchStop::Steady` at qtt_march_run.rs:545-554 tests `(e - prev_e).abs() < tol` on this unnormalized quantity, so the steady-state tolerance silently means something different at every resolution — a user-supplied tol calibrated at 32^2 is 16x too strict at 128^2.

**Recommended fix.** Multiply by dx*dy in `kinetic_energy` (the solver already carries dx/dy in the projector and the run context), or if the unweighted form is wanted for speed, rename the function and the series (`kinetic_energy_unweighted`) and change `MarchStop::Steady` to a relative criterion `|e - prev_e| / max(|e|, eps) < tol`.

---

### 4.11 [MAJOR] TT truncation breaks the mask's documented [0,1] invariant, and nothing clamps or checks it

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/immersed_2d.rs:45`
- **Auditor confidence:** confirmed

**Claim.** The mask is documented as "a `[0, 1]` volume fraction" in three places, but it is produced by TT-SVD truncation, which is not positivity-preserving. At the bond caps the cylinder harness actually uses, the mask goes negative: min chi = -1.780e-3 over 188 of 1024 cells at bond<=4, and -6.5e-5 over 84 cells at bond<=8. A negative chi turns the penalization from a sink into a source.

**Code evidence.**

```
immersed_2d.rs:45-46:
```
/// (spacings `dx`/`dy`, step `dt`, viscosity `nu`, round policy `trunc`) plus the body `mask`
/// (a `[0, 1]` volume fraction, see [`body_mask_2d`](crate::body_mask_2d)), the body velocity
```
mask.rs:9-10: "a mask field `χ_body ∈ [0, 1]` (1 inside the body, 0 outside)".
mask.rs:130-141 builds the exact tanh then hands it to `quantize_2d(&field, trunc)` (mask.rs:55) — TT-SVD under the caller's bond cap. No clamp anywhere. `QttImmersed2d::new` (immersed_2d.rs:52-73) accepts the train without validation.
Measured on the committed cylinder geometry (L=5, radius=0.9425, smoothing=2*dx):
```
bond<=    4: min=-0.001780 max=+0.981896  neg_cells= 188
bond<=    8: min=-0.000065 max=+0.991258  neg_cells=  84
bond<=   16: min=+0.000000 max=+0.991837  neg_cells=   0
```
The harness runs exactly these caps (main.rs:57 `let caps = [4usize, 8, 16, 24];`).
```

**Reference form.** In Brinkman penalization chi is a volume fraction, chi in [0,1]; the forcing -(chi/eta)(u-u_b) is dissipative only for chi >= 0. For chi < 0 the term has eigenvalue +|chi|/eta and amplifies the local velocity, and it contributes with the wrong sign to the force integral F = (1/eta) integral chi (u-u_b) dV.

**Impact.** At bond 4 the amplification rate is |chi|/eta = 0.00178/0.016 = 0.11 per unit time over 18% of the domain — small here, so the run does not visibly blow up, but the invariant the physics depends on is silently violated and would grow with a more aggressive cap or a sharper mask. It also means the bond<=4 and bond<=8 rows of the "accuracy vs bond" table are simulating a different (non-physical) body than the finer rows, so part of the reported "convergence" is the geometry converging, not the numerics.

**Recommended fix.** Clamp the dequantized mask to [0,1] before quantizing (or validate min/max in `QttImmersed2d::new` and return PhysicsError::PhysicalInvariantBroken when the train dequantizes outside [-tol, 1+tol]). Separately, build the mask at a fixed accurate truncation independent of the solver's bond cap so the bond ladder varies only the solver, as `interior_max_speed` already does at print_utils.rs:34-35.

---

### 4.12 [MAJOR] No validation of eta or of the explicit-stability / diffusive time-step limits; eta = 0 silently produces infinities

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/immersed_2d.rs:52`
- **Auditor confidence:** confirmed

**Claim.** `QttImmersed2d::new` and `QttIncompressible2d::new` accept any dt, nu and eta without checking the stability conditions their own doc comments state. eta = 0 yields neg_inv_eta = -inf with no error; dt > 2*eta or dt > dx^2/(4*nu) produce silent divergence. The DEC solver in the same crate has a CFL guard; the QTT solvers have none.

**Code evidence.**

```
immersed_2d.rs:52-73 constructs `Self { inner, mask, ubx, uby, eta }` with no validation of `eta`. immersed_2d.rs:107:
```
let neg_inv_eta = (R::zero() - R::one()) / self.eta;
```
incompressible_2d.rs:61-72 likewise stores dt/nu unvalidated. The stated constraint lives only in prose, immersed_2d.rs:46-47: "explicit stepping needs `Δt ≲ η`".
Contrast dec/dec_ns_solver/step.rs:138-146:
```
pub(super) fn cfl_check(&self, max_speed: R) -> Result<(), PhysicsError> {
    if max_speed > R::zero() {
        let advective_limit = self.cfl_advective * self.dx_min / max_speed;
```
```

**Reference form.** Forward-Euler stability for the penalization ODE du/dt = -(1/eta)u requires dt <= 2*eta. FTCS stability for 2-D diffusion requires dt <= dx^2/(4*nu); the cell-Peclet condition for centered convection requires |u|*dx/nu <= 2. Standard von Neumann analysis (Hirsch, Numerical Computation of Internal and External Flows, Ch. 8).

**Impact.** A user who picks eta smaller than dt/2 to get a harder wall — the natural move, since the docs say "small -> hard wall" — gets a silently divergent run rather than an error. eta = 0 gives NaN fields with no diagnostic. For the avionics consumer this is the difference between a solver that refuses an out-of-envelope configuration and one that returns numbers.

**Recommended fix.** Validate in both constructors: eta > 0 and finite; dt <= 2*eta (or a documented safety factor); dt <= dx^2/(4*nu) for the 2-D diffusive limit. Return PhysicsError with the violated bound and the offending values, mirroring the DEC `cfl_check` message format.

---

### 4.13 [MAJOR] The drag sanity bound of 100 is documented as "a physical O(1) drag coefficient" while gating a measured value of 23.76

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:20`
- **Auditor confidence:** confirmed

**Claim.** DRAG_SANITY_MAX = 100.0 is described as an O(1) bound but is 74x the physical isolated-cylinder value and was necessarily chosen after observing 23.76. main.rs states the gate checks that drag is "positive and `O(1)`", which the code does not do.

**Code evidence.**

```
print_utils.rs:19-20:
```
/// Pinned upper bound on a physical O(1) drag coefficient (sanity).
const DRAG_SANITY_MAX: f64 = 100.0;
```
print_utils.rs:108: `if !(finest.drag > 0.0 && finest.drag < DRAG_SANITY_MAX) {`
main.rs:19:
```
//! 3. **Physical drag** — the streamwise drag is positive and `O(1)`.
```
Measured finest value (baseline.txt:9): C_d = 23.7577. The README (line 21) says "positive and finite", which is accurate; main.rs contradicts it.
```

**Reference form.** A back-fitted bound is one whose value was selected after seeing the measured result. An O(1) sanity bound for a cylinder drag coefficient would be on the order of 2-5 (isolated-cylinder C_d is 1.0-2.0 over Re = 10^1..10^5).

**Impact.** The gate as written admits anything from 0 to 100, which for this quantity is no constraint. Combined with gate 2 (see below) the whole immersed-body verification reduces to: the number is stable in bond, positive, under 100, and the deep-core velocity is under 15% of free stream. None of these tests drag correctness. main.rs's `O(1)` claim tells a reader the opposite.

**Recommended fix.** Correct main.rs:19 to match the README and the code ("positive and finite"). Either drop the misleading "physical O(1)" wording from the constant's docstring and state that 100 is a loose non-divergence guard, or replace it with a bound derived from the case (e.g. a momentum-balance upper limit) and record the derivation.

---

### 4.14 [MAJOR] wall_heat_flux is a volume-integrated source term, not a flux, and the production path hardcodes a wall temperature of zero

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/observe.rs:81`
- **Auditor confidence:** confirmed

**Claim.** The function returns (1/eta)*integral(chi*(T_w - T))dV, whose units are [T]*[L]^2/[t], not a heat flux [W/m^2] and not a heat rate. It is not Fourier's law and involves no temperature gradient, no conductivity, and no surface normal. The one production call site passes T_w = 0 unconditionally, so the published number is proportional to the plasma temperature itself.

**Code evidence.**

```
observe.rs:92-94:
```
// Q = (1/η) ∫ χ_body (T_w − T) dV = −[(1/η) ∫ χ_body (T − T_w) dV].
let q = penalization_integral(mask, temp, t_wall, eta, dx * dy)?;
Ok(R::zero() - q)
```
The only production caller, qtt_march_run.rs:207-215:
```
if let Some((mask, eta)) = &self.wall
    && let Some(t_tr) = field.scalar("T_tr")
{
    ...
    let q = wall_heat_flux(mask, &t_tt, R::zero(), *eta, self.dx, self.dy)?;
    field.set_scalar("wall_heat_flux", Vec::from([q]));
}
```
`t_wall` is not a field of the body config: it appears nowhere in QttMarchConfig (grepped across src/, tests/, verification/, studies/ — the only occurrences are the `advance_scalar` parameter and this hardcoded zero).
```

**Reference form.** Fourier's law: q = -k * dT/dn [W/m^2], with n the wall-outward normal and k the thermal conductivity. A wall heat load is the surface integral of that. The implemented quantity is a Brinkman thermal-penalization source integral, which is a legitimate diagnostic but a different object with different units and no directional content.

**Impact.** The name and the published series key both read as a wall heat flux. For a re-entry TPS consumer this is the safety-critical quantity, and the sign convention here (positive = heat into the fluid, per observe_tests.rs:156) is the opposite of the TPS convention (positive = heat into the wall). Additionally T_w = 0 in a Kelvin field is physically absurd for a re-entry body and maximizes the computed magnitude; a real wall at 1500 K would give a substantially different number, and there is no way to configure it.

**Recommended fix.** Rename to `wall_penalization_heat_integral` (or document the units and sign convention prominently in the first line of the rustdoc), and plumb `t_wall` through `QttMarchConfigBuilder::body` so the wall temperature is a case parameter rather than a hardcoded zero. If a Fourier-law wall flux is wanted, compute it from the temperature gradient at the mask interface with an explicit conductivity.

---

### 4.15 [MAJOR] The bond-convergence gate compares two effectively identical computations against a bound eleven orders of magnitude looser

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:18`
- **Auditor confidence:** confirmed

**Claim.** Gate 2 checks only the relative change between the two finest rows (bond 16 and bond 24). The committed values are identical to 6 significant figures with |dC_d| = 1.89e-11, i.e. a relative change of ~8e-13 against a bound of 0.10. Once the cap exceeds the field's achieved rank, raising it changes nothing, so the gate compares a computation to itself.

**Code evidence.**

```
print_utils.rs:17-18:
```
/// Pinned drag-convergence bound: the relative change between the two finest bond caps.
const CONVERGENCE_BOUND: f64 = 0.10;
```
print_utils.rs:94-103:
```
if rows.len() >= 2 {
    let prev = &rows[rows.len() - 2];
    let rel = (finest.drag - prev.drag).abs() / finest.drag.abs().max(1e-12);
    if rel > CONVERGENCE_BOUND {
```
baseline.txt:8-9:
```
  bond <=  16   C_d = 23.7577   |dC_d| = 7.22e-3   ...
  bond <=  24   C_d = 23.7577   |dC_d| = 1.89e-11  ...
```
The gate does not check that |dC_d| decreases monotonically across the ladder, nor compare against any external value.
```

**Reference form.** A convergence gate should test that the sequence of successive differences decreases at the expected rate across the whole ladder, and should be bounded by a value commensurate with the observed differences. A bound 11 orders above the measured quantity constrains nothing.

**Impact.** Combined with the C_d definition problems above, this is the only quantitative gate on the immersed-body result and it measures saturation of the bond cap rather than accuracy. The README states "This convergence is the verification result" (line 36-37) — so the entire claimed verification rests on this check. A solver computing an entirely wrong but bond-saturated quantity passes.

**Recommended fix.** Gate on the monotone decrease of |dC_d| across all four rungs (as the Taylor-Green harness does for max_err at print_utils.rs:151-160), tighten CONVERGENCE_BOUND to something commensurate with the ladder, and additionally report the achieved bond dimension per row so a saturated cap is visible rather than being read as convergence.

---

### 4.16 [MINOR] Two comments in the spectral Poisson solver state formulas that contradict the code

- **Verification verdict:** pending verification
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/tensor_bridge/projection.rs:157`
- **Auditor confidence:** confirmed

**Claim.** Line 157 states the eigenvalue is -(2-2cos(2*pi*k/N))/Delta^2, but the code computes sin^2(2*pi*k/N)/Delta^2 — different functions (2-2cos(x) = 4 sin^2(x/2)). Line 169 states p_hat = rhs_hat/(-lambda), which with the stated lambda = -(lamx+lamy) means +rhs_hat/(lamx+lamy), but the code computes -rhs_hat/(lamx+lamy). The code is correct in both cases; the comments are wrong.

**Code evidence.**

```
projection.rs:157-171:
```
// λ_k = −(2 − 2cos(2πk/N))/Δ²; the periodic Laplacian eigenvalue (separable in 2-D).
for kx in 0..nx {
    let sx = (tau * from_usize::<R>(kx) / nxf).sin();
    let lamx = sx * sx / dx2;
    ...
        // ∇²p = rhs with λ = −(lamx+lamy): p̂ = rhŝ / (−λ).
        let inv = R::zero() - R::one() / (lamx + lamy);
        spec[idx] = Complex::new(spec[idx].re * inv, spec[idx].im * inv);
```
Line 157 also contradicts lines 153-155 in the same function, which correctly say "eigenvalue -sin^2(2pik/N)/dx^2 (the *consistent* operator, not the compact 5-point Laplacian)".
```

**Reference form.** Grad-of-grad with centered differences: symbol (i sin(2 pi k/N)/dx)^2 = -sin^2(2 pi k/N)/dx^2. The compact 5-point Laplacian: -(2-2cos(2 pi k/N))/dx^2. The code deliberately uses the former (so the projection is exactly consistent with its own divergence); line 157 names the latter.

**Impact.** The two comments describe a different, incompatible discretization from the one implemented, in the one place where the distinction is load-bearing (it is what makes div(project(u)) = 0 exactly). A reviewer checking the Poisson solve against the comment will conclude the code is wrong, or worse, will 'fix' it to match. The sign error at line 169 compounds this.

**Recommended fix.** Delete or correct line 157 to lambda_k = -sin^2(2*pi*k/N)/Delta^2, matching lines 153-155; correct line 169 to p_hat = rhs_hat/lambda = -rhs_hat/(lamx+lamy).

---

### 4.17 [MINOR] strip_pressure_force is an undirected pressure volume integral, dimensionally not a force

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/observe.rs:109`
- **Auditor confidence:** confirmed

**Claim.** The function computes integral(chi_strip * p)dV with no surface normal and no directional projection, so it has units of pressure*area = [N] in 2-D where a per-span surface force is [N/m], and it cannot distinguish an axial from a transverse pressure contribution. The docstring calls it a force and ties it to "the preserved aerodynamic drag the Jarvinen-Adams dataset measured".

**Code evidence.**

```
observe.rs:144:
```
Ok(strip.inner(&p_tt)? * dx * dy)
```
Docstring, observe.rs:101-103:
```
/// and the cell volume — `F = ∫ χ_strip · p dV`, no cut-cell surface or boundary-fiber
/// reconstruction. ... (the preserved aerodynamic drag the
/// Jarvinen–Adams dataset measured), **not** the forcing deficit.
```
```

**Reference form.** A pressure force on a body is F = -integral over the surface of p*n dA — a vector, requiring the outward normal. An axial force coefficient C_A is the axial component of that. A scalar volume integral of p against an indicator has neither the dimension nor the directionality.

**Impact.** Used alone the number is not a force. It is however consumed only through `preserved_drag_fraction` (observe.rs:155-166), a same-configuration ratio in which the geometric factor and the extra length scale cancel — so the practical exposure is limited, and the docstring does say "A same-configuration ratio, so the harness's common geometry biases cancel." The defect is that the intermediate is named and documented as a force, inviting absolute use.

**Recommended fix.** Rename to `strip_pressure_integral`, state the units explicitly in the rustdoc, and note that only the ratio through `preserved_drag_fraction` is meaningful. If an absolute axial force is needed, project against the strip's outward normal and integrate over the interface rather than the volume.

---

### 4.18 [MINOR] ideal_gas_pressure_2d silently falls back to a wrong formula if the 0.5 lift fails

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:277`
- **Auditor confidence:** confirmed

**Claim.** `R::from_f64(0.5).unwrap_or_else(R::one)` substitutes 1.0 for 0.5 on failure, silently computing p = (gamma-1)*(E - |m|^2/rho) — a physically wrong pressure with no error. Every other 0.5 lift in this code path uses `.expect(...)`.

**Code evidence.**

```
marcher_2d.rs:276-279:
```
pub fn ideal_gas_pressure_2d<R: CfdScalar>(rho: R, mx: R, my: R, energy: R, gamma: R) -> R {
    let half = R::from_f64(0.5).unwrap_or_else(R::one);
    (gamma - R::one()) * (energy - half * (mx * mx + my * my) / rho)
}
```
Contrast observe.rs:70 and 182, and mask.rs:87 and 129, which all use `.expect("0.5 lifts into every real field")`.
```

**Reference form.** p = (gamma-1)*(E - 0.5*|m|^2/rho). Substituting 1 for 0.5 doubles the subtracted kinetic energy and gives a pressure that is wrong by (gamma-1)*0.5*|m|^2/rho.

**Impact.** For f64/f32 the fallback is unreachable, so there is no current numerical exposure. It matters because `strip_pressure_force` calls this per cell (observe.rs:140) and the crate advertises precision as a swappable parameter (Float106 and other scalar types); a scalar type with a failing from_f64 would produce a silently wrong pressure field with no diagnostic. This is the one place in the audited path where a numeric-lift failure degrades to wrong physics rather than to an error.

**Recommended fix.** Change to `.expect("0.5 lifts into every real field")`, matching the pattern used consistently elsewhere in the crate.

---

### 4.19 [MINOR] The Taylor-Green README's file-layout table attributes code to config.rs that is not there

- **Verification verdict:** pending verification
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/qtt_taylor_green_verification/README.md:88`
- **Auditor confidence:** confirmed

**Claim.** The table says config.rs holds "the analytic-reference error/compression helpers, and the convection-operator check". config.rs contains none of these — the error and compression measurement live in print_utils.rs and the convection check lives in main.rs. config.rs's own module doc says it "only *describes* the case".

**Code evidence.**

```
README.md:88:
```
| `config.rs` | Case parameters, the `QttMarchConfigBuilder` case, the analytic-reference error/compression helpers, and the convection-operator check. |
```
config.rs is 85 lines and contains only NU/DT/STEPS/MAX_BOND, ft, spacing, decay, tg_u, tg_v, trunc, build_config. config.rs:8-9:
```
//! run (the CfdFlow march) and `print_utils.rs` renders + verifies — this file only *describes* the
//! case.
```
The convection check is `convection_operator_error` at main.rs:106-145; the error/compression measurement is `measure_one` at print_utils.rs:43-86.
```

**Reference form.** Documentation-code parity: a file-responsibility table must reflect where the code actually is.

**Impact.** Minor on its own, but it is the navigation aid an auditor uses to find the convection check — the gate that carries the most weight in this harness. Sending a reviewer to the wrong file makes the duplicated-implementation defect (reported separately) harder to notice.

**Recommended fix.** Correct the table: config.rs holds case parameters and the builder; main.rs holds the convection-operator check; print_utils.rs holds the analytic-reference error and compression measurement plus the gates.

---

### 4.20 [MINOR] The 32-squared grid is described as resolving the cylinder at 9.6 cells per diameter with a 4-cell smoothing skirt

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/main.rs:40`
- **Auditor confidence:** confirmed

**Claim.** The grid gives 9.6 cells across the cylinder diameter, of which the two-cell smoothing skirt occupies roughly 4, leaving about 5-6 cells of solid core (21 cells total have chi > 0.9 out of 1024). Calling this "resolves the smoothed cylinder" overstates it; the mask's effective area is already 1.142x the nominal disc.

**Code evidence.**

```
main.rs:40-41:
```
/// The grid: `2^L × 2^L` (32² — affordable, resolves the smoothed cylinder).
const L: usize = 5;
```
Measured: dx = 0.196350, D = 1.8850, cells across diameter = 9.60; smoothing delta/radius = 0.417; solid cells with chi > 0.9 = 21; mask_area = 3.1871 vs nominal_disc = 2.7906, ratio 1.142.
```

**Reference form.** Immersed-boundary and penalization studies of a circular cylinder typically use 20-40 cells per diameter at low Re for a converged force (e.g. Kevlahan & Ghidaglia 2001; Taira & Colonius 2007). At ~10 cells per diameter with a 4-cell transition the surface is not geometrically resolved.

**Impact.** Compounds the drag-magnitude problems above: an engineer reading "resolves the smoothed cylinder" will attribute the C_d discrepancy to the disclaimed effects rather than to under-resolution. No grid-refinement study exists for the cylinder case (the ladder varies bond, not L), so the resolution claim is unsupported.

**Recommended fix.** State the cells-per-diameter figure and the solid-cell count in the case banner; either add an L ladder alongside the bond ladder or change the comment to say the grid is chosen for cost and that no grid convergence has been established.

---
