# DEC-native incompressible Navier–Stokes rate kernel (`solvers/dec/dec_ns_rate.rs`) and its `FluidTheory` realization (`theories/incompressible_dec.rs`), with the DEC solver traits, marcher, and wrappers

**Production readiness: `needs-work`**

The numerical core is genuinely sound and I verified it against references rather than against itself: the RK4 tableau in `rk4_arrow.rs:26-33` is the exact classical tableau; `laplacian_of` composes dδ+δd in the right order with a codifferential that is the true discrete M-adjoint (`codifferential.rs:78-79`, δ = M⁻¹BM), making Δ_dR positive semi-definite so `−νΔ_dR` provably dissipates; the interior product implements Hirani's `(−1)^{k(D−k)}⋆(⋆ω∧X♭)`, which I derived independently from `i_X(⋆ω)=⋆(ω∧X♭)` and confirmed gives the correct Lamb vector sign; the Leray projector genuinely sits inside each RK4 stage (`step.rs:65-85`), matching the README. What blocks a pre-certification bar is traceability, not arithmetic. The module headline equation at `solvers/dec/mod.rs:12` carries the wrong viscous sign relative to the code, and lines 21-26 of the same file describe a Chorin split with first-order temporal splitting that the implementation abandoned — an engineer reading the module doc gets both the sign and the order of accuracy wrong. Every top-level doc (module, type, `eval_projected`, `eval_unprojected`, the theory, the README) states the convective term is `−i_u(du♭)`, but the code marches the skew-symmetrized `−½(G_ω u − G*_ω u)`, a deliberately different operator whose symmetric-part removal the Leray projector does not undo. The energy-budget CI gate at `energy_budget_tests.rs:141` asserts a quantity that the skew construction makes algebraically zero for every input, so it cannot fail. Finally, the `SolenoidalField` compile-time guarantee that the README sells has a public escape hatch (`with_lift`), and ν reaches the rate through `DecIncompressible::rate` with none of the validation `DecNsRate::new` performs.

- Files read: **33**
- Findings raised: **15** — surviving adversarial verification: **15** (refuted: 0)
- Surviving by severity: major 3, minor 11, info 1
- Independently confirmed-correct items: **8**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| RK4 Butcher tableau | `deep_causality_calculus/src/types/rk4_arrow.rs:26-33` | Classical RK4: k1=f(y), k2=f(y+h/2·k1), k3=f(y+h/2·k2), k4=f(y+h·k3), y_{n+1}=y_n+(h/6)(k1+2k2+2k3+k4). Butcher (1963); any standard ODE text (Hairer–Nørsett–Wanner I, Table 1.2). |
| Hodge–de Rham Laplacian assembled as Δ = dδ + δd in the correct order | `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:761-766 and deep_causality_topology/src/types/manifold/differential/laplacian.rs:48-60` | Laplace–deRham on k-forms: Δ = dδ + δd (Warner, Foundations of Differentiable Manifolds, ch. 6). For a 1-form: Δu = d(δu) + δ(du). |
| Viscous sign: −ν Δ_dR strictly dissipates energy | `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:774 and 515` | δ is the M-adjoint of d, so ⟨u, Δu⟩_M = ⟨δu,δu⟩ + ⟨du,du⟩ ≥ 0 (Δ_dR is PSD). On a flat torus Δ_dR = −∇² on 1-forms in Cartesian components, hence physical +ν∇²u = −ν Δ_dR u♭. |
| Interior-product sign convention (−1)^{k(D−k)}⋆(⋆ω∧X♭) yields the correct Lamb vector | `deep_causality_topology/src/types/manifold/differential/interior_product.rs:57 and 131-135` | From the Riemannian identity i_X(⋆ω) = ⋆(ω ∧ X♭) and ⋆⋆ = (−1)^{k(n−k)} on k-forms: i_X ω = (−1)^{k(n−k)} ⋆(⋆ω ∧ X♭). Cited in the module doc as Hirani, Discrete Exterior Calculus (Caltech 2003), §8. |
| Leray projection is applied to the RATE, inside every RK4 stage, not only to the field | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:65-85` | Leray projection P = I − ∇Δ⁻¹∇· ; marching the projected dynamics ∂u/∂t = P(RHS) has no operator-splitting error, unlike Chorin projection applied after an unprojected step (Chorin, Math. Comp. 22, 19 |
| Skew-symmetrized convective term is the exact M-adjoint construction and is energy-neutral by algebra | `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:591-616 and 643-669` | M-adjoint: ⟨Gx, u⟩_M = ⟨x, G*u⟩_M, so for diagonal M, (G*u)[j] = (Ge_j)ᵀMu / M[j]. Then ⟨u, ½(G−G*)u⟩_M = ½(⟨u,Gu⟩ − ⟨Gu,u⟩) = 0. |
| Spectral viscous eigenvalues | `deep_causality_cfd/src/solvers/dec/spectral_diffusion.rs:116` | Symbol of the 3-point second-difference operator −∂²/∂x² ≈ (−u_{j−1}+2u_j−u_{j+1})/h² is (2 − 2cos θ)/h², θ = 2πk/N. Since Δ_dR = −∇², Δ₁ has eigenvalues Σ_d (2 − 2cos(2πk_d/N_d))/h_d². |
| SolenoidalField has no Add/Mul and no public field constructor | `deep_causality_physics/src/quantities/fluid_dynamics/solenoidal_field.rs:56-59` | Type-state pattern: invariant enforced by making the only constructors the ones that establish it (Rust API guidelines; the crate's own claim at solenoidal_field.rs:16-22). |

## Findings

### 9.1 [MINOR] Energy-budget CI gate asserts a quantity that is algebraically zero by construction — the test cannot fail

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/dec/energy_budget_tests.rs:141`
- **Auditor confidence:** confirmed

**Claim.** The `b.convective().abs() < 1e-9` assertion is a round-off check on a quantity that is exactly zero by construction for every input, so it cannot discriminate a correct convective operator from an incorrect one. It is not vacuous as a regression on the presence of the skew wrapper, and the test's substantive gates (per-step energy monotonicity, march survival) are non-tautological. The test comment already states the neutrality is 'by construction'.

**Code evidence.**

