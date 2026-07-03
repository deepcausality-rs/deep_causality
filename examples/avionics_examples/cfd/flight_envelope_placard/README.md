[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Flight-Envelope Placard Table

A Mach-altitude test matrix goes in; one gated placard table comes out. For every grid point
the study interpolates the freestream state from a cited US-1976 atmosphere table, computes
the dynamic pressure `q = 1/2 * rho * V^2`, the exact Rankine-Hugoniot post-shock stagnation
temperature, and the Sutton-Graves stagnation-point heating, then checks each point against
the stated q-max and stagnation-temperature placards. Any point outside the envelope is named
in the gate detail, not averaged away.

This is the pointwise study path on purpose. The matrix rows go through 
the same `sweep` combinator a solver study would use: 
the study shape (read a case table, compute per case,
gate, write one result table) does not change when the per-case body is a closed form instead
of a full march. Besides `sweep`, the only DSL pieces touched are `Gates` and the group-1
table reader and writer.

## How to Run

```bash
cargo run --release -p avionics_examples --example flight_envelope_placard
```

The default matrix is `mach_alt_matrix.csv`, sixteen points along a supersonic climb corridor
from Mach 0.5 at 5 km to Mach 5 at 40 km. All three gates pass and the process exits 0. The
run completes in well under a second.

The example accepts one optional argument, a path to a different matrix file. The recorded
negative scenario runs the same corridor plus one point beyond the q-max placard:

```bash
cargo run --release -p avionics_examples --example flight_envelope_placard \
    examples/avionics_examples/cfd/flight_envelope_placard/mach_alt_matrix_exceeds.csv
```

The q-max gate fails and names the offending point:

```
[FAIL] q-max placard: q = 85.1 kPa at M 1.50 / 5.0 km exceeds the 60 kPa placard
```

and the process exits 1. Both runs overwrite `placard_table.csv`; rerunning the default
matrix restores the recorded green table.

## What Happens When You Run It

1. **Read.** The Mach-altitude matrix loads through the group-1 table reader
   (`read_table`), which validates the header, the `#units` row, and every cell. A missing
   `mach` or `alt` column, a non-numeric cell, or an empty matrix is a setup failure (exit 2)
   naming the file and the fix.
2. **Compute.** `sweep` maps the placard closure over the matrix rows in input order. Per
   point: the freestream `(n, T, a)` interpolates linearly from the atmosphere table, `q`
   follows from the density and the flight speed, the stagnation temperature goes through the
   exact Rankine-Hugoniot jump (`FittedNormalShock::post_shock`, then isentropic
   re-stagnation of the post-shock state) above Mach 1 and through the shock-free isentropic
   form below it, and the Sutton-Graves correlation gives the stagnation-point heating for
   the stated nose radius. All arithmetic runs in the example's `FloatType` alias.
3. **Gate.** Three gates: every point inside the q-max placard (detail names the max-q
   point), every point inside the stagnation-temperature placard (detail names the hottest
   point), and matrix integrity (every file row computed). Offending points are listed by
   their Mach-altitude coordinates.
4. **Write.** The placard table lands in `placard_table.csv` through the group-1 writer, with
   named and unit-carrying columns: `mach(-)`, `alt(km)`, `q(kPa)`, `t0_post_shock(K)`,
   `qdot(W/cm2)`. The write is the one place the working precision downcasts to raw `f64`.

The recorded default run peaks at q = 23.7 kPa (M 1.20 / 11 km) and T0 = 1502.1 K
(M 5.00 / 40 km), both inside the placards. The full console record is `output.txt`.

## Gates

| Gate | Check | Recorded detail |
|---|---|---|
| q-max placard | every point's q at or below 60 kPa | max q = 23.7 kPa at M 1.20 / 11.0 km |
| stagnation-temperature placard | every point's T0 at or below 1700 K | max T0 = 1502.1 K at M 5.00 / 40.0 km |
| matrix integrity | one computed row per file row | 16 of 16 matrix rows computed |

A failing gate prints its `[FAIL]` line naming every out-of-envelope point and the example
exits 1. Setup and usage failures (an unreadable matrix, a missing column, an altitude
outside the atmosphere table) exit 2 with the file and the fix named.

## Limitations

* **The placards are demonstration values, not certification data.** 60 kPa and 1700 K are
  chosen in `constants.rs` to bound this corridor with margin; a real placard comes from a
  structures and thermal analysis.
* **The gas is calorically perfect.** The Rankine-Hugoniot jump runs with gamma = 1.4, so the
  post-shock stagnation temperature equals the freestream total temperature exactly and the
  supersonic and subsonic branches meet continuously at Mach 1. The approximation is at its
  crudest in the low supersonic range near a blunt body, where the bow shock is detached and
  curved rather than the normal shock assumed here; the stagnation streamline still crosses a
  locally normal shock, and total temperature is conserved regardless, so the placard-level
  numbers stand. Above the grid's hottest point (about 1500 K) vibrational excitation would
  start to matter; this grid stops below that regime.
* **Sutton-Graves is an entry-speed correlation.** At the low-Mach end of the grid the heating
  column is a fraction of a W/cm2 and serves as a trend, not a thermal-protection input.
* **The atmosphere interpolates linearly** between US-1976 rows spaced 5 to 10 km apart, which
  overstates density between rows (the true profile decays exponentially). For a placard
  demonstration the error is benign and conservative; a finer table drops in without code
  changes.

## Where Things Live

| File | Contents |
|---|---|
| `main.rs` | The whole study: read, sweep, gate, write |
| [`model.rs`](model.rs) | Domain logic: atmosphere interpolation and the per-point placard computation |
| [`model_config.rs`](model_config.rs) | Configuration: matrix and table paths, the fitted shock model |
| `constants.rs` | Every constant with its justification: gas model, Sutton-Graves, placards, atmosphere |
| `mach_alt_matrix.csv` | The default sixteen-point corridor (passes all gates) |
| `mach_alt_matrix_exceeds.csv` | The corridor plus one point beyond the q-max placard (the negative scenario) |
| `placard_table.csv` | The written placard table of the recorded default run |
| `output.txt` | The recorded console output of the default run |

The machinery used: `sweep` and `Gates` from `deep_causality_cfd`, `FittedNormalShock` from
the same crate's compressible solver, and `read_table`, `NumericTable::from_columns`, and
`write_table` from `deep_causality_file`.

## References

* U.S. Standard Atmosphere, 1976. NOAA-S/T 76-1562, NOAA/NASA/USAF, Washington, D.C., 1976.
* Sutton, K. and Graves, R. A., "A General Stagnation-Point Convective-Heating Equation for
  Arbitrary Gas Mixtures", NASA TR R-376, 1971.
* Anderson, J. D., *Modern Compressible Flow: With Historical Perspective*, 3rd ed.,
  McGraw-Hill, 2003 (the normal-shock and isentropic relations).
