# deep_causality_cfd — DEC Navier–Stokes solver driver and diagnostics (src/solvers/dec)

**Production readiness: `needs-work`**

The periodic and wall-bounded core is genuinely sound: I confirmed the projection's Poisson operator is div(grad) with the same d0 and the same mass-weighted divergence used to measure the residual (leray.rs:93-115, 936-957), the spectral grade-0 dispatch reproduces exactly the eigenvalues of that same operator (spectral_poisson.rs:81-95), the diffusive CFL constant is the correct 1/(2D) (step.rs:152-155), and the energy/enstrophy diagnostics carry the right 1/2 factors and volume weighting through the diagonal Hodge star (diagnostics.rs:53-90). The immersed-body path is where trust breaks down. Every NoPenetration cut-face row is discarded on a justification that is a non sequitur (no_slip.rs:104-110: global net flux zero does not imply local u.n = 0), the friction-drag diagnostic cites Kirkpatrick et al. 2003 for a wall distance and a traction the paper does not state (surface_force.rs:70-81 vs the paper's eqs. 33-34), the CFL guard divides by the uncut lattice spacing while the Hodge masses are aperture-scaled (mod.rs:79-91 vs has_hodge_star.rs:263-266), and the pressure integral samples vertices whose potential is a pure gauge artifact (leray.rs:612-614, 734-736). Separately, the module-level rustdoc for the whole solver describes a Chorin split-step algorithm with first-order splitting error (mod.rs:21-26) that the code does not implement — an engineer reading the module doc would model the wrong scheme. Finally, the two public "witness" reductions return 0.0 on a NaN field (diagnostics.rs:131-138, 221-224), so the documented projection-exactness witness reports its best possible value for the worst possible input. The periodic Taylor-Green path could be certified on its own evidence; the cut-cell drag/Strouhal path cannot.

- Files read: **32**
- Findings raised: **14** — surviving adversarial verification: **14** (refuted: 0)
- Surviving by severity: major 2, minor 9, info 3
- Independently confirmed-correct items: **11**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Diffusive CFL constant is 1/(2D), correct for the dimension | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:152-155` | Explicit stability for u_t = nu*lap(u) with a centred D-dimensional 2nd difference: nu*dt/dx^2 <= 1/(2D), i.e. 1/2 (1-D), 1/4 (2-D), 1/6 (3-D). Standard von Neumann analysis (Hirsch, Numerical Computa |
| Kinetic energy E = 1/2 * integral \|u\|^2 with correct volume weighting | `deep_causality_cfd/src/solvers/dec/diagnostics.rs:53-68` | E = 1/2 <u,u>_M = 1/2 * sum_e u_e (star u)_e for a diagonal DEC Hodge star, where star_1 diag = \|dual cell\|/\|edge\|. Desbrun/Hirani DEC inner product. |
| Enstrophy Z = 1/2 * integral \|omega\|^2, and the viscous energy budget term equals -2*nu*Z <= 0 | `deep_causality_cfd/src/solvers/dec/diagnostics.rs:74-90 and deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:515` | Z = 1/2 integral \|curl u\|^2; for a divergence-free field on a periodic domain the dissipation is eps = nu*integral\|omega\|^2 = 2*nu*Z, equal to 2*nu*integral\|S\|^2 (Batchelor, Homogeneous Turbulen |
| Drag coefficient normalization C = F/(0.5*rho*U^2*A) with rho = 1 | `deep_causality_cfd/src/solvers/dec/surface_force.rs:227-230` | C_d = F_x / (0.5 * rho * U_inf^2 * D) for a 2-D cylinder of diameter D. |
| Pressure Poisson is div(grad) of the same operators, not a separately discretized Laplacian | `deep_causality_topology/src/types/manifold/differential/leray.rs:93-115 and 936-957` | Leray/Chorin projection: P(w) = w - d(phi) with Delta_0 phi = delta w, where Delta_0 must be delta_1 d_0 formed from the same d_0 and delta_1 used for the gradient correction and the divergence measur |
| Spectral grade-0 Poisson dispatch is exact for the same delta*d operator on a periodic uniform lattice | `deep_causality_topology/src/types/manifold/differential/spectral_poisson.rs:81-95, 99-107` | On a uniform periodic lattice the operator M0^-1 d1 M1 d0 has Fourier symbol lambda_k = sum_d (2 - 2 cos(2 pi k_d / N_d)) / h_d^2, and the k=0 mode spans its kernel. |
| Binary no-slip pins reach exact machine zero; `constrain_edges` is a genuine no-op, not a tolerance-hiding overwrite | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:121-123 vs deep_causality_topology/src/types/manifold/differential/leray.rs:998-1001 and 749-756` | A constrained (KKT) projection onto {u : u\|_E = 0} must return identically zero on E, not merely small values. |
| Pressure diagnostic sign chain: Bernoulli B = phi | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/pressure.rs:44-74` | Rotational-form NS at rho = 1: du/dt = -omega x u + nu lap u + g - grad(p + \|u\|^2/2). Applying the Leray projector P gives du/dt = P(rhs_unproj), hence (I - P) rhs_unproj = grad(B) with B = p + \|u\ |
| Viscous wall traction is the correct rank-one reduction of mu(grad u + grad u^T).n | `deep_causality_cfd/src/solvers/dec/surface_force.rs:166-170` | t_i = mu (du_i/dx_j + du_j/dx_i) n_j. Under a purely wall-normal gradient du_i/dx_j = (du_i/dn) n_j this reduces to t_i = mu (du_i/dn + n_i (du/dn . n)). |
| CFL guard's dx_min is a true geometric edge length, not an aperture-scaled one, so it cannot collapse to zero on a cut lattice | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/mod.rs:80-86 and deep_causality_topology/src/types/cubical_regge_geometry/volumes.rs:31-52` | dx_min must be a positive length for dt <= C*dx/\|u\| to be a meaningful bound. |
| Taylor-Green Re-1600 viscosity and time normalization | `deep_causality_cfd/verification/dec_taylor_green_re1600_verification/config.rs:38-51` | TGV at Re = U L / nu with U = 1 and L = 1/k on a [0,n]^3 unit-spacing lattice with k = 2 pi / n gives nu = 1/(k Re); convective time t* = t k U. |

## Findings

### 16.1 [MAJOR] Solver module rustdoc documents a Chorin split-step march; the code marches the projected rate inside every RK4 stage

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/mod.rs:21`
- **Auditor confidence:** confirmed

**Claim.** The crate's DEC module documentation states the solver uses post-step (Chorin) projection with first-order splitting error. The implementation projects inside each RK4 stage and has no splitting error. An engineer sizing dt or an error budget from the module doc would apply a first-order-in-time splitting model to a scheme that has none.

**Code evidence.**

```
mod.rs:21-26: "//! The march uses the **Chorin placement**: an unprojected `Rk4` step over\n//! the whole-field state, then one gauge-fixed Leray projection back into\n//! the [`SolenoidalField`] type-state, then the\n//! CFL guard. The splitting at the projection is first order in time\n//! regardless of the integrator's interior order; validation therefore\n//! gates on spatial refinement at fixed CFL."

Contradicted by step.rs:6-9: "//! The projected march step: `Rk4` over the **projected rate** (the\n//! Leray projector sits inside each stage ... the marched ODE is exactly the projected dynamics,\n//! with no splitting error)" and by step.rs:65-69: `let rk4 = Rk4::new(self.dt, |s: &VelocityOneForm<R>| { ... self.rate.eval_projected(s, &self.cg_options) ... });`

