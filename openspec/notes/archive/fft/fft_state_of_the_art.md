# State of the Art in FFT / Inverse FFT Implementations

**A reference survey for a from-scratch, dependency-free Rust implementation**

Prepared: 12 June 2026

---

## 1. Executive summary

"Fastest" splits into two distinct questions that point at different code:

1. **Lowest arithmetic operation count (flop-optimal).** For power-of-two
   sizes, the record holder is the **modified split-radix FFT of Johnson &
   Frigo (2007)**, building on unpublished hand-optimization by James Van
   Buskirk (2004). The gain over conventional split-radix is real but modest
   (~6% fewer flops), and flop count is *not* the dominant factor in real
   wall-clock performance.

2. **Fastest wall-clock on real hardware.** Here, memory-access patterns,
   cache behavior, and SIMD dominate. The practical winners (FFTW, Intel MKL,
   PocketFFT, RustFFT) use a **mixed-radix Cooley–Tukey** core with hardcoded
   straight-line "butterfly" kernels for small sizes, plus fallback algorithms
   (Rader's, Bluestein's, Good–Thomas) for awkward sizes. They deliberately do
   *not* use flop-minimal split-radix as the workhorse, because its irregular
   memory access defeats vectorization.

**Recommendation:** For a dependency-free Rust *reference* implementation that
is correct, comprehensible, and genuinely fast, do not chase the Johnson–Frigo
flop record. Build a layered structure: iterative radix-2 baseline → radix-4 /
mixed-radix power-of-two workhorse → hardcoded small-N butterflies → Bluestein's
fallback for arbitrary/prime lengths. Implement the inverse transform by
conjugation reuse of the forward path. Details in §5.

---

## 2. The two notions of "fastest"

### 2.1 Flop-optimal: split-radix and its modern refinement

The split-radix FFT is a Cooley–Tukey variant that blends radices 2 and 4,
recursively expressing a length-N DFT in terms of one length-N/2 DFT and two
length-N/4 DFTs. It held the record for lowest published arithmetic operation
count for power-of-two DFTs for decades. The original count was first improved
in 2004 by James Van Buskirk (hand-optimization for N=64), and the new lowest
count was then achieved by a systematic modification of split radix by Johnson
and Frigo (2007). Importantly, the number of arithmetic operations is not the
sole — or even necessarily the dominant — factor in determining real runtime on
a computer.

- Split-radix FFT — overview, history, Yavne (1968) origin, Duhamel–Hollmann
  (1984) rediscovery, Van Buskirk (2004), Johnson–Frigo (2007):
  https://en.wikipedia.org/wiki/Split-radix_FFT_algorithm
- S. G. Johnson and M. Frigo, "A modified split-radix FFT with fewer arithmetic
  operations," *IEEE Trans. Signal Processing*, 55(1):111–119, 2007:
  https://www.fftw.org/newsplit.pdf
- All radix-2/2^k split-radix variants share the same arithmetic-operation count
  (Bouguezel, Ahmad, Swamy), IEEE ISCAS 2004:
  https://ieeexplore.ieee.org/document/1416259/
- Reduced-complexity radix-4/8/split-radix DIF variant (Qadeer, Khan, Sattar),
  ICTACT J. Communication Technology, 2012:
  https://doaj.org/article/cb811eeba4df43ffbacc503cedb674c3

### 2.2 Wall-clock-optimal: why flops are not enough

On real hardware, the split-radix recursion's irregular memory access pattern
defeats SIMD vectorization, so flop-minimal code is often *slower* in practice
than a more regular radix-4 / mixed-radix structure. This is why production
libraries favor regular access patterns and hardcoded kernels over the
flop-record algorithm.

- Frigo & Johnson, "The Fastest Fourier Transform in the West" (FFTW design
  paper, MIT-LCS-TR-728) — codelet generator, Cooley–Tukey in symbolic
  arithmetic, why real performance differs from flop count:
  https://www.fftw.org/fftw-paper.pdf
- Benchmarking FFT libraries (project-gemmi) — FFTW vs MKL vs PocketFFT vs
  pffft vs muFFT vs KissFFT vs meow_fft measured timings; notes FFTW is "the
  reference point" but no longer the fastest, MKL significantly faster:
  https://github.com/project-gemmi/benchmarking-fft

