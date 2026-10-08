# Sensing verification harnesses

Each harness takes one published paper on atom interferometry or crosstalk and reproduces its
claims through QCL: its attributions through the control stage, its experiment choices through the
planner, and its printed numbers through the paper's own equations. A harness prints every check as
`PASS` or `FAIL` with the numbers behind it and exits with an error naming the checks that failed.

| Harness | Source | Checks |
|---|---|---|
| `verification_v1_wavefront` | Karcher et al., New J. Phys. 20, 113041 (2018), arXiv:1804.04909 | 9 |
| `verification_v2_clipping` | Farah et al., Phys. Rev. A 90, 023606 (2014), arXiv:1406.5998 | 5 |
| `verification_v3_transverse_motion` | Louchet-Chauvet et al., New J. Phys. 13, 065025 (2011) | 8 |
| `verification_v4_crosstalk` | Sarovar et al., Quantum 4, 321 (2020), arXiv:1908.09855 | 8 |
| `verification_v5_light_shift` | Gauguet et al., Phys. Rev. A 78, 043615 (2008), arXiv:0809.0149 | 8 |
| `verification_v6_time_model` | Ménoret et al., Sci. Rep. 8, 12300 (2018), arXiv:1809.04908 | 5 |
| `verification_v7_gradiometer_sweep` | Sorrentino et al., Phys. Rev. A 89, 023607 (2014), arXiv:1312.3741 | 4 |
| `verification_v10_zeeman_field_map` | Hu et al., Phys. Rev. A 96, 033414 (2017), arXiv:1805.05159 | 6 |

The papers' PDFs are in `deep_causality_quantum/papers/`.

## Running

From the repository root:

```
cargo run --release -p deep_causality_quantum --features qcm --example verification_v1_wavefront
bazel run -c opt //deep_causality_quantum:verification_v1_wavefront
```

The harnesses read data from `deep_causality_quantum/papers/`, which the published crate excludes,
so they run from a checkout of the repository.

## Conventions

- A published value and its standard error enter as effective draws on an interferometer fringe
  (`EvidenceSource::Published`). Each candidate is a mechanism whose prediction is a phase channel.
  `common/attribution.rs` runs the control stage over all the measurements and adjudicates them
  together.
- A candidate *holds* when every comparison accepts it. It *survives* when the adjudication names
  it alone. When no candidate holds, the evidence is *outside the model*.
- QCL's comparisons accept within three standard errors, five in V4. V4 makes up to 1,800
  comparisons a scenario, and at five the chance that the generating model fails one by chance
  stays below 10⁻³. A pair of candidates is separated at five bits. V6 compares a predicted scatter
  with a measured one within two standard errors of the measurement; V10 compares within the
  digitisation's uncertainty.
- A prediction printed without an uncertainty carries half a unit of its last digit, added to the
  measurement's standard error in quadrature.
- Where a source contradicts itself, the check states the contradiction and passes when the
  numbers confirm it.

## Data

Every file below regenerates byte for byte from the paper's PDF with its script; the regenerated
data regenerate byte for byte from their script.

| File | Source | Script |
|---|---|---|
| `papers/digitised/gauguet2008_fig7.csv` | Gauguet Fig. 7, vector paths | `extract_gauguet2008_fig7.py` |
| `papers/digitised/karcher2018_fig2.csv`, `karcher2018_fig4.csv` | Karcher Figs. 2 and 4, vector paths | `extract_karcher2018_figs.py` |
| `papers/digitised/louchet2011_fig9_points.csv`, `louchet2011_fig9_fits.csv` | Louchet-Chauvet Fig. 9, vector paths | `extract_louchet2011_fig9.py` |
| `papers/digitised/hu2017_fig6.csv` | Hu Fig. 6(a) and (b), raster | `digitise_hu2017_fig6.py` |
| `papers/regenerated/sarovar2020_*.csv` | Sarovar §7, simulated from the published models | `regenerate_sarovar2020.py` |

Each script's docstring gives the commands that run it, and its calibration or its models. The
Sarovar regeneration ran under Python 3.12 with numpy 2.5.3, scipy 1.18.1, networkx 3.7,
pyGSTi 0.10.2, pcalg 0.2.2 and gsq 0.1.6, seed 20200907.

## What the harnesses find

