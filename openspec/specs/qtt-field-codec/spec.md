# qtt-field-codec Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-tensor-bridge. Update Purpose after archive.
## Requirements
### Requirement: Quantized field codec (lattice ⇄ tensor train)

The `tensor_bridge` module SHALL provide a bidirectional codec between a periodic 1D lattice field of
length `N = 2^L` (a `CausalTensor<R>`) and a `CausalTensorTrain<R>` in quantized (binary-mode) layout:
`quantize` encoding the field as an `L`-mode MPS (physical dimension 2 per mode, most-significant-bit
first) and `dequantize` recovering the dense field. The round trip SHALL reproduce the input to working
precision, and the codec SHALL reject non-power-of-two lengths with a typed error. The bound is real
`R: CfdScalar + ConjugateScalar<Real = R>`.

#### Scenario: Round-trips a field to tolerance
- **WHEN** a length-`2^L` field is `quantize`d and then `dequantize`d
- **THEN** the recovered field equals the original to working precision

#### Scenario: A smooth field compresses
- **WHEN** a smooth (low-rank) field is quantized
- **THEN** the resulting train's bond dimensions are well below the dense size (`≪ 2^L`)

#### Scenario: Non-power-of-two length is rejected
- **WHEN** `quantize` is given a field whose length is not a power of two
- **THEN** it returns a typed error rather than producing an ill-formed train

