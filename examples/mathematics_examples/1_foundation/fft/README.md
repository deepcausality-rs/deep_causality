# Foundation: `deep_causality_fft`

Plan-based transforms. A plan is built once for a length, holds every precomputed twiddle and
stage schedule, and is immutable after construction. Execution borrows caller-supplied scratch,
so a transform can sit inside a hot loop.

Three properties of the crate:

- **Every length is O(N log N).** The planner picks by length: hardcoded kernels for small
  powers of two, an iterative mixed radix-4/radix-2 Stockham pipeline for larger ones, and
  Bluestein's chirp-z for everything else. A prime length runs at the same order as a power of two.
- **The inverse is the forward kernel.** `ifft(x) = conj(fft(conj(x))) / N`, so one kernel
  defines both directions and they stay consistent by construction.
- **Precision is a parameter.** Every plan is generic over `FftScalar`.

| Example | What it covers | Command |
|---|---|---|
| [fft_plans.rs](fft_plans.rs) | `FftPlan` round trip, a sinusoid landing in one bin, `RfftPlan`'s Hermitian half-spectrum, and a prime length going through Bluestein | `cargo run -p mathematics_examples --example fft_plans_examples` |
