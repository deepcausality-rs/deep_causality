## ADDED Requirements

### Requirement: Real-to-complex forward transform (rFFT)
The crate SHALL provide a plan-based real-to-complex forward transform: real
input of length N produces the `N/2 + 1` non-redundant complex bins of the
Hermitian-symmetric spectrum (half-spectrum layout). The implementation SHALL
reuse the complex core via the length-N/2 packing construction rather than
running a complex FFT of redundantly complexified real data. The same
normalization contract as the complex transforms applies (forward
unnormalized).

#### Scenario: rFFT matches the complex FFT of the same data
- **WHEN** the rFFT of a real signal is compared against the first `N/2 + 1` bins of the complex FFT of that signal (imaginary parts zero)
- **THEN** the bins agree within the documented accuracy bound

#### Scenario: Half-spectrum layout
- **WHEN** an rFFT of length N is executed
- **THEN** exactly `N/2 + 1` complex bins are produced, with bin 0 and bin N/2 purely real to rounding

### Requirement: Complex-to-real inverse transform (irFFT)
The crate SHALL provide the inverse transform from a half-spectrum of
`N/2 + 1` complex bins back to N real samples, scaled by `1/N`, completing
the round trip with the forward rFFT.

#### Scenario: Real round-trip identity
- **WHEN** `irfft(rfft(x))` is computed for arbitrary real input
- **THEN** the result equals `x` to within rounding for the precision

#### Scenario: Output is exactly real
- **WHEN** `irfft` is applied to any Hermitian-consistent half-spectrum
- **THEN** the output is a real buffer (no complex residue is surfaced to the caller)
