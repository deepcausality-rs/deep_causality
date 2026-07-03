## ADDED Requirements

### Requirement: Uncalibrated stagnation-line prediction

The stagnation-line verification (`qtt_ramc_stagline`) SHALL be re-measured with the finite-rate network and
**no Saha calibration target**: the peak electron density SHALL be a prediction from the Park rate data and
the evolved thermodynamic state alone. The acceptance band SHALL be pinned from that measurement against the
RAM-C II anchor and justified against the production-code context (DPLR, LAURA, US3D land 2x to 3x on this
anchor with a 2x to 5x chemistry-model spread); the expectation is the ~3x band. The previously calibrated
surrogate result (the 12x-to-1.1x lever-1 history) SHALL remain recorded as history, not overwritten. If the
uncalibrated prediction misses the production band and the miss traces to the partial-equilibrium atom pool,
the designed response is the rated-dissociation promotion, not a re-calibration.

#### Scenario: The anchor is predicted, not reproduced
- **WHEN** the stagnation-line verification runs with the network
- **THEN** no calibrated equilibrium target enters the computation, the peak `n_e` is compared against the
  RAM-C II anchor, and the report labels the band as an uncalibrated prediction with its measured value

#### Scenario: The sheath-renewal A/B is re-run under a loss channel
- **WHEN** the network (with dissociative recombination) is run with and without explicit sheath renewal
- **THEN** both peak values are measured and recorded, the mode matching the stagnation-line closure is
  kept, and the decision with both numbers is written into the example's model labels, superseding the
  first A/B's record