Also contradicted by README.md:41-43: "Each time step marches the Leray-projected rate, so the field stays divergence-free at every step", and by verification/dec_taylor_green_re1600_verification/README.md:171-175: "The projector sits *inside* the `Rk4` stages, not after them. The post-step (Chorin) placement was measured during development to bleed 5-20% of the inviscid energy over `T = 10` ... Marching `du/dt = P(rhs)` directly removes the splitting error entirely".
```

**Reference form.** Chorin projection (Chorin 1968) advances an unprojected predictor then projects once per step, incurring O(dt) splitting error. Marching du/dt = P(rhs) as an ODE on the divergence-free subspace incurs only the integrator's own truncation error. These are different schemes with different temporal convergence orders; the doc names the one the code does not implement.

**Impact.** Anyone reading the module doc (the canonical entry point for the solver) models the wrong temporal accuracy, will not expect RK4-order time convergence, and may reject correct convergence-study results as anomalous. In a certification package the module rustdoc is the specification of record and it disagrees with the code.

**Recommended fix.** Replace mod.rs:21-26 with the projected-rate description already correct in step.rs:6-13 and the TGV README, stating that each RK4 stage evaluates P(rhs) (four CG solves per step) and that the re-entry projection exists only to satisfy the SolenoidalField construction contract.

**Adversarial check.** Both quotes are verbatim and they are flatly contradictory. mod.rs:21-26 says the march is 'an unprojected `Rk4` step over the whole-field state, then one gauge-fixed Leray projection' and that 'the splitting at the projection is first order in time regardless of the integrator's interior order'. step.rs:65-84 constructs `Rk4::new(self.dt, |s| self.rate.eval_projected(s, &self.cg_options))` - the projector is inside the stage closure - and step.rs:6-9 plus dec_ns_rate.rs:9-13 both say explicitly 'there is no splitting error'. step.rs:44-47 further describes the post-RK4 projection as 'a near-no-op solve that exists to keep SolenoidalField's construction contract'. The auditor's reference form is also correct: marching du/dt = P(rhs) is an ODE on the divergence-free subspace and carries only the integrator's truncation error, whereas Chorin (1968) projects a predictor once per step and is O(dt). The module rustdoc is the crate's entry point and it names the scheme the code does not implement.

> Evidence re-read: deep_causality_cfd/src/solvers/dec/mod.rs:21-26 (Chorin placement, 'first order in time'); step.rs:6-9 ('no splitting error'); step.rs:57-88 (Rk4 over eval_projected); step.rs:38-47 (bind-sequence doc); dec_ns_rate.rs:7-14 ('The projector sits **inside** the rate ... there is no splitting error')

---

### 16.2 [MINOR] Every NoPenetration cut-face row is discarded, so the immersed body enforces no local no-penetration condition; the stated justification is a non sequitur

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/no_slip.rs:116`
- **Auditor confidence:** confirmed

**Claim.** The NoPenetration cut-face rows are deliberately dropped per openspec/specs/aperture-resolved-noslip/spec.md; no-penetration is carried in the mass sense by the zero apertures of the cut Hodge star plus per-vertex divergence-freeness plus interior pins, so flux into the body is blocked cell-locally rather than only globally. The residual gap is that the pointwise reconstructed n.u at a wetted fragment is not constrained, a resolution-dependent accuracy term. Separately, the `NoSlipConstraint::new` docstring (no_slip.rs:55-63) says the wall condition 'becomes the weighted cut-face rows' without stating that only the tangential subset is retained - a doc-parity gap against the inline comment ten lines below.

**Code evidence.**

```
no_slip.rs:115-118: `rows = all_rows\n    .into_iter()\n    .filter(|r| r.kind() == CutConstraintKind::Tangential)\n    .collect();`

Justification, no_slip.rs:104-110: "// no-penetration rows of a closed body are linearly dependent (the discrete\n// divergence-theorem identity `n.u dA = 0`), which gives the KKT system a tiny\n// eigenvalue and floors the projection CG. They are also redundant - the body\n// interior is pinned to zero (the solid edges below) and the projection is\n// divergence-free, so the net flux through the body surface already vanishes."

What is being dropped, deep_causality_topology/src/types/cut_cell/registry.rs:324-337: one `CutConstraintKind::NoPenetration` row per cut cell followed by the tangential rows; cut_face_constraint.rs:15-20: "NoPenetration: `n_hat . u_face = 0`: zero flux normal to the wetted cut face".
```

**Reference form.** The immersed no-slip wall condition is u = u_wall on the surface, i.e. all D components: (D-1) tangential rows plus one wall-normal row n.u = 0. Discrete divergence-freeness gives the closed-surface identity integral of n.u dA = 0 (divergence theorem), which is implied by, but does not imply, the pointwise n.u = 0. A body enforcing only the tangential rows is a slip-free but permeable surface.

**Impact.** Wall-normal through-flow at the cylinder surface is unconstrained except by global mass conservation. Separation location, the shedding frequency (Strouhal) and both the pressure and friction drag are set by exactly this condition; C_d and St computed on this path are not traceable to a correctly posed no-slip body. The docstring of `NoSlipConstraint::new` (lines 55-58) tells the reader the wall condition "becomes the weighted cut-face rows ([`CutCellRegistry::cut_face_constraints`])" without disclosing that half of those rows are discarded.

**Recommended fix.** Either restore the no-penetration rows with an explicit rank fix (drop exactly one row per closed body, or deflate the known null direction) rather than dropping all of them, or - if the current behaviour is intentional - measure and publish the residual max |n.u| on the wetted fragments as a first-class diagnostic per run, and state the limitation in the `NoSlipConstraint::new` docstring and in every C_d/St result produced on this path.

**Adversarial check.** The code fact is exact: no_slip.rs:115-118 filters to `CutConstraintKind::Tangential` and registry.rs:322-337 emits one NoPenetration row per cut cell before the tangential rows. But three things the auditor did not check materially change the verdict. (1) This is not an undisclosed slip - it is the normative spec of record. openspec/specs/aperture-resolved-noslip/spec.md:36-46 states 'No-penetration ... SHALL hold in aggregate, carried by the body-interior zero pins together with the projection's divergence-freeness ... rather than as an explicit per-fragment constraint row', with the ill-conditioning rationale. The code comment ends '(design open question 4.3: no-penetration row off)', and tasks.md 4.3 records the ablation as resolved. (2) The mechanism is stronger than the global divergence theorem the auditor attacks. The grade-1 Hodge star is aperture-scaled through `dual_fluid_fraction` (has_hodge_star.rs:263-266), so solid faces carry zero mass and normal flux into the body is blocked at the operator level (design.md:9: 'This already handles no-penetration in the mass sense (blocked flux)'). Combined with per-vertex discrete divergence-freeness and the interior pins, zero net flux is enforced per dual cell, not merely over the whole closed body. (3) The auditor's implied 'dropped rows leave edges unconstrained' does not occur: both row kinds are built from the same `axis_reconstruction` edge set, so `row_edges` (computed from all_rows, pre-filter) covers exactly the edges the surviving tangential rows still govern. What genuinely remains is that the *reconstructed* wall-normal component at a fragment is not driven to zero pointwise - a sub-grid accuracy question that improves with cut resolution, not an unposed body.

> Evidence re-read: no_slip.rs:99-127 (filter + row_edges construction); deep_causality_topology/.../cut_cell/registry.rs:322-337 (row emission), :340-352 (axis_reconstruction shared by both kinds), :411-495 (dual_fluid_fraction); has_hodge_star.rs:263-266 (cut star clip); openspec/specs/aperture-resolved-noslip/spec.md:36-53