```
energy_budget_tests.rs:113-120 doc: "stays energy-non-increasing every step and the convective power stays at zero to rounding — the skew-symmetrized convective term cannot inject energy."
energy_budget_tests.rs:141-145:
```
assert!(
    b.convective().abs() < 1e-9,
    "step {step}: convective power {} not energy-neutral",
    b.convective()
);
```
dec_ns_rate.rs:514: `let convective = R::zero() - m_inner(&conv);`
dec_ns_rate.rs:613-615 (`conv` is built as): `for (c, k) in ws.conv.iter_mut().zip(ws.adj_corr.iter()) { *c = half * (*c - *k); }`
deep_causality_topology/.../stencil/mod.rs:256-258 states it outright: "conv'(u) = ½[G_ω u − G*_ω u] with ω = du: exactly energy-neutral (`⟨u, conv'⟩_M = 0` identically)".
```

**Reference form.** For an M-adjoint pair, ⟨u, ½(G−G*)u⟩_M = ½(⟨u,Gu⟩_M − ⟨u,G*u⟩_M) = ½(⟨u,Gu⟩_M − ⟨Gu,u⟩_M) = 0 for every u, since the M-inner product is symmetric. `apply_convective_vector_adjoint` (stencil/mod.rs:287-294) is the exact transposed chain with diagonal star1/inv_star1, so G* is the exact adjoint of G, not an approximation.

**Impact.** An avionics engineer reading `skew_convection_keeps_energy_monotone` will believe the convective discretization's energy neutrality is under continuous regression test. It is not — that assertion would still pass if the convective operator were replaced with garbage, provided the same skew wrapper were applied. The genuine content of the test is the separate energy-monotonicity assertion at line 152, which is not tautological. The same vacuous comparison appears at dec_ns_rate_tests.rs:490-494 (`b_op.convective()` vs `b_sp.convective()`, both structurally zero, since the spectral option touches only the viscous term).

**Recommended fix.** Delete the convective assertion from `skew_convection_keeps_energy_monotone` and from `energy_budget_spectral_matches_operator`, or replace it with the non-tautological quantity: assert that the *unsymmetrized* gather `⟨u, G_ω u⟩_M` (i.e. `ws.conv` before the ½(c−k) fold, exposed as a diagnostic) is bounded by the expected O(h^p) adjoint defect. Correspondingly correct `energy_budget.rs:36-38`, which advertises `convective` as "its discrete residue is the aliasing diagnostic" — after the skew fix there is no residue and the field has no diagnostic value.

**Adversarial check.** The algebra is right. energy_budget (dec_ns_rate.rs:514) computes convective = −⟨u, conv'⟩_M with conv' = ½(G_ω u − G*_ω u) (dec_ns_rate.rs:610-615 fused, 671-678 generic). Since M is diagonal and (G*u)[j] = ⟨G e_j, Mu⟩/M[j], one gets ⟨u,G*u⟩_M = Σ_j u_j⟨G e_j, Mu⟩ = ⟨Gu, Mu⟩ = ⟨u,Gu⟩_M exactly, so the quantity is identically zero for every u (the m1[j]≈0 branch also contributes zero on both sides). The fused adjoint is verified as the exact transposed chain with diagonal star1/inv_star1 (stencil/mod.rs:287-294), so no discretization slack rescues the gate. No input can make the assertion fail. HOWEVER the auditor's framing overstates the defect: (a) the test itself says so — the in-test comment at energy_budget_tests.rs:137-138 reads 'The convective power is energy-neutral by construction at every visited state', so nothing is 'masquerading'; (b) the assertion is not vacuous as a code-change regression — deleting the skew wrapper (the dec-ns-stability fix) would make it fail, which is a real structural guard; (c) the test's headline property (energy monotonicity, line 151-154) and its survival of the march are non-tautological and are what the test name refers to. Severity 'critical / certification blocker' is not supportable for a redundant assertion in a test whose primary gate is sound.

> Evidence re-read: energy_budget_tests.rs:113-157 (doc, comment 137-138, assertions 141-145 and 151-154 read verbatim as cited); dec_ns_rate.rs:460-529 (energy_budget), 591-616, 623-679; deep_causality_topology/src/types/manifold/differential/stencil/mod.rs:251-296 (adjoint is exact transposed chain with star1/inv_star1); dec_ns_rate_tests.rs:486-497 (the secondary claim is also correct — both sides are structurally zero)

---

### 9.2 [MINOR] Module-level governing equation has the viscous term with the wrong sign relative to its own definition of Δ_dR

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/mod.rs:12`
- **Auditor confidence:** confirmed

**Claim.** solvers/dec/mod.rs:12 overloads `Δ`: it writes `+ ν Δ u♭` (Δ = ∇², correct) on the same line as the definition `Δ_dR = −∇²`, inviting a reader to substitute the subscripted operator into the unsubscripted slot and derive anti-diffusion. The fix is to write `− ν Δ_dR u♭` to match mod.rs:29 and dec_ns_rate.rs:7. It is a notation-clarity defect, not a stated sign error.

**Code evidence.**

```
solvers/dec/mod.rs:11-13:
```
//! ```text
//! ∂u♭/∂t = P( − i_u ω + ν Δ u♭ + g♭ ),   ω = d u♭,   Δ_dR = −∇²
//! ```
```
Contradicted by dec_ns_rate.rs:7 (`P(−i_u(du♭) − ν Δ_dR u♭ + g♭)`), dec_ns_rate.rs:47-49 ("on a flat torus the Hodge–de Rham Laplacian satisfies `Δ_dR = −∇²`, so the physical diffusion `+ν∇²u` enters as `−ν Δ_dR u♭`"), and by the code at dec_ns_rate.rs:774: `R::zero() - ws.conv[i] - nu * (ws.visc_a[i] + ws.visc_b[i])`.
```

**Reference form.** Incompressible NS momentum in rotational (Lamb) exterior-calculus form: ∂u♭/∂t = P(−i_u(du♭) − ν Δ_dR u♭ + g♭), where Δ_dR = dδ + δd is PSD and equals −∇² on a flat torus. Equivalently ∂u/∂t + ω×u = −∇(p+|u|²/2) + ν∇²u + g (Batchelor, An Introduction to Fluid Dynamics, §3.2).

**Impact.** This is the first equation an engineer opening the DEC solver module reads. As written it states the sign the crate's own regression test `viscous_sign_decays_energy_f64` exists to rule out. Anyone cross-checking an independent implementation against this doc will introduce an anti-diffusion sign error.

**Recommended fix.** Rewrite solvers/dec/mod.rs:12 as `∂u♭/∂t = P( − i_u ω − ν Δ_dR u♭ + g♭ ),   ω = d u♭,   Δ_dR = dδ + δd = −∇² on a flat torus`, so the glyph used in the equation is the one that is defined, and the sign matches dec_ns_rate.rs:774.

**Adversarial check.** The quoted line exists verbatim at solvers/dec/mod.rs:12. But the auditor's reading is not the only one, and probably not the intended one: the equation writes an unsubscripted `Δ`, and the trailing annotation `Δ_dR = −∇²` introduces a *different, subscripted* symbol. Read with Δ = ∇² (the ordinary vector Laplacian), `+ ν Δ u♭` is the correct physical diffusion and the annotation is the bridge to the implemented `−ν Δ_dR u♭`. Seven lines lower the same file (mod.rs:29) writes the RHS correctly as `−i_u(du♭) − ν Δ_dR u♭ + g♭`, and dec_ns_rate.rs:47-49 states the convention explicitly. So this is symbol overloading in one line of a module doc, not an asserted anti-diffusion sign. The auditor's reference form for rotational-form NS is itself correct.

> Evidence re-read: solvers/dec/mod.rs:11-13 (quoted line present exactly as cited) and mod.rs:29 ('the right-hand side `−i_u(du♭) − ν Δ_dR u♭ + g♭`'); dec_ns_rate.rs:6-7, 47-49; dec_ns_rate.rs:774 and 716/722 (implemented as `− nu * lap`, i.e. −ν Δ_dR)

---

### 9.3 [MAJOR] Module doc describes a Chorin split with first-order temporal splitting; the code marches the projected rate with no splitting

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/mod.rs:21`
- **Auditor confidence:** confirmed

**Claim.** `solvers/dec/mod.rs` states the march is an unprojected RK4 step followed by one projection, and that the resulting scheme is first order in time, and derives a validation policy from that. The implemented step projects inside every RK4 stage and has no splitting error, as `step.rs` and `dec_ns_rate.rs` both state. The module doc is stale and misstates the scheme's temporal order of accuracy.

**Code evidence.**

```
solvers/dec/mod.rs:21-26:
```
//! The march uses the **Chorin placement**: an unprojected `Rk4` step over
//! the whole-field state, then one gauge-fixed Leray projection back into
//! the [`SolenoidalField`] type-state, then the
//! CFL guard. The splitting at the projection is first order in time
//! regardless of the integrator's interior order; validation therefore
//! gates on spatial refinement at fixed CFL.
```
Directly contradicted by step.rs:6-11: "`Rk4` over the **projected rate** (the Leray projector sits inside each stage ... the marched ODE is exactly the projected dynamics, with no splitting error)" and by dec_ns_rate.rs:9-13: "The projector sits **inside** the rate ... there is no splitting error and no per-step energy discard."
Code confirms step.rs:65-69: `let rk4 = Rk4::new(self.dt, |s: &VelocityOneForm<R>| { match self.rate.eval_projected(s, &self.cg_options) {`
```

**Reference form.** Chorin projection (Chorin, Math. Comp. 22:745, 1968) applies the projector after an unprojected advance and is first-order accurate in time. Marching ∂u/∂t = P(RHS) directly retains the integrator's order. The two are different schemes with different temporal orders.

**Impact.** An engineer sizing dt or planning a verification campaign from this module doc will assume first-order temporal accuracy and will design a spatial-only refinement study, as the doc explicitly instructs ("validation therefore gates on spatial refinement at fixed CFL"). The actual scheme supports a temporal refinement study, and any published order-of-accuracy claim derived from this paragraph is wrong. For a pre-certification evidence package, the documented scheme and the implemented scheme must be the same scheme.

**Recommended fix.** Replace solvers/dec/mod.rs:21-26 with the description already in step.rs:6-13 (projector inside each RK4 stage; four CG solves per step plus one near-free re-entry solve; no splitting error), and remove the derived claim about gating validation on spatial refinement, or re-justify it on grounds that survive the corrected premise.

**Adversarial check.** solvers/dec/mod.rs:21-26 is present verbatim and says exactly what the auditor quotes, including the derived validation policy. The implementation contradicts it directly: step.rs:65-84 constructs `Rk4::new(self.dt, |s| self.rate.eval_projected(s, ...))`, i.e. the Leray projector is evaluated inside every stage, and step.rs:6-13 plus dec_ns_rate.rs:9-14 both state there is no splitting error. There is no unprojected Rk4 run anywhere in the step path; the only post-march projection (step.rs:104-113) is the type-state re-entry, documented as a near-no-op on an already divergence-free input. The auditor's characterization of Chorin (1968) and of the order retention of marching P(RHS) is correct.

> Evidence re-read: solvers/dec/mod.rs:21-26 verbatim; dec_ns_solver/step.rs:6-13 (module doc), 30-47 (bind sequence doc), 57-113 (implementation); dec_ns_rate.rs:9-14

---

### 9.4 [MAJOR] Every headline doc states the convective term is `−i_u(du♭)`; the code marches the skew-symmetrized `−½(G_ω u − G*_ω u)`, a different operator the projector does not reconcile

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:681`
- **Auditor confidence:** confirmed

