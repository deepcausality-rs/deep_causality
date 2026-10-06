<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->
# Context geometry: TDD record

The context crate's geometry was brought to the textbook definitions it names (see
`deep_causality_context/README.md`, section Geometry). The implementation existed before the suite,
so the unified-math TDD protocol (`openspec/specs/unified-math-tdd-protocol/spec.md`) applied in
modified form: phases 1 and 2 could not precede the code, and the protocol's later phases ran in
full: independent expected values, the corner-case rows, error variants, the defect audit and
mutation testing.

## Independent sources of expected values

| Kernel | Source of the expected values |
|---|---|
| `GeoSpace::distance` | IOGP 373-7-2 §2.2.1 worked example (geocentric coordinates to the millimetre); WGS 84 `a`, `b` (NGA.STND.0036 Tables 3.1, 3.5): pole to pole `2b`, equatorial antipodes `2a`, pole to equator `√(a² + b²)`, equatorial chord `2a·sin(1.5°)` |
| `utils::seconds` | Exact unit products, Gregorian mean year 365.2425 d = 31 556 952 s |
| `utils::inertia` via `TangentSpacetime::update_metric_tensor` | Closed-form eigenvalues (`[[1, 2], [2, 1]]` → 3, −1; `[[0, 1], [1, 0]]` → ±1); diagonals; Sylvester's law of inertia over generated congruences `SᵀDS` with known `D` (Horn & Johnson 2013, Thm 4.5.8); `numpy.linalg.eigvalsh` for the fixture tensors |
| Newtonian, Galilean, Minkowski intervals and distances | 3-4-12 triangle; `η = diag(−1, 1, 1, 1)` contractions in integers; the signature's own sign pattern as a quadratic form |
| `CausalSetSpacetime` | Sorkin 2003 p. 5 axioms (irreflexivity) |

## Corner-case rows

| Row | `GeoSpace` distance and bounds | Metric-tensor check (`inertia`) | Classical and Minkowski spacetimes |
|---|---|---|---|
| A empty | n/a: two points always | n/a: always 4×4 | n/a |
| B single | n/a | n/a | n/a |
| C coinciding | same point at longitudes 360° apart; distance to itself 0 beside known non-zero values | zero diagonal (all diagonal entries equal) | events away from the origin, so a sum cannot pass for a difference |
| D degenerate index | the poles, where longitude degenerates | the 2×2-block pivot (zero diagonal, generated congruences) | coordinate index `0 => t` on every type |
| E thresholds | latitude ±90° accepted, ±90.000001° refused, for `new`, `update`, `adjust` | pivot threshold: equivalent (D5) | simultaneity across time scales, exactly |
| F zero | equator, zero height | a zero eigenvalue (`diag(−1, 1, 1, 0)`), reported in the refusal | zero time and position |
| G negative | south pole, negative longitude | one, two and three negative eigenvalues | negative coordinates in the record tests |
| H exact boundary | exactly ±90° | n/a | n/a |
| I non-finite | NaN and ±∞ refused by `new`, `update`, `adjust` and the record restore | NaN off and on the diagonal, −∞: refused (this row found the defect below) | time scales without a duration give NaN |
| J overflow reach | not covered: an altitude near the type's maximum overflows `ν + h` | scale invariance at λ = 10⁻³, 1, 10³ | n/a |
| K precision | IOGP example at `f32` (< 4 m), `f64`, `Float106` (< 2 mm) | the check at `f32`, `Float106` | simultaneity, distance and interval at `f32`, `Float106` |

## Defect found by the corner cases

`TangentSpacetime::update_metric_tensor` accepted `diag(−∞, 1, 1, 1)`, counting −∞ as a negative
eigenvalue. A tensor with a non-finite entry is not a real symmetric matrix and has no signature;
it is now refused before the symmetry and signature checks
(`test_a_tensor_with_a_non_finite_entry_is_refused`).

## Defect audit

34 defects injected one at a time into the source, each run against the context integration suite
and restored byte for byte (script: scratchpad `defect_audit.py`).

| Class | Defects | Result |
|---|---|---|
| 1 off-by-one / index | D25, D28, D29, D30 (coordinate index), D31 (grid position) | all killed |
| 2 flipped comparison | D5 (pivot threshold `>=` → `>`), D18 (latitude bound) | D18 killed; D5 survives, equivalent: at the threshold the 2×2 determinant is still negative |
| 3 inverted sign | D2 (ms multiply), D10 (Schur update `-=` → `+=`), D13 (`e²` sign), D34 (default `−c²`) | all killed; D10 first survived and was killed by `test_a_lorentzian_tensor_that_couples_t_and_x_is_accepted` |
| 4 changed constant | D1 (minute 61 s), D14 (Earth radius), D19 (90° → 180°) | all killed |
| 5 dropped normalisation | D15 (`(1 − e²)` dropped), D22 (seconds conversion skipped) | all killed |
| 6 loosened tolerance | D12 (only positive-definite refused) | killed |
| 7 skipped case | D7 (2×2 branch skipped) | killed |
| 8 removed guard | D4, D11, D16, D20, D21, D23, D32, D33 | all killed |
| 9 plausible neighbour | D3 (365-day year), D8, D9 (zero count), D17, D24, D26, D27 | all killed; D9 first survived and was killed by `test_the_refusal_reports_the_inertia_it_found` |

## Mutation testing

`cargo mutants -p deep_causality_context -j 8` over the added and edited geometry files: 455
mutants, 319 caught, 46 missed, 1 timeout, 89 unviable on the first pass. The missed ones exposed
two gaps, both closed: every distance and interval test placed one event at the origin, where
`x − 0 = x + 0` (`…away_from_the_origin` tests), and no accepted tensor coupled the remaining rows
to a 2×2 pivot (`test_the_signature_check_is_invariant_under_congruence`, which builds `SᵀDS` with
known `D`). Rerunning the 51 missed and timed-out mutants: 42 caught, 1 timeout (`inertia.rs`
`||` → `&&` in the loop exit, which never terminates), 8 missed. The 8 are equivalent:

| Mutant | Why it cannot change behaviour |
|---|---|
| `inertia.rs:27` `+` → `*` in the pivot threshold | Any threshold in (0, 1] of the largest entry is correct: in the 2×2 branch both diagonal entries are below the largest entry `b`, so the determinant is below `b² − b² = 0` |
| `inertia.rs:36`, `:41` `>` → `>=` | Tie-breaks between equally large entries; any choice is a valid pivot |
| `inertia.rs:53` `>` → `>=` | The pivot is at least half the largest entry, never zero |
| `metric_tensor.rs` `(i + 1)..4` → `(i * 1)..4` | Adds the pairs `(i, i)`; every entry is finite by then and equals itself |
| `NoSpace::id`, `NoTime::id` → `Default::default()` | Both return 0 already |
| `NoTime::time_scale` → `Default::default()` | `TimeScale::default()` is `NoScale`, the value returned |

Not done: the `.cargo/mutants.toml` entries for these 8 with the `comm` check. `replace > with >=
in inertia` matches exactly the three tie-break mutants; the `inertia.rs:27` mutant needs a line
anchor, or the threshold written as `diag + diag >= big`, which removes it.
