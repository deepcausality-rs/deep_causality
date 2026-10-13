# Gentamicin Dosing from Two Serum Levels

This example individualises a gentamicin regimen with the Sawchuk–Zaske method: two measured serum
levels give the patient's elimination rate, and one first-order law reconstructs the peak and the
trough that nobody sampled.

```bash
cargo run -p medicine_examples --example gentamicin_dosing
```

## What the run prints

```text
Regimen: one dose every 8 h, infused 08:00 to 08:30
Measured: 7.00 mg/L at 09:30 and 3.50 mg/L at 15:30

1. Elimination rate:  k = 0.1155 /h, half-life 6.0 h
2. Peak (backward):   7.86 mg/L at 08:30   target 6-10 mg/L: in range
3. Trough (forward):  3.30 mg/L at 16:00   target at most 2 mg/L: OUT OF RANGE
4. Interval:          the peak falls to 1 mg/L after 18.3 h; dose every 24 h

Check: the law run forward from the peak returns 7.00 mg/L at 09:30 (difference 0.0e0 mg/L)
```

The peak is effective, but a 6-hour half-life leaves 3.3 mg/L in the blood when the next dose
starts, above the 2 mg/L that limits accumulation. Stretching the interval to 24 hours lets the
peak fall to 1 mg/L first.

## The causal chain

```rust
let report = CausalFlow::value(MEASURED)
    .try_step(elimination_rate) // the patient's elimination rate; fails unless the levels decay
    .map(peak_at_end_of_infusion) // backward in time, to 08:30
    .map(trough_before_next_dose) // forward in time, to 16:00
    .map(dosing_interval) // the verdict and the interval
    .map(round_trip) // forward again from the peak: must return the 09:30 level
    .finish()?;
```

Each step needs the result of the one before it, so the causal order runs one way. Physical time
does not: the peak step runs `concentration_after(c, k, hours)` with a negative `hours`, back from
09:30 to 08:30, and the trough step runs the same law forward to 16:00. First-order elimination,
`c · exp(-k · hours)`, is exact in either direction, which the last step confirms.

## The pharmacology

* **Elimination rate.** Two levels on one decay give `k = ln(c1 / c2) / (t2 - t1)`.
* **Peak.** The first level, run back to the end of the infusion.
* **Trough.** The peak, run forward to the start of the next dose.
* **Interval.** `tau = t_infusion + ln(peak / 1 mg/L) / k`, rounded up to the next interval a ward
  schedules (8, 12, 24, 36 or 48 hours).

The targets, a peak of 6–10 mg/L and a trough of at most 2 mg/L, are those for conventional
multiple-daily dosing. Many hospitals now give aminoglycosides once daily; Begg et al. review
both regimens and the monitoring each needs.

## Precision

`FloatType` sets the working precision. The program runs unchanged at `f32`, `f64` and
`Float106`; at `f32` the round trip differs by 4.8e-7 mg/L, at `f64` and `Float106` by none.

## References

* Sawchuk RJ, Zaske DE, Cipolle RJ, Wargin WA, Strate RG. Kinetic model for gentamicin dosing with
  the use of individual patient parameters. *Clin Pharmacol Ther.* 1977;21(3):362–369.
  [PubMed 837654](https://pubmed.ncbi.nlm.nih.gov/837654/)
* Begg EJ, Barclay ML, Kirkpatrick CJM. The therapeutic monitoring of antimicrobial agents.
  *Br J Clin Pharmacol.* 1999;47(1):23–30.
  [PMC2014209](https://pmc.ncbi.nlm.nih.gov/articles/PMC2014209/)