**V1, Karcher.** QCL attributes the temperature scan to the wavefront and rejects Coriolis of a
centred cloud, the one-photon light shift and a wavefront linear in temperature. The linear
extrapolation from above 2 µK lands 4.4 standard errors from Table I's −56(13) nm/s². Refitting
Fig. 2 with Fig. 4's responses reproduces Table I's extrapolations within one standard error, but
not its correlation coefficients. The figures disagree in two ways. Fig. 4 computes each of Fig. 2's
temperatures above the lowest at 1.5 times its value, within 1.5 %. And the drawn fit has
χ² = 17.7, below the 36.7 that is the best any combination of Fig. 4's five responses at those
temperatures reaches. The bound on interactions, under
1 nm/s² per thousand atoms, is (7 + 12)/20, one standard error. At three, interactions remain a
possible cause of the rise at 650 nK. The planner needs the turn, the atom-number change and a Rabi
change at constant pulse area. The paper ran the first two, so a light shift with the wavefront's
response to temperature remains unseparated. Fig. 2's axis label reads mm/s² and means nm/s².

**V2, Farah.** With Coriolis and the wavefront alone, the +46(2) µGal per unit of imbalance is
outside the model. With clipping added, only clipping with Coriolis holds. A 180° turn does not
separate Coriolis from clipping; a 1 mm cloud displacement does. §VII converts velocity at 10.94 and 10.96 µGal/(mm/s)
against the 9.72 of §VIII. The printed "kHz/mm" reads as Hz/mm.

**V3, Louchet-Chauvet.** Table 1's corrections sum to the printed totals. The device's 5.1 µGal is
not the quadrature of its rows, 4.16. Eq. (4) as printed doubles the light shift. Only the turn
separates Coriolis from the wavefront, and its 0.5 ± 0.4 µGal does not resolve a Coriolis shift
from none. The temperature scan leaves the wavefront ambiguous: the fits that hold extrapolate from
−4.50 to 4.82 µGal at 0 K, where the text gives −4 to 15.

**V4, Sarovar.** In every scenario of the paper the survivor is the generating model, and the pairs
it implies are the published edges. The crosstalk-free control establishes none. QCL separates
detection crosstalk from a shared bath, though both imply only R0–R1. Without the ZZ candidate,
the ZZ data leave no survivor. On the written draw, pyGSTi's PC misses the published edges for the
one-way, detection and ladder scenarios. Over ten further draws it finds them in 9, 9 and 8, and the
ZZ edges in 7. Its edge weights halve the sum eq. (12) prints (pyGSTi 0.10.2,
`extras/crosstalk/core.py`).

**V5, Gauguet.** Fig. 7's error bars alone reject every candidate. With the printed ±10 %
reproducibility, the light shift of eq. (7), 33 mrad, survives; eq. (8)'s counter-propagating part
alone and quadratic Zeeman do not. The same holds with the pooled scatter of the repeated ratios,
2.74 mrad, in place of ±10 %. The fit gives 32 mrad. The three measurements at ratio ½ spread
±16 %, beyond the printed ±10 %. Eq. (8) gives 21.7 mrad from the printed Doppler shifts and
20.0 mrad from the pulses' timing.

**V6, Ménoret.** The white-noise time model predicts the 10-minute and 1-hour scatter at Talence,
24.5 and 10.0 nm/s² against 25.2 and 10.7, and does not explain the 1-day scatter at Larzac, 2.55
against 9.4. The planner sizes a separation inside the white-noise range and refuses one past it.

**V7, Sorrentino.** The mirror tilt's slope attributes to Coriolis, eq. (10) giving −35.2 mrad/mrad
from the printed inputs against the printed −34. The magnetic pulse's slope attributes to the
quadratic Zeeman shift of eq. (12). No quantified candidate explains the MOT ratio's slope, which leaves the temperature,
as the paper concludes. Ranked by slope times daily fluctuation, Table I puts Raman total intensity
third; the text's list of dominant parameters omits it.

**V10, Hu.** A fountain through the printed first- and second-pulse heights, 45.1 and 85.7 cm,
reproduces the printed third pulse, 60.0 cm, and apex, 86.1 cm. The coefficient γ₂ works in
Hz/µT², not the printed Hz/nT². The nominal field map reproduces the printed 2.04 µGal bias as
2.010, within the digitisation's 0.040. The cross term between the coil's field and the background
carries the bias, and QCL's prediction carries it at both currents.