---

### 16.3 [MINOR] Friction-drag wall distance and traction are attributed to Kirkpatrick et al. 2003, which states a different distance and a tangential-only shear stress

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/surface_force.rs:72`
- **Auditor confidence:** confirmed

**Claim.** The docstring attributes to Kirkpatrick et al. 2003 a sampling distance and a traction form the paper does not state: the paper uses the node-to-surface perpendicular distance with the existing node velocity and a tangential-only shear (eqs. 33-34), while the code uses a fixed one-cell support width along n with an interpolated sample and adds a wall-normal term. The construction itself is self-consistent (the sample is placed at exactly Delta_h along n from the wall anchor), so this is a citation/attribution defect, not a formula error.

**Code evidence.**

```
surface_force.rs:71-77: "The wall shear is evaluated with a **one-sided wall-normal\n/// gradient to the true surface distance `dh`** (Kirkpatrick et al. 2003) ... The Kirkpatrick wall traction `t = mu S.n` with the rank-one wall-normal\n/// gradient reduces to `t_i = mu (u_sample,i + n_i (u_sample.n)) / dh`"

surface_force.rs:148-158: `let mut delta_h = R::zero(); ... for i in 0..D { delta_h += n[i].abs() * dx[i]; } ... *s = centroid[i] + delta_h * n[i];`

surface_force.rs:79-81 asserts: "this reads the gradient from the wall to the first fluid sample over the actual perpendicular distance".

papers/kirkpatrick2003.pdf, sec. 2.3.3 (extracted text): "the perpendicular distance between the node and the quadric surface is used in conjunction with the area of the surface within the cell to calculate the shear force on the fluid" and "Assuming that u is tangential to the surface, for a no-slip boundary the shear stress and velocity vectors are parallel, so that tau = mu du/dh (33) ... tau ~ mu u / Dh (34)".
```

**Reference form.** Kirkpatrick, Armfield & Kent, J. Comput. Phys. 184 (2003) 1-36, eqs. (33)-(34): tau = mu * u_node / Delta_h, with u_node the existing staggered-grid node velocity and Delta_h the node-to-surface perpendicular distance, and with u assumed tangential (no normal-component term).

**Impact.** An engineer verifying the friction drag against the cited paper will find the code computes something else: a different sampling distance (up to sqrt(2)*dx in 2-D, sqrt(3)*dx in 3-D rather than a sub-cell node distance) and an extra normal-component term. Because Delta_h is the cell support width and not a computed wall distance, the sample point sits 1.4-1.7 cells into the fluid, outside the linear near-wall region at any resolved boundary layer, and the multilinear stencil around it can include solid-side vertices whose velocity is pinned to zero - both bias C_f low. The doc sentence "the actual perpendicular distance" is false as written: Delta_h does not depend on where the wall sits inside the cell.

**Recommended fix.** Give the full reference in the docstring (repo convention) and state plainly that Delta_h is the cell support width along n, not the paper's node-to-wall distance, and that the traction retains the normal component the paper drops. Better: use the fragment centroid-to-node perpendicular distance the registry already carries, and publish a grid-convergence study of C_f against a reference to quantify the current bias.

**Adversarial check.** I extracted papers/kirkpatrick2003.pdf and the auditor's quotation is accurate. Sec. 2.3.3: 'the perpendicular distance between the node and the quadric surface is used ... to calculate the shear force'; eq. (33) 'Assuming that u is tangential to the surface ... tau = mu du/dh'; eq. (34) 'tau ~ mu (u/Dh i + v/Dh j + w/Dh k)'. Kirkpatrick's Delta_h is node-to-surface (sub-cell), u is the existing node velocity, and there is no normal-component term. The code's Delta_h = sum_i |n_i|*dx_i (surface_force.rs:148-152) is the cell support width along n, in [dx, sqrt(D)*dx], and the traction carries the extra +n_i(u.n) term (surface_force.rs:166-168). So the attribution is loose and the phrase 'The Kirkpatrick wall traction' overclaims. However the auditor's sharpest sub-claim is wrong: 'the doc sentence "the actual perpendicular distance" is false as written' does not hold. The code anchors the zero-velocity point at the fragment centroid c and places the sample at c + Delta_h*n (surface_force.rs:156-158), so the sample genuinely sits at perpendicular distance Delta_h from the wall - the sample moves with the wall even though Delta_h does not. The scheme is self-consistent; only the citation is inaccurate. The 'biases C_f low' consequence is plausible (a sample 1.0-1.7 cells out, with a multilinear stencil that can touch pinned solid-side vertices) but I could not settle its sign or magnitude without running the cylinder case.

> Evidence re-read: deep_causality_cfd/src/solvers/dec/surface_force.rs:67-83 (docstring), :146-170 (delta_h, sample point, rank-one traction); papers/kirkpatrick2003.pdf sec. 2.3.3 extracted text, eqs. (33)-(34) and Fig. 5 caption

---

### 16.4 [MINOR] CFL guard divides by the uncut lattice spacing while the Hodge masses are aperture-scaled, so it ignores the cut-cell small-cell stability restriction

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/mod.rs:80`
- **Auditor confidence:** likely

**Claim.** dx_min is the geometric lattice edge length and the CFL guard never consults the cut-cell registry's cell-merging floor. The crate does implement the small-cell stabilizer (CutCellRegistry::with_cell_merging), but flow_config Body defaults merge_floor to 0, which disables it, so on the default immersed-body path a sliver cut cell of fluid fraction f makes the true explicit limit ~f times smaller (dt <= C*f*dx^2/(2 D nu)) than the value the guard checks. With a non-zero merge floor the overshoot is bounded by 1/min_fraction.

**Code evidence.**

```
mod.rs:79-91: `let mut dx_min: Option<R> = None;\nfor cell in complex.iter_cells(1) {\n    let len = metric.cell_volume(complex, &cell);\n    dx_min = Some(match dx_min { Some(m) if m <= len => m, _ => len });\n}` - `cell_volume` returns the pure geometric edge length (volumes.rs:31-52), with no aperture factor.

mod.rs:303-306: `/// The minimum lattice edge length the CFL guard divides by.\npub fn dx_min(&self) -> R { self.dx_min }`

