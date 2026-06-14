## Context

The DEC NS solver is stateless (`step(&self, field)`); state lives in the monad
(`add-cut-cells-and-immersed-boundaries` Group C). Boundary conditions, however, are
implemented ad hoc at five unrelated places. This change introduces one vocabulary — the
**boundary zone** — and the layered dispatch that routes each zone to the solver stage it
actually affects, plus the single new numerical operator (net-flux projection) that open
boundaries require. The design is grounded in the existing step:

```text
SolenoidalField ──as_one_form──► VelocityOneForm
    ──Rk4 over P∘rhs──► VelocityOneForm        (the arrow: projected stages)
    ──constrained Leray project──► SolenoidalField
    ──constrain_edges ∘ with_lift──► SolenoidalField   (re-assert Dirichlet edges)
    ──cfl_check──► StepOutput
```

## Goals / Non-Goals

- **Goal:** one composable `BoundaryZone` term; every existing BC re-expressed with zero
  behavioural change; inflow/outflow expressible; one write-once net-flux projection.
- **Goal:** the uncertain-data-zone mechanism generalized to a cross-domain source.
- **Non-Goal:** the `FluidDynamics` DSL (third change); new IO; advanced non-reflecting BCs.

## Decisions

### D1: A boundary zone is a layered dispatch, not a single `apply()`

Boundary conditions are **not** interchangeable: they act at different stages of a step. A naïve
"one trait, one `apply()`" is wrong — outflow is a boundary *time-evolution*, inflow is a
*projection constraint plus a flux source*, an immersed body is *geometry*. The zone trait
therefore exposes one hook **per solver stage**, each defaulting to a no-op; a concrete zone
overrides only the layers it touches.

| Layer | Stage | Hook (illustrative) | Who overrides it |
|-------|-------|---------------------|------------------|
| L0 | topology | `periodic_axes()` | (lattice; construction-time, informational) |
| L1 | metric / Hodge star | `metric_overlay(&self) -> Option<&CutCellRegistry>` | ImmersedBody |
| L2 | rate source | `rate_source(&self, complex) -> Option<CausalTensor>` | BodyForce |
| L3a | projection constraint set | `constrained_edges(&self, complex) -> Vec<usize>` | NoSlip, MovingWall, Inflow, ImmersedBody |
| L3b | inhomogeneous lift (time-dependent) | `lift_values(&self, complex, step) -> Vec<(usize, R)>` | MovingWall, Inflow |
| L4 | projection net-flux / compatibility | `flux_role(&self) -> FluxRole` (`Closed` \| `Source` \| `Reference`) | Inflow, Outflow |
| L5 | boundary time-update | `boundary_update(&self, field, dt) -> Vec<(usize, R)>` | Outflow |

Each hook *collects into* an accumulator rather than returning an owned `Vec`, so a composite zone
simply sequences the calls of its members — no per-zone allocation, fully monomorphized:

```rust
pub trait BoundaryZone<const D: usize, R: RealField> {
    fn metric_overlay(&self) -> Option<&CutCellRegistry<D, R>> { None }
    fn collect_rate_source(&self, _complex: &LatticeComplex<D, R>, _acc: &mut CausalTensor<R>) {}
    fn collect_constrained_edges(&self, _complex: &LatticeComplex<D, R>, _out: &mut Vec<usize>) {}
    fn collect_lift(&self, _complex: &LatticeComplex<D, R>, _step: usize, _out: &mut Vec<(usize, R)>) {}
    fn collect_flux_roles(&self, _out: &mut Vec<BoundaryFlux<R>>) {} // (face, role)
    fn collect_boundary_update(&self, _field: &SolenoidalField<R>, _dt: R, _out: &mut Vec<(usize, R)>) {}
}
```

Composition is **static (HKT), never `dyn`** — per the repo's static-dispatch rule and the
`causal_cfd.md` HKT goal. A composite is a typed cons/tuple that is itself a `BoundaryZone` and
folds each stage over its members (built on the `deep_causality_haft` foundation):