---

## 3. The algorithm toolbox (what production FFTs actually compose)

A state-of-the-art practical FFT is not one algorithm but a planner that
dispatches across several, by size:

| Algorithm | Best for | Note |
|---|---|---|
| Cooley–Tukey (radix-2 DIT/DIF) | power-of-two | simplest, the correctness baseline |
| Radix-4 / mixed radix-2-4 | power-of-two | ~halves nontrivial multiplies vs radix-2; regular, vectorizable |
| Split-radix (incl. Johnson–Frigo) | power-of-two | flop-optimal but irregular access |
| Mixed-radix Cooley–Tukey | highly composite N | the general workhorse |
| Good–Thomas (Prime Factor Algorithm) | coprime factors | no twiddle factors between stages |
| Rader's algorithm | prime N | re-expresses prime-size DFT as cyclic convolution |
| Bluestein's (chirp-z) algorithm | arbitrary / large-prime N | universal O(N log N) fallback via convolution |
| Four-step / Bailey | very large N (out-of-cache) | blocking strategy for cache/memory locality |

References:

- Cooley–Tukey FFT algorithm — recursive decomposition to O(N log N) for smooth
  N; historical background:
  https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm
- Rader's FFT algorithm — prime-size DFT as cyclic convolution:
  https://en.wikipedia.org/wiki/Rader%27s_FFT_algorithm
- Bluestein's algorithm — for large prime factors, an FFT of length n becomes a
  convolution of length n2 ≥ 2n−1 chosen to be highly composite (see PocketFFT
  README under §4).
- Four-step / Bailey framework, and Rader's/Bluestein's as
  convolution reformulations — discussion in "Permutation-Avoiding FFT-Based
  Convolution," arXiv:2506.12718:
  https://arxiv.org/pdf/2506.12718
- R. Singleton, "An algorithm for computing the mixed radix fast Fourier
  transform," *IEEE Trans. Audio Electroacoust.*, 17(2):93–103, 1969
  (foundational mixed-radix reference), DOI 10.1109/TAU.1969.1162042.

---

## 4. Best reference implementations to study (source code)

### 4.1 PocketFFT — the engine behind NumPy / SciPy

FFTPACK-derived, header-only C++. Implements mixed-radix Cooley–Tukey and
automatically selects between FFTPACK and Bluestein based on the size's
factorization: `cfftp` handles highly composite sizes, `fftblue` handles large
prime factors. Worst-case complexity stays O(N log N) because Bluestein covers
the bad cases. Twiddle generation is numerically careful (≈2√n sincospi calls,
remainder via angle-addition theorems). Plans are read-only and thread-safe.

- Canonical repo (Martin Reinecke, MPCDF GitLab):
  https://gitlab.mpcdf.mpg.de/mtr/pocketfft/-/blob/cpp/pocketfft_hdronly.h
- GitHub mirror (simplifies external contributions):
  https://github.com/mreineck/pocketfft
- Architecture / algorithm-selection write-up (DeepWiki):
  https://deepwiki.com/mreineck/pocketfft

### 4.2 RustFFT — the most directly relevant prior art

Pure-Rust planner composing: hardcoded butterflies for sizes 2–32; Radix-3,
Radix-4; Mixed-Radix and Mixed-Radix-Small for composites; Good–Thomas
(+ small variant); Rader's for primes; Bluestein's as universal fallback; and a
naïve O(n²) DFT kept purely as a validation reference. SIMD (AVX/SSE/NEON/WASM)
is selected automatically by the planner. Measured behavior: sizes of the form
2^n·3^m are fastest; sizes with all prime factors ≤ 11 are very fast; large
prime factors are noticeably slower.

- Algorithm module docs:
  https://docs.rs/rustfft/latest/rustfft/algorithm/index.html
- Algorithm-implementation overview & complexity table (DeepWiki):
  https://deepwiki.com/ejmahler/RustFFT/3-algorithm-implementations
- ZFFT — a pure-Zig port of RustFFT; useful as a second, cleanly-structured
  reading of the same algorithm set and portable-SIMD approach:
  https://github.com/10d9e/zfft

