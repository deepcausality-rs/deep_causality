## ADDED Requirements

### Requirement: `Complex<F>` <-> `CausalMultiVector<F>` in Cl(0,1) implements `FieldIso` and `DivisionAlgebraIso`

The crate `deep_causality_multivector` SHALL expose a forward `impl<F> From<Complex<F>> for CausalMultiVector<F>` and a Tier 2 named witness `ComplexCl01Iso` implementing `Iso<CausalMultiVector<F>, Complex<F>>` in a new module `src/iso/`. The forward direction SHALL embed the complex value into Cl(0,1) by setting the scalar coefficient to `re` and the e₁ coefficient to `im`. The reverse direction SHALL panic when the input multivector's metric is not Cl(0,1). The iso SHALL satisfy `FieldIso<CausalMultiVector<F>, Complex<F>>` and `DivisionAlgebraIso<CausalMultiVector<F>, Complex<F>, F>` via blanket impls on `ComplexCl01Iso`.

#### Scenario: Forward From embeds Complex into Cl(0,1) multivector

- **WHEN** a downstream user invokes `CausalMultiVector::<f64>::from(Complex::new(2.0, 3.0))`
- **THEN** the result SHALL be a 2-element multivector with the Cl(0,1) metric
- **AND** the scalar coefficient SHALL equal `2.0`
- **AND** the e₁ coefficient SHALL equal `3.0`

#### Scenario: Reverse to_source recovers Complex from Cl(0,1) multivector

- **WHEN** a downstream user invokes `<ComplexCl01Iso as Iso<CausalMultiVector<f64>, Complex<f64>>>::to_source(mv)` for `mv` constructed via the forward direction with `Complex::new(2.0, 3.0)`
- **THEN** the result SHALL equal `Complex::new(2.0, 3.0)`

#### Scenario: Reverse panics on wrong-metric multivector

- **WHEN** a downstream user calls `to_source` with a multivector whose metric is not Cl(0,1)
- **THEN** the call SHALL panic with a message identifying the metric mismatch

#### Scenario: FieldIso homomorphism holds

- **WHEN** the test suite runs `assert_witness_field_iso_laws::<ComplexCl01Iso, CausalMultiVector<f64>, Complex<f64>>` with a non-zero multivector input
- **THEN** the assertion SHALL pass
- **AND** the test SHALL additionally exercise `assert_witness_ring_iso_laws`, `assert_witness_group_iso_law`, and `assert_witness_iso_round_trip` to cover the inherited laws

#### Scenario: DivisionAlgebraIso conjugation is preserved

- **WHEN** the test suite runs `assert_witness_division_algebra_iso_law::<ComplexCl01Iso, CausalMultiVector<f64>, Complex<f64>, f64>`
- **THEN** the assertion SHALL pass

### Requirement: `Quaternion<F>` <-> `CausalMultiVector<F>` as Cl(3,0)-even rotor implements `DivisionAlgebraIso`

The crate `deep_causality_multivector` SHALL expose a forward `impl<F> From<Quaternion<F>> for CausalMultiVector<F>` that always lifts cleanly into the Cl(3,0) even subalgebra (scalar + bivector components only; vector and pseudoscalar set to zero). The reverse direction SHALL be expressed as `impl<F> TryFrom<CausalMultiVector<F>> for Quaternion<F>` returning a typed error variant when the input has non-zero coefficients on odd-grade basis blades. A Tier 2 named witness `QuaternionRotorIso` SHALL implement `Iso<CausalMultiVector<F>, Quaternion<F>>` for the always-valid path; its `to_source` MAY panic when invariants are violated and SHALL document the invariant in the doc comment. The iso SHALL satisfy `DivisionAlgebraIso<CausalMultiVector<F>, Quaternion<F>, F>` (but NOT `FieldIso`, because quaternions are non-commutative).

#### Scenario: Forward From maps quaternion to Cl(3,0)-even rotor

- **WHEN** a downstream user invokes `CausalMultiVector::<f64>::from(Quaternion::new(1.0, 2.0, 3.0, 4.0))`
- **THEN** the result SHALL be a multivector with the Cl(3,0) metric
- **AND** the scalar coefficient SHALL equal `1.0`
- **AND** the e₂e₃ bivector coefficient SHALL equal `2.0`
- **AND** the e₃e₁ bivector coefficient SHALL equal `3.0`
- **AND** the e₁e₂ bivector coefficient SHALL equal `4.0`
- **AND** all vector and pseudoscalar coefficients SHALL be zero

#### Scenario: TryFrom recovers Quaternion when input is a pure rotor

- **WHEN** a downstream user invokes `Quaternion::<f64>::try_from(mv)` for `mv` constructed via the forward direction
- **THEN** the result SHALL be `Ok(Quaternion::new(...))` matching the original
- **AND** the round-trip SHALL be byte-identical for any quaternion input

#### Scenario: TryFrom rejects multivectors with non-zero odd-grade coefficients

- **WHEN** a downstream user invokes `Quaternion::<f64>::try_from(mv)` for `mv` with any non-zero coefficient on a vector or pseudoscalar basis blade
- **THEN** the result SHALL be `Err(...)` with an error variant naming the non-rotor failure
- **AND** the result SHALL NOT panic

#### Scenario: QuaternionRotorIso satisfies DivisionAlgebraIso

- **WHEN** the test suite runs `assert_witness_division_algebra_iso_law::<QuaternionRotorIso, CausalMultiVector<f64>, Quaternion<f64>, f64>` on a pure-rotor input
- **THEN** the assertion SHALL pass
- **AND** the test SHALL additionally exercise `assert_witness_ring_iso_laws`, `assert_witness_group_iso_law`, and `assert_witness_iso_round_trip`

#### Scenario: FieldIso is NOT implemented

- **WHEN** a reviewer inspects `QuaternionRotorIso`
- **THEN** there SHALL be NO `impl FieldIso<...> for QuaternionRotorIso` block
- **AND** any code attempting to use `QuaternionRotorIso` against a `FieldIso` bound SHALL fail to compile, citing the non-commutativity of quaternions
