## ADDED Requirements

### Requirement: Real trait for analytic real scalars without field invertibility

`deep_causality_num` SHALL provide a `Real` trait modelling an analytic real scalar decoupled from field invertibility. Its supertraits SHALL be `CommutativeRing + PartialOrd + Neg<Output = Self> + Copy + Clone + AddAssign + SubAssign + MulAssign`, and SHALL NOT include `Div`, `DivAssign`, `InvMonoid`, or `Field`. It SHALL declare the analytic surface currently on `RealField` that does not depend on division: the constants (`pi`, `e`, `epsilon`); the elementary functions (`sqrt`, `exp`, `ln`, `log`, `log2`, `log10`, `powf`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`); sign/rounding/shape (`abs`, `floor`, `ceil`, `round`, `clamp`); and exceptional-value predicates/constructors (`nan`, `is_nan`, `is_infinite`, `is_finite`). The trait SHALL be re-exported from the crate root.

#### Scenario: Real is implementable without division

- **WHEN** a type provides commutative-ring arithmetic (`+ − ×`) and the analytic surface but has no total multiplicative inverse
- **THEN** it can implement `Real` without implementing `Field`, `Div`, or `DivAssign`

#### Scenario: f32, f64, and Float106 are Real

- **WHEN** the elementary functions and constants are invoked through a `Real` bound on `f32`, `f64`, or `Float106`
- **THEN** they produce results identical to the prior `RealField` implementation for those types

### Requirement: RealField extends Real and Field, preserving its surface

`RealField` SHALL be refactored to `RealField: Real + Field`, with the analytic method declarations relocated from `RealField` into `Real` and only the field-specific surface (division-based operations such as `inverse`) remaining on `RealField`. The refactor SHALL be behavior-preserving: any existing `T: RealField` bound SHALL continue to resolve the identical method set (the analytic surface now inherited via the `Real` supertrait, the field surface via `Field`), so no existing `RealField`-generic code requires modification. The concrete `RealField` implementations for `f32`, `f64`, and `Float106` SHALL relocate their analytic method bodies into `impl Real` blocks verbatim, yielding bit-identical numeric results.

#### Scenario: Existing RealField-generic code is unchanged

- **WHEN** a function bounded on `T: RealField` calls analytic operations (for example `x.sin()`, `x.exp()`, `T::pi()`) and field operations (`x / y`)
- **THEN** it compiles and runs unchanged after the refactor, resolving the analytic operations through the `Real` supertrait and the field operations through `Field`

#### Scenario: RealField implies Real

- **WHEN** a generic context holds `T: RealField`
- **THEN** `T: Real` holds automatically (a `T: RealField` value is accepted wherever `Real` is required), and every `RealField` value is a `Real` value

#### Scenario: A non-field analytic type is Real but not RealField

- **WHEN** a type implements `Real` but not `Field` (it lacks a total inverse)
- **THEN** it is accepted by `Real` bounds and rejected by `RealField` bounds at compile time