### 4.3 FFTW — the canonical fast library and split-radix codelet source

If you later decide the flop-optimal path is worth the complexity, FFTW's
generated split-radix codelets and the Johnson–Frigo paper are the canonical
source.

- FFTW design paper: https://www.fftw.org/fftw-paper.pdf
- Modified split-radix paper: https://www.fftw.org/newsplit.pdf

---

## 5. Recommended build plan for DeepCausality (dependency-free Rust)

Build in priority order. Each layer is independently testable against the one
above it.

1. **Iterative radix-2 Cooley–Tukey (DIT)** with bit-reversal permutation and
   precomputed twiddle factors. Your correctness baseline (~40 lines). Validate
   against a naïve O(n²) DFT, exactly as RustFFT keeps its DFT for testing.

2. **Radix-4 / mixed radix-2-4** as the power-of-two workhorse. Roughly halves
   nontrivial multiplies vs radix-2 with regular, vectorizable access — most of
   split-radix's benefit at far lower code and cache complexity.

3. **Hardcoded butterflies for small N** (2, 3, 4, 5, 8, …) as straight-line,
   loop-free, twiddle-lookup-free code. This is where libraries get a large
   fraction of real-world speed.

4. **Bluestein's algorithm** as the universal fallback for arbitrary/prime N,
   layered on the power-of-two core (pad to a power of two ≥ 2n−1). Keeps the
   transform O(N log N) for *any* length.

### Inverse FFT

Do not write a separate inverse kernel. Conjugate the input, run the forward
transform, conjugate the output, scale by 1/N. One line of reuse, and it stays
provably consistent with the forward path — which matters more for a causality
framework than shaving cycles.

### If your workload is convolution-dominated

If DeepCausality propagates effects via spectral convolution, look at
**permutation-avoiding FFT-based convolution**: it skips bit-reversal/permutation
entirely because the forward scramble and inverse unscramble cancel across a
convolution, and can beat a flop-optimal FFT in that pipeline.

- "Permutation-Avoiding FFT-Based Convolution," arXiv:2506.12718:
  https://arxiv.org/pdf/2506.12718

---

## 6. Consolidated source list

**Algorithms & theory**

- Split-radix FFT: https://en.wikipedia.org/wiki/Split-radix_FFT_algorithm
- Johnson & Frigo, modified split-radix (2007): https://www.fftw.org/newsplit.pdf
- Cooley–Tukey: https://en.wikipedia.org/wiki/Cooley%E2%80%93Tukey_FFT_algorithm
- Rader's algorithm: https://en.wikipedia.org/wiki/Rader%27s_FFT_algorithm
- Split-radix variants share flop count (ISCAS 2004): https://ieeexplore.ieee.org/document/1416259/
- Radix-4/8/split-radix reduced complexity (ICTACT 2012): https://doaj.org/article/cb811eeba4df43ffbacc503cedb674c3
- Permutation-avoiding convolution (arXiv:2506.12718): https://arxiv.org/pdf/2506.12718

**Design papers & benchmarks**

- FFTW design paper (MIT-LCS-TR-728): https://www.fftw.org/fftw-paper.pdf
- FFT library benchmarks: https://github.com/project-gemmi/benchmarking-fft

**Reference source code**

- PocketFFT (canonical): https://gitlab.mpcdf.mpg.de/mtr/pocketfft/-/blob/cpp/pocketfft_hdronly.h
- PocketFFT (GitHub mirror): https://github.com/mreineck/pocketfft
- PocketFFT architecture notes: https://deepwiki.com/mreineck/pocketfft
- RustFFT algorithm docs: https://docs.rs/rustfft/latest/rustfft/algorithm/index.html
- RustFFT implementation overview: https://deepwiki.com/ejmahler/RustFFT/3-algorithm-implementations
- ZFFT (Zig port of RustFFT): https://github.com/10d9e/zfft

---

*Note: Numeric flop-reduction percentages and size-class performance
characteristics are drawn from the cited papers and library documentation
above. The Johnson–Frigo result is the standard reference for lowest-known
power-of-two arithmetic count; library wall-clock rankings change over time and
should be re-benchmarked on your target hardware (Apple Silicon, in your case).*