**Claim.** The module doc, the type doc, `eval_projected`, `eval_unprojected`, the solver module doc, the `FluidTheory` realization, and the README all state the rate's convective term is `−i_u(du♭)`. The implemented term is `−½(G_ω u − G*_ω u)` where G* is the exact M-adjoint. These differ by `+½(G_ω + G*_ω)u`, the symmetric part, which is not a discrete gradient and is therefore NOT removed by the Leray projector. The marched equation is a stabilized/regularized NS, not the stated one.

**Code evidence.**

```
dec_ns_rate.rs:681: `/// Evaluates the **unprojected** assembly `−i_u(du♭) − ν Δ_dR u♭ + g♭`.`
dec_ns_rate.rs:35: `/// The rate field `u♭ ↦ −i_u(du♭) − ν Δ_dR u♭ + g♭``
dec_ns_rate.rs:418: `/// Evaluates `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)`: the projected rate the`
incompressible_dec.rs:42: `/// `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)` at the ambient `ν`.`
But the actual assembly, dec_ns_rate.rs:753 then 774: `Self::fill_convective_skew_fused(t, ws, u_slice);` ... `R::zero() - ws.conv[i] - nu * (...)`
and dec_ns_rate.rs:610-615:
```
let half = R::from_f64(0.5)...;
for (c, k) in ws.conv.iter_mut().zip(ws.adj_corr.iter()) {
    *c = half * (*c - *k);
}
```
Only the two private helpers document this honestly (dec_ns_rate.rs:582-590, 618-622).
```

**Reference form.** Rotational-form NS: ∂u♭/∂t = P(−i_u(du♭) − νΔ_dR u♭ + g♭). Leray annihilates exact 1-forms (gradients) only. The added term ½(G+G*)u is symmetric, not exact, so P(½(G+G*)u) ≠ 0 in general — it is a genuine modification of the marched dynamics, vanishing only as the discrete adjoint defect ‖G+G*‖ → 0 under refinement.

**Impact.** Two consequences. (1) An engineer implementing an independent cross-check from the documented equation will not reproduce this code's rate, and will misattribute the discrepancy to a bug. (2) More seriously, for certification the deviation from the target PDE must be a declared, bounded modelling choice with a stated order. Here it is undeclared at every level a reader is likely to consult, including the README's "Calculus-based: the DEC-native Navier-Stokes solver" section. The change was made for a real reason (an energy-injection instability measured 2026-06-12, per dec_ns_rate.rs:588-590), which makes it all the more important to document at the headline.

**Recommended fix.** Amend the module doc (dec_ns_rate.rs:6-7), the type doc (line 35), `eval_projected` (line 418), `eval_unprojected` (line 681), `incompressible_dec.rs:42`, and the README section to state the marched convective operator as `conv'(u) = ½(G_ω u − G*_ω u)` with `G_ω : x ↦ i_x(du♭)`, noting that it is consistent with `i_u(du♭)` at the discretization order and that the difference is a deliberate energy-neutrality stabilization. State the measured order of `½(G+G*)u` so the modelling error is bounded, not merely asserted to be consistent.

**Adversarial check.** All cited doc sites read as quoted (dec_ns_rate.rs:35, 418, 681; incompressible_dec.rs:42; solvers/dec/mod.rs:29). Both marching paths assemble the skew form: eval_unprojected_fused calls fill_convective_skew_fused (dec_ns_rate.rs:753 → 610-615) and the generic path calls convective_skew_generic (704 → 671-678). A repo-wide grep for 'skew' in deep_causality_cfd/src and README.md returns hits only in private helper docs, internal comments and field comments (lines 122, 476, 498, 582-591, 618-623, 702-704, 753) — nothing at any public doc level and nothing in the README. The math is right: the deviation from the documented operator is ½(G+G*)u, the M-symmetric part, which is not an exact 1-form and is therefore not annihilated by Leray; it vanishes only with the discrete adjoint defect under refinement. Worth stating in mitigation (does not change the verdict): the skew-symmetric convective form is a standard, consistent discretization choice in the incompressible-NS literature, and the private helper doc at dec_ns_rate.rs:582-590 does record the rationale and the 2026-06-12 measurement — the defect is that this never surfaces at any level a reader or an independent cross-checker consults.

> Evidence re-read: dec_ns_rate.rs:35-36, 418-419, 681, 591-616, 623-679, 704, 753; incompressible_dec.rs:42; solvers/dec/mod.rs:29; `grep -rn skew deep_causality_cfd/src deep_causality_cfd/README.md` (12 hits, all private/internal)

---

