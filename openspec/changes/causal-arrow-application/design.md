## Context

The third coordinated change. `causal-arrow-autodiff` (Layer-1 scalar differentiation) and `causal-arrow-autointegration` (Layer-2 integration operator) supply the primitives; this change spends them across the example suite and fills the gap the recent fluid-dynamics kernels left — no examples, because they return `∂u/∂t` and demand `∇u` / `∇²u` / `∇p` as inputs. The targets are not guesses: they come from a four-domain survey (physics, avionics, medicine, mathematics) that classified every derivative and integration site.

Constraints (`AGENTS.md`): example crates follow the same rules as library crates — tests register in their `mod` tree and `BUILD.bazel`; new examples are added to Cargo and Bazel; the writing guides bind prose and comments; `unsafe_code = "forbid"`; no new external deps. Golden rules: never commit (owner commits), never delete files without asking.

## Goals / Non-Goals

**Goals**
- Prove each primitive against real code: replace hand-coded derivatives with `forward-autodiff` and hand-rolled stepping with `numeric-integration`, **behavior-preserving**, with tests asserting the numbers are unchanged.
- Add the fluid-dynamics examples the kernels were waiting for, with at least one **avionics CFD** example verified by the Method of Manufactured Solutions.
- Show the two primitives compose (the Leibniz bridge) in one small example.

**Non-Goals (optional or out)**
- Behavior-*changing* showcases (tumor-treatment annealing → AD gradient ascent) — optional P2, clearly separated from the behavior-preserving rewrites.
- GR gravity-family enhancements (`event_horizon_probe`, `gauge_gr`) — optional P1.
- Any library-API change; the broader physics-kernel bound audit; reverse-mode AD; adaptive integration.
- Differentiating discrete-mesh fields with no closed form — the heat-diffusion rewrites touch only the *time* loop; the spatial Laplacian stays exterior calculus.

## Decisions

### D1 — Behavior-preserving rewrites, proven by equality tests

A "rewrite" here must not change the output. Each AD rewrite keeps the old hand-coded expression as the test oracle: the example (or its test) asserts `gradient(...)` equals the previously hard-coded value (`maxwell`'s `da_dt = −ω·sin(phase)`, etc.) within floating-point tolerance. This makes the simplification auditable and stops an AD substitution from silently drifting. Integrator rewrites assert the new loop reproduces the prior `Euler` trajectory step-for-step; `Rk4` is then offered as a strict accuracy upgrade, not a behavior change.

### D2 — AD targets are closed-form fields only (the analytic boundary)

Per `causal-arrow-autodiff` D7, forward-mode AD applies only where the field is a closed-form function the example can evaluate on `Dual`. The chosen AD targets satisfy this: `maxwell`'s `A_x(t,z) = cos(ω(t−z))`, `magnav`'s synthetic `B(x,y)`, decompression's Schreiner `p(t)`. The heat-diffusion examples are *not* AD targets — their spatial derivative is a discrete mesh Laplacian with no closed form; they receive the *integration* rewrite (the time loop) and keep the exterior-calculus Laplacian. Stating this keeps the change honest about where each primitive belongs.

### D3 — Integration rewrites de-duplicate and unlock accuracy

The Kuramoto rate field appears verbatim in two trees (epilepsy, counterfactual-resection); both become `Euler.integrate(..., &kuramoto_rhs)` over the same rate-field shape, removing the duplicated loop body. The three heat-diffusion examples and a position-Euler example follow. Each demonstrates the property the hand loops cannot: swapping `Euler` for `Rk4` improves accuracy with no change to the model. The Chern-number Riemann sum becomes `quadrature` over the Brillouin interval.

### D4 — The avionics CFD example: Method of Manufactured Solutions on Taylor–Green

The flagship new example exercises the whole stack. It uses an analytic incompressible velocity field with a known exact solution — the **Taylor–Green vortex**, a closed-form solution of the incompressible Navier–Stokes equations — as the manufactured solution:

1. define `u(x, t)` and `p(x, t)` as closures;
2. take `∇u` (Jacobian), `∇²u`, and `∇p` *exactly* with `gradient` / nested duals — no finite differences;
3. feed them to `incompressible_ns_rhs_kernel` to get `∂u/∂t`;
4. march with `Rk4` from the integration operator;
5. verify the marched field matches the exact Taylor–Green field within tolerance.

The avionics framing wraps this as a flow / boundary-layer scenario in `avionics_examples`. MMS is the standard CFD verification method, and AD makes the exact manufactured derivatives free — the single clearest justification for type-based differentiation in this domain. Taylor–Green is chosen because it is genuinely an exact NS solution, so the verification is real, not a tautology.

### D5 — Example registration and tests

Each modified example keeps its `main` and gains (or extends) an assertion test proving the AD / integrator result matches the oracle. Each new example is added to its domain crate's `Cargo.toml` and `BUILD.bazel`, with a smoke/assertion test, per the crate conventions in `AGENTS.md`. No file is deleted; superseded hand-coded lines are replaced in place, and the explanatory comments are preserved or updated to describe the AD/integrator call.

### D6 — Tiering: required core vs optional showcases

- **P0 (required):** maxwell ×2, magnav `∇B`, decompression `dp/dt` (AD); Kuramoto ×2, one heat-diffusion, one position-Euler (integrator); the avionics MMS CFD example; the Leibniz-bridge example.
- **P1 (recommended):** the remaining two heat-diffusion loops and remaining position-Euler loops; the Chern `quadrature`; the GR gravity-family enhancements.
- **P2 (optional):** tumor-treatment AD gradient ascent (behavior-changing); a second fluid example in `physics_examples`.

The spec's requirements are written against P0; P1/P2 are tasks that extend coverage without gating the change.

### D7 — Honest scope on "the majority of examples"

The survey showed that a literal AD *replacement* fits only a handful of examples (the rest are gauge / complex-matrix / mesh / integral code where AD is a poor fit). This change does not pretend otherwise: it rewrites the genuine AD targets, applies the integrator broadly (where duplication is real), and adds new examples rather than forcing AD onto code that has no closed-form derivative. The non-continuous cases remain served by the differential-geometry / topology exterior-calculus surface.

## Risks / Trade-offs

- **Example-test brittleness.** Equality-to-oracle tests use tolerances, not exact bit equality, to absorb the difference between a hand-simplified expression and the AD evaluation order.
- **MMS scope creep.** Bounded by using a single, standard analytic solution (Taylor–Green) and verifying one or two time steps, not a production solver.

## Migration / Rollout

Applied only after both primitive changes are archived. Example-only; no library consumer is affected. Owner commits.
