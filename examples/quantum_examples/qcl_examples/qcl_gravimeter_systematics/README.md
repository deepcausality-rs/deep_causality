# Gravimeter systematics: which effect explains the offset?

After the Earth-tide correction, an atom gravimeter reads −5 µGal against a trusted reference.
Seven systematics could produce that offset, and a passive run cannot tell them apart. This
example plans the cheapest experiments that do, runs them as a campaign that stops when the
evidence decides, and shows three ways the decision fails when the model is incomplete.

```bash
cargo run --release -p quantum_examples --example qcl_gravimeter_systematics
```

The instrument is the cold-atom gravimeter of Louchet-Chauvet et al., New J. Phys. 13, 065025
(2011): `T` = 70 ms and a repetition rate of about 3 Hz. Its four-configuration protocol reaches
70 µGal at 1 s, which the paper puts at a factor √10 above one configuration, so one configuration
reads 22.1 µGal/√Hz. The noise stays white to 5000 s. The 10 mG bias field and the π/2 pulse of
about 10 µs come from Farah et al., on the same instrument. The papers on it do not state a
contrast, so the example takes the 40 % of Ménoret et al.'s rubidium gravimeter
(arXiv:1809.04908); sizing does not depend on it. Two sets of values remain placeholders, marked
in `constants.rs`: the setup times, which no paper read here reports, and the tide, which is the
site's.

## The candidates

Each candidate is a mechanism: a phase channel on the interferometer's read-out, with the fringe
contrast as a dephasing channel. A response model computes what each configuration of the
instrument does to it.

| Candidate | Response | Source |
|---|---|---|
| H1 Coriolis | flips under a 180° turn about vertical | Louchet-Chauvet et al. |
| H2 quadratic Zeeman | flips when the wave vector reverses, being independent of its sign; scales with coil current `s` as `a s² + b s + c` | Louchet-Chauvet et al., eq. 2; Hu et al., arXiv:1805.05159, eqs. 7 to 9 and Fig. 6 |
| H3 tilt | reads `−g(1 − cos θ)`, so only low; a deliberate tilt adds to `θ` | |
| H4 mirror vibration | removed by the accelerometer correction | |
| H5 wavefront | vanishes at zero atom temperature | Karcher et al., arXiv:1804.04909 |
| H6 two-photon light shift | linear in the Rabi frequency at constant pulse area; its co-propagating third, 11 of 33 mrad, follows the field | Gauguet et al., arXiv:0809.0149 |
| H7 clipping | flips under a turn; moves 14.2 µGal per mm of east-west cloud displacement | Farah et al., arXiv:1406.5998 |

Every candidate is calibrated to produce the offset in the passive configuration. A tilt only reads
low, so for a positive offset its closest size is vertical, which produces none.

The Zeeman shares are GAIN's. Hu et al. mapped GAIN's field at 13 mA and at 6.5 mA (Fig. 6(a) and
(b)); `deep_causality_quantum/papers/digitised/digitise_hu2017_fig6.py` digitises both maps and
integrates eq. 9 along the fountain trajectory through the printed pulse heights. The nominal
map gives 2.01 µGal against the printed 2.04 µGal. Of that bias, the cross term `b` carries
1.005, the coil's own term `a` −0.038 and the background's `c` 0.033, so halving the current
roughly halves the bias.

## The experiments

E1 reverses the wave vector, E2 turns the sensor head 180°, E3 applies the accelerometer
correction, and E4 tilts the head by 1.5 mrad, the end of Ménoret et al.'s tilt scan. E5 halves
the coil current, as Hu et al. did, and E6 extrapolates the atom temperature to zero. E7 halves the
Rabi frequency, as Louchet-Chauvet et al.'s protocol does, and E8 displaces the cloud by 1 mm,
inside the ±1.6 mm Farah et al. scanned. Each
is an instrument context, the configuration with one knob turned, holding the Earth tide recorded
at the time the experiment runs. E6 costs 20,000 s of setup. That figure is derived from Karcher et
al.'s ultracold operation, which puts each temperature point at 10⁴ s or more.

## What happens

Each run observes the passive configuration first through `baseline_with`, which refuses a
candidate whose prediction the observation contradicts. It then plans in instrument time and runs
a sequential campaign. After every step the campaign's contexts move to the new time, and a tide
that moved a planned prediction forces a re-plan. Every move re-plans, because draws sized at an
earlier tide can leave a pair a fraction of a bit short of the floor, and the campaign then buys
another experiment to close the pair, even the 20,000 s temperature scan.

For each of the seven systematics as the cause, the campaign names it. The plan is E1, E2, E3,
E4, E7 and E8 at 4956 s, and it leaves the wavefront to elimination. No planned experiment
singles it out, so it survives only when every other candidate is rejected. The campaign spends
166 s when the Zeeman shift is the cause and 4953 s when the wavefront or the light shift is.
Every campaign spends no more than its plan.

Three runs show the model's limits:

- **A positive offset.** With the light shift producing +5 µGal, the baseline refuses the tilt:
  it predicts 76.43 µGal with the tide and the instrument reads 82.54. The campaign then names
  the light shift.
- **No tide in the contexts.** The tide's amplitude here is 80 µGal. Without it the baseline
  refuses every candidate, since each predicts −5 µGal and the instrument reads 72.41.
- **Clipping left off the list.** With clipping producing the offset and absent from the
  candidates, nothing needs the cloud displacement, so the plan drops it. The campaign
  misattributes the offset to Coriolis, which a turn flips the same way. Farah et al. met the
  same confusion between a clipping shift and Coriolis.

Predictions, calibrations and plans are computed; observations are sampled. The example exits
nonzero if a check fails. The campaign must name each of the seven causes and spend no more than
its plan, and each plan must leave the wavefront to elimination. The baseline must refuse only the
tilt at a positive offset and every candidate without the tide.

## Not built

The correction each survivor implies and the verification run after it are not built. Neither is
the wavefront's dependence on atom temperature between zero and the operating 2 µK: the example
reads it at those two points only.

## Precision

The working scalar is `Float106`, and the run takes about 5 s in release.