```rust
impl<const D: usize, R, A, B> BoundaryZone<D, R> for (A, B)
where R: RealField, A: BoundaryZone<D, R>, B: BoundaryZone<D, R>
{
    fn collect_constrained_edges(&self, c: &LatticeComplex<D, R>, out: &mut Vec<usize>) {
        self.0.collect_constrained_edges(c, out);
        self.1.collect_constrained_edges(c, out);
    }
    // …each hook delegates to .0 then .1
}
```

The solver is **generic over the composed zone type** `Z: BoundaryZone<D, R>` (an empty `NoZones`
identity for the closed-domain default; `solver.with(zone)` returns a solver over `(Zone, Z)`).
Everything is resolved at compile time. `collect_lift` and `collect_boundary_update` take the step
index / `dt` so a zone can be **time-dependent** (the seam the uncertain source and a future
turbulent inflow plug into).

**Reduction guarantee (numerical, not API):** the existing BCs map onto exactly the layers they
already use, so a solver carrying the `(NoSlip, BodyForce)` zone composition produces byte-identical
output to today's Poiseuille/cavity physics. The API around it changes freely; only the marched
result is held fixed. This is the numerical-equivalence gate.

### D2: The net-flux mixed-BC Leray projection is the one irreducible operator

A DSL cannot compose a new discrete operator into existence. The closed-domain projection solves
the pressure-Poisson `δ d φ = δ u` with pure-Neumann pressure BCs, which presupposes the
compatibility condition `∮ u·n = 0` (zero net flux) — exactly why a wall-normal lift is rejected
today. Open boundaries need a mixed-BC solve:

- **Inflow faces (`FluxRole::Source`)**: prescribe `u·n` (the L3 lift on the wall-normal boundary
  edges). In the pressure Poisson this is a Neumann condition with a *known* boundary flux — it
  injects a source into the RHS.
- **Outflow face (`FluxRole::Reference`)**: pin a pressure reference (`φ = 0`) on that face. This
  (a) fixes the otherwise-singular pure-Neumann nullspace and (b) lets `u·n` on the outflow
  **adjust freely** to conserve mass — it absorbs whatever net flux the inflow injects. Exactly
  one reference face is required when any `Source` is present.
- **Walls (`FluxRole::Closed`)**: zero normal flux (Neumann), unchanged.

Discretely on the cubical DEC lattice this is a change to the boundary rows of the codifferential
`δ = M⁻¹ ∂ M` used by the projection's CG solve: the inflow boundary edges carry their prescribed
flux into the divergence RHS, and the reference face removes one pressure DOF. The interior solve,
the cut-star metric, and the CG machinery are unchanged. **When no zone reports a non-`Closed`
flux role, the operator is bit-identical to the current projection** (the reference/source code
paths are skipped) — the second numerical-equivalence gate.

Compatibility/well-posedness: with ≥1 `Source` and a `Reference`, the discrete system is
non-singular and globally mass-conservative by construction (the reference face's flux equals the
negative sum of all sources). With sources but no reference → an explicit error (an unbalanced
closed domain). This is the standard fractional-step open-boundary treatment recast in DEC.

**Implementation finding (the inlet disconnection).** Masking an inflow edge from the operator
disconnects its *ghostless* inlet vertex from the interior in the free-edge graph (its only
interior link is the masked edge), and the injected net flux cannot leave under an all-Neumann
gauge. So the divergence is enforced only on the component reachable from the reference (a flood
fill over the free edges); the disconnected inlet ring carries the prescribed velocity with its
divergence left unconstrained — which is exactly the open-boundary condition. This is realized in
`leray_project_open_opts` and is why a prescribed inflow genuinely requires a reference (confirming
the original decision over a brief "no reference needed" detour).

### D3: Outflow is a boundary time-update, not a projection constraint (L5)