The masses that actually set the eigenvalue, deep_causality_topology/src/types/cubical_regge_geometry/has_hodge_star.rs:263-266: `let build = || match &self.cut_registry {\n    Some(registry) => self.build_star_diagonal(complex, k, &|cell| registry.dual_fluid_fraction(complex, cell)),` - and no floor is applied to that fraction anywhere in `build_star_diagonal` (has_hodge_star.rs:96-151).
```

**Reference form.** For a cut-cell discretization, the explicit stability limit is set by the smallest control volume, not the background spacing: dt <= C * (f * dx)^2 / (2 D nu) and dt <= C * f * dx / |u| for a cell of fluid fraction f. This is the classical small-cell problem that Kirkpatrick et al. 2003 (the paper this crate cites) addresses with cell-linking - see the abstract's "a novel cell-linking method ... which overcomes problems associated with the creation of small cells".

**Impact.** On the cylinder/immersed-body path the CFL guard can pass a dt that is unstable near the body. Because the guard is the solver's only automatic stability check and it is evaluated after the step (step.rs:127-128), an instability seeded at a sliver cut cell can grow for many steps before any error surfaces. The public accessor `dx_min()` and its docstring give no hint that the value is not the stability-relevant length on a cut lattice.

**Recommended fix.** Either fold the minimum wetted fraction from the cut registry into dx_min (dx_eff = min over cells of f*dx), or implement the cell-linking/merging the cited paper prescribes, and in either case document in `dx_min()` and `cfl_check` that the bound is a background-grid bound when no registry is attached.

**Adversarial check.** The mechanism is real: dec_ns_solver/mod.rs:79-91 takes dx_min from `metric.cell_volume`, which volumes.rs:31-52 shows is the pure product of geometric edge lengths with no aperture factor, while the grade-1 masses used by every operator are aperture-scaled (has_hodge_star.rs:263-266). But two parts of the finding are wrong. (1) The cited evidence 'no floor is applied to that fraction anywhere in build_star_diagonal' is false. The floor is not in build_star_diagonal, it is inside the clip closure itself - registry.rs:485-495 inflates any free, body-adjacent, non-zero dual fraction to `min_fluid_fraction`, leaving fully-dry (exactly 0) edges at 0. (2) The claim that Kirkpatrick's cell-linking addresses a hazard this crate ignores is refuted: `CutCellRegistry::with_cell_merging` (registry.rs:68-88) is exactly that stabilizer, documented against Berger-Helzel with an explicit statement of the hazard ('the viscous stencil's 1/mass factor then makes the explicit operator stiff enough to violate the CFL bound for any usable time step - the canonical cut-cell hazard') and an explicit rejection of Colella-Graves-Modiano flux redistribution on architectural grounds. The auditor's reference scaling is also off by a power: with M0^-1 d^T M1 d and a dual volume scaled by f, the eigenvalue grows like 1/f, giving dt <= C*f*dx^2/(2 D nu), not (f*dx)^2. What survives is narrower but real: the guard never consults the floor, and the Flow config defaults `merge_floor` to zero (flow_config/body.rs:25), i.e. `with_cell_merging(0)` never triggers, so on a default immersed-body run the guard can pass a dt above the true limit by ~1/f.

> Evidence re-read: dec_ns_solver/mod.rs:79-91 and :303-306; deep_causality_topology/.../volumes.rs:31-52 (geometric product only); has_hodge_star.rs:250-266 (cut clip wiring), :96-151 (build_star_diagonal - no floor here, correctly); cut_cell/registry.rs:68-88 (with_cell_merging doc), :485-495 (the floor, applied inside dual_fluid_fraction); deep_causality_cfd/src/types/flow_config/body.rs:16-46 (merge_floor defaults to zero); flow_config/mesh.rs:124,189

---

### 16.5 [MINOR] dec_divergence_residual and dec_max_speed return 0.0 on a NaN field, so the documented projection-exactness witness reports its best value for the worst input

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/solvers/dec/diagnostics.rs:221`
- **Auditor confidence:** confirmed

**Claim.** Both max-norm reductions silently discard NaN because `>` is false for unordered comparisons, so an all-NaN field yields 0.0 from a function documented as 'the projection-exactness witness', and the advective CFL branch is skipped. The step path is indirectly protected because a NaN state fails the stage CG first; the exposure is to direct callers of the public dec_max_speed / dec_divergence_residual. Fix by rejecting non-finite input or making the fold NaN-propagating.

**Code evidence.**

```
diagnostics.rs:220-224: `let div = manifold.codifferential_of(edge_form.as_slice(), 1);\nOk(div.as_slice().iter().fold(\n    R::zero(),\n    |acc, x| if x.abs() > acc { x.abs() } else { acc },\n))`

diagnostics.rs:131-138: `let mut max_sq = R::zero();\nfor chunk in v.chunks_exact(D) {\n    let norm_sq = chunk.iter().fold(R::zero(), |acc, x| acc + *x * *x);\n    if norm_sq > max_sq { max_sq = norm_sq; }\n}\nOk(max_sq.sqrt())`

Consumed as the health metric, step.rs:127-132: `let max_speed = dec_max_speed(...)?; self.cfl_check(max_speed)?; let divergence_residual = dec_divergence_residual(...)?;` and step.rs:139: `if max_speed > R::zero() { ... }` - the advective limit is skipped when max_speed is 0.

Documented as a witness, diagnostics.rs:210-211: "/// Post-projection divergence residual `||du||_inf` - the projection-\n/// exactness witness." Both functions are public API (mod.rs:83-86).
```

**Reference form.** A max-norm reduction must be NaN-propagating (or explicitly reject non-finite input) for its value to be a witness: max_inf over a set containing NaN is NaN, not 0. IEEE-754 unordered comparisons make `x > acc` false for NaN, which silently discards the worst element.

**Impact.** A public function documented as the projection-exactness witness returns 0.0 for a fully diverged field. The same blindness disables the advective CFL branch. Upstream, `SolenoidalField::from_open_leray_projection_weighted_opts` performs no finiteness validation (solenoidal_field.rs:121-146), so nothing in the step path re-checks. In practice a NaN right-hand side usually causes the CG to exhaust its budget and error (cg.rs:312, 324-327), but that is an accidental safety net, not a check, and it does not protect direct callers of these two public functions.

**Recommended fix.** Make both reductions non-finite-aware: return `PhysicsError::NumericalInstability` when any coefficient is not finite, or use a NaN-propagating max. Additionally make `cfl_check` reject a non-finite `max_speed` explicitly rather than falling through the `> zero` guard.

**Adversarial check.** The code is exactly as quoted (diagnostics.rs:220-224 and :131-138) and the IEEE-754 reasoning is right: `x.abs() > acc` and `norm_sq > max_sq` are both false for NaN, so an all-NaN field folds to the zero-initialised accumulator and both functions return 0. The CFL consequence is also real - step.rs:139 gates the advective branch on `max_speed > R::zero()`, which a NaN-induced 0 skips. I confirmed there is no finiteness check upstream: `leray_project_constrained_weighted_opts` (leray.rs:293-313) and the open weighted re-entry validate lengths and constraint sets, not finiteness. The correction is severity, not substance. Inside `step` the NaN would have to survive four `eval_projected` CG solves first, and the CG errors out on a non-convergent residual, so the in-solver exposure is indirect; the exposure is to direct callers of the two public functions. This is a robustness/API-contract defect in a diagnostic, not a physics error in the march. I also note the axis label is off - this is not a tautological gate, it is an unordered-comparison reduction defect.

> Evidence re-read: diagnostics.rs:123-139 (dec_max_speed fold), :210-225 (dec_divergence_residual fold, and the 'projection-exactness witness' doc); step.rs:127-132 and :139 (max_speed > 0 gate); leray.rs:293-313 (no finiteness validation on the weighted constrained entry point); mod.rs:83-86 (both are public API)

---

### 16.6 [MAJOR] Pressure surface force samples the potential at solid-interior vertices where it is a pure gauge artifact

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/surface_force.rs:31`
- **Auditor confidence:** likely

**Claim.** `pressure_surface_force` requires a per-cut-cell pressure. The crate's only producer is `pressure_diagnostic`, whose 0-form is the projection potential. Vertices whose incident edges are all pinned inside the body are structurally eliminated from the solve (phi = 0), then the uniform mean subtraction gives them phi = -mean(phi). The crate's adapter corner-averages the cell's 2^D vertices, so at a cut cell it mixes real fluid pressures with that gauge artifact.

**Code evidence.**

```
surface_force.rs:31-34 requires a cell-indexed pressure: `pub fn pressure_surface_force<const D: usize, R: DecNsScalar>(registry: &CutCellRegistry<D, R>, cell_pressure: impl Fn(usize) -> R) -> [R; D]`

