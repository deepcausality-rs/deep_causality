## ADDED Requirements

### Requirement: The vortex-shedding resonance margin is a self-verifying example

`examples/avionics_examples/cfd/viv_resonance_margin` SHALL sweep airspeed over a circular
cross-section using `sweep` and `run_owned` on the validated cylinder-wake configuration,
extract the vortex-shedding frequency per airspeed with the existing frequency observables,
and produce one margin table (airspeed, shedding frequency, margin to a stated structural
natural frequency) through the group-1 writer. Gates SHALL check that the extracted Strouhal
number stays inside the solver's validated band for the flown Reynolds-number range and that
the resonance margin at every swept airspeed clears the stated minimum. The example SHALL
state its Reynolds-number range plainly and claim nothing beyond the laminar-wake regime the
solver validates in; it SHALL compute in its `FloatType` alias and exit nonzero on regression.

#### Scenario: The margin table is produced and gated

- **WHEN** the example runs its recorded airspeed schedule
- **THEN** each airspeed yields a shedding frequency from the computed wake, the margin
  column is gated against the stated minimum, the Strouhal check stays inside the validated
  band, and all gates pass with exit code zero
