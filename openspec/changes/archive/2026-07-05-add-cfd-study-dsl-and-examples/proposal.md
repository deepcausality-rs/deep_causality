# Proposal: add-cfd-study-dsl-and-examples

## Why

The DSL review (`openspec/notes/cfd-examples/dsl-review.md`) drafted five planned examples in
the current `CfdFlow` syntax and found the same friction in each: sweeps, gates, and result
tables are hand-rolled per program, the gate block already exists as three near-identical
copies in the tree, and the nozzle case cannot be expressed in the DSL at all because no 1-D
duct path exists. The review's additive syntax program (S1 to S5) removes that friction with
no breaking change. This change ships the syntax program together with the three examples it
was verified against first, so the syntax lands proven against real programs rather than
speculation.

## What Changes

- **Study primitives in `deep_causality_cfd`** (review items S1, S2, S4): `sweep`, an
  order-preserving, `Result`-collecting, deterministically parallel map over case inputs;
  `Gates`, the acceptance-gate builder that replaces the hand-rolled PASS/FAIL block; and
  `run_owned`, one-shot geometry materialization on the march pipelines for sweep bodies that
  do not reuse a caller-owned manifold.
- **`NumericTable::from_columns` in `deep_causality_file`** (review item S3): one-call table
  construction from column descriptors and rows.
- **The duct march in `deep_causality_cfd`** (review item S5): an owned `DuctConfig` (area
  profile, inlet stagnation state, back pressure, stop condition) lowered onto the 1-D
  compressible Euler solver through a new `CfdFlow::duct_march` entry, returning the standard
  `Report` with duct observables (Mach and pressure profiles, shock position, thrust
  coefficient).
- **Isentropic duct relations in `deep_causality_physics`**: the area-Mach relation and the
  supporting isentropic ratios as cited pointwise kernels, needed to gate the duct march
  against closed forms.
- **Three examples in `examples/avionics_examples/cfd/`**, each self-verifying (gates, exit
  nonzero on regression), each computing in the example's `FloatType` alias, each writing its
  result table through the group-1 writer, each with a README in the established convention:
  - `nozzle_operating_map`: sweeps back pressure on a converging-diverging duct and tables
    choking, normal-shock position, and thrust coefficient, gated per row against the
    area-Mach and normal-shock relations.
  - `viv_resonance_margin`: sweeps airspeed over a circular cross-section (an antenna mast or
    strut), extracts the vortex-shedding frequency from the computed wake, and gates the
    margin to a stated structural natural frequency.
  - `flight_envelope_placard`: reads a Mach-altitude test matrix, computes dynamic pressure,
    post-shock stagnation temperature, and stagnation-point heating per point, and gates the
    grid against q-max and temperature placards.
- **Optional migration**: the three existing hand-rolled gate blocks (corridor, stagline,
  weather) may migrate to `Gates` when next touched; nothing forces it.

## Capabilities

### New Capabilities

- `cfd-study-dsl`: the `sweep` combinator, the `Gates` builder, and `run_owned` one-shot
  geometry on the march pipelines.
- `duct-march`: the 1-D compressible duct path (config, runner, observables) plus the cited
  isentropic-relation kernels that gate it.
- `nozzle-operating-map`: the back-pressure sweep example with its analytic gates and table.
- `viv-resonance-margin`: the airspeed sweep example with its frequency extraction and
  margin gates.
- `flight-envelope-placard`: the test-matrix example with its placard gates and table.

### Modified Capabilities

- `typed-table-io`: gains the `from_columns` constructor requirement (one-call table
  construction preserving the existing validation and round-trip guarantees).

## Impact

- `deep_causality_cfd`: new `sweep`/`Gates` in the flow module, `run_owned` on
  `MarchPipeline` (and the uncertain twin), the `DuctConfig` container and `duct_march`
  runner, duct observables; tests mirrored.
- `deep_causality_physics`: isentropic area-Mach and ratio kernels with citations and the
  source PDF in `papers/`; tests mirrored.
- `deep_causality_file`: the `from_columns` constructor; tests mirrored.
- `examples/avionics_examples`: three new example binaries under `cfd/`, registered in
  `Cargo.toml`, each with `README.md` and a recorded `output.txt`.
- Existing behavior: unchanged; all additions. The optional gate-block migration is out of
  this change's scope.
- Downstream: unblocks the remaining two planned examples (they need only S1 to S4 plus
  roadmap item 3) and retires the dsl-review's open program.
