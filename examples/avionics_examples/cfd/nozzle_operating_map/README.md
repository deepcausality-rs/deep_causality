[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Nozzle Operating Map

Dropping the back pressure on a converging-diverging duct walks it through its operating
regimes. Above the first critical ratio the duct flows subsonic throughout. Below it the
throat chokes and a normal shock stands in the diverging section, marching toward the exit as
the back pressure falls. Below the exit-shock ratio the duct runs supersonic all the way out.
Where the shock sits and what thrust coefficient each point produces is the operating map, and
computing one is routine sizing work in propulsion and test.

This example computes that map with the 1-D compressible duct march and gates every row
against gas-dynamics closed forms. Nothing in the gates comes from the solver itself: choking
checks against the sonic throat, shock positions check against the isentropic-plus-
Rankine-Hugoniot construction, and shock-free profiles check against the area-Mach relation.

## How to Run

From the repository root:

```bash
cargo run --release -p avionics_examples --example nozzle_operating_map
```

Wall-clock is under a second: six duct marches at 128 cells, run concurrently under the
`parallel` feature. The whole computation runs in the shared example `FloatType` alias; `f64`
appears only where the result table is written.

To see the reader refuse a malformed schedule (the wrong-usage path), pass the provided bad
file and watch the error name the file, the row, and the column:

```bash
cargo run --release -p avionics_examples --example nozzle_operating_map \
  examples/avionics_examples/cfd/nozzle_operating_map/back_pressures_bad.csv
```

The run exits nonzero and writes no table.

## What Happens When You Run It

1. The schedule loads from [`back_pressures.csv`](back_pressures.csv): six ratios
   `p_back / p0` from 0.90 down to 0.10, spanning the internal-shock window (the analytic
   references print with the run: first critical at 0.937, exit-plane shock at 0.513).
2. `sweep` runs one `CfdFlow::duct_march` per ratio on the 2:1:2 parabolic nozzle
   (reservoir 300 kPa, 500 K, gamma 1.4).
3. Each report reduces to one row: exit Mach, shock station, thrust coefficient. The measured
   map: the shock marches from 0.656 m to 0.930 m as the ratio falls from 0.90 to 0.60, and
   the two supersonic-exit rows leave at Mach 2.12 against the design value 2.197.
4. The operating map writes to `operating_map.csv` with named, unit-carrying columns
   (`shock_x` uses -1 for shock-free rows; the in-memory report omits the series instead).

## Gates

Five gates, exit nonzero on any regression:

1. **Schedule integrity.** Every scheduled ratio produced a converged march.
2. **Choking.** Every choked row crosses Mach 1 within the stated band of the throat.
3. **Shock position.** Every internal-shock row lands within the measured band (twelve cell
   widths at 128 cells) of the closed-form position.
4. **Shock-free profiles.** Interior stations track the area-Mach relation within the
   measured 5 percent band.
5. **Physical thrust.** The thrust coefficient is finite and positive on every row.

The bands live in [`constants.rs`](constants.rs) with their derivations; they were measured by
the duct-march verification tests at this resolution, not chosen.

## Limitations

The duct march is a first-order quasi-1-D scheme: shocks smear over a few cells and the
profiles carry a resolution-dependent bias, which is exactly what the measured bands express.
The geometry is the parabolic demonstration nozzle, perfect gas at gamma 1.4; real nozzle
work adds wall friction, heat transfer, and real-gas effects this example does not model.
The back-pressure schedule stays inside the choked regimes; the unchoked window above the
first critical ratio is not swept.

## Where Things Live

| File | Contents |
|---|---|
| [`main.rs`](main.rs) | Orchestration: schedule in, sweep, table out, gate verdict |
| [`model.rs`](model.rs) | Domain logic: the march-and-reduce step and the closed-form references |
| [`model_config.rs`](model_config.rs) | Configuration: the duct case description per back pressure |
| [`utils_print.rs`](utils_print.rs) | Console rendering and the five gates |
| [`constants.rs`](constants.rs) | Geometry, reservoir state, and the measured gate bands |
| [`back_pressures.csv`](back_pressures.csv) | The swept schedule |
| [`back_pressures_bad.csv`](back_pressures_bad.csv) | The wrong-usage demonstration input |
| `operating_map.csv` | The written map (produced by the run) |
| [`output.txt`](output.txt) | The recorded console run |

## References

- Anderson, "Modern Compressible Flow: With Historical Perspective": the isentropic relations,
  the area-Mach relation, and the normal-shock construction behind every gate.
- The duct-march verification (`deep_causality_cfd/tests/types/flow/duct_march_tests.rs`):
  where the bands were measured.
