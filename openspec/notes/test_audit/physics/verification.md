<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Verification of the physics test audit

The audit's numbers reproduce exactly and its named findings are real. Its one factual error is in
the "Done" table, and the classes it declared healthy hold the largest remaining gap: 402 tests
assert that a kernel refused without pinning which of thirteen refusals it gave.

Scope is `deep_causality_physics` alone: 1766 tests, 190 files, 28 230 lines. Baseline green.

## 1. The counts reproduce

Both scanners were rerun against the current tree. Every published figure came back identical.

| Class | Published | Rerun |
|---|---:|---:|
| single-input / single-case | 1585 / 1603 | 1585 / 1603 |
| cherry-picked / unproven-literal-oracle | 632 / 545 | 632 / 545 |
| tautology / answer-blind | 116 / 119 | 116 / 119 |
| circular / derived-oracle | 52 / 54 | 52 / 54 |
| no-observation | 5 | 5 |
| conditional-only | 5 | 5 |

## 2. A scanner defect, fixed

`mask_non_code` blanked the character after a backslash inside a string literal. Where that
character is a newline, which is Rust's line continuation, the mask lost a line and every line
number after it shifted.

```rust
"AS E14 2016 07 01 00 00 00.000000  2  0.000123456789\n\
 AS E18 2016 07 01 00 05 00.000000  2  0.5\n\
```

The masker now keeps the newline. Measured effect: the scanner crashed outright on
`deep_causality_file`, and 29 files across 12 crates had shifted line numbers, one of them in
physics. Rerunning physics after the fix moved no count, so the audit's conclusions stand on it.

## 3. The proof holds

The audit's falsifiable claim was that `weber_number` could be replaced by a constant zero and the
suite would pass. Replacing its body with `PropagatingEffect::pure(R::zero())` now fails two tests:

```
test_the_dimensionless_wrappers_carry_the_value_not_merely_an_ok
  weber_number: wrapper gave 0.00000000000000000e0, oracle 5.55555555555555571e1
test_weber_number_wrapper_error_path
```

The source file was restored; `git status` is clean apart from the scanner fix.

## 4. Ten named findings, all confirmed

Every no-observation and conditional-only site was read. All ten are real.

`test_energy_momentum_tensor_em_wrapper_dimension_error` is the sharpest: its name promises a
dimension check and its body is `let _ = result.is_ok() || result.is_err();`.

`test_proper_time_si` is the most revealing. Its assertion sits inside `if let Ok(tau) = ...`, and
the file carries a second test, `test_proper_time_si_runs_with_square_connection`, whose comment
explains that a Lorentz connection has shape `[N, 4, 6]` so `proper_time` cannot return `Ok`. The
author knew the first test's body never runs and wrote a second rather than repairing the first.

## 5. One correction to the audit

The "Done" table reads:

> `kernels/fluids/wrappers_tests.rs` | 119 tests, 80 tautology | the 18 dimensionless wrappers
> assert the carried value against the oracle, over every table row

The repair exists, but not in that shape. One consolidated test,
`test_the_dimensionless_wrappers_carry_the_value_not_merely_an_ok`, drives all eighteen wrappers
over roughly a hundred oracle rows. The eighteen `test_*_wrapper` functions were not rewritten.
Seventeen of the eighteen still read:

```rust
#[test]
fn test_weber_number_wrapper() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.001).unwrap();
    assert!(weber_number(&rho, &u, &l, 0.072_f64).is_ok());
}
```

Across that file, 83 of 120 tests inspect no value. The section header calls them "smoke tests".
They are now redundant with the consolidated test and should go, which also explains the file's
persistent answer-blind count of 38.

`kernels/fluids/dimensionless_tests.rs` is repaired as described: oracle tables, cited definitions,
three cross-kernel identities.

## 6. What the scanners cannot see

Six classes, counted over all 1766 tests. The scanners model assertion syntax, so an assertion that
is well-formed and cannot fail reads to them as a real observation.

| Class | Count |
|---|---:|
| U6 refusal asserted, error variant unpinned | 402 |
| U1 unfalsifiable `!is_empty()` | 12 |
| U5 comment admits the test checks nothing | 7 |
| U2 `is_ok() \|\| is_err()` | 4 |
| U3 `is_finite()` as the only observation | 4 |
| U4 compares magnitudes, drops the sign | 1 |

### U6 is the large one

`PhysicsError` has thirteen variants. `assert!(result.is_err())` accepts all thirteen, so a kernel
that rejects valid input for an unrelated reason passes a test named for the reason it did not
give. 402 tests do this; 85 pin the variant or the message.

The audit removed this class deliberately. Its calibration note reads: "`assert!(x.is_err())` is a
real claim. An error-path test asserting only a refusal is doing its job." That took the count from
535 to 169. The claim is real but weak, and the gap it leaves is the largest single one left.

### U1 is the sharpest

Four of the twelve are the only value assertion in a kernel's `_valid` test:

```rust
let result = lorentz_force_kernel(&j, &b);
assert!(result.is_ok());
let force = result.unwrap();
assert!(!force.data().is_empty());
```

`force` is a `CausalMultiVector` over `Metric::Euclidean(3)`, so `data()` has eight entries for
every input and every implementation. The assertion cannot fail. The scanner read
`!force.data().is_empty()` as an observation of the answer and classified the test `single-case`
only. The same shape covers `poynting_vector_kernel`, `torque_kernel` and
`angular_momentum_kernel`.

### The connection term of the GR kernels is pinned by nothing

`parallel_transport_kernel` and `geodesic_integrator_kernel` are called by eleven tests. Every
finite fixture sets the Christoffel symbol to zero, where the whole connection term vanishes and
both kernels reduce to the identity and to straight-line motion. The two nonzero fixtures are
`1e150` and `NaN`, and both assert only `is_err()`.

A wrong sign, a transposed index pair or a wrong factor on `Γ^μ_αβ u^α u^β` survives all eleven.

The oracle already exists in the crate. `theories/general_relativity/metrics_tests.rs` builds a
Schwarzschild connection with `schwarzschild_christoffel_at` and checks four components against
closed forms. It is never fed to either consumer.

### Two classes are clean

Stated because a zero can mean a miscalibrated control, so both were calibrated first.

- **Tolerances never admit zero.** 324 assertions compare against a literal with an absolute
  tolerance. The loosest tolerance is 11.7% of its expected value; none permits the answer to be 0.
- **No duplicated tests.** All 1766 bodies are distinct, and distinct after numeric literals are
  replaced by a placeholder. Injecting one duplicate proved the detector fires.

## 7. Mutation evidence

`cargo mutants` over `kernels/em/fields.rs` and `forces.rs`: 72 mutants, 42 caught, 7 missed,
23 unviable.

```
fields.rs:167:19  replace * with + in proca_equation_kernel        (m*m becomes m+m)
fields.rs:177:21  replace < with <= in proca_equation_kernel
fields.rs:209:21  replace + with - in proca_equation_kernel        (sign of the Proca current)
fields.rs:209:21  replace + with * in proca_equation_kernel
fields.rs:254:31  replace || with && in energy_density_kernel
fields.rs:298:48  replace || with && in lagrangian_density_kernel
fields.rs:307:31  replace || with && in lagrangian_density_kernel
```

Four sit in `proca_equation_kernel`, whose only success-path test is `assert!(result.is_ok())`.
One flips the sign of the current. This is the answer-blind class producing a wrong answer that
nothing objects to, which is what the class was predicted to do.

The three `||` survivors guard squared magnitudes:

```rust
if !e_squared.is_finite() || !b_squared.is_finite() {
```

An earlier guard already rejects non-finite *inputs*, so this one is reachable only by overflow on
finite input. Both overflow tests set `e` and `b` to the same huge value, which makes a guard
needing one operand indistinguishable from a guard needing both. Overflowing one field at a time
kills all three.

## 8. Repair order

Ordered by severity over effort. Items 1 to 3 are bounded and independent.

1. **The 18 vacuous tests.** The union of no-observation, conditional-only, `is_ok() || is_err()`,
   the four unfalsifiable `!is_empty()` kernel `_valid` tests, and `is_finite()`-only. Each needs
   one real assertion. The failure mechanism is visible without interpretation.

2. **The GR connection gap.** Feed `schwarzschild_christoffel_at` into `parallel_transport_kernel`
   and `geodesic_integrator_kernel`. Parallel transport around a closed loop returns the vector
   rotated by the holonomy, which is an independent textbook oracle and nonzero on a curved
   connection. This is the largest physics risk in the crate and the fixture already exists.

3. **The five mutation survivors.** Give `proca_equation_kernel` and `energy_density_kernel` a
   success-path test that asserts the value. Rerun the same mutants to confirm they die.

4. **The 402 unpinned refusals.** Assert the variant, not merely that something failed. Mechanical
   per test, large in aggregate; worth doing per family alongside item 6.

5. **The 17 redundant wrapper tautologies.** Delete them. The consolidated test covers what they
   claim to cover, and leaving them implies coverage that is not there.

6. **The 545 unproven literal oracles.** The bulk, and the shape the audit already described: an
   oracle per kernel family in `scripts/physics_oracles.py`, then table-driven tests against it.

Each family closes only when targeted mutation testing kills the wrong constants, signs and
comparisons the rewrite was aimed at.

## Reproducing this

```console
python3 openspec/notes/test_audit/physics/audit_tests.py
python3 openspec/notes/test_audit/physics/audit_tests_v2.py
cargo mutants -p deep_causality_physics \
  --file 'deep_causality_physics/src/kernels/em/fields.rs' \
  --file 'deep_causality_physics/src/kernels/em/forces.rs' -j 8
```

`audit_tests.py` hardcodes its root; the README's claim that it takes a root directory is wrong.
`audit_tests_v2.py` takes `--root` and `--json`.