### 9.5 [MAJOR] `SolenoidalField`'s advertised compile-time guarantee has a public escape hatch: `with_lift` writes arbitrary values into a projected field

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_physics/src/quantities/fluid_dynamics/solenoidal_field.rs:219`
- **Auditor confidence:** confirmed

**Claim.** The type doc claims "there is no path that re-wraps a modified tensor" and the README claims the type-state "rejects time-stepping an unprojected field at compile time". `with_lift` and `constrain_edges` are `pub fn` on the public type, take `self` and return `Self`, and overwrite arbitrary coefficients. Any external caller holding one projected field can produce an arbitrarily non-solenoidal value that still has type `SolenoidalField` and is accepted by `DecNsSolver::step`.

**Code evidence.**

```
solenoidal_field.rs:16-22 (module doc): "\"You cannot time-step an unprojected field\" is thereby a compile-time fact: the type has no other constructor, and **no arithmetic** ... there is no path that re-wraps a modified tensor."
solenoidal_field.rs:213-232:
```
/// Crate-internal wall-bounded path: set the prescribed tangential wall
/// values ...
pub fn with_lift(self, lift: &[(usize, R)]) -> Self {
    if lift.is_empty() { return self; }
    let mut data = self.field.into_vec();
    for &(e, value) in lift { data[e] = value; }
```
solenoidal_field.rs:198: `pub fn constrain_edges(self, edges: &[usize]) -> Self {`
Both doc comments say "Crate-internal", but neither is `pub(crate)`. The type is re-exported publicly: quantities/mod.rs:57 `pub use fluid_dynamics::solenoidal_field::SolenoidalField;`
README.md:41-42: "the `SolenoidalField` type-state rejects time-stepping an unprojected field at compile time."
```

**Reference form.** A type-state invariant is a compile-time fact only when every public operation preserves it. Here the invariant is `‖δu‖ ≈ 0` to CG tolerance; `with_lift(&[(0, 1e9)])` violates it while preserving the type.

**Impact.** The headline safety property the crate sells is not enforced. `solver.step(&s.with_lift(&[(0, 1e9)]))` compiles and runs, marching a field with large divergence. Additionally both functions index `data[e]` with no bounds check (lines 204, 226), so an out-of-range edge index panics rather than returning an error — a runtime abort in a marching loop.

**Recommended fix.** Change both to `pub(crate)` — their own doc comments already say "Crate-internal", and the only in-tree callers are step.rs:121-123 within the same workspace, which would need a `pub(crate)`-visible seam or a dedicated re-projecting constructor. If they must stay public, rename them to signal the invariant break (e.g. `reproject_with_lift`) and have them re-run the constrained projection, and add bounds checks returning `PhysicsError::DimensionMismatch`. Soften README.md:41-42 to describe what is actually guaranteed (no arithmetic, no public field constructor).

**Adversarial check.** Every cited fact checks out. solenoidal_field.rs:17-22 claims 'there is no path that re-wraps a modified tensor'. `constrain_edges` (line 198) and `with_lift` (line 219) are both `pub fn`, both take `self` and return `Self`, both call `self.field.into_vec()`, mutate, and re-wrap into `Self { field }` — the exact re-wrapping path the module doc denies. Both doc comments open with 'Crate-internal' but neither is `pub(crate)`. The type is publicly re-exported and the README paragraph quoted ('the `SolenoidalField` type-state rejects time-stepping an unprojected field at compile time') is present. `with_lift(&[(0, 1e9)])` yields a value of type SolenoidalField with arbitrary divergence that `DecNsSolver::step` accepts. The unchecked indexing is real too: `data[e] = R::zero()` (line 204) and `data[e] = value` (line 226) panic on an out-of-range edge index. Note `constrain_edges` only writes zeros, but zeroing arbitrary edges also breaks the divergence-free invariant, so the auditor's inclusion of it stands.

> Evidence re-read: deep_causality_physics/src/quantities/fluid_dynamics/solenoidal_field.rs:17-22 (module doc), 36-55 (compile_fail doctests, which cover only field construction and Add), 189-232 (constrain_edges/with_lift, pub, unchecked indexing at 204 and 226); deep_causality_cfd/README.md 'Three Solver Paradigms' section; dec_ns_solver/step.rs:121-123 (the only intended callers)

---

### 9.6 [MINOR] `DecIncompressible::rate` forwards ambient ν into the rate with none of the validation `DecNsRate::new` performs — negative or NaN ν silently produces anti-diffusion

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/theories/incompressible_dec.rs:50`
- **Auditor confidence:** confirmed

**Claim.** `DecNsRate::new` rejects non-finite and negative ν (dec_ns_rate.rs:156-165) and `set_nu`'s doc places the burden on the caller. `DecIncompressible::rate` is that caller and validates nothing; `Ambient::new` and `Ambient::set_nu` validate nothing either. A negative ambient ν flips `−νΔ_dR` to `+νΔ_dR`, i.e. anti-diffusion, and the march grows energy without any error being raised.

**Code evidence.**

```
incompressible_dec.rs:45-52:
```
fn rate(&self, state: &VelocityOneForm<R>, ambient: &Ambient<R>) -> Result<VelocityOneForm<R>, PhysicsError> {
    self.rate.set_nu(*ambient.nu());
    self.rate.eval_projected(state, &self.opts)
}
```
dec_ns_rate.rs:410-416: `/// The caller guarantees `ν` is finite and non-negative (the construction invariant).` ... `pub fn set_nu(&self, nu: R) { self.nu.set(nu); }`
dec_ns_rate.rs:161-165 (the invariant that is bypassed):
```
if nu < R::zero() {
    return Err(PhysicsError::PhysicalInvariantBroken(
        "DecNsRate: viscosity cannot be negative".into(),
```
types/ambient.rs:25-31: `pub fn new(nu: R, freestream: R, body_force: ...) -> Self { Self { nu, freestream, body_force } }` — no checks.
Same gap in solvers/dec/marcher.rs:29: `self.rate().set_nu(*ambient.nu());`
```

**Reference form.** Kinematic viscosity ν ≥ 0 is a physical invariant (second law); ν < 0 makes the heat-type operator ill-posed and the initial-value problem unstable at every wavenumber. The crate's own `DecNsRate::new` encodes this at construction; the theory seam must not weaken it.

**Impact.** The class doc at dec_ns_rate.rs:38-45 explicitly sells construction-time validation as the reason per-step evaluation is safe. That guarantee does not survive the documented `FluidTheory` seam, which is the crate's advertised extension point for coupling stages and dynamic-law counterfactuals (`ν(T)` feedback per the same doc). A coupling stage computing ν from a temperature field that transiently goes negative yields silently unphysical, energy-growing results rather than an error. The tests for this seam (incompressible_dec_tests.rs) never exercise a bad ν.

**Recommended fix.** Make `DecNsRate::set_nu` fallible (`-> Result<(), PhysicsError>`) applying the same `is_finite` / `>= 0` checks as `new`, and propagate from `DecIncompressible::rate` (which already returns `Result`) and from `Marcher::advance`. Alternatively validate in `Ambient::new`/`set_nu`. Add a test that `FluidTheory::rate` with `Ambient::new(-0.01, ...)` returns `PhysicalInvariantBroken`.

**Adversarial check.** incompressible_dec.rs:45-52 reads exactly as quoted: `self.rate.set_nu(*ambient.nu())` then `eval_projected`, with no checks. `Ambient::new` (ambient.rs:25-31) and `Ambient::set_nu` (49-51) validate nothing. `DecNsRate::new` does reject non-finite (dec_ns_rate.rs:156-160) and negative (161-165) ν. `set_nu` (414-416) is a bare `Cell::set` whose doc places the burden on the caller (410-413). marcher.rs:29 has the same gap. I additionally confirmed no downstream guard catches it: cfl_check's diffusive branch is gated on `nu > R::zero()` (step.rs:151), so a negative ν skips the CFL check entirely and marches anti-diffusion silently. The physics reference (ν ≥ 0; ν < 0 makes the heat operator ill-posed at every wavenumber) is correct. Severity: the failure requires a caller to supply a physically invalid ν, and set_nu's doc does state the contract explicitly, so this is a hardening gap in a documented-contract API rather than a latent wrong-answer path in normal use.

> Evidence re-read: theories/incompressible_dec.rs:38-53; types/ambient.rs:22-62; dec_ns_rate.rs:156-165, 404-416; solvers/dec/marcher.rs:24-31; dec_ns_solver/step.rs:150-167 (diffusive CFL guard skipped when nu <= 0); tests/theories/incompressible_dec_tests.rs (69 lines, no bad-ν case)

---

### 9.7 [MINOR] `with_spectral_diffusion` is silently ignored on the generic-assembly path

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:707`
- **Auditor confidence:** confirmed

**Claim.** `with_spectral_diffusion()` succeeds and sets `self.spectral = Some(...)`, but neither `eval_unprojected`'s generic branch nor `energy_budget`'s generic branch consults `self.spectral`. A rate configured with both `with_generic_assembly()` and `with_spectral_diffusion()` silently evaluates the operator Laplacian, with no error and no diagnostic.

**Code evidence.**

