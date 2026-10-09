# Gradiometer crosstalk: is the correlation between the clouds benign?

A gravity gradiometer runs two atom clouds off one Raman laser and one mirror, and their read-outs
fluctuate together. If the shared phase is the whole story, differential extraction removes it and
the gradient is unbiased. If light from one cloud reaches the other's detection signal, or the
platform rotates, the same correlation biases the gradient. This example decides which, and says
what the answer costs in Eötvös.

```bash
cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk
```

The instrument is the dual-cloud gradiometer of Sorrentino et al., Phys. Rev. A 89, 023607 (2014),
arXiv:1312.3741: `T` = 160 ms, contrast 0.45, a 1.9 s cycle, a 30 cm baseline, and a sensitivity of
3 × 10⁻⁹ g/√Hz that holds to 8000 s. The leak, the dark background and the setup times are
placeholders. No paper read for this example measures light leaking between the clouds, so a
lab's own measurement replaces them; `constants.rs` marks each one.

## The candidates

Every world shares one model of a shot. The common phase takes four equally spaced values. Each
cloud's atoms read excited with the cloud's fringe, Sorrentino et al.'s eq. 1. Each detection signal
reads excited when its own atoms are excited, when background light fires a dark cloud's signal,
or when light leaking in from the other cloud fires it. The atoms and the detection signals are
separate nodes, so a leak runs from one cloud into the other's signal and never forms a cycle.

| Candidate | Mechanism | Gradient |
|---|---|---|
| H1 | Cloud A's fluorescence reaches B's detection signal | biased |
| H2 | Cloud B's fluorescence reaches A's detection signal | biased |
| H3 | The shared phase alone | unbiased |
| H4 | Rotation adds `2 b k_eff T² Ω²` to the differential phase (Lellouch & Holynski, arXiv:2504.11119) | biased, always positive |

A leak carries the other cloud's whole population, so it depends on the shared phase through that
cloud's fringe. Each candidate is fitted to the passive read-out through its two fringe centres and
its differential phase. That is three numbers for the three a joint of two binary read-outs carries,
so the fit is exact and the passive read-out cannot tell the candidates apart. The instrument
operates a quarter fringe from the gradient's own phase, where the ellipse is closest to a circle.

## The experiments

| Experiment | Configuration | Reads | Separates |
|---|---|---|---|
| E0 passive | none | both signals excited | nothing |
| E1 brighten A | A's atoms all bright, B's dark with no pulses | B's signal excited | H1: `b + ε(1 − b)` against `b` |
| E2 brighten B | the mirror image | A's signal excited | H2 |
| E3 rotation step | 1 mrad/s about a horizontal axis | the two signals disagreeing | H4 |

The disagreement of the two signals depends on the differential phase alone, because the shared
phase averages out of it. Near the quarter-fringe operating point it moves most for a given phase
step. At Earth's horizontal rate the centrifugal phase is 6.9 × 10⁻⁴ rad. E3 rotates the sensor head
deliberately, and its step adds 0.247 rad.

## What happens

For each of five worlds the observations are drawn from, one per candidate and a two-way leak, the
example:

1. fits the four candidates to that world's passive read-out;
2. screens them with `validate`: each is a normalised process, Markov, and free of a C₃;
3. plans in instrument time with `design_with`. Each chosen experiment takes the fewest effective
   draws that reach the 5-bit floor, and the plan is E1, E2 and E3 at about 722 s. The setup times
   dominate: E1 needs 340 draws and E3 about 60,000, at 3 × 10⁻⁵ s each;
4. runs the static plan and adjudicates all three readings together;
5. runs a sequential campaign that stops as soon as one candidate holds and separates;
6. branches on the survivor to its corrective action, and computes the gradient bias it implies.
   A flow reads the instrument's differential phase in the survivor's world, alternates the context
   to the same world without the mechanism, and reads it again.

In the four candidate worlds the campaign and the static plan name the cause the observations came
from. The campaign spends 60 s when the leak runs from A to B and 120 s when it runs from B to A.
It runs all three experiments for the shared phase and for rotation. A 6 % leak shifts the
instrument's reading by −0.044 rad, −356 E. Rotation at Earth's horizontal rate shifts it by
+6.9 × 10⁻⁴ rad, +5.5 E.

The two-way leak is no candidate. The static plan reports it outside the model, because E2 shows a
leak that H1 does not predict. The sequential campaign stops at H1 after E1. A survivor ends the
campaign before the experiment that would contradict it runs, so a cause outside the candidate list
is caught only by an experiment the campaign has a reason to run.

Predictions, fits and plans are computed; the observations are sampled. The example exits nonzero
if a check fails. Both the campaign and the static plan must name each candidate world's cause, and
the static plan must report the two-way leak outside the model. Every campaign must spend no more
than the plan, and every plan must be E1, E2 and E3 and cover every pair.

## Precision

The working scalar is `Float106`, and the run takes about 16 s in release. At `f64` it takes about
2 s and prints the same output.
