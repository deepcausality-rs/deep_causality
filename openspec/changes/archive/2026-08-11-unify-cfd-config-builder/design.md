## Context

`CfdConfigBuilder` was introduced to mirror the Discovery `CdlConfigBuilder` → `CdlBuilder` split:
configuration (the "what") is built by one entry, and `CfdFlow` composes and runs it (the "how").
Four families follow that shape. Five do not, and the divergence grew family by family as the
compressible carrier, the QTT march, the duct march and the blended coordinate map each landed with
their own entry.

Measured state at the start of this change:

| Family | Entry today | Sites |
|---|---|---|
| `DecNsConfig` | `CfdConfigBuilder::dec_ns()` **and** `DecNs::config()` | 8 bypass |
| `MarchConfig` | `CfdConfigBuilder::march::<D, R>(name)` | — |
| `VerifyConfig` | `CfdConfigBuilder::verify::<R, M>(name, m)` | — |
| `UncertainMarchConfig` | `CfdConfigBuilder::uncertain_march::<R>(name)` | — |
| `QttMarchConfig` | `QttMarchConfigBuilder::<R>::new()` + `.name(..)` | 24 |
| `CompressibleMarchConfig` | `CompressibleMarchConfigBuilder::<R>::new()` + `.name(..)` | 16 |
| `DuctConfig` | `DuctConfig::new(profile, inlet, gamma, back_p, cells, stop)` | 23 |
| `BlendedMapConfig` | `BlendedMapConfig::new(lx, ly, r0, dr, theta0, dtheta, lambda)` | 7 |

Plus six public-field types built as struct literals: `ReferenceScales` (6), `PlumeImprint` (2),
`PlumeNozzle` (3), `DuctInlet`/`DuctStop` (13), `AtmosphereRow` (11).

The pattern to converge on already exists in the crate. `MarchConfigBuilder::new`,
`UncertainMarchConfigBuilder::new` and `VerifyConfigBuilder::new` are `pub(crate)`; the case name is
taken at the `CfdConfigBuilder` entry; `build()` validates and returns `Result<_, PhysicsError>`.
This change applies that shape to the rest.

## Goals / Non-Goals

**Goals:**

- One public door per configuration family, with the case name taken at the door.
- No configuration type reachable through a public bypass constructor or public fields.
- A stated, checkable boundary between a configuration (a `CfdConfigBuilder` entry) and a
  configuration *input* (a `Mesh`-peer value type).
- A named, reasoned exemption class for verification harnesses that drive solvers directly, so the
  remaining direct-construction sites are a decision on the record rather than an omission.
- Bit-identical numerical output from every touched binary and example.

**Non-Goals:**

- Changing any solver, kernel, or numerical path. Every tranche is re-routing.
- Introducing configuration families for solver primitives (1-D Euler, 3-D compressible,
  DEC scalar transport). That is a separate change, and the exemption class below is written so it
  does not block one.
- Moving `DescentSchedule` or `ForcingRegion` into `CfdConfigBuilder`.
- Retiring `AtmosphereRow`'s public fields.

## Decisions

### D1 — Three tranches, each independently green

Tranche A is single-entry re-routing, Tranche B absorbs the struct-literal configs, Tranche C
handles the remaining bundles and the raw-solver harnesses. Each tranche ends with the workspace
building, `bazel test //deep_causality_cfd/...` green, and the touched binaries reproducing their
recorded numbers.

*Alternative considered:* one sweep. Rejected — the change touches ~145 call sites across 30 files,
and a single sweep makes a numerical regression expensive to bisect. The tranche boundaries are
chosen so each is separately revertible.

### D2 — The name is taken at the entry, and required

`CfdConfigBuilder::qtt_march::<R>(name)`, `::compressible_march::<R>(name)` and `::duct::<R>(name)`
follow `march` / `verify` / `uncertain_march`. The `.name(..)` builder methods and the
`"qtt_march"` / `"compressible_march"` default names go away, and `DuctConfig` gains a name so the
duct report stops being labelled `"duct_march"` for every case.