```
dec_ns_rate.rs:694-707 (generic branch of `eval_unprojected` — no mention of `self.spectral`):
```
if let Some(engine) = &self.engine {
    return self.eval_unprojected_fused(engine, u);
}
...
let lap = self.manifold.laplacian_of(u_slice, 1);
```
dec_ns_rate.rs:497-501 (generic branch of `energy_budget`, same omission):
```
} else {
    let conv = self.convective_skew_generic(u);
    let mut lap = self.manifold.laplacian_of(u_slice, 1).into_vec();
```
Contrast the fused paths, which do check: line 754 `if let Some(spectral) = &self.spectral {` and line 471-489.
Both builders are `pub` and composable: line 391 `pub fn with_spectral_diffusion(mut self) -> Result<Self, PhysicsError>` and line 399 `pub fn with_generic_assembly(mut self) -> Self`.
```

**Reference form.** A builder that accepts a configuration must either honour it or reject the combination. Silently discarding a requested numerical method is a traceability defect: the run does not compute what the caller configured.

**Impact.** A cross-validation run comparing the spectral viscous evaluation against the generic compositional oracle — exactly the stated purpose of `with_generic_assembly` ("the equivalence oracle", line 398) — would compare the operator path against the operator path and report perfect agreement, falsely validating the spectral option. The existing test `energy_budget_spectral_matches_operator` (dec_ns_rate_tests.rs:468-497) uses the fused path for both and so does not catch this.

**Recommended fix.** Either wire `self.spectral` into the two generic branches (dec_ns_rate.rs:707 and 499), or make the combination an error: have `with_generic_assembly` return `Result` and reject when `self.spectral.is_some()` (and vice versa in `with_spectral_diffusion`). Document whichever choice is made on both builders.

**Adversarial check.** Verified in both branches. eval_unprojected's generic path (dec_ns_rate.rs:698-725) calls `self.manifold.laplacian_of(u_slice, 1)` with no reference to `self.spectral`; energy_budget's generic branch (497-502) does the same. The fused paths do consult it (754-759 and 477-489). Both builders are `pub` and freely composable — `with_spectral_diffusion` (391-394) sets `self.spectral`, `with_generic_assembly` (396-402) sets `self.engine = None`, neither inspects the other, and neither errors on the combination in either order. The impact claim is right: a spectral-vs-generic-oracle cross-validation would silently compare the operator Laplacian against itself. The cited existing test (dec_ns_rate_tests.rs:467-498) indeed uses the fused path on both sides, so it does not cover this.

> Evidence re-read: dec_ns_rate.rs:384-402 (both builders), 470-502 (energy_budget both branches), 687-732 (eval_unprojected generic branch), 738-790 (fused branch checks spectral at 754); tests/solvers/dec/dec_ns_rate_tests.rs:464-498

---

### 9.8 [MINOR] Type doc claims `eval_projected` is infallible `Fn(&S) -> S`; it returns `Result` and the solver needs a deferred-error cell precisely because it is fallible

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:42`
- **Auditor confidence:** confirmed

**Claim.** The type doc justifies its construction-time validation by stating that it makes `eval_projected` infallible with signature `Fn(&S) -> S`, composing directly with `Rk4`. The method returns `Result<VelocityOneForm<R>, PhysicsError>`, and `step.rs` wraps it in an error-parking `Cell` to satisfy `Rk4`. The infallible method is `eval_unprojected`; the doc names the wrong one.

**Code evidence.**

```
dec_ns_rate.rs:38-45:
```
/// body-force edge count matching the lattice, `ν` finite and
/// non-negative — so that [`eval_projected`](Self::eval_projected) is **infallible**
/// (`Fn(&S) -> S`) and composes directly with
/// `deep_causality_calculus::Rk4`.
```
dec_ns_rate.rs:424-428:
```
pub fn eval_projected(
    &self, u: &VelocityOneForm<R>, opts: &HodgeDecomposeOptions<R>,
) -> Result<VelocityOneForm<R>, PhysicsError> {
```
step.rs:58-63 and 70-72 (the workaround the doc denies is needed): "`Rk4` requires an infallible `Fn(&S) -> S`, so a CG failure inside a stage parks its error in the deferred slot and yields a zero rate" ... `Err(e) => { deferred.set(Some(e)); ... }`
dec_ns_rate.rs:687 (the actually-infallible one): `pub fn eval_unprojected(&self, u: &VelocityOneForm<R>) -> VelocityOneForm<R> {`
```

**Reference form.** The doc's own stated contract: construction-time validation removes per-step failure modes. That is true for the pure operator assembly and false for the CG projection solve, which can legitimately fail to converge — as `eval_projected`'s own `# Errors` section (lines 421-423) admits.

**Impact.** The doc invites a reader to conclude the projected rate cannot fail at run time, which is the opposite of the design: on wall-bounded and immersed-body lattices the constrained CG is the most likely per-step failure and the deferred-error machinery in step.rs exists for it. A consumer writing their own integrator around this rate would omit the error path.

**Recommended fix.** Change the reference at dec_ns_rate.rs:42 to `eval_unprojected`, and add a sentence stating that `eval_projected` remains fallible because of the projection CG, pointing at the deferred-error adaptation in `DecNsSolver::step`.

**Adversarial check.** dec_ns_rate.rs:37-45 states verbatim that construction-time validation is done '...so that [`eval_projected`](Self::eval_projected) is **infallible** (`Fn(&S) -> S`) and composes directly with `deep_causality_calculus::Rk4`'. The signature at 424-428 returns `Result<VelocityOneForm<R>, PhysicsError>` and its own `# Errors` section (421-423) documents CG non-convergence. step.rs:58-84 implements exactly the workaround the doc denies is needed: a `Cell<Option<PhysicsError>>` deferred slot, a zero-rate fallback on Err, and a check that short-circuits after the run. The genuinely infallible method is `eval_unprojected` (687). solvers/dec/mod.rs:29-31 repeats the same overclaim ('validated at construction so per-step evaluation is infallible').

> Evidence re-read: dec_ns_rate.rs:37-45, 418-433, 681-687; dec_ns_solver/step.rs:49-56 (# Errors listing the stage CG failure), 57-88; solvers/dec/mod.rs:29-31

---

### 9.9 [MINOR] Projection warm start makes the RK4 stage rate a stateful function of call history, violating the integrator's purity requirement

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:577`
- **Auditor confidence:** likely

**Claim.** With `warm_start` enabled, `project_raw` mutates `self.proj_warm` and `self.proj_warm_lambda` on every evaluation. That evaluation is the closure handed to `Rk4`, whose `Arrow` impl requires `F: Fn(&S) -> S` — a pure function of state. Stage k2's CG is seeded by k1's solution, k3's by k2's, so the rate returned for a given state depends on which stage produced it. RK4's order guarantee assumes a state-only rate function.

**Code evidence.**

```
dec_ns_rate.rs:563-579:
```
let (projection, lambda) = { ... leray_project_constrained_weighted_warm( ..., phi_guess.as_deref(), lambda_guess.as_deref()) ... };
*self.proj_warm.borrow_mut() = Some(projection.potential().as_slice().to_vec());
*self.proj_warm_lambda.borrow_mut() = Some(lambda);
```
step.rs:65-69: `let rk4 = Rk4::new(self.dt, |s: &VelocityOneForm<R>| { match self.rate.eval_projected(s, &self.cg_options) {`
rk4_arrow.rs:13: `F: Fn(&S) -> S,` — and rk4_arrow.rs:26-29 calls it four times per step at four different states.
dec_ns_solver/mod.rs:240-243 claims: "the marched result is identical to the cold path within the CG tolerance."
HodgeDecomposeOptions::default() is `tolerance: None, max_iterations: None` (hodge_decomposition_impl.rs:70-73), so the internal defaults govern termination.
```

**Reference form.** RK4's fourth-order local truncation error is derived for ẏ = f(y) with f a function of state alone. A rate with hidden mutable state between stage evaluations is not such an f; the residual difference is bounded by the CG termination tolerance, so the scheme degrades gracefully but is no longer bit-reproducible or formally fourth-order.

**Impact.** Bounded in magnitude but real for a certification context: with `with_warm_start()` enabled, two runs of the same case can differ (the number of CG iterations, and therefore the residual at termination, depends on the seed), and the trajectory is not reproducible from state alone. The docs at dec_ns_solver/mod.rs:240-243 and dec_ns_rate.rs:559-562 acknowledge the tolerance-level difference but never mention the purity violation or the reproducibility consequence. Off by default (dec_ns_rate.rs:290 `warm_start: false`), which limits exposure.

**Recommended fix.** Document at `with_warm_start` (dec_ns_solver/mod.rs:240) that enabling it makes the rate stateful across RK4 stages, that runs are reproducible only to CG tolerance, and that it should be disabled for any run producing certification evidence. UNCERTAIN whether the marched trajectory actually differs in practice: settle it by marching an identical case (same manifold, seed, dt, N steps) with and without `with_warm_start()` and comparing the final edge cochain bit-for-bit; a non-zero diff confirms the reproducibility gap, a zero diff confirms the CG converges to the same fixed point regardless of seed.

**Adversarial check.** project_raw's warm branch (dec_ns_rate.rs:558-579) reads `proj_warm`/`proj_warm_lambda` as CG guesses and writes the new potential and multipliers back on every evaluation, and project_raw is reached from eval_projected (430), which is the closure handed to `Rk4::new` in step.rs:65-84 — so the four stage evaluations of one step are chained through mutable state. The warm cell is `RefCell` behind `&self`, so this happens without any `&mut` at the call site. `warm_start: false` at construction (line 290) is confirmed, opt-in via `with_warm_start` (dec_ns_solver/mod.rs:244-247), whose doc (240-243) claims only 'identical to the cold path within the CG tolerance' and never mentions the state dependence or the reproducibility consequence. The auditor's reasoning about RK4's order derivation assuming f = f(y) is correct, as is the observation that the residual difference is bounded by the CG termination tolerance so the degradation is graceful. Severity minor is right: off by default, bounded by tolerance, and the tolerance-level caveat is at least partially disclosed.

> Evidence re-read: dec_ns_rate.rs:92-100 (warm fields), 279-293 (warm_start: false default), 535-580 (project_raw, both branches), 424-433; dec_ns_solver/step.rs:65-85; dec_ns_solver/mod.rs:240-247

---

### 9.10 [MINOR] CFL safety factor 0.9 is applied to both the advective and the diffusive limit with no traceable stability analysis, and the diffusive reference form is the explicit-Euler bound, not RK4's

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/mod.rs:93`
- **Auditor confidence:** confirmed

**Claim.** `default_safety = 0.9` is used for both `cfl_advective` and `cfl_diffusive` with no cited source. The diffusive limit `dt ≤ C·dx²/(2Dν)` is the classical forward-Euler bound; for RK4 the real-axis stability interval is ≈2.79× larger, so the guard is inconsistent with the integrator actually in use. Neither factor is traced to a stability analysis of the RK4 + DEC-operator pair.

**Code evidence.**

```
dec_ns_solver/mod.rs:93-95:
```
let default_safety = R::from_f64(0.9)
    // Coverage exemption: 0.9 lifts into every real field.
    .expect("0.9 lifts into R");
```
dec_ns_solver/mod.rs:102-103: `cfl_advective: default_safety, cfl_diffusive: default_safety,`
step.rs:135-137 (doc): "Enforces the advective limit `dt ≤ C_adv · dx_min / max|u|` ... and the diffusive limit `dt ≤ C_diff · dx_min² / (2·D·ν)`"
step.rs:155: `let diffusive_limit = self.cfl_diffusive * self.dx_min * self.dx_min / (two_d * nu);`
```

**Reference form.** Forward Euler on the D-dimensional heat equation with the 3-point Laplacian is stable for dt ≤ dx²/(2Dν) (von Neumann analysis; Hirsch, Numerical Computation of Internal and External Flows, §8). RK4's stability region extends to |z| ≈ 2.785 on the negative real axis, giving dt ≤ 2.785·dx²/(2Dν). For advection with central differences, forward Euler is unconditionally unstable while RK4 is stable to CFL ≈ 2.8/√3 in 3D — so a single factor 0.9 applied to a Euler-derived bound is neither the RK4 limit nor a stated fraction of it.

**Impact.** The guard is conservative in the diffusive direction (rejecting dt values that RK4 would handle) and its advective form is not tied to any stated stability region, so a user cannot tell whether a rejected dt is genuinely unstable or merely outside an arbitrary margin. Bounded because both factors are overridable via `with_cfl_factors` (validated finite and positive, lines 277-291) and because erring conservative does not produce wrong numbers. Secondary observation: the guard runs on the *post-step* max speed (step.rs:127-128), so it detects a violation one step after the fact and never checks the seed state before step 1.

**Recommended fix.** State the reference stability limits in the `cfl_check` doc (step.rs:135-137) and express the defaults as an explicit fraction of them — e.g. `C_diff = 0.9` relative to the Euler bound is ≈0.32 of RK4's actual limit; say so. Cite the von Neumann analysis or the RK4 stability-region source. If the pre-step guard matters, additionally evaluate `dec_max_speed` on the incoming state before the RK4 call.

**Adversarial check.** dec_ns_solver/mod.rs:93-95 and 102-103 read exactly as quoted; the only nearby comment is a coverage exemption ('0.9 lifts into every real field'), not a justification. Grep over the file finds no other mention of the factor or any citation. deep_causality_cfd/papers/ contains Droege2005, kirkpatrick2003, mittal2005, mohamed2016 — none is a time-integration stability reference. The implemented diffusive limit (step.rs:155) is C·dx²/(2Dν), which is the forward-Euler von Neumann bound; the auditor's RK4 real-axis figure (≈2.785) is correct, so the guard is roughly 3× conservative against the integrator actually in use, and the advective form is not tied to any stated stability region. Both secondary observations check out: the factors are overridable and validated finite/positive (mod.rs:277-291), and cfl_check runs on the post-step max speed (step.rs:127-128), so a violation is detected one step late and the seed state is never checked. Erring conservative does not corrupt results, which is why minor is the right severity.

> Evidence re-read: dec_ns_solver/mod.rs:93-107, 272-291; dec_ns_solver/step.rs:127-128, 135-170; `ls deep_causality_cfd/papers` (4 PDFs, none on time-stepping stability)

---

### 9.11 [INFO] Hard-coded absolute tolerance 1e-12 gates the adjoint columns in the generic convective path with no scaling to the lattice metric

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:637`
- **Auditor confidence:** confirmed

**Claim.** `zero_tol = 1e-12` (dec_ns_rate.rs:637, mirrored at stencil/mod.rs:140) is an absolute, unit-dependent threshold on a dimensional quantity with no traceable justification; a relative test against max_j|m_j| would be the correct form. Its intended target — exactly-zero dual masses on immersed-solid edges — is served correctly in both f32 and f64, and the construction-time star-positivity acceptance narrows the residual exposure to legitimately tiny cut-edge masses.

**Code evidence.**

```
dec_ns_rate.rs:635-639:
```
let m1 = self.manifold.hodge_star_of(&vec![R::one(); self.n1], 1);
let m1 = m1.as_slice();
let zero_tol = <R as FromPrimitive>::from_f64(1e-12)
    // Coverage exemption: 1e-12 lifts into every real field.
    .expect("1e-12 is representable in every RealField");
```
dec_ns_rate.rs:665-669:
```
*slot = if m1[j].abs() <= zero_tol { R::zero() } else { dot / m1[j] };
```
The same unscaled literal appears in the fused path's inverse-star build (topology stencil/mod.rs:140-151) and in `codifferential_of` (codifferential.rs:88-89).
```

**Reference form.** A zero test on a dimensional quantity should be relative — e.g. `|m| <= eps_rel * max_j |m_j|` with eps_rel tied to machine epsilon — so that it behaves identically under a change of length units. A fixed 1e-12 has different meaning on a lattice with h=1 (grade-1 star ≈ 1 in 2D, ≈ h in 3D) than on one with h=1e-5.

**Impact.** On a 3D lattice the grade-1 dual mass scales as h, and with an aperture-resolved cut-cell registry it is further multiplied by a wetted fraction that can be arbitrarily small near a body surface. A legitimate small-but-nonzero cut edge whose mass falls under 1e-12 silently has its adjoint column zeroed, breaking the exact skew-symmetry the stabilization relies on, with no diagnostic. In `f32` builds (the `CfdScalar` bound admits f32, and `viscous_sign_decays_energy_f32` exercises it) 1e-12 is below f32's smallest normal relative resolution against O(1) values, making the branch effectively unreachable rather than protective. Bounded because it affects the generic oracle path only; the fused default path has the analogous literal in the topology crate.

**Recommended fix.** Replace with a relative threshold: compute `m_max = m1.iter().map(|m| m.abs()).fold(zero, max)` and test `m1[j].abs() <= eps_rel * m_max`, with `eps_rel` derived from `R::epsilon()` rather than a decimal literal. Document the choice in the function doc. Apply the same treatment to stencil/mod.rs:140 and codifferential.rs:88 for consistency across the two assemblies.

**Adversarial check.** The literal is present exactly as cited (dec_ns_rate.rs:637-639, used at 665) and the same unscaled literal appears in the fused inverse-star build (stencil/mod.rs:140-151). It is an absolute threshold on a dimensional quantity with no justification beyond a coverage-exemption comment, and the auditor's point that a relative test scaled to max_j|m_j| would be unit-invariant is correct — that part I confirm. But two sub-claims are wrong. (1) The f32 argument is backwards: the guard's actual purpose is to catch *exactly zero* dual masses (dec_ns_rate.rs:220-238 documents that immersed-solid edges legitimately have zero dual mass and are exempted from the star-positivity acceptance), and |0.0| <= 1e-12 holds in f32 as in f64, so the branch is reachable and protective in f32, not 'effectively unreachable'. (2) The cut-cell risk is overstated as stated: DecNsRate::new runs a star-positivity acceptance over all non-immersed edges (dec_ns_rate.rs:201-239) that rejects any free edge with non-positive or non-finite dual mass at construction, so a legitimate small-but-nonzero cut edge dropping under 1e-12 is a narrow band rather than an unguarded regime — though that acceptance checks positivity, not magnitude, so the band does exist.

> Evidence re-read: dec_ns_rate.rs:634-670 (zero_tol definition and use), 201-239 (star-positivity acceptance and the immersed-edge exemption rationale); deep_causality_topology/src/types/manifold/differential/stencil/mod.rs:140-151 (same literal, inv_star1)

---

### 9.12 [MINOR] `DecNsRate` and the DEC solver modules are documented as periodic-only while the code implements walls, moving walls, immersed cut-cell bodies, free slip, and open inflow/outflow boundaries

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:36`
- **Auditor confidence:** confirmed

**Claim.** The type doc scopes the rate to a "periodic lattice manifold" and both solver module docs say "periodic", but `DecNsRate::new` explicitly handles wall-bounded lattices, immersed cut-cell registries, aperture-resolved vs staircase no-slip, free-slip un-pinning, and open-boundary inflow/outflow — a large capability set the headline docs do not mention.

**Code evidence.**

```
dec_ns_rate.rs:35-36: `/// The rate field ... on a metric-bearing /// **periodic** lattice manifold.`
solvers/dec/mod.rs:6: `//! The periodic DEC-native incompressible Navier–Stokes solver.`
dec_ns_solver/mod.rs:6 and 25: `//! The periodic DEC-native incompressible Navier–Stokes solver` / `/// The DEC Navier–Stokes solver on a periodic lattice manifold.`
Contradicted by dec_ns_rate.rs:184-199 (wall-axis extent acceptance), 175-179 (`cut_registry` / aperture-resolved `NoSlipConstraint`), 208-239 (star-positivity acceptance run "whenever there is a wall or an immersed body"), 310-325 (`set_open_boundary`), 329-335 (`apply_slip`), 341-347 (`set_staircase_noslip`), and by dec_ns_solver/mod.rs:182-232 (`with_moving_wall`).
README.md:42-44 confirms the capabilities are real and validated: "Validated against Taylor-Green decay, exact Couette and Poiseuille states, the Ghia et al. (1982) lid-driven cavity tables, and cylinder wake references."
```

**Reference form.** Bidirectional docs-vs-code parity: documented scope must cover implemented scope. Here the code's capability strictly exceeds the doc's stated scope.

**Impact.** A reader consulting the module headline concludes the solver cannot do wall-bounded or immersed-body flow and either does not use it or reimplements it. The gap is also load-bearing for review: the wall-bounded and cut-cell paths carry their own constraint semantics (`rate_constrained`, aperture-resolved cut-face rows, the masked-CG normal form) that a reviewer told "periodic" would not think to audit. Secondary: `DecNsRate::new`'s `# Errors` list (lines 132-137) omits the wall-axis-extent `DimensionMismatch` (line 193), the star-positivity `TopologyError` (line 233), and the stencil-compilation `TopologyError` (line 256).

**Recommended fix.** Rewrite the three module/type headlines to state the supported boundary configurations: fully periodic, wall-bounded with no-slip and prescribed moving-wall lift, free-slip, immersed cut-cell bodies (aperture-resolved by default, staircase as fallback), and open inflow/outflow with a pressure reference. Extend the `# Errors` section at dec_ns_rate.rs:132-137 with the three missing rejection paths.

**Adversarial check.** All four doc sites say 'periodic' as quoted (dec_ns_rate.rs:35-36; solvers/dec/mod.rs:6; dec_ns_solver/mod.rs:6 and 25). All the contradicting code sites are present at or very near the cited lines: wall-axis extent acceptance (dec_ns_rate.rs:181-199), cut_registry + aperture-resolved NoSlipConstraint (175-179), star-positivity acceptance run 'whenever there is a wall or an immersed body' (201-239), set_open_boundary (306-325), apply_slip (327-335), set_staircase_noslip (337-347), with_moving_wall and with_staircase_noslip (dec_ns_solver/mod.rs:~182-232, 249-257). The secondary claim also holds: DecNsRate::new's `# Errors` block (132-137) lists only D<2 / body-force DimensionMismatch, negative-ν, non-finite-ν and missing-metric TopologyError, and does not mention the wall-axis-extent rejection (193), the star-positivity rejection (233), or the stencil-compilation failure (256).

> Evidence re-read: dec_ns_rate.rs:35-36, 130-137, 170-199, 201-239, 255-256, 306-347; solvers/dec/mod.rs:6; dec_ns_solver/mod.rs:6, 25-31, 249-257; README.md validation paragraph

---

### 9.13 [MINOR] `Ambient::body_force` is never read by the DEC rate; a coupling stage that drives it has no effect on the march

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/theories/incompressible_dec.rs:50`
- **Auditor confidence:** confirmed

**Claim.** `Ambient` carries an optional body force and exposes `set_body_force` documented as a coupling-stage driver, but the only consumer in the marching path — `DecIncompressible::rate` — reads only `nu`. The DEC rate's body force is fixed at `DecNsRate::new` and cannot be changed after construction. The same is true of `Marcher::advance`.

**Code evidence.**

```
incompressible_dec.rs:50-51 (reads nu only):
```
self.rate.set_nu(*ambient.nu());
self.rate.eval_projected(state, &self.opts)
```
solvers/dec/marcher.rs:29-30 (same):
```
self.rate().set_nu(*ambient.nu());
self.step(state)
```
types/ambient.rs:58-61: `/// Drive the body force from a coupling stage.` / `pub fn set_body_force(&mut self, body_force: Option<BodyForceOneForm<R>>) { self.body_force = body_force; }`
types/ambient.rs:9-14 (module doc implies all three are read): "The per-step ambient a marcher reads each step: kinematic viscosity, the freestream inflow speed, and an optional body force. Coupling stages ... write into it *between* steps ... the marching rate only reads it."
A repo-wide grep for `body_force()` in deep_causality_cfd/src returns exactly one call site — types/flow/state_snapshot.rs:148 — which only refuses to serialize it. `DecNsRate` stores its own: dec_ns_rate.rs:63 `body_force: Option<CausalTensor<R>>` set once at line 241-253 and read at lines 769-783.
```

**Reference form.** Docs-vs-code parity: an API documented as a per-step driver of the marched dynamics must actually reach the marched dynamics. `Ambient::freestream` has the same shape — I did not trace whether it reaches the DEC path either.

**Impact.** A coupling stage implementing, say, a time-varying thrust or buoyancy term via `ambient.set_body_force(...)` compiles, runs, and produces a march in which the forcing never appears — silently, with no error. The result would be wrong and the cause invisible from the call site. UNCERTAIN whether some non-DEC marcher family consumes `Ambient::body_force`; the grep covers `body_force()` accessor calls in `deep_causality_cfd/src` and found only the snapshot guard, but a destructuring or field-level access in another family would not match. Settle by grepping for `\.body_force` (field and method) across all crates and confirming no marcher family consumes it.

**Recommended fix.** Either wire the ambient body force into the rate — add a `DecNsRate::set_body_force` mirroring `set_nu` and call it from `DecIncompressible::rate` and `Marcher::advance` — or document on `Ambient::set_body_force` and in the `Ambient` module doc which marcher families actually consume the field and which ignore it. Add a test that a body force driven through the ambient changes the projected rate.

**Adversarial check.** Confirmed, and I closed the auditor's own stated uncertainty. incompressible_dec.rs:50-51 and marcher.rs:29-30 read only `ambient.nu()`. A repo-wide grep for `\.body_force` (field access *and* accessor call) across all crates returns, in marching code, only types/flow/state_snapshot.rs:148 (a serialization refusal) plus the Ambient definition/accessors themselves and unrelated `body_force_per_mass` kernel parameters in the pointwise theories. I additionally grepped the other Marcher family (deep_causality_cfd/src/solvers/qtt, 6 impls) for any `ambient.` field or method access: zero hits, so no marcher family consumes it. DecNsRate carries its own construction-fixed body force (dec_ns_rate.rs:63, set at 241-253, read at 711 and 769) with no setter. types/ambient.rs:9-14 and 58-61 read as quoted. Same shape applies to `freestream`, which likewise reaches only the snapshot writer (state_snapshot.rs:197) and no marcher.

> Evidence re-read: theories/incompressible_dec.rs:45-52; solvers/dec/marcher.rs:24-31; types/ambient.rs:9-14, 43-61; `grep -rn '\.body_force\b|body_force()' --include=*.rs .` (only state_snapshot.rs:148 in marching code); `grep -rn 'ambient\.' deep_causality_cfd/src/solvers/qtt` (no hits); dec_ns_rate.rs:63, 241-253, 711, 769

---

### 9.14 [MINOR] `DecIncompressible`'s only physics assertion is that the rate of the zero state is zero — a tautology; the theory realization has no test tying it to `DecNsRate::eval_projected`

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/theories/incompressible_dec_tests.rs:57`
- **Auditor confidence:** confirmed

**Claim.** The three tests for the exported `FluidTheory` realization are a getter check, a `Debug` format check, and a rate evaluation on the zero state. Every term of the rate (skew convective, viscous Laplacian, absent body force) annihilates the zero vector by linearity/bilinearity, and the Leray projection of zero is zero, so the assertion holds for any implementation of the trait that returns something. There is no test that `DecIncompressible::rate` equals `DecNsRate::eval_projected` on a non-trivial state.

**Code evidence.**

```
incompressible_dec_tests.rs:49-59:
```
let u = zero_velocity(&manifold);
let ambient = Ambient::new(0.05_f64, 0.0, None);
let result = FluidTheory::rate(&theory, &u, &ambient).expect("projected rate evaluates");
assert_eq!(theory.rate().nu(), 0.05);
// A divergence-free projection of the zero state is again the zero rate.
for &c in result.as_tensor().as_slice() {
    assert!(c.abs() < 1e-12, "rate of rest should be zero, got {c}");
}
```
The whole test file is 69 lines (incompressible_dec_tests.rs:1-68) for a publicly exported type (theories/mod.rs:25 `pub use incompressible_dec::DecIncompressible;`).
```

**Reference form.** A meaningful test of a delegating wrapper compares its output to the delegate's on a state where the delegate produces a non-trivial value. Here: assert `FluidTheory::rate(&theory, &u_tg, &ambient)` is elementwise equal to `rate.eval_projected(&u_tg, &opts)` for a Taylor–Green state, and that the ambient ν actually changes the result.

**Impact.** The `FluidTheory` seam is documented as the abstraction over both the DEC rate and the pointwise evaluators (fluid_theory.rs:10-13) and is a public extension point. Its sole realization is effectively unverified: the `nu` assertion checks a getter round-trip, and the rate assertion cannot distinguish a correct delegation from one that returns zeros unconditionally. Combined with the missing ν validation reported separately, the seam has no negative-path coverage either.

**Recommended fix.** Add a test seeding a Taylor–Green edge cochain, evaluating both `FluidTheory::rate(&theory, &u, &ambient)` and `theory.rate().eval_projected(&u, &opts)`, and asserting elementwise equality; add a second asserting the projected rate changes when the ambient ν changes; add a negative test for non-finite and negative ambient ν once `set_nu` is made fallible.

**Adversarial check.** The file is 68 lines with exactly the three tests described: test_new_and_rate_getter (getter), test_fluid_theory_rate_reads_nu_from_ambient (lines 43-60, quoted accurately), and test_debug_impl. The rate assertion is tautological as claimed: the marched rate is a sum of the skew convective term (bilinear in u through ω = du and the vector slot), −ν Δ_dR u (linear), and an absent body force, so it annihilates u = 0; the Leray projection of the zero vector is zero regardless of the projector. No test compares DecIncompressible::rate against DecNsRate::eval_projected on a non-trivial state, and none exercises a bad ν. The one non-vacuous byte of the test is `assert_eq!(theory.rate().nu(), 0.05)`, which does confirm set_nu is invoked through the seam — a getter round-trip, as the auditor concedes. The suggested Taylor-Green delegation-equality test is the right remedy.

> Evidence re-read: tests/theories/incompressible_dec_tests.rs:1-68 (entire file); theories/incompressible_dec.rs:38-53; theories/mod.rs (DecIncompressible re-export); dec_ns_rate.rs:687-732 (linearity/bilinearity of every rate term)

---

### 9.15 [MINOR] "Second order" oracle convergence test gates on a factor-3 error reduction, which admits observed order 1.58

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/tests/solvers/dec/dec_ns_rate_tests.rs:168`
- **Auditor confidence:** confirmed

**Claim.** The test is named and documented as verifying the assembled rate matches the pointwise oracle "at second order", but the gate requires only that the relative error fall by a factor greater than 3 per doubling. Second order requires a factor of 4; a factor of 3 corresponds to observed order log2(3) ≈ 1.58.

**Code evidence.**

```
dec_ns_rate_tests.rs:135-138 (doc): "The full assembled rate (`−i_u du − ν Δ u`) matches the pointwise oracle at **second observed order** over the refinement ladder"
dec_ns_rate_tests.rs:168-175:
```
assert!(
    rel_errors[1] < rel_errors[0] / 3.0,
    "rate vs oracle not second order (first refinement): {rel_errors:?}"
);
assert!(
    rel_errors[2] < rel_errors[1] / 3.0,
    "rate vs oracle not second order (second refinement): {rel_errors:?}"
);
```
```

**Reference form.** For a scheme of order p, halving h reduces the error by 2^p. p = 2 requires a factor of 4. Standard practice reports the estimated order p̂ = log2(e_coarse/e_fine) and gates on p̂ exceeding a stated threshold with a stated margin, rather than on a bare ratio.

**Impact.** Bounded — the test is otherwise genuinely non-circular and is the strongest piece of evidence in the scope: the oracle is built independently from the analytic Taylor–Green field via automatic differentiation (`TgComponent::gradient`, lines 106-107), the separate `incompressible_ns_rhs` pointwise kernel (line 116), and an analytic Laplacian `-2k²u` (line 112), corrected by the Lamb-form kinetic-energy gradient (line 127). But the failure message asserts "not second order" for a gate that a 1.6-order scheme passes, so a genuine order degradation from 2 to ~1.6 would not be caught, and the doc claim of "second observed order" is not what the gate enforces.

**Recommended fix.** Compute and assert the observed order explicitly: `let p = (rel_errors[0]/rel_errors[1]).log2(); assert!(p > 1.8, "observed order {p} below 2");` for each refinement pair, and print the observed orders in the failure message. If a factor of 3 was chosen because the asymptotic regime is not reached at n=8, extend the ladder or say so in the doc rather than relaxing the claim silently.

**Adversarial check.** Both quotes are exact. dec_ns_rate_tests.rs:135-137 documents the test as matching the pointwise oracle 'at **second observed order** over the refinement ladder', the test is named rate_matches_pointwise_oracle_at_second_order (line 139), and the two gates at 168-175 assert only rel_errors[i+1] < rel_errors[i] / 3.0 while the failure message says 'not second order'. The arithmetic is right: a factor-3 reduction per halving corresponds to p̂ = log2(3) ≈ 1.585, so a degradation from 2 to ~1.6 passes. The auditor's mitigation is also accurate — the oracle really is independently constructed (TgComponent::gradient AD at ~106-107, the separate incompressible_ns_rhs kernel at ~116, the analytic −2k²u Laplacian, the Lamb kinetic-energy gradient correction at 127-128), so this remains the strongest non-circular evidence in scope; the defect is the loose gate and the mismatched failure message, not the test's design.

> Evidence re-read: tests/solvers/dec/dec_ns_rate_tests.rs:131-176 (doc, test body, both assertions and their messages, oracle construction at 116-129)

---
