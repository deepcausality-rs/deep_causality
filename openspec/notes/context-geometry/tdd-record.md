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
| `utils::inertia` via `TangentSpacetime::update_metric_tensor` | Closed-form eigenvalues (`[[1, 2], [2, 1]]` → 3, −1; `[[0, 1], [1, 0]]` → ±1; `coupled_null_block(1)` → (5 ± √13)/2, (−3 ± √29)/2 and `coupled_null_block(−1)` → (3 ± √29)/2, (−5 ± √13)/2, from its restrictions to the vectors symmetric and antisymmetric under swapping t with x and y with z); diagonals; Sylvester's law of inertia over generated congruences `SᵀDS` with known `D` (Horn & Johnson 2013, Thm 4.5.8), and over exact power-of-two scalings; `numpy.linalg.eigvalsh` for the fixture tensors |
| Newtonian, Galilean, Minkowski intervals and distances | 3-4-12 triangle; `η = diag(−1, 1, 1, 1)` contractions in integers; the signature's own sign pattern as a quadratic form |
| `CausalSetSpacetime` | Sorkin 2003 p. 5 axioms (irreflexivity) |

## Corner-case rows

| Row | `GeoSpace` distance and bounds | Metric-tensor check (`inertia`) | Classical and Minkowski spacetimes |
|---|---|---|---|
| A empty | n/a: two points always | n/a: always 4×4 | n/a |
| B single | n/a | n/a | n/a |
| C coinciding | same point at longitudes 360° apart; distance to itself 0 beside known non-zero values | zero diagonal (all diagonal entries equal) | events away from the origin, so a sum cannot pass for a difference |
| D degenerate index | the poles, where longitude degenerates | the 2×2-block pivot (zero diagonal, generated congruences, a 2×2 pivot coupled to the other rows in `coupled_null_block`) | coordinate index `0 => t` on every type |
| E thresholds | latitude ±90° accepted, ±90.000001° refused, for `new`, `update`, `adjust` | pivot threshold: equivalent (D5) | simultaneity across time scales, exactly |
| F zero | equator, zero height | a zero eigenvalue (`diag(−1, 1, 1, 0)`), reported in the refusal | zero time and position |
| G negative | south pole, negative longitude | one, two and three negative eigenvalues | negative coordinates in the record tests |
| H exact boundary | exactly ±90° | n/a | n/a |
| I non-finite | NaN and ±∞ refused by `new`, `update`, `adjust` and the record restore | NaN off and on the diagonal, −∞: refused (this row found the first defect below) | time scales without a duration give NaN |
| J overflow reach | not covered: an altitude near the type's maximum overflows `ν + h` | scale invariance at λ = 10⁻³, 1, 10³, and at 2^±665 in `f64` and 2^±100 in `f32`, where the product of two entries overflows or underflows (the second defect below); not covered within about 2^10 of the maximum or in the subnormal range | n/a |
| K precision | IOGP example at `f32` (< 4 m), `f64`, `Float106` (< 2 mm) | the check, including the default tensor's, at `f32`, `Float106` | simultaneity, distance and interval at `f32`, `Float106` |

## Defects found

`TangentSpacetime::update_metric_tensor` accepted `diag(−∞, 1, 1, 1)`, counting −∞ as a negative
eigenvalue. A tensor with a non-finite entry is not a real symmetric matrix and has no signature;
it is now refused before the symmetry and signature checks
(`test_a_tensor_with_a_non_finite_entry_is_refused`). Row I found it.

`utils::inertia` formed products of two entries (`a_pp·a_qq`, `a_pq²`, `a_jp·a_pk`) and, in the 2×2
correction, products of three (`a_jp·a_qq·a_pk`, `a_jp·a_pq·a_qk`), which overflow or underflow while
the entries themselves are representable. The products of three fail first, near the cube root of
the range: at 2^400 in `f64` it refused 2 of the 52 Lorentzian tensors of
`tensors_of_known_inertia` and at 2^−400 one; `f32` refused 2 at 2^50, below the default tensor's
own c² ≈ 9·10¹⁶, and 50 at 2^70. At 2^665 in `f64` it refused 50 of the 52; at 2^−665 it refused
49 of them and accepted 6 of the 52 tensors of other signature. `f32` at 2^±100 gave the same
counts. Every update is now an
entry times a ratio of entries that the pivot choice bounds
(`test_the_signature_check_holds_where_products_of_entries_leave_the_range`). Review of the pull request found it;
row J had stopped at λ = 10³. The check still errs at the two ends of the range: within about 2^10
of the type's maximum, where one update can grow an entry to four times the largest entry and so
past the maximum (`f64` refused 1 of the 52 at 2^1014 and 34 at 2^1021), and in the subnormal
range, where the entries carry too few digits (`f64` refused 10 at 2^−1070). `Float106` is clean
from 2^−1000 to 2^1000.

