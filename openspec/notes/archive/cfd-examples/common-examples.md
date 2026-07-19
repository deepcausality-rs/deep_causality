# Common CFD examples

Date: 2026-07-03. Status: **planning note, nothing proposed or built.** Related:
[dsl-review](../cfd-dsl/01-dsl-review.md) (the five examples drafted in today's DSL, and the five additive
syntax pieces S1 to S5 they justify; group 2/3 specs should reference it),
[cfd-roadmap](../../cfd-plasma-retropulsion/cfd-roadmap.md),
[qmc-presence-gate-followup](../cfd/qmc-presence-gate-followup.md).

The examples directory currently spans canonical verifications (cylinder, cavity,
Taylor-Green) and the plasma-blackout corridor. Between them sits the everyday work an
aerospace engineer actually does: sweep a parameter, build a table, check it against a
placard or a closed form, sign it. The five examples below fill that middle ground with
machinery that exists today; none of them waits on turbulence. The common shape is
*sweep, table, gate, CSV*, and each example mirrors a different desk (propulsion,
structures, loads, thermal, test), so the directory reads as coverage of departments,
not of one heroic case.

Honest boundary: until the staged turbulence work lands, high-Reynolds external
aerodynamics over a real component stays out of this set.

## Group 1: precondition, the `deep_causality_file` crate

> **STATUS: shipped (2026-07-03)** through `openspec/changes/add-cfd-file-io/`: the typed
> table reader, the units-aware result-table writer, the sensor-trace loader, and the two-tier
> snapshot/resume (checksummed container, `force_load` with reported warnings, world
> fingerprint, bit-exact scalars, one-line `save_state_snapshot` on a paused march and
> `load_resume_state` for the continuing workflow). Groups 2 and 3 are unblocked.

Every example below reads an input table and writes a result table, so the file seam
comes first. `deep_causality_file` today provides IO-monad loaders
(`deep_causality_haft::IoAction`, lazy until `.run()`) for RINEX GNSS precise products
(SP3 orbits, `.clk` clocks), precision-generic over the scalar `R`. The CFD side owns
only `write_csv` / `write_xy_csv` for output. What the everyday set needs from the crate:

