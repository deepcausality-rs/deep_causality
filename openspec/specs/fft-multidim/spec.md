## ADDED Requirements

### Requirement: D-dimensional transforms by row-column decomposition
The crate SHALL provide a multi-dimensional plan (`FftPlanNd<R>`, with a real
variant for the rFFT entry axis) over an arbitrary-rank shape, implemented as
batched 1-D transforms applied axis by axis. Strided (non-contiguous) axes
SHALL be processed by gathering into contiguous scratch, transforming with
the 1-D kernels, and scattering back, so the 1-D kernels are reused
unchanged. One 1-D plan per distinct axis length SHALL be built and shared.

#### Scenario: 3-D transform matches the naïve multi-dimensional DFT
- **WHEN** a 3-D forward transform on a small grid is compared against the naïve DFT applied independently along each axis
- **THEN** the results agree within the documented accuracy bound

#### Scenario: Multi-dimensional round trip
- **WHEN** the inverse 3-D transform is applied to the forward 3-D transform of arbitrary input at 16³ and 32³
- **THEN** the result equals the input to within rounding

#### Scenario: Anisotropic shapes
- **WHEN** a transform is planned over a shape with unequal axis lengths (e.g. 16×32×8)
- **THEN** the transform is correct against the naïve reference and plans are shared between equal-length axes

### Requirement: Opt-in parallel execution with granularity thresholds
The crate SHALL provide a `parallel` cargo feature following the established
workspace pattern (Rayon under the feature, `MaybeParallel`-style bounds, no
serial API change). Under the feature, the independent 1-D batches of a
multi-dimensional transform SHALL fan out across threads behind a measured
granularity threshold so that small transforms remain serial. With the
feature disabled the crate SHALL compile and run fully serially.

#### Scenario: Identical results in both feature configurations
- **WHEN** the same multi-dimensional transform runs with and without the `parallel` feature
- **THEN** the outputs are identical to rounding

#### Scenario: Small transforms stay serial
- **WHEN** a transform below the granularity threshold executes under the `parallel` feature
- **THEN** no parallel fan-out is performed