## Defect audit

33 defects, D1–D5 and D7–D34 (D6 was never assigned), injected one at a time into the source, each
run against the context integration suite and restored byte for byte (script: scratchpad
`defect_audit.py`). The five `inertia.rs` defects, D5 and D7–D10, were injected again into the
current `inertia.rs`, with patterns quoting its text (script: scratchpad `defect_audit_inertia.py`);
their rows below give that result.

| Class | Defects | Result |
|---|---|---|
| 1 off-by-one / index | D25, D28, D29, D30 (coordinate index), D31 (grid position) | all killed |
| 2 flipped comparison | D5 (pivot threshold `diag + diag >= big` → `>`), D18 (latitude bound) | D18 killed; D5 survives, equivalent: at `diag + diag == big` every diagonal entry is at most half the largest entry `b`, which lies off the diagonal, so the 2×2 block's determinant is at most `b²/4 − b² < 0` |
| 3 inverted sign | D2 (ms multiply), D10 (1×1 Schur update `-=` → `+=`), D13 (`e²` sign), D34 (default `−c²`) | all killed; D10 first survived and was killed by `test_a_lorentzian_tensor_that_couples_t_and_x_is_accepted`, and now fails 4 tests |
| 4 changed constant | D1 (minute 61 s), D14 (Earth radius), D19 (90° → 180°) | all killed |
| 5 dropped normalisation | D15 (`(1 − e²)` dropped), D22 (seconds conversion skipped) | all killed |
| 6 loosened tolerance | D12 (only positive-definite refused) | killed |
| 7 skipped case | D7 (2×2 branch skipped) | killed: 2 tests fail and `test_the_signature_check_is_invariant_under_congruence` does not terminate |
| 8 removed guard | D4, D11, D16, D20, D21, D23, D32, D33 | all killed |
| 9 plausible neighbour | D3 (365-day year), D8 (2×2 block counted as two positive), D9 (zero count), D17, D24, D26, D27 | all killed; D8 fails 5 tests; D9 first survived and was killed by `test_the_refusal_reports_the_inertia_it_found` |

## Mutation testing

`cargo mutants -p deep_causality_context -f deep_causality_context/src/utils/inertia.rs -f
deep_causality_context/src/types/context_node_types/space_time/tangent_spacetime/metric_tensor.rs
-j 6` on the current code: 76 mutants, 63 caught, 4 missed, 1 timeout, 8 unviable. The timeout is
`inertia.rs:52` `||` → `&&` in the loop exit, which never terminates on a degenerate tensor. The
pivot threshold mutant `inertia.rs:56` `+` → `*` is caught: for entries below 1,
`diag * diag >= big` sends a tensor whose largest entry is on the diagonal to the 2×2 branch with
`p == q`, and `diag(−1, 1, 1, 1) · 10⁻³` is refused. The 4 missed are equivalent:

| Mutant | Why it cannot change behaviour |
|---|---|
| `inertia.rs:41` `>` → `>=` | Tie-break between equally large entries. In the 2×2 branch every largest entry is off the diagonal, since a diagonal one makes `diag + diag >= big` hold, and each is a valid pivot |
| `inertia.rs:46` `>` → `>=` | Tie-break between equally large diagonal entries, each at least half the largest entry when it is used. A zero diagonal entry is used only if `big` is zero, and the loop returns first |
| `inertia.rs:58` `>` → `>=` | The pivot is at least half the largest entry, which is not zero, so the pivot is never zero |
| `metric_tensor.rs:27` `(i + 1)..4` → `(i * 1)..4` | Adds the pairs `(i, i)`; every entry is finite by then and equals itself |

A pass over all the added and edited geometry files ran before the change to `inertia.rs` recorded
under Defects found: 455 mutants, 319 caught, 46 missed, 1 timeout, 89 unviable. The missed ones
exposed two gaps, both closed: every distance and interval test placed one event at the origin,
where `x − 0 = x + 0` (`…away_from_the_origin` tests), and no accepted tensor coupled the remaining
rows to a 2×2 pivot (`test_the_signature_check_is_invariant_under_congruence`, which builds `SᵀDS`
with known `D`). Its survivors outside the two files above are three equivalent mutants, missed
again by a run over their files on the current code:

| Mutant | Why it cannot change behaviour |
|---|---|
| `NoSpace::id`, `NoTime::id` → `Default::default()` | Both return 0 already |
| `NoTime::time_scale` → `Default::default()` | `TimeScale::default()` is `NoScale`, the value returned |

Not done: `.cargo/mutants.toml` entries for these 7 equivalent mutants, each checked with `comm` to
match one mutant.