1. **A generic typed table reader.** One loader for delimited numeric tables returning
   typed rows through the same `IoAction` pattern: test matrices (a Mach-altitude grid),
   atmosphere tables, flow-rate schedules, and measured sensor traces all arrive as CSV
   in practice. Precision-generic like the GNSS loaders, with the exact-`f64`-literal
   lift convention preserved (parse to `f64`, lift via the caller's `ft`).
2. **A result-table writer.** A header-and-units-aware companion to `write_xy_csv`, so a
   placard table or an operating map serializes with its column meanings, not as bare
   columns.
3. **A sensor-trace loader for example 5.** Time-stamped noisy samples per channel,
   feeding `MaybeUncertain` / `Uncertain` construction on the way in (an intermittent
   channel maps to `MaybeUncertain`, per the presence-gate note).
4. **Snapshot and resume: save the running state to disk, load it in another workflow.**
   The everyday reality this serves: work gets suspended for any reason, and the
   researcher writes one line in the flow (save to disk with a file path), then a
   different workflow days later loads the full package and continues. A tensor-train
   state makes this cheap where dense-CFD checkpointing is painful: the snapshot is the
   compressed representation itself, order `chi^2 * L` numbers (kilobytes for the
   corridor's layer), so the serialized artifact carries the compression for free.

   The package is the *full* state, never a bare tensor: the tensor cores with ranks,
   every carried scalar field, the navigation engine (filter state and covariance), the
   provenance log, the step index, the scalar type (`f64` vs `Float106` must round-trip
   bit-exact), and a fingerprint of the world description it belongs to, so loading into
   a mismatched world is refused loudly instead of diverging quietly.

   Integrity is a hash checksum computed over the whole package (tensor data and all
   metadata) at save time and re-verified at load time. On mismatch the loader reports a
   corrupt-file error naming the file; a `force_load` override exists for salvage work,
   but the default informs the user the moment anything corrupted the data. Same
   `IoAction` pattern as the other loaders: the save and the load are lazy descriptions,
   run at the edge.

   Two tiers, scoped separately: a field snapshot (cores plus metadata, for
   area-of-interest artifacts and golden field baselines) and the full resume package
   described above. The field tier is also the payload convention the roadmap's ROM
   export (item 4) and self-describing archive (item 5) both need, so building it here
   back-fills their foundation.

5. **Practitioner-grade errors (open; added 2026-07-03).** The seams above return good
   errors for bad *files*; this item audits what a bad *usage* produces, across the group-1
   surfaces and the DSL pieces the examples use (S1 to S5 of the [dsl-review](../cfd-dsl/01-dsl-review.md)).
   The standard: every error a practitioner can reach speaks engineering, names the file,
   column, row, field, or gate involved, and says what to do next; no trait-bound or
   generic-parameter language ever reaches an example user. Each everyday example's tests
   include at least one wrong-usage case asserting the message quality, the way the group-1
   loaders already assert theirs.

This group also connects to the roadmap's self-describing-results item: the reader and
writer are the file half of the config-as-data story, the snapshot package is the field
payload of the archive, and all of it belongs in `deep_causality_file` rather than in
each example.

## Group 2: examples 1 to 3 (analytic anchors, existing solvers, small effort)

> **STATUS: shipped (2026-07-03)** through `openspec/changes/add-cfd-study-dsl-and-examples/`,
> together with the DSL review's S1 to S5 (sweep, Gates, run_owned, from_columns, duct_march).
> Measured: the nozzle map's shocks land within 12 cells of the closed form and the supersonic
> rows exit at M 2.12 vs design 2.197; the VIV sweep reads St 0.182 to 0.191 at Re 100 to 160
> on the Williamson laminar reference; the placard grid peaks at q 23.7 kPa (M 1.20 / 11 km)
> inside its stated placards. Examples 4 and 5 remain open (example 5 waits on roadmap item 3).

**Example 1: nozzle operating map (propulsion desk).** Sweep back pressure on a
converging-diverging duct; find choking, the normal-shock position, and the thrust
coefficient per point. Machinery: the 1-D compressible Euler solver, exact
Rankine-Hugoniot jumps. Gates: area-Mach relation and normal-shock relations in closed
form per row. Output: one operating-map table. The sweep is a vector of configs, so the
example doubles as the plainest demonstration of configuration-as-data.

**Example 2: vortex-induced-vibration margin check (structures desk).** The validated
cylinder wake dressed as the question an installation engineer signs: at what airspeed
does an antenna mast or strut shed at the structure's natural frequency, and what is the
margin? Sweep airspeed, extract the shedding frequency with the existing
`dominant_frequency` / `strouhal_number` observables, gate the resonance margin.
Near-zero new physics; the value is the reframing from benchmark to deliverable.

**Example 3: flight-envelope placard table (loads desk).** Over a Mach-altitude grid
(read from a test-matrix file, group 1), compute dynamic pressure, Reynolds per meter,
post-shock stagnation temperature, and Sutton-Graves heating; gate against q-max and
temperature placards. Pointwise closures plus the atmosphere machinery only; minutes of
wall-clock; the least glamorous and most recognizable artifact of the set.

## Group 3: examples 4 and 5 (larger, each pulls one roadmap item forward)

**Example 4: cooling-channel pressure-drop sizing (thermal desk).** Sweep flow rate
through a laminar channel; compute pressure drop; gate against the exact Poiseuille
state and the laminar friction-factor law the DEC solver already validates against.
Pulls the wall-bounded DEC solver out of benchmark territory into a sizing memo.
Optional second axis: an immersed obstacle in the channel via the existing mask path.

**Example 5: wind-tunnel data reduction with error bars (test desk).** Measured noisy
inflow in (sensor-trace loader, group 1), force coefficients with uncertainty out, and
the significance of a configuration delta gated instead of eyeballed. Machinery: the
sensor-fed `UncertainMarchPipeline` and the `deep_causality_uncertain` statistics. Two
recorded design facts apply directly: the presence gate stays the hybrid SPRT-MC design
with QMC reserved for the collapse (see
[qmc-presence-gate-followup](../cfd/qmc-presence-gate-followup.md); a fully-QMC pass would use
the fixed-N `estimate_probability_qmc` opt-in, not a QMC SPRT), and deterministic
seeding keeps the example bit-reproducible. This example doubles as the demonstrator
for the roadmap's dispersion-sweep combinator (item 3) and is best built alongside it.

## Roadmap interactions: what to preload, what not to

The rule: preload a [roadmap item](../../cfd-plasma-retropulsion/cfd-roadmap.md) only where it makes an
everyday example simpler or keeps its output from being throwaway; never where it makes
an example bigger, because this set's value is smallness.

**Preload.**
- *Item 1 (objective convergence gates)*: costless and already the house convention;
  every example ships with its analytic gate from day one.
- *Item 3 (the `Uncertain<T>` dispersion combinator)*: land it before example 5. It
  shrinks the example rather than growing it: hand-rolled means and sigma rules become
  `probability_exceeds(...)` at a stated confidence, and the sample count turns adaptive.
- *Item 5 (self-describing results), file half only*: do not preload the archive; do
  design the group-1 result-table writer as the archive's table payload (headers, units,
  column semantics), so the everyday examples become archive-ready later with zero
  rework.

**Do not preload.**
- *Item 2 (spectra, vorticity)*: the VIV example needs only the existing frequency
  observables; item 2's priority comes from turbulence.
- *Item 4 (ROM export)*: no everyday example produces a state worth re-shipping.
- *Item 6 (data fusion)*: the dependency points the other way; example 5 builds the
  measured-data ingestion and noise bookkeeping item 6 later consumes.
- *Item 7 (rasterizer)*: the set uses analytic geometry on purpose; complex geometry
  doubles each example for no gain in relatability.

The reverse flow is part of the value: examples 1 and 3 exercise the typed table reader
the enterprise config story needs, and example 5 is the live proving ground for roadmap
items 3 and 6.

## Sequencing

Group 1 first (with the item-5-aligned result-table writer; the snapshot tier of item 4
can land with it or immediately after); it is small and everything
else consumes it. Examples 1 to 3 are each a day or two against analytic gates. Roadmap
item 3 next, then examples 4 and 5, so example 5's statistics arrive typed instead of
hand-rolled.
