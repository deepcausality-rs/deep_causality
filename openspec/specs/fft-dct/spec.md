# fft-dct

## Purpose

Plan-based real-to-real discrete cosine transforms (types I, II, III) in
`deep_causality_fft`, generic over `R: RealField`, built on the power-of-two core via
even-symmetric embeddings with a Bluestein fallback so every length runs in O(N log N).

## Requirements
### Requirement: Plan-based discrete cosine transforms
`deep_causality_fft` SHALL provide plan-based real-to-real DCT transforms
of types I, II, and III, generic over `R: RealField`, built on the existing
power-of-two core via even-symmetric embeddings with the Bluestein fallback
covering awkward lengths (every length O(N log N)). Plans SHALL follow the
crate's contracts: immutable after construction, caller-provided scratch,
no per-execution heap allocation, documented normalization with DCT-III as
the inverse of DCT-II under that convention.

#### Scenario: DCTs match the naïve definitions
- **WHEN** each DCT type is evaluated against a naïve O(n²) cosine-sum reference at representative lengths (power-of-two, odd, prime) at f64
- **THEN** results agree within the documented per-precision accuracy bound

#### Scenario: II/III round trip
- **WHEN** DCT-III is applied to the DCT-II of arbitrary real input
- **THEN** the input is recovered to rounding under the documented normalization

#### Scenario: DCT-I self-inverse round trip
- **WHEN** DCT-I is applied twice with the documented normalization
- **THEN** the input is recovered to rounding

#### Scenario: Precision-generic
- **WHEN** a DCT plan is instantiated at f32, f64, and Float106
- **THEN** all compile and meet their per-precision accuracy bounds against the naïve reference