The adapter, src/types/flow/march_run.rs:489-501: `let cell_pressure = |cell_id: usize| -> R { let base = *cells[cell_id].position(); let mut sum = R::zero(); for corner in 0..(1usize << D) { ... if let Some(&pv) = pressure.get(&pos) { sum += pv; } } sum * inv_corners };` where `pressure` is built from `static_p.as_tensor()`, the vertex 0-form.

Where the artifact is created, deep_causality_topology/src/types/manifold/differential/leray.rs:612-614: `let inactive: Vec<bool> = (0..n0).map(|i| diag[i] == R::zero() || (eliminate && (is_ref[i] || !live[i]))).collect();` - the RHS is zeroed on those rows (line 626-627) and the operator zeroes them on input and output (lines 666-670, 683-685), so the CG leaves them at 0. Then leray.rs:734-736: `if !eliminate { subtract_mean_in_place(&mut phi); }` is applied to the whole vector, giving phi = -mean at every inactive vertex.
```

**Reference form.** F_p = -integral over S of p n dA requires p to be the physical fluid pressure evaluated on the wetted surface. A potential value at a vertex excluded from the Poisson solve carries no pressure information; it is a gauge constant. A uniform constant added to every cell would cancel because the closed-body fragment normals sum to zero (the crate's own `fragment_area_vector` check, surface_force.rs:51-65), but a constant injected into only some corners of only the cut cells does not cancel.

**Impact.** The pressure-drag component of C_d - the dominant term for a bluff body - is contaminated by a gauge constant at every cut cell, with a magnitude set by mean(phi), which itself depends on how many solid vertices the body occupies. This is the headline number of the cylinder verification. Neither `pressure_surface_force`'s docstring nor `pressure_diagnostic`'s mentions that the 0-form is undefined inside the body.

**Recommended fix.** Restrict the corner average to vertices that were active in the solve (expose the active mask from the projection, or reconstruct it from the no-slip edge set), renormalizing by the count of contributing corners; and document in both `pressure_diagnostic` and `pressure_surface_force` that the potential is meaningless at solid-interior vertices. Re-run the cylinder C_d after the fix and report the delta.

**Adversarial check.** I traced the whole chain and it holds, including the gauge branch the auditor's claim depends on. pressure_diagnostic (pressure.rs:44-53) calls eval_projected_with_potential -> project_raw, and project_raw (dec_ns_rate.rs:546-557) calls `leray_project_constrained_weighted_opts`, which (leray.rs:301-313) forwards with EMPTY reference_vertices. That makes `eliminate = false` at leray.rs:610, so the inactive branch at leray.rs:736-741 is not taken and leray.rs:734-735 runs `subtract_mean_in_place(&mut phi)` over the whole vector - including the rows marked inactive at leray.rs:612-614 (`diag[i] == R::zero()`, i.e. vertices with no free incidence, which the CG left at 0 per the operator masking at :666-670/:683-685). Those vertices therefore come back as exactly -mean(phi). march_run.rs:489-501 then corner-averages all 2^D vertices of each cut cell, mixing that constant into cells that straddle the body, and pressure_surface_force (surface_force.rs:31-47) integrates it against -n dA. Because only some corners of only the cut cells carry it, the closed-body cancellation the docstring relies on (surface_force.rs:29-30, fragment normals summing to zero) does not apply. Static pressure compounds it: pressure.rs:69-74 subtracts |u|^2/2, which is also 0 at those vertices since their velocity is pinned. Neither pressure_surface_force's nor pressure_diagnostic's docstring warns that the 0-form is undefined inside the body.

> Evidence re-read: pressure.rs:38-74; dec_ns_rate.rs:438-447 and :535-557 (project_raw -> constrained weighted, no reference vertices); leray.rs:293-313 (reference_vertices = &[] -> eliminate=false), :610-614 (inactive), :626-627, :666-670, :683-685 (masking), :733-741 (subtract_mean on the !eliminate branch); flow/march_run.rs:479-505 (corner average); surface_force.rs:25-47

---

### 16.7 [MINOR] Moving-wall lift is written onto the state after projection, so the divergence residual reported for cavity/Couette runs contains a structural wall artifact

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:121`
- **Auditor confidence:** likely

**Claim.** After the re-entry projection the lift values are assigned onto constrained edges that the projection zeroed and never saw. For a uniform tangential lift along a flat wall the two incident wall edges cancel in the discrete divergence at interior wall vertices, but at the two ends of the moving wall only one lifted edge exists, so the state carries a nonzero divergence there. The value that `StepOutput::divergence_residual` reports as the projection-exactness witness is then dominated by that geometric artifact rather than by CG convergence.

**Code evidence.**

```
step.rs:121-123: `let projected = projected\n    .constrain_edges(self.rate.no_slip_edges())\n    .with_lift(&self.lift);`
followed immediately by step.rs:130: `let divergence_residual = dec_divergence_residual(self.manifold, projected.as_one_form())?;`

The lift set, mod.rs:219-229: every tangential edge on the wall row gets `velocity[axis] * length`, including the edges at the ends of the wall.

The projection never sees these values, leray.rs:821-824: `let mut v: Vec<R> = field.as_slice().to_vec(); for &e in zeroed_edges { v[e] = R::zero(); }` - consistent with the claim in mod.rs:44-46 that "the constrained projector ignores constrained-edge input values, so `P(u) = P(u - lift)` exactly".
```

**Reference form.** delta u = 0 must hold on the marched state, not only on the projector's output. For a lid-driven cavity the discrete divergence at a lid-corner vertex receives the flux of the single incident lifted edge, giving |delta u| ~ U * M1 / M0 there.

