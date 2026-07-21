# deep_causality_cfd — DEC boundary-condition zones (`src/solvers/dec/boundary/`) and the sensor-fed uncertain-inflow machinery (`src/solvers/dec/uncertain_inflow/`)

**Production readiness: `needs-work`**

The core numerics I could check are right: the inflow Dirichlet value is imposed on edge 1-form DOFs as `speed·|e|` (inflow.rs:84-85), the per-stage rate is pinned to zero on `no_slip ∪ inflow` (dec_ns_rate.rs:350-356) so the value cannot drift through RK4, and the lift is re-applied *after* the projection (step.rs:104-123) — the exact ordering bug the audit asked about is absent. Mass consistency of the pressure Poisson problem is genuinely handled: a prescribed inflow without reference vertices is hard-rejected (leray.rs:793-799), and the reference pin makes the system Dirichlet rather than pure-Neumann, so no compatibility condition is silently violated. What blocks certification is the surrounding contract layer. Two documented `BoundaryZone` hooks do not do what the trait doc says: `collect_constrained_edges` is never called by the solver at all (repo-wide grep), and `collect_lift`'s `step` argument is only ever passed `0` (dec_ns_solver/mod.rs:149) despite being documented as step-dependent — a user-authored zone using either gets silent wrong physics with no error. `BodyForceZone` silently zip-truncates a mis-sized force cochain while the doc claims the solver validates it, and the validation is structurally incapable of failing. The `Outflow` doc characterizes the implementation as "a zero-gradient / convective outflow" when the code implements a pressure-pinned outlet with freed tangential edges — no convective condition exists. Finally, the verification evidence for the moving-wall lift factor is weak: the "bit-identical" zone test compares two byte-identical copies of the same loop, and the only value-level inflow test runs at h = 1 where the edge-length factor is unobservable. The uncertain-inflow path is honest about collapsing to a scalar before the solver, and the `log_entries == 2 × dropouts` invariant is genuinely produced by two independent code paths (not by construction), but the sensor's `Uncertain` id-keyed global sample cache means a cloned stream produces a frozen, non-varying "noisy" sensor while the docs claim per-step independence.