A convective outflow `∂u/∂t + U_c ∂u/∂n = 0` (with `U_c` the local or mean normal speed) is
advanced **before** the projection each step on the outflow boundary edges, by upwinded
extrapolation from the interior. It is *not* pinned in the constraint set (that would over-specify
the outflow and reflect waves). The projection's `Reference` role (D2) then makes the outflow
field divergence-compatible. A zero-gradient (Neumann velocity) outflow is the v1 default;
characteristic/non-reflecting variants are a later refinement behind the same L5 hook.

### D4: The uncertain boundary source is a cross-domain wrapper, not a CFD type

The Group-C `UncertainInflowZone` conflated two things: *which boundary* (a wall) and *how its
value is sourced* (a presence-gated uncertain stream with dropout intervention). This change
separates them. `UncertainBoundarySource<R>` wraps the value channel of **any** Dirichlet-bearing
zone (anything with an L3b `lift_values`): each step it presence-gates the `MaybeUncertain<R>`
sample (`lift_to_uncertain`), collapses it (`expected_value`/`sample`), updates last-good in the
monad `State`, and on a dropout substitutes last-good via `.intervene` recorded in the `EffectLog`
at a configurable `DropoutVerbosity`. It composes as `Inflow::uniform(u).sourced_by(stream, cfg)`
or `MovingWall::tangential(u).sourced_by(...)`.

Cross-domain: the wrapper depends only on `MaybeUncertain<R>` + the monad, not on fluid concepts,
so the same "sensor-fed, presence-gated, dropout-intervening time-varying value" serves any
parameter in any domain (a control setpoint, a forcing amplitude, a material property). The
existing `UncertainInflowZone` is re-expressed as `UncertainBoundarySource` over an `Inflow`/
`MovingWall` zone, preserving its tests bit-for-bit.

### D5: Zones carry no simulation state; the monad still carries state

Zones are immutable configuration (Context, design D10 of the cut-cells change). Time-varying
values come from the step index / `dt` passed to `lift_values`/`boundary_update`, and the
last-good of the uncertain source lives in the monad `State`. The solver stays stateless; this
change does **not** reintroduce solver-held state.

### D6: Composition is static (HKT) from the start — no `dyn`

Per the repo's static-dispatch rule (AGENTS.md: "avoid `dyn`, trait objects, and dynamic
dispatch") and the `causal_cfd.md` HKT goal, the zone set is a **typed, monomorphized composition**
on the `deep_causality_haft` foundation — there is no `Vec<dyn>`. The solver is generic over the
composed zone type `Z: BoundaryZone<D, R>`; `solver.with(zone)` returns a solver over `(Zone, Z)`;
the closed-domain default is the identity zone `NoZones`. Each stage folds over the tuple at compile
time (D1). The `FluidDynamics` DSL (third change) reuses this same static composition as its zone
vocabulary — it does not introduce a parallel dynamic one.

## Risks / Trade-offs

- **Net-flux projection correctness** is the load-bearing risk: the mixed-BC pressure solve must
  be mass-conservative and stable. Mitigation: gate against an analytic channel (uniform inflow →
  uniform outflow, exact), a mass-conservation invariant per step, and the bit-identical
  closed-domain reduction.
- **Outflow reflections** can corrupt the wake. Mitigation: v1 convective outflow placed far
  downstream; the L5 seam admits non-reflecting variants without re-plumbing.
- **Refactor blast radius**: re-expressing existing BCs as zones touches the solver wiring.
  Mitigation: the reduction guarantees (D1, D2) are pinned by bit-identical tests before any new
  zone is added.

## No legacy preservation

None of this CFD code is published (it was written over the last few days), so there is **no
back-compat constraint**: the zone set is *the* boundary surface, and the ad-hoc entry points
(`with_moving_wall`, the implicit no-slip-from-walls, the body-force argument, the cut-registry
attachment) are **replaced** by zone constructors, not kept as shims. Downstream callers (the
cut-cells change, the examples) are updated to the zone API wherever that yields a cleaner surface.
The only thing preserved is **numerics**: the reduction gates (D1, D2) prove the new operator
reproduces the validated physics, independent of the API change.