*Alternative considered:* keep `.name(..)` optional with defaults. Rejected — two ways to name a
case is the inconsistency this change exists to remove, and an unnamed case produces reports that
cannot be told apart in a study sweep.

### D3 — `CfdConfigBuilder` starts solver and case configurations, not every parameter bundle

The boundary: a `CfdConfigBuilder` entry starts a configuration that `CfdFlow` runs (a solver config
or a marching case). Everything a case *reads* — `Mesh`, `Body`, `Seed`, `Observe`, `QttObserve`,
`DescentSchedule`, `ForcingRegion`, `BlendedMapConfig` — is a value type with its own validated
constructor and fluent overrides.

This is what keeps `DescentSchedule` and `ForcingRegion` where they are, and it is why
`BlendedMapConfig` gets a builder without getting an entry: a coordinate map is an input to a
compressible march, one level below the case.

*Alternative considered:* every configuration-shaped type gets a `CfdConfigBuilder` entry. Rejected
— it makes the entry list a directory of the crate's types rather than a list of things that can be
run, and `CfdFlow::march` has nothing to dispatch on for a coordinate map.

### D4 — Validation moves with the constructor, unchanged

`DuctConfig::new` carries ~90 lines of validation. It moves verbatim into `DuctConfigBuilder::build`,
and `DuctConfig::new` becomes a crate-private field constructor. The 15 rejection tests in
`duct_config_tests.rs` move to a new `duct_builder_tests.rs` and keep their assertions; only the
construction call changes. Same for the `DecNsConfigReady::build` validation, which is untouched.

### D5 — Case-by-case ruling on the eight raw-solver binaries

Ruled on configuration size and on whether the binary's subject *is* the direct construction.

| Binary | Configuration surface | Ruling |
|---|---|---|
| `dec_cylinder_verification` (512 L) | 9 environment knobs, cut-cell registry with merge fraction, zone tuple, CG options + warm start + staircase toggle, perturbed seed, wake probe, per-step drag with pressure/friction split | **Lift** onto `CfdConfigBuilder::march` |
| `qtt_blunt_body_2d` (212 L) | 9 constants, two 7-argument `BlendedMapConfig::new` calls | **Benefits from C1**, no case config |
| `srp_momentum_jet` (618 L + 178 L config) | ~30 constants in a dedicated `config.rs`, 3 forcing regions, environment dials | **Keep** — already honours config/execution separation |
| `qtt_sod` (93 L) | 9 scalars, one solver constructor | **Keep** — exemption |
| `qtt_reentry_3d` (239 L) | 8 constants, one marcher | **Keep** — a rank-measurement harness, mostly `quantize_3d` |
| `dec_graded_mms_verification` (312 L) | 1 constant, operator-level MMS on manifolds | **Keep** — below the config layer, no march |
| `dec_wall_heat_flux_verification` (220 L) | 5 constants, cut-cell fixtures | **Keep** — a kernel gate |
| `compressible_carrier_timing` (310 L) | 5 constants, sweeps marcher assembly | **Keep** — marcher assembly cost is the measurement |

The five "keep" harnesses become a named exemption class in `cfd-config-entry`: a binary whose
subject is a solver, kernel, or codec below the case layer constructs it directly, and says so in
its module docstring. Without that, the next reader has to re-derive the reasoning per binary.

*Alternative considered:* lift all eight. Rejected — three of them (1-D Euler, 3-D compressible,
DEC scalar transport) have no config family to lift onto, so "lift" means inventing three new
families to serve one binary each. `compressible_carrier_timing` would additionally measure the
wrapper it was written to measure without.

### D6 — Lifting the cylinder harness adds two library capabilities

`MarchPipeline` already computes what the harness computes by hand: `surface_force_coefficients`
calls `pressure_surface_force` + `viscous_surface_force`, driven by `Observe::drag(u_ref)` and
`Mesh::cut_registry()`, and `run_with` gives a per-step `StepView` for the harness's own sampling.
Two gaps remain:

