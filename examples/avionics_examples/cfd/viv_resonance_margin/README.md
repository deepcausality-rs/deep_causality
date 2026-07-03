[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# The Vortex-Shedding Resonance Margin

A circular member in a stream sheds a von Karman street, and the street pushes on the
structure at the shedding frequency `f = St * V / D`. When that frequency approaches a
structural natural mode, the member locks in and shakes itself apart. Checking the margin
between the two is a standard installation task for masts, booms, and struts.

This example runs that check as a computed study. It sweeps airspeed over the validated
isolated-cylinder configuration, computes each wake with the DEC incompressible solver,
extracts the shedding frequency from the wake probe, and gates the margin to a stated
structural mode. The result is one table an engineer can read row by row: airspeed, Reynolds
number, Strouhal number, shedding frequency, margin.

## How to Run

From the repository root:

```bash
cargo run --release -p avionics_examples --example viv_resonance_margin
```

Wall-clock is about two and a half minutes: four wake marches, run concurrently under the
`parallel` feature, bit-identical to the sequential run. The whole computation runs in the
example's `FloatType` alias; `f64` appears only where the result table is written.

## What Happens When You Run It

1. The airspeed schedule loads from [`airspeeds.csv`](airspeeds.csv) through the typed table
   reader. Four airspeeds, chosen so the member's Reynolds number spans 100 to 160.
2. `sweep` runs one wake march per airspeed on the isolated-cylinder configuration, each
   through `CfdFlow::march(..).run_owned()`. The configuration reuses the grid, body, and
   probe placement of the cylinder wake verification, so the example inherits validated
   territory instead of claiming new ground.
3. The shedding frequency per airspeed comes from the Strouhal extraction on the probe's
   developed tail, then dimensionalizes as `f = St * V / D`.
4. The margin table writes to `viv_resonance_margin.csv` with named, unit-carrying columns.

The measured run: St 0.1818 to 0.1909 across the sweep, shedding frequencies 68 to 115 Hz
against the stated 150 Hz structural mode, worst margin 0.236.

## Gates

Three gates, exit nonzero on any regression:

1. **Strouhal band.** The extracted St at every airspeed stays inside the band measured for
   this grid (0.16 to 0.21), which brackets the unconfined laminar reference (Williamson:
   about 0.164 at Re 100 rising to about 0.185 at Re 160). A dead or numerically broken wake
   fails here.
2. **Resonance margin.** The worst row's margin `|f_struct - f_shed| / f_struct` clears the
   stated placard minimum of 0.15.
3. **Run integrity.** Every sweep returns a finite, oscillating wake.

## Limitations

The solver validates cylinder shedding in the laminar-wake regime, and the sweep stays inside
it (Re 100 to 160). Nothing here claims turbulent shedding; a flight-Reynolds strut sits far
above this range and waits on the staged turbulence work. The structural frequency and the
margin placard are stated demonstration values in [`constants.rs`](constants.rs), not measured
properties of a real member; every constant carries its justification there. Blockage and the
coarse grid shift St slightly against the unconfined reference, and the gate band records the
measured shift rather than hiding it.

## Where Things Live

| File | Contents |
|---|---|
| [`main.rs`](main.rs) | Orchestration: schedule in, sweep, table out, gate verdict |
| [`model.rs`](model.rs) | Domain logic: the march-and-reduce step and the margin row |
| [`model_config.rs`](model_config.rs) | Configuration: the validated cylinder-wake case per Reynolds number |
| [`utils_print.rs`](utils_print.rs) | Console rendering and the three gates |
| [`constants.rs`](constants.rs) | Every tuned constant with its justification |
| [`airspeeds.csv`](airspeeds.csv) | The swept airspeed schedule |
| `viv_resonance_margin.csv` | The written margin table (produced by the run) |
| [`output.txt`](output.txt) | The recorded console run |

## References

- Williamson, "Vortex dynamics in the cylinder wake," Annual Review of Fluid Mechanics (1996):
  the laminar St-Re relation the band brackets.
- The in-repo cylinder verifications (`deep_causality_cfd/verification/dec_cylinder_*`): the
  validated configuration this example reuses.