**Impact.** On any moving-wall case (lid-driven cavity, Couette - both named in the crate README's validation list) the reported divergence residual is O(U/h) rather than O(CG tolerance), so it cannot be used to detect a genuine projection failure, and a user calibrating a tolerance against observed values would back-fit it to a structural artifact.

**Recommended fix.** Either report the residual over the free (unconstrained, unlifted) vertex set only, or state in `StepOutput::divergence_residual`'s docstring that on lifted walls the value includes the wall-end lift flux and give the expected magnitude. A settling check: run the cavity case and print delta u at the two lid corners versus the interior maximum - if the corners dominate by orders of magnitude, this is confirmed.

**Adversarial check.** Ordering is verbatim: step.rs:121-123 applies `.constrain_edges(...).with_lift(&self.lift)` after the re-entry projection, and step.rs:130 computes the reported divergence_residual on that post-lift state. The projector genuinely never sees the lift values (leray.rs zeroes constrained-edge input, consistent with dec_ns_solver/mod.rs:41-47 'P(u) = P(u - lift) exactly'). The end-of-wall argument checks out against the lift construction at mod.rs:216-229: the loop admits every tangential edge on the wall row including the two at the ends, so at a lid-corner vertex only one lifted edge is incident (the perpendicular wall-tangential edge there is pinned to zero by the no-slip set, no_slip.rs:81-85), leaving an uncancelled flux of order U*h in the codifferential. On a fully periodic-in-x Couette lattice the wall row is closed and the two incident lifted edges cancel, so the artifact is specific to the lid-driven-cavity geometry the README lists. Severity minor is right: it corrupts a reported diagnostic, not the marched state's physics.

> Evidence re-read: step.rs:114-132 (lift applied at :121-123, residual measured at :130); dec_ns_solver/mod.rs:41-47 (lift/projection commutation doc), :216-229 (lift includes the wall-row end edges); no_slip.rs:79-86 (perpendicular wall edges pinned)

---

### 16.8 [MINOR] Advective CFL uses the one-dimensional form with the Euclidean speed instead of the multidimensional sum over axes

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:140`
- **Auditor confidence:** confirmed

**Claim.** The guard evaluates dt <= C * dx_min / max|u| with max|u| the Euclidean vertex speed. The standard multidimensional Courant number is dt * sum_i |u_i|/dx_i. For a flow diagonal to the grid these differ by up to sqrt(D), so the guard permits dt up to sqrt(D) times larger than the standard condition (about 1.41x in 2-D, 1.73x in 3-D).

**Code evidence.**

```
step.rs:140-141: `let advective_limit = self.cfl_advective * self.dx_min / max_speed;\nif self.dt > advective_limit {`

max_speed comes from the Euclidean norm, diagnostics.rs:133: `let norm_sq = chunk.iter().fold(R::zero(), |acc, x| acc + *x * *x);` then `max_sq.sqrt()`.

Documented as, step.rs:135-136: "/// Enforces the advective limit `dt <= C_adv . dx_min / max|u|`".
```

**Reference form.** Courant-Friedrichs-Lewy in D dimensions: CFL = dt * sum_{i=1..D} |u_i| / dx_i <= C. With dx_i = h and u at 45 degrees in 2-D with speed s, the true CFL number is dt*s*sqrt(2)/h while the code computes dt*s/h. (Hirsch, Numerical Computation of Internal and External Flows, sec. 8.4.)

**Impact.** The stated safety margin (default C_adv = 0.9) is optimistic by up to sqrt(D) for grid-diagonal flow. RK4's larger stability region usually absorbs this, so the practical risk is bounded, but the documented condition is the 1-D form presented as the general one, and a user tightening C_adv to get a specific Courant number will not get it.

**Recommended fix.** Either compute the per-axis sum (requires the sharp-reconstructed component vector, already available inside `dec_max_speed`) or document explicitly that the bound is the per-direction form and is non-conservative by up to sqrt(D) for diagonal flow.

**Adversarial check.** step.rs:140 is `self.cfl_advective * self.dx_min / max_speed` and max_speed is the Euclidean vertex norm (diagnostics.rs:131-138, sum of squares then sqrt). The auditor's reference is correct: the multidimensional Courant number is dt * sum_i |u_i|/dx_i, and for a unit vector sum_i|n_i| ranges over [1, sqrt(D)], so on a uniform grid the code's number is smaller than the standard one by up to sqrt(D) and the guard admits a correspondingly larger dt. The docstring at step.rs:135-136 states the 1-D form as if general. I checked for compensation and found none - dx_min is a min over axes, which is conservative only on anisotropic grids and gives nothing on the uniform lattices this solver targets. Severity minor is right: RK4's stability region absorbs the factor in practice, and the guard errors rather than silently changing physics.

> Evidence re-read: step.rs:135-148 (advective limit and its docstring); diagnostics.rs:123-139 (Euclidean max_speed); dec_ns_solver/mod.rs:79-91 (dx_min is a min over all grade-1 cells, not a per-axis vector)

---

### 16.9 [INFO] Module doc writes the governing equation with a viscous sign that contradicts the implemented rate

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/mod.rs:12`
- **Auditor confidence:** confirmed

**Claim.** The module-level governing equation (mod.rs:12) uses a bare 'Delta' next to a definition of 'Delta_dR' without saying which operator the code applies, so the viscous sign convention is ambiguous at the crate's entry point. The rate module (dec_ns_rate.rs:7, :47-49) states it correctly as -nu Delta_dR u, and the implementation matches. The module doc should be rewritten to use the same symbol.

**Code evidence.**

```
mod.rs:11-13: "```text\n//! du/dt = P( - i_u omega + nu Delta u + g ),   omega = d u,   Delta_dR = -grad^2\n//! ```"

The implementation, dec_ns_rate.rs:716 and 722: `.map(|i| R::zero() - conv_s[i] - nu * lap_s[i] + g_s[i])` and `.map(|i| R::zero() - conv_s[i] - nu * lap_s[i])`, with lap = `self.manifold.laplacian_of(u_slice, 1)` (line 707) which is Delta_dR.

The correct statement is already in dec_ns_rate.rs:7: "P(-i_u(du) - nu Delta_dR u + g)" and dec_ns_rate.rs:47-49: "the Hodge-de Rham Laplacian satisfies `Delta_dR = -grad^2`, so the physical diffusion `+nu grad^2 u` enters as `-nu Delta_dR u`."
```

**Reference form.** Incompressible NS in rotational form: du/dt = P(-omega x u + nu*grad^2 u + g). With Delta_dR = -grad^2 the viscous term must appear as -nu*Delta_dR u.

**Impact.** A reader checking the sign convention against the module doc will conclude the solver applies anti-diffusion, or will fail to reconcile the module doc with the rate doc. On a certification review this is exactly the kind of ambiguity that triggers a finding against the whole viscous term.

**Recommended fix.** Write mod.rs:12 as `du/dt = P(- i_u omega - nu Delta_dR u + g)` so the symbol in the equation and the symbol in the definition are the same operator.

**Adversarial check.** The quoted line is verbatim (mod.rs:11-13) and the implementation is as cited: dec_ns_rate.rs:707 computes `laplacian_of(u_slice, 1)` (Delta_dR) and :716/:722 build `-conv - nu*lap`, i.e. -nu*Delta_dR, which with Delta_dR = -grad^2 is the physical +nu*grad^2. The auditor's reference form is right. But the over-read is on the auditor's side: the doc line writes the equation with an unsubscripted 'Delta' and then separately notes 'Delta_dR = -grad^2' - it distinguishes the two symbols rather than equating them, so the natural reading of 'nu Delta u' is nu*grad^2, which is correct. This is an unlabelled-symbol ambiguity, not a stated sign error, and dec_ns_rate.rs:7 and :47-49 state the convention unambiguously. Downgrading to info: worth fixing for a certification package (write '-nu Delta_dR u' in the module doc to match the rate doc), but nothing in the doc asserts anti-diffusion.

> Evidence re-read: mod.rs:11-13 (equation with both Delta and Delta_dR); dec_ns_rate.rs:701-726 (rhs = -conv - nu*lap with lap = laplacian_of(...,1)); dec_ns_rate.rs:7 and :46-49 (correct convention stated twice)

---

### 16.10 [MINOR] Pressure diagnostic documents both outputs as mean-zero-gauged; the static pressure is not

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/pressure.rs:14`
- **Auditor confidence:** confirmed

**Claim.** The doc states both returned 0-forms come back mean-zero-gauged. Only the Bernoulli form does (it is the projection potential, mean-subtracted). The static form is B - |u|^2/2, whose mean is -mean(|u|^2/2), which is strictly negative for any nonzero field.

**Code evidence.**

```
pressure.rs:13-15: "//! Both come back mean-zero-gauged through the projection's gauge\n//! fixing; absolute pressure on a torus is defined only up to a constant." and pressure.rs:31-33: "both\n/// up to the additive gauge constant fixed by the projection's\n/// mean-zero convention."

pressure.rs:69-74: `let stat: Vec<R> = bernoulli.iter().zip(kinetic.iter()).map(|(b, k)| *b - *k).collect();` where `bernoulli` is the mean-subtracted potential (leray.rs:103 `subtract_mean_in_place(&mut phi)`) and `kinetic` is the strictly non-negative `|u|^2/2` (pressure.rs:63-67).
```

**Reference form.** A field is mean-zero-gauged iff the sum of its coefficients is zero. mean(B - |u|^2/2) = mean(B) - mean(|u|^2/2) = 0 - mean(|u|^2/2) != 0 whenever u is not identically zero.

**Impact.** A user comparing absolute static-pressure levels between two runs or against a reference (for example forming C_p without an explicit freestream reference point) will pick up an offset equal to the mean kinetic energy, which differs between runs and between grids.

**Recommended fix.** State that the Bernoulli form is mean-zero and the static form inherits that gauge shifted by -mean(|u|^2/2), or explicitly re-gauge the static form to mean zero before returning it.

**Adversarial check.** The doc quotes are verbatim: pressure.rs:14-15 'Both come back mean-zero-gauged through the projection's gauge fixing' and :31-33 'both up to the additive gauge constant fixed by the projection's mean-zero convention'. The code contradicts it. `bernoulli` is the projection potential, which is mean-subtracted (leray.rs:734-735 on the eliminate=false branch that this path takes - verified via project_raw -> leray_project_constrained_weighted_opts with empty reference vertices). `kinetic` (pressure.rs:63-67) is a sum of squares times 0.5, hence non-negative and strictly positive somewhere for any non-zero field. `stat = bernoulli - kinetic` (pressure.rs:69-74) therefore has mean -mean(|u|^2/2) < 0. The arithmetic is trivially decisive and there is no compensating re-gauge anywhere between the subtraction and the PressureZeroForm construction at :81-86.

> Evidence re-read: pressure.rs:13-15, :29-33 (both doc claims), :53 (bernoulli = potential), :60-67 (kinetic, non-negative), :69-74 (stat = B - k), :81-86 (no re-gauge); leray.rs:733-735 (mean subtraction applies to phi only)

---

### 16.11 [INFO] Absolute 1e-12 threshold applied to a dimensional Hodge mass silently drops operator rows

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:637`
- **Auditor confidence:** confirmed

**Claim.** 1e-12 is an untraced absolute tolerance compared against the grade-1 Hodge mass, which is dimensional for D >= 3 (length^(D-2)); the test should be relative to max_j|m_j|. In practice it targets structural zeros - the cell-merging floor keeps free duals above it and fully-dry duals at exactly 0 - so the hazard is limited to D >= 3 runs in small units with the merge floor left at its zero default.

**Code evidence.**

```
dec_ns_rate.rs:637-639: `let zero_tol = <R as FromPrimitive>::from_f64(1e-12)\n    // Coverage exemption: 1e-12 lifts into every real field.\n    .expect(\"1e-12 is representable in every RealField\");`
used at dec_ns_rate.rs:665-669: `*slot = if m1[j].abs() <= zero_tol { R::zero() } else { dot / m1[j] };`

The same absolute constant governs the fused path's codifferential assembly, deep_causality_topology/src/types/manifold/differential/stencil/build.rs:92-100: `let zero_tol = <R as FromPrimitive>::from_f64(1e-12)...;\nfor r in 0..rows {\n    if star_km1[r].abs() <= zero_tol {\n        entries.push(Vec::new());\n        continue;\n    }` - the row is emptied, i.e. the operator silently outputs zero there.

The masses being tested are aperture-scaled with no floor (has_hodge_star.rs:263-266).
```

**Reference form.** A degeneracy test on a dimensional diagonal should be relative: |m_i| <= eps * max_j |m_j|, so the decision is invariant under a change of units and under uniform grid refinement.

**Impact.** In SI units on a fine grid (h = 1e-4 m, 3-D, grade 1 mass = h) a cut cell with wetted fraction below 1e-8 crosses the threshold and its codifferential row is silently emptied - the operator returns zero there with no diagnostic. Conversely the same geometry expressed in millimetres would not cross it. The behaviour of the discretization depends on the unit system.

**Recommended fix.** Replace both absolute 1e-12 constants with a relative test against the maximum mass of the same grade, and emit a counted diagnostic (number of dropped rows) rather than failing silently.

**Adversarial check.** Both literals are exactly where claimed (dec_ns_rate.rs:637-639 used at :665-669; stencil/build.rs:92-100 emptying the row) and the general point that an absolute tolerance on a dimensional diagonal should be relative (|m_i| <= eps*max_j|m_j|) is correct. Two corrections. (1) The dimensionality is D-dependent: the grade-1 star magnitude is length^(D-2k) = length^(D-2), so it is dimensional only for D >= 3 - in the 2-D cylinder case that dominates this crate's validation it reduces to the dimensionless clip fraction in [0,1] and the unit-system argument does not bite. (2) The design intent is to catch structural zeros, not small values: registry.rs:485-495 deliberately keeps fully-dry duals at exactly 0 (interior-solid, pinned and dropped from the dynamics) while inflating every free body-adjacent fraction to the cell-merging floor, so with the floor enabled nothing lands in the (0, 1e-12) band by construction. The residual defect is real but narrow: with the floor left at its zero default and D >= 3 in small SI units, a legitimate sliver row can be emptied with no diagnostic. Also worth noting the dec_ns_rate.rs copy sits in `convective_skew_generic`, which the docstring at :620-622 scopes to 'test-scale lattices only'.

> Evidence re-read: dec_ns_rate.rs:636-639, :665-669, and the generic-path scope note at :617-622; deep_causality_topology/.../stencil/build.rs:92-100 (row emptied); has_hodge_star.rs:130-133 (magnitude = length^(D-2k)); cut_cell/registry.rs:485-495 (floor keeps free fractions above the band, dry stays exactly 0)

---

### 16.12 [MINOR] Default CFL safety factor 0.9 is an untraced constant duplicated in two places

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/mod.rs:93`
- **Auditor confidence:** confirmed

**Claim.** The default advective and diffusive safety factors are both 0.9, with no cited source and no statement of what stability analysis they are a margin against. The same literal is duplicated in the config builder, so the two can drift apart.

**Code evidence.**

```
mod.rs:93-95: `let default_safety = R::from_f64(0.9)\n    // Coverage exemption: 0.9 lifts into every real field.\n    .expect(\"0.9 lifts into R\");` then `cfl_advective: default_safety, cfl_diffusive: default_safety,`

Duplicated at dec_config/mod.rs:76: `let safety = R::from_f64(0.9).expect(\"0.9 lifts into R\");` with `cfl_advective: safety, cfl_diffusive: safety,`

The bound the factor multiplies is the forward-Euler bound (step.rs:155 `dx_min^2/(2*D*nu)`), while the integrator is RK4 (step.rs:65), whose real-axis stability limit is roughly 2.79x the forward-Euler one - so the effective conservatism is unstated.
```

**Reference form.** A CFL safety factor is a margin against a named stability limit. Applying 0.9 to a forward-Euler diffusive bound while marching RK4 is conservative by an unstated factor; applying it to the 1-D advective form is non-conservative by up to sqrt(D) (see the separate advective finding). Neither is documented.

**Impact.** The default is conventional and user-overridable, and it gates only whether a run errors rather than the computed physics, so the risk is bounded. But for certification the constant has no traceable provenance and the two copies are an obvious drift hazard.

**Recommended fix.** Define the factor once as a named, documented constant (stating the stability analysis it margins against and that the diffusive bound is the forward-Euler one applied under RK4) and have `DecNsConfigReady::new` and `DecNsSolver::new` both read it.

**Adversarial check.** Both literals are verbatim at the cited lines: dec_ns_solver/mod.rs:93-103 (`default_safety` fed to both cfl_advective and cfl_diffusive) and dec_config/mod.rs:76-83 (an independent `let safety = R::from_f64(0.9)` fed to the same two fields). Neither references a shared constant, and I found no `const` or doc block anywhere in the crate tying 0.9 to a stability analysis - the only nearby comment is the coverage exemption ('0.9 lifts into every real field'), which explains the `expect`, not the value. The auditor's supporting arithmetic is also right: step.rs:155 applies the factor to the forward-Euler diffusive bound dx_min^2/(2*D*nu) while step.rs:65 marches RK4, whose real-axis stability boundary is about 2.78 - so the effective margin is unstated in both directions. Correctly scoped as minor: the constant is user-overridable via with_cfl_factors and gates only whether a run errors.

> Evidence re-read: dec_ns_solver/mod.rs:93-106 (first copy) and :272-291 (with_cfl_factors override); dec_config/mod.rs:74-83 (second, independent copy); step.rs:150-167 (forward-Euler diffusive bound) vs step.rs:65 (Rk4); no named constant found for 0.9 in the crate

---

### 16.13 [MINOR] Module docs describe the solver as periodic while the code implements walls, moving walls, free slip, open boundaries and immersed cut bodies

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/dec/mod.rs:6`
- **Auditor confidence:** confirmed

**Claim.** Both the DEC module doc and the solver module doc open by describing a periodic solver, and the module-layout list enumerates only rate, solver, outputs, diagnostics and wrappers. The implemented capability set is substantially larger and none of the boundary machinery is mentioned.

**Code evidence.**

```
mod.rs:6: "//! The periodic DEC-native incompressible Navier-Stokes solver." and dec_ns_solver/mod.rs:6: "//! The periodic DEC-native incompressible Navier-Stokes solver: owns the ..."

Undocumented in that list but present: `pub(crate) mod boundary;` (mod.rs:62) exporting `BodyForceZone, BoundaryZone, Inflow, MovingWall, Outflow, SlipWall` (mod.rs:80); `surface_force` (mod.rs:71, 89-91); `spectral_diffusion` (mod.rs:69); `energy_budget` (mod.rs:67); `uncertain_inflow` (mod.rs:74-75, 92-96); `dec_config` (mod.rs:78). In the solver: `with_zones` (dec_ns_solver/mod.rs:123), `with_moving_wall` (:182), `with_staircase_noslip` (:254), `with_warm_start` (:244), `with_spectral_diffusion` (:267).
```

**Reference form.** Module documentation should enumerate the module's capabilities; the module-layout list in mod.rs:28-39 is presented as that enumeration and omits roughly half the public surface.

**Impact.** An engineer scoping the crate from the module docs will not discover the boundary-zone abstraction, the surface-force diagnostics, the energy budget, or the immersed-body support, and may conclude the solver cannot handle wall-bounded or external-flow cases - which is precisely the avionics use case.

**Recommended fix.** Drop "periodic" from both opening lines and extend the module-layout list in mod.rs:28-39 with `boundary`, `surface_force`, `energy_budget`, `spectral_diffusion`, `dec_config` and the immersed cut-cell no-slip.

**Adversarial check.** Both openings are verbatim - mod.rs:6 'The periodic DEC-native incompressible Navier-Stokes solver.' and dec_ns_solver/mod.rs:6 the same phrase - and the module-layout list at mod.rs:28-39 enumerates only DecNsRate, DecNsSolver, StepOutput/RunOutput, diagnostics and wrappers. I checked the declared surface against it: mod.rs:62-78 also declares boundary, energy_budget, spectral_diffusion, surface_force, uncertain_inflow and dec_config, and mod.rs:80-96 re-exports BoundaryZone/Inflow/MovingWall/Outflow/SlipWall/BodyForceZone, EnergyBudget, the three surface-force functions and the whole uncertain-inflow surface. On the solver, with_zones (:123), with_moving_wall (:182), with_warm_start (:244), with_staircase_noslip (:254) and with_spectral_diffusion (:267) are all present and all absent from the doc. The word 'periodic' is actively misleading given that no_slip.rs, boundary.rs and the cut-cell path exist precisely to handle the non-periodic cases.

> Evidence re-read: mod.rs:6, :28-39 (layout list), :62-96 (declared modules and re-exports); dec_ns_solver/mod.rs:6, :123, :182, :244, :254, :267; no_slip.rs:6-34 (wall machinery the doc never mentions)

---

### 16.14 [INFO] dec_sample_velocity's floor search has no downward branch, so a negative query coordinate silently extrapolates

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/diagnostics.rs:182`
- **Auditor confidence:** confirmed

**Claim.** The floor search starts at k = 0 and only steps upward. For a query point with p[j] < 0 the loop exits immediately with lo[j] = 0 and frac[j] = g < 0, producing negative interpolation weights - an extrapolation reported as a sample. The twin routine in surface_force.rs has the missing downward branch.

**Code evidence.**

```
diagnostics.rs:180-187: `let g = p[j] / dx[j];\nlet mut k = 0usize;\nwhile R::from_usize(k + 1).unwrap_or_else(R::one) <= g {\n    k += 1;\n}\nlo[j] = k;\nfrac[j] = g - R::from_usize(k).unwrap_or_else(R::zero);`

Compare surface_force.rs:190-199, which has both directions: `let mut k = base[j];\nwhile R::from_usize(k + 1).unwrap_or_else(R::one) <= g { k += 1; }\nwhile k > 0 && R::from_usize(k).unwrap_or_else(R::zero) > g { k -= 1; }`

The docstring claims only that out-of-domain corners contribute zero (diagnostics.rs:145-146), which does not cover negative fractions.
```

**Reference form.** Multilinear interpolation requires 0 <= frac < 1 in every axis; outside that range the weights are no longer a convex combination and the result is an extrapolation, not an interpolation.

**Impact.** Bounded: this is a read-only probe used for the Strouhal wake signal and the Ghia centerline profile, both of which query in-domain points. A caller passing a negative coordinate gets a silently extrapolated value rather than an error. Also, the `unwrap_or_else(R::one)` fallback at line 182 would make the loop non-terminating for a scalar type where `from_usize` fails, though no workspace scalar has that property.

**Recommended fix.** Reject p[j] < 0 with a `PhysicsError::DimensionMismatch`, or mirror the surface_force twin's clamped two-directional search, and replace the `unwrap_or_else` fallbacks with an explicit error.

**Adversarial check.** Verified line by line. diagnostics.rs:179-187 seeds k = 0 and has only the ascending `while from_usize(k+1) <= g` loop; for p[j] < 0 the guard 1 <= g is false immediately, so lo[j] = 0 and frac[j] = g < 0, and the corner weights at :195-199 become one negative and one greater than 1 - an extrapolation, not a convex combination. The twin at surface_force.rs:188-200 does carry the descending branch (`while k > 0 && from_usize(k) > g { k -= 1; }`), so the asymmetry is real and the sibling routine is the correct pattern. The docstring at diagnostics.rs:144-146 only promises that out-of-domain corners contribute zero, which is a different guarantee. The auditor's non-termination note about `unwrap_or_else(R::one)` is also technically sound and correctly flagged as unreachable for workspace scalars. Info severity is right: both in-tree callers (wake probe, centerline profile) query in-domain points.

> Evidence re-read: diagnostics.rs:141-149 (docstring), :177-187 (upward-only floor search), :189-206 (corner weights); surface_force.rs:176-200 (the twin routine, with both branches at :192-197)

---