1. **Seed.** The harness seeds a uniform stream plus a single-signed transverse Gaussian blob to
   break the top–bottom symmetry that otherwise suppresses shedding. `Seed` has `UniformX` but no
   perturbed variant. Add one carrying the blob offset, width and amplitude, keeping `Seed`
   `Copy` (no boxed closures, so a case stays cloneable).
2. **Drag split.** The pipeline sums pressure and friction into one `drag` series; the harness
   reports the split against the Dröge & Verstappen reference. Add an opt-in that emits the two
   contributions as separate series alongside the combined one.

Both are small, both are reusable beyond this binary, and both are the reason this lift is worth
doing rather than exempting the harness.

*Alternative considered:* exempt `dec_cylinder_verification` too. Rejected — it is the largest
configuration surface in the crate and a hand-built duplicate of the family it should be using, so
exempting it would exempt the case that motivates the rule.

### D7 — `AtmosphereRow` keeps public fields

It is a table row: 11 of its sites are literal atmosphere tables, and a constructor per row would
add noise without adding validation that `DescentSchedule::new` does not already perform over the
whole table (ascending altitude, finite and positive `n_tot` / `temperature` / `sound_speed`).

## Risks / Trade-offs

**Numerical drift from the cylinder lift** → The only tranche that changes an execution path. Gate
it by running `dec_cylinder_verification` at its default knobs before and after and requiring the
reported `St` and `C_d` (and the pressure/friction split) to match to the printed precision. The
lift is the last item in Tranche C, so it can be dropped without affecting the rest.

**The perturbed seed changes the seed field for existing users** → It is a new `Seed` variant;
`UniformX` is untouched, so no existing case changes.

**Wide source-breaking surface** → ~145 call sites, all inside the workspace (`publish = false`).
Mitigated by tranche boundaries and by doing the shared test fixtures first
(`tests/types/flow/compressible_march_run/mod.rs` feeds four sibling files).

**Intra-doc link breakage** → `march_builder.rs` links `[DecNs::config](crate::DecNs::config)` as
the contrasting type-state pattern. Unexporting `DecNs` breaks that link; the paragraph is rewritten
to name `CfdConfigBuilder::dec_ns` instead.

**`Default` removal on two builders** → `QttMarchConfigBuilder` and `CompressibleMarchConfigBuilder`
implement `Default`, which cannot be made crate-private. Both impls are removed. Clippy's
`new_without_default` does not fire on a `pub(crate) fn new`, so no suppression is needed.

**Spec drift is corrected in the same change** → `qtt-flow` requires a `CfdFlow::qtt_march(&config)`
entry the code replaced with the unified `CfdFlow::march` + `MarchDispatch`. Left alone, the delta
would restate a requirement the code already fails. It is corrected here rather than deferred.

## Migration Plan

1. **Tranche A** — library entries, then shared test fixtures, then the remaining tests, then the
   6 binaries and 3 example files. Gate: workspace builds, `bazel test //deep_causality_cfd/...`
   green, touched binaries reproduce recorded numbers.
2. **Tranche B** — absorb `DuctInlet`/`DuctStop`/`ReferenceScales` into builder methods; add
   validated constructors to `PlumeImprint`/`PlumeNozzle`. Same gate.
3. **Tranche C** — `BlendedMapConfig` builder; exemption docstrings on the five kept harnesses; the
   `Seed` variant and drag-split observable; the cylinder lift. Same gate plus the
   before/after numeric diff of `dec_cylinder_verification`.
4. Docs (`deep_causality_cfd/README.md`, two verification READMEs) after Tranche C.

Rollback: each tranche is a separate commit; reverting one leaves the others coherent, since a
later tranche never depends on an earlier tranche's *removals*, only on its entries existing.

## Open Questions

- `DuctInlet` / `DuctStop` become unused public types once the builder absorbs them. Deleting them
  needs explicit approval under the repository's no-deletion rule; until then they stay exported and
  unused, which Tranche B records as a follow-up.
