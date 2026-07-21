# Close the QTT solver's numerical envelope

## Why

The DEC solver family refuses a configuration it cannot integrate. Its sibling QTT family accepts
anything and returns numbers. That asymmetry is the theme of four audit findings, and it is the reason
the QTT immersed-cylinder harness is the only verification program failing today.

`dec_ns_solver/step.rs::cfl_check` rejects an advective or diffusive CFL violation with
`PhysicsError::PhysicalInvariantBroken`, naming the limit and the values that broke it. Against that:

- `QttImmersed2d::new` and `QttIncompressible2d::new` validate **nothing** — they destructure their
  arguments straight into `Ok(Self { .. })`. Neither `η`, `dt`, `ν` nor the mask is checked, at the
  constructor or at the `QttMarchConfigBuilder::build` layer above it (which validates only that the
  grid is `2^L` and the seed shapes match). `η = 0` yields `−1/η = −inf` with no error path.
- All **four** compressible marchers — `euler_1d`, `marcher_2d`, `marcher_3d`, `marcher_3d_fitted` —
  enforce density positivity and then push an **unfloored, possibly negative** pressure into the flux
  (`f[1] = mx·vx + p`, `f[3] = (e+p)·vx`) while flooring it only for the wave speed `c`. The
  hyperbolicity the scheme assumes is not checked, and the state is not rejected.
- The `1e-300` floor those four sites share evaluates to **exactly 0.0 at `f32`** — `num-traits`'
  `f64→f32` conversion is an infallible cast, so `from_f64(1e-300)` returns `Some(0.0)` rather than
  `None` and the `unwrap_or_else` fallback never fires. The floor silently disappears in a precision
  mode the crate documents as supported.
- The body mask is documented as a `[0, 1]` volume fraction in two places, but tensor-train truncation
  drives it negative and nothing clamps or checks it: measured `min χ = −1.78e-3` (188 cells) at bond
  cap 4 and `−6.5e-5` (84 cells) at cap 8 — two of the four caps the shipped cylinder ladder runs. A
  negative `χ` flips the penalization forcing from damping to amplifying.
- The Brinkman envelope itself is unenforced and currently violated: the physical layer
  `√(ην) = 0.144·dx` is ~7× thinner than one cell, and the resolution criterion `η ≥ dx²/ν = 0.771` is
  violated **48×** by the configured `η = 0.016`.

The last of these is already failing nightly. Phase 1 added η and mask-smoothing ladders to
`qtt_cylinder_verification`; both report `NOT CONVERGING`, which is what established that the reported
`C_d ≈ 23.8` is a property of the mask blur width rather than of a cylinder. **This change therefore
has its acceptance test already written and already red.**

Audit `AUDIT-REPORT.md` §4b, §5b and §9 Phase 2 items 10, 12, 13, 14.

## What Changes

- **Reject non-hyperbolic states.** Every compressible marcher rejects non-positive pressure the same
  way it already rejects non-positive density — an error naming the offending quantity, not a silent
  floor. All four sites, treated uniformly.
- **Fix the precision-dependent floor.** Whatever positivity guard survives must behave identically at
  `f32`, `f64` and `Float106`, rather than vanishing in one of them.
- **Validate the QTT numerical envelope at construction**, matching the DEC family's contract: the
  penalization parameter, the time step against the explicit-stability and diffusive limits, and the
  viscosity. A configuration outside the envelope is refused, not integrated.
- **Enforce the mask invariant.** `χ ∈ [0, 1]` is either guaranteed after quantization or checked, so
  the documented invariant and the shipped behaviour agree.
- **Resolve the Brinkman envelope.** Choose `η` from a stated wall-error target rather than from the
  explicit-stability ratio `dt/η = 0.25`, document the `η ≥ dx²/ν` resolution constraint, and state
  the configuration's standing against it. **This is the item that closes the failing gate.**
- **BREAKING (result-level):** resolving the envelope changes the immersed-cylinder configuration, so
  its reported `C_d`, its ladders and its baseline all move. The harness's own ladders are the
  acceptance test for whether the new configuration is defensible.

Explicitly **not** in scope: the penalization force definition itself (confirmed correct against Angot
et al. during the audit — the defect is parameter choice, not the force law), the DEC family, the
tensor-train codec, and the compressible marchers' flux scheme beyond the positivity guard.

## Capabilities

### New Capabilities

- `qtt-numerical-envelope`: the QTT solver family's admissibility contract — which configurations it
  accepts, which it refuses, and where the refusal happens. This is the capability the DEC family has
  in practice and the QTT family does not: a stated envelope, enforced at construction, with the same
  error type and the same "name the limit and the values" diagnostic quality.

### Modified Capabilities

- `qtt-immersed-body`: the Brinkman-penalized marcher's requirements gain the resolution constraint
  relating `η`, `ν` and `dx`, and the mask requirement's `[0, 1]` invariant becomes enforced rather
  than asserted. The immersed-cylinder validation requirement added in Phase 1 (the η and
  smoothing ladders) is the acceptance test for the envelope choice.
- `compressible-qtt-flux`: the flux requirement gains pressure positivity as a rejected invariant
  alongside density, uniformly across the 1-D, 2-D, 3-D and fitted-3-D marchers.

## Impact

**Code**
- `deep_causality_cfd/src/solvers/qtt/compressible/{euler_1d,marcher_2d,marcher_3d,marcher_3d_fitted}.rs`
  — the pressure guard and the precision-dependent floor, four sites, identical shape.
- `deep_causality_cfd/src/solvers/qtt/{immersed_2d,incompressible_2d}.rs` — constructor validation.
- `deep_causality_cfd/src/types/flow_config/qtt_march_config.rs` — whether the builder validates or
  defers to the constructor (see design).
- `deep_causality_cfd/src/tensor_bridge/mask.rs` — the `[0, 1]` invariant.
- `deep_causality_cfd/verification/qtt_cylinder_verification/config.rs` — `ETA`, `DT`, `SMOOTH_CELLS`
  and their derivation.

**Evidence**
- `qtt_cylinder_verification`'s ladders, gates and `baseline.txt` — currently the one red harness.
- `verification/README.md`'s `KNOWN-FAILING` block, which this change is expected to retire.
- Any harness whose configuration is now refused by the new envelope checks must be brought inside it
  or have its configuration justified.

**Risk**
- The envelope may refuse configurations the shipped harnesses currently use. That is the point, but
  it means this change can surface further work rather than only closing it — see the design's risks.
- Resolving `η` for a wall-error target may raise cost: resolving the Brinkman layer needs either a
  larger `η` (a softer wall) or a finer grid. The trade-off is stated, not assumed away.
- No public API change beyond constructors gaining rejection paths they already return `Result` for.