- Files read: **41**
- Findings raised: **17** — surviving adversarial verification: **17** (refuted: 0)
- Surviving by severity: major 5, minor 11, info 1
- Independently confirmed-correct items: **9**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Inflow imposes the Dirichlet velocity on edge 1-form DOFs (not node values), as the edge line integral | `deep_causality_cfd/src/solvers/dec/boundary/inflow.rs:84-85` | For a velocity 1-form u♭, the DEC degree of freedom on edge e is the line integral ∫_e u·dl. For an edge normal to the inflow face with uniform normal speed U, ∫_e u·dl = U·\|e\|. (Standard DEC/Whitne |
| Inflow edge column selection matches the lattice's non-periodic edge indexing | `deep_causality_cfd/src/solvers/dec/boundary/inflow.rs:55-61` | On a non-periodic axis a with N=shape[a] vertices, edges oriented along a occupy position[a] ∈ 0..=N-2 (N-1 edges). The last such column is N-2; the first is 0. |
| The prescribed inflow value is re-imposed AFTER the projection, and cannot drift during the RK4 stages | `deep_causality_cfd/src/solvers/dec/dec_ns_solver/step.rs:104-123 and deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:350-356` | A Dirichlet BC on a projection-based incompressible step must be enforced as the final operation of the step, or the Leray gradient correction dφ overwrites it; and the marched rate must be zero on th |
| The open-boundary pressure Poisson problem cannot be silently inconsistent: inflow without an outflow reference is rejected | `deep_causality_topology/src/types/manifold/differential/leray.rs:793-799 (binary path) and :436-442 (weighted path)` | For the pressure-Poisson/Leray projection ∇²φ = ∇·u* with pure-Neumann BCs, a solution exists only if ∮ u*·n dA = 0 (the discrete compatibility condition). Prescribing a nonzero net inflow with no Dir |
| SlipWall no-penetration (u·n = 0) is structural, so no outward-normal sign can be wrong | `deep_causality_cfd/src/solvers/dec/boundary/slip_wall.rs:49-68 and deep_causality_topology/src/types/lattice_complex/mod.rs:288-297` | Free-slip = u·n = 0 (no penetration) + ∂u_t/∂n = 0 (zero tangential shear), with n the outward face normal. |
| MovingWall rejects a non-zero wall-normal velocity component | `deep_causality_cfd/src/solvers/dec/boundary/moving_wall.rs:49-53` | A moving solid wall may translate tangentially only; a normal component would be a mass source through an impermeable face, incompatible with the projection's Neumann/no-penetration condition. |
| Body force enters the momentum rate with the correct (+) sign and dimensionally consistent units | `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:716 and :774` | Incompressible NS at ρ = 1: ∂u/∂t = −(u·∇)u + ν∇²u + g. In DEC edge-integral form: d(∫_e u·dl)/dt = −i_u(du♭) − νΔ_dR u♭ + ∫_e g·dl. |
| The `log_entries == 2 × dropouts` invariant is produced by two independent code paths, not by construction | `deep_causality_cfd/src/solvers/dec/uncertain_inflow/uncertain_boundary_source.rs:173-178; deep_causality_core/src/types/causal_effect_propagation_process/alternatable_value.rs:27-30; deep_causality_core/src/types/causal_effect_propagation_process/mod.rs:149-151` | A non-tautological accounting gate must fail if any contributing mechanism is removed, and its expected value must come from an independent source (here, the injection schedule STEPS/DROPOUT_EVERY, no |
| The uncertain collapse is genuine Monte-Carlo/QMC sampling, not a cosmetic pass-through | `deep_causality_uncertain/src/types/uncertain/uncertain_statistics.rs:17-29; deep_causality_cfd/src/solvers/dec/uncertain_inflow/uncertain_boundary_source.rs:131-159` | E[X] estimated by (1/n)Σ x_i over n draws; a cosmetic wrapper would return a stored parameter without drawing. |

## Findings

### 8.1 [MAJOR] `BoundaryZone::collect_constrained_edges` is documented as a solver stage but is never called by the solver

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/boundary_zone.rs:22`
- **Auditor confidence:** confirmed

**Claim.** The trait documents `collect_constrained_edges` as a folded solver stage, but `DecNsSolver::with_zones` never invokes it — the hook is dead. No in-crate zone implements it, so the impact is confined to user-authored zones, which would silently apply no boundary condition.

**Code evidence.**

```
boundary_zone.rs:18-22:
```
/// A zone declares **which solver stage(s) it affects** ... The solver folds every zone's
/// contribution at the matching stage:
///
/// * `collect_rate_source` — adds a forcing term ...
/// * `collect_constrained_edges` — edges pinned to zero in the constrained projection.
```
boundary_zone.rs:42-47 defines the hook with a no-op default. `DecNsSolver::with_zones` (dec_ns_solver/mod.rs:132-166) calls `collect_rate_source` (:136), `collect_lift` (:149), `collect_slip_edges` (:155), `collect_prescribed_edges` (:159) and `collect_reference_vertices` (:161) — and NOT `collect_constrained_edges`. Repo-wide grep for `collect_constrained_edges` returns only the trait definition, the `(A,B)` fold (:99-106), and one test call whose result is never asserted (tests/solvers/dec/boundary_construction_tests.rs:228-229, `let mut constrained = Vec::new(); zones.collect_constrained_edges(&m, &mut constrained);` — `constrained` is never read).
```

**Reference form.** Docs-vs-code parity: a trait hook documented as a solver stage must be consumed by the solver. Reference is the crate's own trait doc at boundary_zone.rs:18-22.

**Impact.** An engineer writing a custom `BoundaryZone` (e.g. a porous baffle, a sponge layer, an internal obstacle) that pins DOFs through `collect_constrained_edges` gets a solver that silently ignores the boundary condition. The march completes, reports a converged divergence-free field, and produces physically wrong results with no diagnostic. This is exactly the 'verification gate that cannot fail / silently wrong physics' category.

**Recommended fix.** Either (a) wire the hook into `with_zones` — collect it and union it into `rate.no_slip` before `recompute_rate_constrained()`, alongside the existing `apply_slip`/`set_open_boundary` calls; or (b) delete the hook from the trait and remove the claim from the doc. If it is retained as future API, mark it `#[doc(hidden)]`/`unimplemented` and state explicitly in the doc that the solver does not yet consume it. Add a test that a zone pinning an edge via this hook actually produces a zero on that edge after a step.

**Adversarial check.** The factual core is exactly right. boundary_zone.rs:22 does list `collect_constrained_edges` as a solver stage that 'the solver folds ... at the matching stage'. `DecNsSolver::with_zones` (mod.rs:132-166) calls collect_rate_source, collect_lift, collect_slip_edges, collect_prescribed_edges, collect_reference_vertices — the hook is absent. A repo-wide grep confirms only the trait definition, the (A,B) fold, and one test call whose `constrained` vector is never asserted. However 'critical' is overclaimed: no zone shipped in the crate (MovingWall, Inflow, Outflow, SlipWall, BodyForceZone) overrides the hook, so no shipped result is affected. The defect is a dead documented extension point, a latent hazard for a user-authored zone, not a live physics error.

> Evidence re-read: boundary_zone.rs:16-28 (trait doc listing the five folded stages incl. collect_constrained_edges), :42-47 (no-op default), :99-106 ((A,B) fold); dec_ns_solver/mod.rs:132-166 (with_zones — no collect_constrained_edges call); `grep -rn collect_constrained_edges --include=*.rs` returns only boundary_zone.rs and boundary_construction_tests.rs:229.

---

### 8.2 [MINOR] `collect_lift`'s `step` parameter is documented as step-dependent but is only ever passed 0

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/boundary_zone.rs:23`
- **Auditor confidence:** confirmed

**Claim.** The `step` parameter of `collect_lift` is vestigial: the solver folds the lift once at step 0 and never re-collects. The trait doc at :23/:49 implies step-dependence, though `with_zones`'s own doc discloses the step-0 restriction.

**Code evidence.**

```
boundary_zone.rs:23-24: `/// * `collect_lift` — prescribed (inhomogeneous, possibly step-dependent) edge values, e.g. a\n/// moving-wall tangential velocity.`
boundary_zone.rs:49: `/// Add this zone's prescribed (inhomogeneous) edge values for march step `step`.`
Sole solver call, dec_ns_solver/mod.rs:147-150:
```
// Fold the prescribed lift (static zones evaluate at step 0).
let mut lift = alloc::vec::Vec::new();
zones.collect_lift(manifold, 0, &mut lift);
solver.lift = lift;
```
Repo-wide grep for `collect_lift` finds no other non-test caller; every test call also passes `0`. `DecNsSolver::step` (step.rs:57-133) reuses the stored `self.lift` verbatim and never re-collects.
```

**Reference form.** Docs-vs-code parity: an API parameter documented as varying per march step must actually vary. Reference is the crate's own trait doc at boundary_zone.rs:23 and :49.

**Impact.** A user implementing a ramped inlet, an oscillating wall, or a gust profile — the natural avionics use cases — writes `fn collect_lift(&self, m, step, out)` keyed on `step`, sees it compile and run, and silently gets a constant boundary condition equal to the t=0 value. The transient the study exists to measure is never applied.

**Recommended fix.** Either move the lift collection into `step` (re-collect per step with the true step index, at the cost of `&mut self` or interior mutability), or remove the `step` parameter and restate the doc as 'static, evaluated once at construction'. Until then, change boundary_zone.rs:23 and :49 to state explicitly that the solver evaluates this hook once at step 0 and that step-dependent zones are not supported.

**Adversarial check.** The call-site facts hold: `zones.collect_lift(manifold, 0, &mut lift)` at mod.rs:149 is the only non-test caller, step.rs:121-123 reuses `self.lift` verbatim, and both shipped implementors (moving_wall.rs:63, inflow.rs:65) bind the parameter as `_step`. But the auditor omits two mitigations that change the severity. First, `with_zones`'s own rustdoc states the restriction explicitly — mod.rs:113-114 'every zone's collect_lift (at step 0) forms the prescribed lift' — and the inline comment at :147 repeats 'static zones evaluate at step 0'. Second, the crate's actual time-varying boundary path is the uncertain-inflow march, which re-derives the lift every step through `with_moving_wall` (inflow_march.rs:219), not through collect_lift. So this is an unused, misleadingly-documented parameter on the trait, not a silently-broken transient path.

> Evidence re-read: boundary_zone.rs:23-24 and :49 (doc text as quoted); dec_ns_solver/mod.rs:113-114 (doc: '(at step 0)'), :147-150 (comment + call); moving_wall.rs:63-67 and inflow.rs:65-70 (`_step` unused); step.rs:121-123 (`with_lift(&self.lift)`, no re-collect); grep confirms every other collect_lift call site is a test passing 0.

---

### 8.3 [MAJOR] `with_moving_wall` replaces the entire lift vector, silently discarding lifts contributed by other boundary zones

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_solver/mod.rs:230`
- **Auditor confidence:** confirmed

**Claim.** `with_moving_wall` assigns `self.lift = lift`, overwriting rather than merging. A solver built by `with_zones` with an `Inflow` (and/or a `MovingWall`) loses its entire prescribed-value set on any later `with_moving_wall` call, while the inflow edges remain registered as `prescribed` in the projection — so those edges get held at zero instead of the prescribed inflow speed.

**Code evidence.**

```
dec_ns_solver/mod.rs:218-231:
```
let mut lift = alloc::vec::Vec::new();
for (idx, cell) in complex.iter_cells(1).enumerate() { ... lift.push((idx, velocity[axis] * length)); }
self.lift = lift;
Ok(self)
```
The inflow edge set is stored separately and is NOT cleared: `set_open_boundary` (dec_ns_rate.rs:310-325) persists `self.inflow_edges`, and `step.rs:104-113` still passes `self.rate.inflow_edges()` to the projection as prescribed. `step.rs:121-123` then applies only the (now overwritten) `self.lift`.
The uncertain-inflow march calls this every single step: inflow_march.rs:219 `solver.with_moving_wall(zone.wall_axis(), zone.max_side(), velocity)`.
```

**Reference form.** A boundary-value setter must be composable with, or explicitly documented as exclusive of, the zone-composition path that also writes the same state. The crate's own doc (dec_ns_solver/mod.rs:41-47) describes `lift` as 'Prescribed tangential wall values ... populated by `Self::with_moving_wall`' without stating that it is destructive of `with_zones` output.

**Impact.** `DecNsSolver::with_zones(m, nu, dt, (Inflow::new(0,false,U), Outflow::new(0,true)))?.with_moving_wall(1, true, [v, 0.0])` compiles, runs, and produces a channel with zero inflow velocity but an inflow-typed projection — a silently wrong flow field with no error. Today's shipped uncertain path escapes this only because `uncertain_march_run.rs:88` materializes with the empty `()` zone set; nothing in the type system or a runtime check enforces that.

**Recommended fix.** Make `with_moving_wall` merge into (rather than replace) `self.lift`, de-duplicating by edge index with a documented last-writer-wins rule; or reject the call with `PhysicsError::PhysicalInvariantBroken` when `!self.lift.is_empty()` or `!self.rate.inflow_edges().is_empty()`, directing the caller to the `MovingWall` zone. Add a test composing `Inflow` + `with_moving_wall` and asserting the inflow edge still carries `U·h`.

**Adversarial check.** dec_ns_solver/mod.rs:230 is `self.lift = lift;` — assignment, not merge, and the loop at :218-229 builds `lift` fresh from the wall face alone. The inflow prescribed-edge set is stored separately in the rate (`set_open_boundary`) and is not cleared, and step.rs:104-113 still feeds `self.rate.inflow_edges()` to the open projection. I traced the consequence through the projector: leray.rs sets `mass_free[e] = 0` for both zeroed and prescribed edges and keeps prescribed edges at their field value (`v[e]` untouched), so the projected output at an inflow edge equals its input. With the inflow lift discarded, that input is 0 and stays 0 for the whole march — a channel with an inflow-typed projection and zero inflow. No error is raised and `with_zones` performs no check. The doc at mod.rs:41-47 and :169-181 never states the setter is destructive of `with_zones` output.

> Evidence re-read: dec_ns_solver/mod.rs:218-231 (fresh `lift`, `self.lift = lift`), :41-47 (field doc), :169-181 (method doc — no exclusivity note), :158-164 (with_zones sets open boundary separately); step.rs:104-113, :121-123; leray.rs:816-824 ('keep the prescribed (inflow) edges at their field value'), :806-815 (mass_free zeroed for prescribed); inflow_march.rs:219 (per-step call).

---

### 8.4 [MAJOR] Inflow edges that lie on a no-slip lateral wall are double-claimed: the projection zeroes them but the post-projection lift restores them, injecting flux the projection never balanced

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/inflow.rs:99`
- **Auditor confidence:** likely

**Claim.** `Inflow` claims every edge in the inlet column, including the ones sitting on lateral no-slip walls, which `NoSlipConstraint` also claims. In the projection the `zeroed` loop wins (their flux is excluded from the divergence right-hand side), but `step.rs` applies `with_lift` after `constrain_edges`, so the returned field carries a nonzero value on those edges. The result is a field that is neither no-slip nor divergence-balanced at the inlet corners, with no error raised.

**Code evidence.**

```
inflow.rs:99-104 claims all edges at the inlet column regardless of lateral wall membership:
```
for (idx, cell) in complex.iter_cells(1).enumerate() {
    let axis = cell.orientation().trailing_zeros() as usize;
    if axis == self.wall_axis && cell.position()[self.wall_axis] == col { out.push(idx); }
}
```
no_slip.rs:81-85 claims the same edge when a lateral axis is non-periodic:
```
let tangential = (0..D).any(|w| w != axis && !periodic[w] && (pos[w] == 0 || pos[w] + 1 == shape[w]));
```
For D=2, an axis-0 edge at position [0,0] on a fully open lattice satisfies BOTH.
The projection resolves the conflict in favour of zero — leray.rs:820-824:
```
let mut v: Vec<R> = field.as_slice().to_vec();
for &e in zeroed_edges { v[e] = R::zero(); }
```
(prescribed edges are NOT re-set afterwards), while the step resolves it in favour of the lift — step.rs:121-123:
```
let projected = projected.constrain_edges(self.rate.no_slip_edges()).with_lift(&self.lift);
```
`DecNsSolver::with_zones` (mod.rs:152-164) performs no disjointness check between `prescribed` and the no-slip set.
```

**Reference form.** A well-posed mixed BC requires each DOF to carry exactly one condition. In the Leray/pressure-projection framework, any boundary flux that appears in the final velocity must have been counted in the divergence RHS ∇·u* used to solve for φ, otherwise ∇·u ≠ 0 at the adjacent vertices (standard fractional-step consistency requirement, e.g. Chorin 1968 / Kim & Moin 1985).

**Impact.** For the natural channel configuration `Inflow(0,false,U) + Outflow(0,true)` on a fully open lattice (no `SlipWall` on the lateral axes), the inlet corner edges violate no-slip and inject an unbalanced flux. The divergence residual reported by `dec_divergence_residual` degrades at those vertices, but nothing errors and nothing gates it inside `step`. The two in-repo cases dodge this by construction (inflow_outflow_tests.rs:23 uses a periodic y-axis; dec_cylinder_verification/main.rs adds `SlipWall(1,false)`+`SlipWall(1,true)` which un-pin exactly those corner edges) — so the defect is latent and untested rather than absent.

**Recommended fix.** Reject overlapping claims at construction: in `with_zones`, after collecting `prescribed` and applying `slip`, assert `prescribed ∩ rate.no_slip_edges() == ∅` and return `PhysicsError::PhysicalInvariantBroken` naming the offending edges. Alternatively define an explicit precedence (e.g. Inflow taps a wall-clipped profile that is zero at the lateral walls) and document it on `Inflow`. Add a regression test on a fully open lattice with Inflow+Outflow and no SlipWall asserting the divergence residual stays at solve tolerance.

**Adversarial check.** I re-derived the overlap and it is real. Inflow::collect_prescribed_edges (inflow.rs:99-104) and collect_lift (:79-86) claim every axis-`wall_axis` edge in the inlet column with no lateral-wall exclusion. NoSlipConstraint::new (no_slip.rs:81-85) marks an axis-`a` edge tangential when any other non-periodic axis `w` has pos[w] at 0 or shape[w]-1 — for D=2, wall_axis=0, max_side=false, the edge at position [0,0] satisfies both. The resolution asymmetry is as claimed: leray.rs:817-820 zeroes `v[e]` for every zeroed (no-slip) edge and does not re-set prescribed edges afterwards, so that edge contributes zero flux to the divergence RHS; step.rs:121-123 then applies `constrain_edges(no_slip)` followed by `with_lift(&self.lift)`, so the returned field carries `speed·length` there. The auditor's reference form is correct standard fractional-step consistency (Chorin/Kim-Moin): boundary flux present in the final velocity must appear in the RHS used to solve for phi. `with_zones` (mod.rs:152-164) does no disjointness check. I also confirmed both in-repo cases dodge it — inflow_outflow_tests.rs:24 uses periodic y, so no lateral no-slip exists at all. The defect is genuine but latent and untested.

> Evidence re-read: inflow.rs:79-86, :94-105; no_slip.rs:69-87 (tangential predicate); leray.rs:806-824 (mass_free zeroing, `for &e in zeroed_edges { v[e] = R::zero(); }`, prescribed kept at field value); step.rs:104-123; dec_ns_solver/mod.rs:152-164 (no disjointness check); inflow_outflow_tests.rs:24 `LatticeComplex::new([nx, ny], [false, true])`.

---

### 8.5 [MAJOR] `BodyForceZone` silently truncates or zero-pads a mis-sized force cochain, and the documented validation is structurally unable to catch it

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/body_force_zone.rs:37`
- **Auditor confidence:** confirmed

**Claim.** `collect_rate_source` uses `zip`, which truncates to the shorter of the accumulator and the force tensor. Because `with_zones` allocates the accumulator at the correct length `n1` and only then hands it to `BodyForceOneForm::new`, the length validation the doc points to can never fail — a force cochain of the wrong length is silently zero-padded (too short) or clipped (too long).

**Code evidence.**

```
body_force_zone.rs:36-40:
```
fn collect_rate_source(&self, _manifold: &Manifold<LatticeComplex<D, R>, R>, acc: &mut [R]) {
    for (a, f) in acc.iter_mut().zip(self.force.as_slice().iter()) { *a += *f; }
}
```
`BodyForceZone::new` (body_force_zone.rs:25-27) performs no validation at all.
dec_ns_solver/mod.rs:132-143:
```
let n1 = manifold.complex().num_cells(1);
let mut source = alloc::vec![R::zero(); n1];
zones.collect_rate_source(manifold, &mut source);
... let tensor = CausalTensor::new(source, alloc::vec![n1]) ...
Some(BodyForceOneForm::new(tensor, manifold)?)
```
The tensor passed to `BodyForceOneForm::new` always has length `n1`, so the `validate_graded_field(&field, 1, ...)` length check (body_force_one_form.rs:32) is unreachable-as-failing.
Doc claim, body_force_zone.rs:16-17: `/// gradient `G·h` on the x-edges) added to the rate source. The carried tensor is the grade-1\n/// edge cochain; the solver validates and wraps it as a `BodyForceOneForm` when assembling.`
The truncating behaviour is pinned as expected by tests/solvers/dec/boundary_construction_tests.rs:196-203, which feeds a length-3 force and a length-3 `acc` on a 5×5 manifold (n1 = 40).
```

**Reference form.** A validation gate must be able to fail on the input it claims to validate. Reference: the crate's own doc claim at body_force_zone.rs:17 plus `BodyForceOneForm::new`'s documented `DimensionMismatch` contract (body_force_one_form.rs:25-27).

**Impact.** A user who builds a body force from a subset of edges (a common way to force only the x-edges), or who changes the lattice shape without rebuilding the tensor, gets a silently zero-padded forcing — the flow is driven over only part of the domain — with the solver reporting success. For a Poiseuille/Couette calibration this shifts the whole velocity profile with no diagnostic.

**Recommended fix.** Validate in `BodyForceZone::new` against a supplied edge count, or add a length check inside `collect_rate_source` (returning a `Result`, or panicking with a clear message in a `debug_assert_eq!(acc.len(), self.force.len())` plus a hard check in `with_zones` comparing `zone.force().len()` to `n1`). Correct the doc at body_force_zone.rs:17 to state where validation actually happens. Change the test at boundary_construction_tests.rs:196-203 to assert rejection rather than truncation.

**Adversarial check.** body_force_zone.rs:36-40 is the quoted `zip` loop, which truncates to the shorter side; `new` (:25-27) validates nothing. In `with_zones` the accumulator is `alloc::vec![R::zero(); n1]` (mod.rs:135) and only that n1-length vector is handed to `CausalTensor::new(source, vec![n1])` → `BodyForceOneForm::new` (mod.rs:138-140), so the length check downstream can never observe a wrong length on this path: a short cochain is zero-padded, a long one clipped, both silently. The doc at :17 ('the solver validates and wraps it as a BodyForceOneForm') therefore points at a gate that cannot fail for zone-supplied input. The truncation is pinned as expected behaviour by boundary_construction_tests.rs:196-203, which feeds a length-3 force and a length-3 acc while the manifold is 5x5 (N=5 at :16, so n1 = 2·4·5 = 40) — the test never exercises the real accumulator length.

> Evidence re-read: body_force_zone.rs:15-17 (doc), :25-27 (no validation), :36-40 (zip); dec_ns_solver/mod.rs:132-143 (n1 accumulator → tensor → BodyForceOneForm::new); boundary_construction_tests.rs:16 (N=5), :196-203 (len-3 force, len-3 acc, asserts [1.5,1.0,1.25]).

---

### 8.6 [MINOR] `Outflow` is documented as a 'zero-gradient / convective outflow'; the implementation is a pressure-pinned outlet with no convective condition

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/outflow.rs:22`
- **Auditor confidence:** confirmed

**Claim.** The word 'convective' in Outflow's doc is unsupported — no convection velocity or normal-derivative operator exists — and the parenthetical about a 'boundary time-update' refers to absent code. The implemented condition is a pressure-pinned ('do-nothing') outlet with freed tangential edges, for which 'zero-gradient' is a loose but defensible description.

**Code evidence.**

```
outflow.rs:18-22:
```
/// ... It carries no prescribed velocity — the outflow is determined by incompressibility
/// (a zero-gradient / convective outflow; the boundary time-update lands with the outflow group).
```
The whole implementation is outflow.rs:47-87 and contains exactly two hooks:
`collect_reference_vertices` (:48-64) — `for (idx, cell) in complex.iter_cells(0).enumerate() { if cell.position()[self.wall_axis] == pos { out.push(idx); } }`
`collect_slip_edges` (:66-86) — `if axis != self.wall_axis && cell.position()[self.wall_axis] == pos { out.push(idx); }`
There is no time-update, no normal-derivative stencil, and no convection velocity anywhere in the file. The parenthetical 'the boundary time-update lands with the outflow group' refers to work that is not present in the code.
```

**Reference form.** A convective outflow BC is ∂u/∂t + U_c ∂u/∂n = 0 integrated on the outflow plane; a zero-gradient outflow is ∂u/∂n = 0 (see e.g. Ferziger & Perić, 'Computational Methods for Fluid Dynamics', §8.5 on outflow boundary conditions). Neither is implemented; what is implemented is a Dirichlet pressure outlet (φ = 0) with a free tangential velocity.

**Impact.** An avionics reviewer reading 'convective outflow' will assume the wake convects out without reflection at the documented rate and will size the downstream domain accordingly. A pressure-pinned outlet has different reflection characteristics; the difference matters directly for the wake/shedding studies this zone exists to support. The claim also cannot be traced to any code the reviewer can check.

**Recommended fix.** Restate the doc as what the code does: 'an outflow pressure reference (φ = 0 on the face) with free tangential edges; the outflow velocity is whatever the projection produces'. Remove 'zero-gradient / convective' and the forward reference to a time-update that does not exist, or track it as an explicit TODO with an issue link. Also promote the `collect_slip_edges` behaviour from an in-method comment (:71-73) into the type doc, since it materially changes the no-slip set.

**Adversarial check.** The doc text is verbatim at outflow.rs:21-22 and the implementation is exactly the two hooks cited (collect_reference_vertices :48-64, collect_slip_edges :66-86) — no normal-derivative stencil, no convection speed, no boundary time-update anywhere in the file. The auditor's reference forms are correct (convective: du/dt + U_c du/dn = 0; zero-gradient: du/dn = 0; Ferziger & Peric section 8.5). So 'convective' is an unsupported label and the parenthetical 'the boundary time-update lands with the outflow group' describes work that is not present. I part company on two points. First, what is implemented is not merely a Dirichlet pressure pin: it also frees the face's tangential edges from the auto no-slip set (:71-73, with the stated rationale that pinning them 'would reflect the wake'), which is the discrete analogue of the 'do-nothing' / traction-free outlet, and calling that loosely 'zero-gradient' is defensible rather than false. Second, the wake harness README explicitly disclaims a true outflow surface ('a confined, periodic-x harness (a prescribed moving wall, not a true inflow/outflow surface)'), so no shipped result rests on the label. This is a doc-wording overclaim, not a physics defect.

> Evidence re-read: outflow.rs:18-22 (doc), :47-87 (entire impl — two hooks only), :71-73 (slip-edge rationale comment); verification/README.md:139-145 (harness disclaims a true inflow/outflow surface).

---

### 8.7 [MINOR] `march_inflow`'s documented error contract is not implemented: equal `wall_axis`/`flow_axis` is neither rejected nor reported as documented, and the 'cannot fail on a valid stream' guarantee is false

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/uncertain_inflow/inflow_march.rs:287`
- **Auditor confidence:** confirmed

**Claim.** `wall_axis` range IS validated with the documented `DimensionMismatch` (via the pre-flight `with_moving_wall`). What fails: axis equality is not checked as such, is reported as `PhysicalInvariantBroken` rather than the documented `DimensionMismatch`, and escapes pre-flight entirely when `default_inflow == 0`, surfacing mid-march — falsifying the 'cannot fail on a valid stream' guarantee at :283-284.

**Code evidence.**

```
inflow_march.rs:280-290 (doc):
```
/// The zone configuration is validated once against the lattice before the march, so the per-step
/// boundary reconfiguration cannot fail on a valid, finite stream.
///
/// # Errors
/// * `PhysicsError::DimensionMismatch` when `wall_axis`/`flow_axis` are out of range, equal, or
///   the stream is shorter than `steps`.
```
inflow_march.rs:301-318 (the actual validation):
```
if zone.flow_axis() >= D { ... DimensionMismatch ... }
if stream.len() < steps { ... DimensionMismatch ... }
let mut probe = [R::zero(); D];
probe[zone.flow_axis()] = zone.default_inflow();
let solver = solver.with_moving_wall(zone.wall_axis(), zone.max_side(), probe)?;
```
With `flow_axis == wall_axis` and `default_inflow == 0`, `probe` is all zeros, so `with_moving_wall`'s guard `if velocity[wall_axis] != R::zero()` (dec_ns_solver/mod.rs:204) does not fire and construction succeeds. At the first present sample, inflow_march.rs:217-219 sets `velocity[zone.flow_axis()] = inflow` (nonzero) with `flow_axis == wall_axis`, and `with_moving_wall` returns `PhysicalInvariantBroken` — a different variant, at a different time, than documented. Even with `default_inflow != 0` the error is `PhysicalInvariantBroken`, not the documented `DimensionMismatch`.
```

**Reference form.** Docs-vs-code parity on an `# Errors` contract: every listed condition must be checked, and the listed error variant must be the one returned. Reference: the crate's own rustdoc at inflow_march.rs:286-290.

**Impact.** A caller writing `match err { DimensionMismatch(_) => reconfigure, _ => abort }` (exactly the pattern the crate's own tests use — uncertain_inflow_tests.rs:241 and :256 match on `PhysicsErrorEnum::DimensionMismatch`) mishandles the equal-axis case. Worse, with a zero fallback the misconfiguration escapes pre-flight validation entirely and aborts the run partway through a long march, wasting the run and yielding a partial state that a caller could mistake for a completed one.

**Recommended fix.** Add explicit checks in `march_inflow` before the probe: `wall_axis >= D` → `DimensionMismatch`; `flow_axis == wall_axis` → `DimensionMismatch` (or `PhysicalInvariantBroken`, and fix the doc to match). Make the probe use a nonzero sentinel (e.g. `R::one()`) so the wall-normal guard is always exercised regardless of `default_inflow`. Then the 'cannot fail on a valid stream' sentence becomes true.

**Adversarial check.** One sub-claim is wrong: `wall_axis` range IS checked and DOES return the documented variant — the pre-flight `with_moving_wall` at inflow_march.rs:318 hits mod.rs:188-192, which returns `DimensionMismatch` for `wall_axis >= D`. The rest holds. Axis equality is never checked directly; the pre-flight relies on `with_moving_wall`'s `velocity[wall_axis] != zero` guard (mod.rs:204), and with `default_inflow == 0` the probe at :316-317 is all zeros so the guard does not fire and construction succeeds. At the first present, nonzero sample, :217-219 sets velocity[flow_axis]=inflow with flow_axis==wall_axis and the reconfiguration is rejected — as `PhysicalInvariantBroken`, not the documented `DimensionMismatch`, and mid-march rather than pre-flight. That falsifies the adjacent guarantee at :283-284 that 'the per-step boundary reconfiguration cannot fail on a valid, finite stream', and the in-source comment at :314-315 claiming the 'equal-axis rule' is validated once. Severity: the failure is loud (the process carries an Err and short-circuits), it requires a misconfiguration plus a zero fallback, and no shipped harness uses that combination.

> Evidence re-read: inflow_march.rs:283-290 (doc), :301-318 (validation: flow_axis range, stream length, probe + with_moving_wall), :314-315 (comment claiming equal-axis validation), :217-219, :219-235 (mid-march error_process); dec_ns_solver/mod.rs:188-192 (wall_axis DimensionMismatch — contradicts the auditor), :204-209 (PhysicalInvariantBroken).

---

### 8.8 [MAJOR] 'Every step is an independent realization' is false for a cloned sensor stream: the global sample cache is keyed on the `Uncertain` id, which survives `Clone`

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/uncertain_inflow/uncertain_boundary_source.rs:91`
- **Auditor confidence:** confirmed

**Claim.** The source documents that each step's collapse is an independent realization. Both the MC and QMC sample paths key a process-global memoization cache on `(uncertain.id, sample_index, sampler_kind)`, and `Uncertain` derives `Clone` carrying that id. A stream built by cloning one `MaybeUncertain` therefore yields byte-identical draws — and hence an identical collapsed inflow — at every march step: a frozen sensor wearing a noise wrapper.

**Code evidence.**

```
Doc claim, uncertain_boundary_source.rs:90-92: `/// per-sample Sobol digital shift is `base_seed ⊕ present.id()`, so every step is an independent,\n/// reproducible randomized-QMC realization.`
Id survives clone — deep_causality_uncertain/src/types/uncertain/mod.rs:31 `#[derive(Clone, Debug)]`, :58 `pub fn id(&self) -> usize`.
Cache key ignores everything but the id — deep_causality_uncertain/src/types/uncertain/uncertain_sampling.rs:21-29:
```
pub fn sample_with_index(&self, sample_index: u64) -> Result<T, UncertainError> {
    let key = (self.id, sample_index, SamplerKind::Mc);
    let computed_value = with_global_cache(|cache| { cache.get_or_compute(key, || { ... }) })?;
```
and :39-46 for the QMC variant, `let key = (self.id, sample_index, SamplerKind::Qmc);` — note the digital shift is not part of the key either.
The cloning construction is used in the crate's own tests: tests/solvers/dec/uncertain_inflow_tests.rs:77 `let stream = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS];` (also :207, :380).
```

**Reference form.** A per-step 'independent realization' requires the draws at step i and step j to be statistically independent. Under an id-keyed memoization cache, cloned nodes share an id and therefore return identical draws for identical sample indices — the draws are perfectly correlated, not independent.

**Impact.** The whole point of the uncertain-inflow machinery is that the boundary condition tracks a noisy sensor. A user who constructs the stream the obvious way (`vec![sample; N]`, or by cloning a template `Uncertain`) gets a constant boundary velocity while believing the sensor noise is being propagated — an unquantified, silent loss of the study's independent variable. The shipped verification harness escapes this only because `config.rs::sensor_stream` happens to build a fresh `Uncertain::normal(...)` inside the per-step closure; nothing documents that this is required.

**Recommended fix.** Document the requirement prominently on `UncertainBoundarySource`, `UncertainInflowZone` and `march_inflow`: each stream element must be a distinct `Uncertain` node (do not clone), because sampling is memoized per node id. Better, make it structural — have `InflowContext::new` (or the config builder) detect duplicate `Uncertain` ids in the stream and reject, or offset the sample index by the step index (`sample_with_index(step * n + i)`) so cloned nodes still draw distinct values. Also add the QMC digital shift to the QMC cache key upstream, since collapsing one node twice under different shifts currently returns the first shift's cached draws.

**Adversarial check.** I traced the full chain and it is worse than a clone-only issue. `expected_value` (uncertain_statistics.rs:17-29) and `expected_value_qmc` (:68-80) both sum `sample_with_index(i)` over the DETERMINISTIC indices 0..num_samples — not the random `next_sample_index()` path. Those calls key the process-global memoization cache on `(self.id, sample_index, SamplerKind)` (uncertain_sampling.rs:22 and :44) with no seed component. `Uncertain` derives Clone (mod.rs:30-35) and carries `id` through it. So any two stream entries sharing an id return byte-identical draws and hence an identical collapsed inflow — perfectly correlated, not independent, contradicting the doc at uncertain_boundary_source.rs:91-92. The crate's own tests build the stream exactly that way (`vec![MaybeUncertain::from_value(U_IN); STEPS]`). I confirmed the shipped harness escapes only by accident: verification/dec_cylinder_wake_verification/config.rs:194-207 constructs a fresh `Uncertain::normal(...)` inside the per-step closure, giving each step a new id; nothing in the API doc says that is required.

> Evidence re-read: uncertain_statistics.rs:17-29 (`for i in 0..num_samples { sum += self.sample_with_index(i as u64)? }`), :68-80 (same for QMC); uncertain_sampling.rs:21-29, :39-52 (cache keys); uncertain/mod.rs:30-35 (`#[derive(Clone)]` over `id`), :58-60 (`id()`); uncertain_boundary_source.rs:90-92 (doc), :140-146; verification/dec_cylinder_wake_verification/config.rs:194-207 (fresh Uncertain per step).

---

### 8.9 [MINOR] The moving-wall lift factor is 'verified' by comparing two byte-identical copies of the same loop

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/dec/boundary_zone_tests.rs:89`
- **Auditor confidence:** confirmed

**Claim.** The bit-identity test is tautological with respect to the shared lift formula and no unit test asserts the numeric lift value. However, the lid-cavity Ghia gate runs at h = 1/16 with an explicit /h conversion, so it does provide an external check that would catch a gross missing- or doubled-`h` factor; only subtle lift errors are uncovered.

**Code evidence.**

```
moving_wall.rs:83-93:
```
for (idx, cell) in complex.iter_cells(1).enumerate() {
    let axis = cell.orientation().trailing_zeros() as usize;
    if axis == self.wall_axis || self.velocity[axis] == R::zero() || cell.position()[self.wall_axis] != wall_pos { continue; }
    let length = metric.cell_volume(complex, &cell);
    out.push((idx, self.velocity[axis] * length));
}
```
dec_ns_solver/mod.rs:219-229 — the identical loop with `velocity[axis]` in place of `self.velocity[axis]`.
The test, boundary_zone_tests.rs:89-102:
```
fn moving_wall_zone_marches_bit_identically_to_with_moving_wall() {
    let legacy = DecNsSolver::new(&m, NU, dt, None).unwrap().with_moving_wall(1, true, [lid, 0.0]).unwrap();
    let zoned = DecNsSolver::with_zones(&m, NU, dt, MovingWall::new(1, true, [lid, 0.0]).unwrap()).unwrap();
    ... assert_eq!(a, b, "moving-wall zone must reproduce with_moving_wall bit-for-bit");
```
No unit test anywhere asserts the numeric lift value: boundary_construction_tests.rs:102-106 asserts only `!out.is_empty()`.
```

**Reference form.** A verification must compare against a source independent of the code under test. Comparing two copies of the same expression is a tautology — the audit's axis-C definition of 'a verification that compares a result against a number derived from the same code path'.

**Impact.** A factor error in the moving-wall lift (a missing `·h`, a doubled `h`, a wrong sign) passes this test unconditionally. The only surviving check is the lid-cavity Ghia RMSE gate, whose thresholds are themselves back-fitted (see the separate finding), so the effective coverage of the single most load-bearing boundary value in the crate is a loose end-to-end comparison.

**Recommended fix.** Delete one of the two duplicate loops — have `with_moving_wall` construct a `MovingWall` zone and call `collect_lift`, so there is a single source of truth. Replace the bit-identity test with a value test on a lattice with `h != 1`: assert `lift[idx] == velocity[axis] * h` for a known edge, and assert the edge count equals the number of tangential edges on the face.

**Adversarial check.** The duplication and the tautology are real: moving_wall.rs:83-93 and dec_ns_solver/mod.rs:219-229 are the same loop modulo `self.`, and boundary_zone_tests.rs:84-104 asserts bit-identity between the two, so no error in the shared `velocity[axis] * length` expression can make it fail. boundary_construction_tests.rs:98-119 does only assert non-emptiness. But the impact claim overreaches. The lid-cavity Ghia gate is NOT a loose end-to-end check of the length factor — cavity_tests.rs:65 builds the manifold at h = 1/(n−1) (0.0625 at n=17) and cavity_centerline_rmse divides edge integrals by h before comparing to Ghia's tables. Dropping the `* length` factor would put the lid at u = 1/h = 16, blowing the centerline RMSE orders of magnitude past 0.32. So a gross factor error is caught by genuinely external reference data at h != 1; what the tautological unit test fails to catch is only a subtle error. (Note the Couette test at no_slip_tests.rs:260-302 does NOT help — manifold_2d uses `CubicalReggeGeometry::unit()`, spacing 1, so it is length-degenerate.)

> Evidence re-read: moving_wall.rs:78-93; dec_ns_solver/mod.rs:216-230; boundary_zone_tests.rs:84-104; boundary_construction_tests.rs:98-119 (non-emptiness only); cavity_tests.rs:65-73 (h = 1/(n-1)), :95-160 (edge integrals /h vs Ghia tables), :180-190 (rmse < 0.32); no_slip_tests.rs:16-22 (`unit()` metric), :260-302.

---

### 8.10 [MINOR] The Inflow lift's edge-length factor is unfalsifiable by the test suite: the only value-level assertion runs at h = 1

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/dec/inflow_outflow_tests.rs:26`
- **Auditor confidence:** confirmed

**Claim.** `Inflow::collect_lift` pushes `speed * length`. The only test that checks an inflow value numerically uses a uniform metric with spacing 1.0, where `speed * length == speed`, so it cannot distinguish the correct edge-integral form from a bare `speed`. Every other inflow test asserts only non-emptiness.

**Code evidence.**

```
inflow_outflow_tests.rs:26: `let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);`
Assertion, inflow_outflow_tests.rs:68-76:
```
// Steady state is uniform `u_x = U_IN` (edge integrals at h = 1), `u_y = 0`.
...
assert!((u[i] - U_IN).abs() < 1e-6, "x-edge {i} not uniform: {} vs {U_IN}", u[i]);
```
The comment itself concedes the degeneracy ('edge integrals at h = 1').
boundary_construction_tests.rs:126-131 and :154-160 assert only `assert!(!lift.is_empty())` / `assert!(!prescribed.is_empty())`; :164-171 checks positions, never values.
The code under test, inflow.rs:84-85: `let length = metric.cell_volume(complex, &cell); out.push((idx, self.speed * length));`
```

**Reference form.** A test that fixes a scale factor to unity cannot verify that factor. To verify `∫_e u·dl = U·|e|` the test must run at `|e| != 1`. (The slip-wall suite does this correctly — slip_wall_tests.rs:41 uses `h = 1.0/(ny-1)` and asserts `u[i] ≈ U * h` at :62 — so the pattern exists in the repo.)

**Impact.** The Dirichlet inflow magnitude is the primary input of every open-boundary study. A missing or spurious edge-length factor would scale the entire inflow by `h` (a factor of ~31 in the cylinder configuration) and no test in the suite would fail. The implementation is in fact correct, but an avionics reviewer cannot establish that from the test evidence.

**Recommended fix.** Re-run `uniform_inflow_outflow_channel_marches_to_uniform_flow` with `CubicalReggeGeometry::uniform(h)` for some `h != 1` and assert `u[i] ≈ U_IN * h` on the x-edges. Additionally add a direct unit test on `Inflow::collect_lift` asserting the exact pair `(idx, U * h)` for a known inlet edge on both the min and max faces.

**Adversarial check.** inflow_outflow_tests.rs:27 is `CubicalReggeGeometry::uniform(1.0)` and the only numeric assertion (:68-76) compares x-edges to U_IN with the comment conceding 'edge integrals at h = 1'. At h = 1, `self.speed * length` (inflow.rs:84-85) is indistinguishable from a bare `self.speed`. Every other inflow test asserts only non-emptiness or edge positions (boundary_construction_tests.rs:121-171). The auditor's reference reasoning is sound and the counter-pattern exists in-repo: slip_wall_tests.rs:31 builds at h = 1/(ny−1) and asserts `U * h` at :58-62. The auditor correctly notes the implementation is in fact correct; the finding is about test evidence, and I could not find any test that would fail if the factor were dropped from Inflow specifically.

> Evidence re-read: inflow_outflow_tests.rs:22-28 (uniform(1.0)), :68-80 (assert |u - U_IN| < 1e-6); inflow.rs:79-86; boundary_construction_tests.rs:121-171 (non-emptiness / position only); slip_wall_tests.rs:31 (h = 1/(ny-1)), :56-63 (asserts U*h).

---

### 8.11 [MINOR] The QMC collapse seed is documented as `base ⊕ id` (XOR) in three places but implemented as wrapping addition

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/dec/uncertain_inflow/uncertain_boundary_source.rs:143`
- **Auditor confidence:** confirmed

**Claim.** The reproducibility specification for the per-step Sobol digital shift is stated as an XOR of the base seed with the sample id in the API doc, the inline comment, and the verification harness comment. The code uses `wrapping_add`.

**Code evidence.**

```
Code, uncertain_boundary_source.rs:140-146:
```
let collapsed = match self.qmc_collapse_seed {
    Some(base) => present.expected_value_qmc(
        self.collapse_samples,
        base.wrapping_add(present.id() as u64),
    ),
```
Doc, uncertain_boundary_source.rs:91-92: `/// per-sample Sobol digital shift is `base_seed ⊕ present.id()`, so every step is an independent,`
Inline comment, uncertain_boundary_source.rs:138-139: `// Opt-in QMC collapse uses a per-sample reproducible Sobol shift (base ⊕ id), so`
Harness, verification/dec_cylinder_wake_verification/config.rs (QMC_COLLAPSE_SEED doc): `/// CDF). The per-step Sobol shift is `base ⊕ sample.id()`, so the collapse is reproducible and`
```

**Reference form.** Docs-vs-code parity on a reproducibility specification. `a ⊕ b` (XOR) and `a + b mod 2^64` are different functions; for a randomized-QMC digital shift the conventional operator is XOR (Owen, 'Randomly Permuted (t,m,s)-Nets'; L'Ecuyer & Lemieux on randomized QMC digital shifts).

**Impact.** An engineer reproducing a recorded run in another tool, or auditing the sequence of Sobol shifts, will compute a different shift set from the documented formula and conclude the run is irreproducible. For a certification artifact where the whole selling point of the harness is bit-identical reproducibility (verification/dec_cylinder_wake_verification/README.md), a wrong seed-derivation formula in the spec is a traceability break.

**Recommended fix.** Change the code to `base ^ (present.id() as u64)` (matching the documented and conventional randomized-QMC digital shift) and note the resulting baseline change, or change all three doc sites to say 'wrapping addition (`base + id mod 2^64`)'. Pick one and add a unit test asserting the shift value for a known `(base, id)`.

**Adversarial check.** All four sites are exactly as quoted. Code: uncertain_boundary_source.rs:143 `base.wrapping_add(present.id() as u64)`. Docs: :91 'base_seed ⊕ present.id()', inline comment :138 '(base ⊕ id)', and verification/dec_cylinder_wake_verification/config.rs:59 'The per-step Sobol shift is `base ⊕ sample.id()`'. XOR and mod-2^64 addition are different functions, and the auditor's note that XOR is the conventional randomized-QMC digital-shift operator is right. Severity minor is correct: the run is still deterministic, only the published derivation formula is wrong. Worth flagging alongside it — the QMC cache key `(id, sample_index, SamplerKind::Qmc)` (uncertain_sampling.rs:44) does not include the shift at all, so the seed derivation only ever matters across distinct ids anyway.

> Evidence re-read: uncertain_boundary_source.rs:138-146 (wrapping_add), :91-92 (doc XOR); verification/dec_cylinder_wake_verification/config.rs:58-61 (doc XOR); uncertain_sampling.rs:39-52 (QMC key omits the seed).

---

### 8.12 [MINOR] `shape[axis] - 2` / `- 1` underflow on a degenerate axis extent, with no shape validation upstream

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/inflow.rs:57`
- **Auditor confidence:** confirmed

**Claim.** `Inflow::edge_column`'s `shape[wall_axis] - 2` underflows on an axis of extent 1 and is reachable through `with_zones`. The `Outflow`/`SlipWall` `- 1` variants underflow only at extent 0, which `DecNsSolver::new`'s empty-lattice rejection already blocks on the solver path; they are reachable only by calling the public hooks directly.

**Code evidence.**

```
inflow.rs:55-61:
```
fn edge_column(&self, complex: &LatticeComplex<D, R>) -> usize {
    if self.max_side { complex.shape()[self.wall_axis] - 2 } else { 0 }
}
```
called unconditionally at inflow.rs:78 and :98 before the cell loop.
outflow.rs:54-58 and slip_wall.rs:55-59: `let pos = if self.max_side { complex.shape()[self.wall_axis] - 1 } else { 0 };`
No guard upstream — deep_causality_topology/src/types/lattice_complex/mod.rs:84-92:
```
pub fn new(shape: [usize; D], periodic: [bool; D]) -> Self { Self { shape, periodic, ... } }
```
`Inflow::new` (inflow.rs:36-52) validates only `wall_axis < D` and `speed.is_finite()`; it never sees the lattice.
```

**Reference form.** An index expression on `usize` must be guarded against underflow when the operand can be smaller than the subtrahend. The valid edge-column range is `0..=shape[a]-2` (established by `valid_positions`, lattice_complex/mod.rs:288-297), which is empty when `shape[a] < 2`.

**Impact.** Narrow but real: a degenerate or quasi-1D lattice (a single node across an axis, e.g. a 2D case run as a 3D lattice with one cell in z) panics inside a boundary zone rather than returning a `PhysicsError`, and in a release build silently produces a wrapped index that matches no cell — yielding an empty, silently absent boundary condition instead of a diagnostic.

**Recommended fix.** Guard in the collect hooks: `let Some(col) = shape[wall_axis].checked_sub(2) else { return; }` (and `checked_sub(1)` for Outflow/SlipWall), or validate `shape[wall_axis] >= 2` on the non-periodic axes in `DecNsSolver::with_zones` and return `PhysicsError::DimensionMismatch`. Add a unit test on a `[1, N]` lattice.

**Adversarial check.** The `Inflow` half is confirmed and reachable: inflow.rs:55-61 computes `shape[wall_axis] - 2` on usize, is called at :78 (after only the periodic and metric guards) and :98, and `LatticeComplex::new` (lattice_complex/mod.rs:84-92) stores shape with no validation. A 2-D lattice like [1, 5] with a max-side Inflow on axis 0 reaches `1 - 2` and panics in debug / wraps in release, and `Inflow::new` (:36-52) never sees the lattice so cannot pre-reject it. The `Outflow`/`SlipWall` `- 1` half is much weaker than stated: it underflows only at extent 0, and a shape with a zero extent has zero cells at every grade, so `DecNsSolver::new` rejects it first with 'the lattice has no edges (empty shape)' (mod.rs:87-91) before any zone hook that could underflow runs. Only direct user calls to the public `collect_*` hooks reach it.

> Evidence re-read: inflow.rs:36-52 (new — no lattice), :54-61 (edge_column), :71-78 and :94-98 (call order); outflow.rs:53-58, slip_wall.rs (same `-1` pattern); lattice_complex/mod.rs:84-92 (no validation); dec_ns_solver/mod.rs:87-91 (empty-shape rejection ordering).

---

### 8.13 [MINOR] The wake harness's incompressibility gate cannot fire on a diagnostic error, and the residual is only sampled every 10th step

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_cylinder_wake_verification/main.rs:89`
- **Auditor confidence:** confirmed

**Claim.** On a divergence-diagnostic failure the harness substitutes `f64::NAN` and folds it with `f64::max`, which discards NaN — so `max_div` never becomes NaN and the `max_div.is_nan()` branch of the gate is unreachable from this path. Separately, the residual is evaluated only at the report cadence (every `STEPS/200` steps), while `verification/README.md` states the gate as an unqualified property of the projector.

**Code evidence.**

```
main.rs:77-89:
```
if sv.step() % (STEPS / 200).max(1) == 0 {
    ...
    let div = match sv.divergence() { Ok(v) => Into::<f64>::into(v), Err(_) => f64::NAN };
    max_div = max_div.max(div.abs());
```
main.rs:112-118:
```
const DIV_TOL: f64 = 1e-6;
if max_div.is_nan() || max_div >= DIV_TOL { eprintln!("FAIL: ..."); failed = true; }
```
`f64::max` returns the non-NaN operand when exactly one argument is NaN (IEEE-754 maxNum semantics, as documented for Rust's `f64::max`), so `max_div.max(NAN) == max_div`.
verification/README.md:143-145: `The gate is (a) incompressibility — the\nconstrained Leray projector keeps the divergence residual at machine precision` — no sampling caveat (the example's own README does say 'sampled').
```

**Reference form.** A gate must be able to fail on the failure mode it names. Here the NaN branch is dead code, and 'the divergence residual stays below 1e-6' is asserted from 200 samples out of 2000 steps, not from the full march.

**Impact.** If the divergence diagnostic starts erroring — a plausible symptom of the very lattice/metric corruption the gate exists to catch — the harness prints `NaN` rows to stdout and still exits zero with 'verified: incompressibility held'. The reported PASS is weaker than the README's wording implies.

**Recommended fix.** Track diagnostic errors explicitly (`if sv.divergence().is_err() { failed = true; }`) rather than folding NaN, and either evaluate the residual every step (it is cheap relative to a projection solve) or state the sampling cadence in verification/README.md as the example README already does.

**Adversarial check.** Both halves check out. `max_div` is initialized to 0.0 and updated only by `max_div = max_div.max(div.abs())`; Rust's `f64::max` returns the other operand when one is NaN, so a `sv.divergence()` error substituting `f64::NAN` leaves `max_div` unchanged and the `max_div.is_nan()` branch at main.rs:112-118 is unreachable from that path — the harness would print NaN rows and still exit zero claiming 'incompressibility held'. The sampling is also as described: the diagnostic block is inside `if sv.step() % (STEPS / 200).max(1) == 0`, i.e. 200 of 2000 steps. verification/README.md:143-145 states the gate as an unqualified property of the projector with no sampling caveat. Severity minor is right — this is gate strength, not a wrong physical result, and the measured value (3.33e-15) has huge margin.

> Evidence re-read: verification/dec_cylinder_wake_verification/main.rs:66 (`let mut max_div = 0.0f64`), :77-89 (cadence gate + NAN substitution + `max_div.max`), :112-118 (`if max_div.is_nan() || max_div >= DIV_TOL`), :138-140 (success message); verification/README.md:141-145.

---

### 8.14 [MINOR] The wake harness's presence gate is exercised only at its degenerate endpoints, so the SPRT parameters are unverified by the gate that claims to verify the dropout model

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_cylinder_wake_verification/config.rs:190`
- **Auditor confidence:** confirmed

**Claim.** The sensor stream alternates between an always-present reading and `always_none()`. The presence channel is therefore a point mass at true or at false, so the SPRT decides deterministically regardless of `threshold`, `confidence`, `epsilon` or `max_samples`. The `log_entries == 2 × dropouts` gate consequently validates the log bookkeeping, not the presence detector or its four tuned parameters.

**Code evidence.**

```
config.rs (sensor_stream):
```
if (s + 1) % DROPOUT_EVERY == 0 { MaybeUncertain::<FloatType>::always_none() }
else { MaybeUncertain::<FloatType>::from_uncertain(Uncertain::normal(ft(U_BULK), ft(SENSOR_SIGMA))) }
```
Gate parameters that never affect the outcome, config.rs (build_uncertain_config):
```
let zone = UncertainInflowZone::new(1, true, 0, ft(U_BULK))
    .with_presence_gate(0.5, 0.95, 0.05, PRESENCE_SAMPLES)
```
(defaults of the same shape at uncertain_boundary_source.rs:55-60: `threshold: 0.5, confidence: 0.95, epsilon: 0.05, max_samples: 1000, collapse_samples: 1000`).
The gate, main.rs:119-126: `let expected_log = 2 * n_dropouts; ... if got_log != expected_log { ... failed = true; }`
```

**Reference form.** A verification of a probabilistic detector must exercise it in the regime where its decision is non-trivial — i.e. with a presence probability inside the indifference region [threshold − epsilon, threshold + epsilon] or near it. With point-mass presence channels, a detector that ignores all four parameters passes identically.

**Impact.** The gate is not tautological — deleting either `UncertainBoundarySource::record` or `alternate_value` makes it fail (both were confirmed as independent contributors) — but it is much weaker than the README's 'effect accounting' framing suggests. A mis-tuned or broken SPRT (wrong threshold sense, ignored confidence, budget exhaustion misclassified) would pass. The four constants 0.5 / 0.95 / 0.05 / 256 are therefore untraceable to any measurement: no evidence in the repo justifies these values or shows the result is insensitive to them.

**Recommended fix.** Add a stream segment with an intermediate presence probability (e.g. `Uncertain<bool>` Bernoulli at 0.5 ± epsilon) and gate the observed dropout rate against the SPRT's stated operating characteristic; or, at minimum, add a sensitivity note recording the dropout count under a perturbed `(threshold, confidence, epsilon, max_samples)` so a reviewer can see the result does not hinge on the tuned values. Document the provenance of the four defaults at uncertain_boundary_source.rs:53-60.

**Adversarial check.** config.rs:194-207 alternates `MaybeUncertain::always_none()` with `from_uncertain(Uncertain::normal(...))`, so the presence channel is a point mass at false or at true. Against a point mass the SPRT terminates on the same decision regardless of threshold, confidence, epsilon or max_samples, so the four values passed at config.rs:174 (0.5, 0.95, 0.05, PRESENCE_SAMPLES) and the defaults at uncertain_boundary_source.rs:55-60 are never load-bearing. The gate at main.rs:119-126 compares `log_entries` to `2 * n_dropouts`, which is log bookkeeping. The auditor's reference form — a probabilistic detector must be exercised near its indifference region — is correct, and their own honesty about the gate not being strictly tautological is accurate. I found no sensitivity study or measurement in the repo justifying the four constants.

> Evidence re-read: verification/dec_cylinder_wake_verification/config.rs:193-207 (sensor_stream: always_none / from_uncertain), :173-176 (with_presence_gate(0.5, 0.95, 0.05, PRESENCE_SAMPLES)); uncertain_boundary_source.rs:54-65 (identical defaults), :131-136 (lift_to_uncertain gate); main.rs:119-126 (expected_log = 2 * n_dropouts).

---

### 8.15 [MINOR] The lid-cavity refinement gates — the only external check on the moving-wall lift magnitude — are back-fitted from the measurement with disclosed headroom

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/config.rs:83`
- **Auditor confidence:** confirmed

**Claim.** The two absolute RMSE bounds are back-fitted (measured 0.252 / 0.133 plus ~25 % headroom), with the provenance disclosed in-source, so a PASS on those two alone is a non-regression claim rather than an accuracy claim. The accompanying strict refinement-trend margin (fine < coarse - 0.04) is a genuine convergence criterion and is not back-fitted.

**Code evidence.**

```
config.rs:82-85:
```
/// Pinned RMSE gate for the coarse (17²) grid.
pub const TREND_COARSE_GATE: f64 = 0.32;
/// Pinned RMSE gate for the fine (33²) grid.
pub const TREND_FINE_GATE: f64 = 0.20;
```
main.rs:117-118 (the provenance, stated in-source):
```
// Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom) plus the
// strict refinement-trend margin.
```
Applied at main.rs:120-137 (`if coarse >= ft(TREND_COARSE_GATE) { ... }`, likewise for fine).
```

**Reference form.** Axis-C definition: 'a gate whose bound was picked after seeing the measured value (back-fitted bound)'. A defensible accuracy gate is derived from a target discretization error or an accepted benchmark tolerance, not from the current run.

**Impact.** An engineer reading 'RMSE gated against Ghia (1982)' will read the PASS as an accuracy claim. It is a non-regression claim with 25 % slack: a change that degrades centerline accuracy by up to a quarter still passes. Since this is the only end-to-end external check on the `velocity[axis] * length` moving-wall lift (the unit-level test being tautological — see the separate finding), the trust chain for that boundary value bottoms out on a self-referential threshold. Mitigant: the provenance is disclosed in-source, and a gross factor error in the lift would still blow past 0.32.

**Recommended fix.** State the intent explicitly in config.rs and the README — label these `TREND_*_REGRESSION_GATE` and say they are non-regression bounds, not accuracy criteria. Separately record the measured RMSE per grid in the README table (0.252 / 0.133) so a reviewer sees the actual accuracy alongside the bound, and if an accuracy claim is wanted, derive a target from the expected second-order convergence rate rather than from the measurement.

**Adversarial check.** The provenance claim is verbatim correct: config.rs:82-85 pins TREND_COARSE_GATE = 0.32 / TREND_FINE_GATE = 0.20, and main.rs:117-118 states in-source 'Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom)'. cavity_tests.rs:176-180 carries the same disclosure. So the bound is current-output-plus-margin, i.e. a non-regression gate presented alongside external reference data. Two corrections. First, the auditor omits that the gate is not only the two absolute bounds: main.rs:139 additionally requires `fine < coarse - TREND_MARGIN` (0.04), a strict refinement-trend condition that is a genuine discretization-convergence criterion and is not back-fitted in the same sense. Second, per my verdict on the moving-wall finding, this gate is a real external check at h = 1/16 that would catch a gross lift-factor error by orders of magnitude, so the 'trust chain bottoms out on a self-referential threshold' framing overstates it.

> Evidence re-read: verification/dec_lid_cavity_re1000_verification/config.rs:78-87 (TREND_* constants incl. TREND_MARGIN = 0.04); main.rs:105-140 (provenance comment at :117-118, both absolute gates, and the strict `fine >= coarse - TREND_MARGIN` trend gate at :139); cavity_tests.rs:176-190 (same 0.32 gate, same disclosed provenance).

---

### 8.16 [MINOR] The body-force cochain's units (acceleration vs force density) are not stated in the zone doc

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/body_force_zone.rs:15`
- **Auditor confidence:** confirmed

**Claim.** `BodyForceZone` documents its payload as 'the edge-integral cochain g♭ (e.g. a streamwise pressure gradient G·h)' without stating that `g` must be a force per unit MASS (an acceleration), nor that the crate assumes ρ = 1. The rate assembly adds it directly to d(∫u·dl)/dt, which fixes the units, but that is only discoverable by reading `dec_ns_rate.rs`.

**Code evidence.**

```
body_force_zone.rs:15-17:
```
/// A body force on the velocity edges: the edge-integral cochain `g♭` (e.g. a streamwise pressure
/// gradient `G·h` on the x-edges) added to the rate source. The carried tensor is the grade-1
/// edge cochain; ...
```
The unit-fixing line, dec_ns_rate.rs:711-717:
```
Some(g) => { let g_s = g.as_slice(); let nu = self.nu.get();
    (0..self.n1).map(|i| R::zero() - conv_s[i] - nu * lap_s[i] + g_s[i]).collect() }
```
The ρ = 1 convention appears only elsewhere, dec_ns_rate.rs:437: `/// (`dφ = −∇(p + ½|u|²)` at `ρ = 1`).`
```

**Reference form.** Incompressible NS in acceleration form: ∂u/∂t = −(u·∇)u + ν∇²u + g, with g = f/ρ (force per unit mass). Adding the term directly to ∂u/∂t requires g in m/s²; a force per unit volume would need division by ρ first.

**Impact.** A user supplying a pressure-gradient forcing as −dp/dx (Pa/m, force per unit volume) rather than −(1/ρ)dp/dx (m/s²) gets a forcing wrong by a factor of ρ — with no error, since both are finite grade-1 cochains of the right length. At ρ = 1 the two coincide, which makes the mistake invisible in the crate's own tests and visible only when a real fluid density is used.

**Recommended fix.** State in the `BodyForceZone` doc that the cochain is `∫_e (f/ρ)·dl` — an acceleration line integral in m²/s² — and that the solver's incompressible formulation assumes ρ = 1 so `g` is numerically the force density. Cross-reference the rate assembly. Do the same on `BodyForceOneForm`.

**Adversarial check.** The doc at body_force_zone.rs:15-17 is verbatim as quoted and states neither units nor the rho = 1 convention. The unit-fixing line is where the auditor says: dec_ns_rate.rs adds `+ g_s[i]` directly into the rate expression `-conv - nu*lap + g`, which is d/dt of the edge integral, so g must be per unit mass (m/s^2). The auditor's reference form (incompressible NS in acceleration form, g = f/rho) is correct. I also confirmed the convention is essentially undocumented crate-wide: a grep for rho/density over dec_ns_rate.rs returns exactly one hit, the pressure-diagnostic doc at :437 ('dphi = -grad(p + |u|^2/2) at rho = 1'). Minor is the right severity — it is a doc gap, not a code defect, and the whole solver is uniformly kinematic (nu is kinematic viscosity; no density parameter exists anywhere), so the convention is inferable but never stated at the point of use.

> Evidence re-read: body_force_zone.rs:15-17 (doc, no units); dec_ns_rate.rs:710-717 (`-conv_s[i] - nu * lap_s[i] + g_s[i]`); `grep -n 'rho|density|rho-symbol' dec_ns_rate.rs` returns only :437.

---

### 8.17 [INFO] `Inflow`'s documented requirement of a matching `Outflow` is enforced only at the first projection, not at zone composition

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/solvers/dec/boundary/inflow.rs:22`
- **Auditor confidence:** confirmed

**Claim.** `Inflow`'s doc states it 'Requires a matching Outflow reference to balance the net flux'. `DecNsSolver::with_zones` accepts an `Inflow` with no `Outflow` without complaint; the failure surfaces later, as a wrapped `TopologyError` from the first seed or step projection rather than as a construction-time configuration error.

**Code evidence.**

```
Doc, inflow.rs:21-22: `/// counted in the open-boundary projection — and the lift that sets that value. Requires a\n/// matching [`Outflow`](super::outflow::Outflow) reference to balance the net flux.`
No construction check, dec_ns_solver/mod.rs:158-164:
```
let mut prescribed = alloc::vec::Vec::new();
zones.collect_prescribed_edges(manifold, &mut prescribed);
let mut reference = alloc::vec::Vec::new();
zones.collect_reference_vertices(manifold, &mut reference);
if !prescribed.is_empty() || !reference.is_empty() { solver.rate.set_open_boundary(prescribed, reference); }
```
The check exists only downstream, leray.rs:793-799: `if !prescribed_edges.is_empty() && reference_vertices.is_empty() { return Err(TopologyError::InvalidInput("leray_project_open: a prescribed inflow requires a reference (outflow) face to balance the net flux")) }` (mirrored at :436-442 for the weighted path).
```

**Reference form.** Fail-fast principle for configuration invariants: a documented requirement of the zone set should be checked where the zone set is assembled. The invariant itself is correctly enforced (see verified_correct entry on Poisson consistency); only its timing and error type are at issue.

**Impact.** Low. The condition cannot be silently violated — the run does fail. But the failure appears as a `PhysicsError::TopologyError("open weighted Leray projection failed: ...")` from `seed_from_vertex_vectors`, which is an obscure place to learn that the zone tuple was missing an `Outflow`.

**Recommended fix.** Add a check in `with_zones`: if `!prescribed.is_empty() && reference.is_empty()`, return `PhysicsError::PhysicalInvariantBroken` naming the missing `Outflow` zone. Note the deferred-failure behaviour on `Inflow`'s doc until then.

**Adversarial check.** Confirmed on all points. inflow.rs:21-22 carries the 'Requires a matching Outflow reference to balance the net flux' sentence. `with_zones` (dec_ns_solver/mod.rs:158-164) collects prescribed edges and reference vertices and calls `set_open_boundary` if either is non-empty, with no check that a non-empty prescribed set implies a non-empty reference set. The enforcement lives downstream in the projector, exactly where cited: leray.rs:791-799 in `leray_project_open_guess` and the mirrored check in the weighted path both return `TopologyError::InvalidInput('... a prescribed inflow requires a reference (outflow) face to balance the net flux')`. The invariant cannot be silently violated — the run does fail — so the auditor's own 'info' severity and 'low impact' framing are accurate; the issue is failure timing and error type (a wrapped TopologyError from seeding rather than a construction-time configuration error).

> Evidence re-read: inflow.rs:18-22 (doc); dec_ns_solver/mod.rs:158-164 (no construction-time check); leray.rs:791-799 (open path guard), and the identical guard in the weighted path at leray.rs:435-442.

---
