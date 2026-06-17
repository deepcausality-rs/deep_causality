# `deep_causality_cfd` — Solver Benchmark Performance

Sequential vs. parallel (`--features parallel`) timings for the three `CfdFlow` solver benchmarks.
The `parallel` feature forwards to the Rayon-backed DEC operator loops in `deep_causality_topology`
(via the shared `MaybeParallel` marker); without it the same loops run serially.

## Methodology

- **Machine:** Apple M3 Max, 16 logical / 16 physical cores, 128 GB, macOS 26.5.1 (arm64).
- **Toolchain:** rustc 1.96.0, `bench` profile (optimized).
- **Harness:** Criterion, `--warm-up-time 1 --measurement-time 2 --sample-size 30`. Reported value is
  Criterion's point estimate (median of the slope/mean estimate).
- **Date:** 2026-06-17.
- **Commands:**

  ```bash
  # sequential (default features)
  cargo bench -p deep_causality_cfd --bench bench_dec_ns_march    -- --warm-up-time 1 --measurement-time 2 --sample-size 30
  cargo bench -p deep_causality_cfd --bench bench_mms_verify      -- --warm-up-time 1 --measurement-time 2 --sample-size 30
  cargo bench -p deep_causality_cfd --bench bench_operator_study  -- --warm-up-time 1 --measurement-time 2 --sample-size 30

  # parallel (Rayon-backed DEC loops)
  cargo bench -p deep_causality_cfd --features parallel --bench bench_dec_ns_march   -- ...
  cargo bench -p deep_causality_cfd --features parallel --bench bench_mms_verify     -- ...
  cargo bench -p deep_causality_cfd --features parallel --bench bench_operator_study -- ...
  ```

- **Speedup** = sequential ÷ parallel. `> 1.00×` means parallel is faster; `< 1.00×` means parallel is
  **slower** (overhead exceeds the work).

> **Summary:** at the resolutions these benches use, the `parallel` feature does **not** help and
> actively **hurts** the marching solver — the per-loop Rayon fan-out costs more than the DEC operator
> work it parallelizes on small grids. It is a knob for large grids, not the default. See
> [Interpretation](#interpretation).

## 1. DEC Navier–Stokes marching (`bench_dec_ns_march`)

`CfdFlow::march` on a lid-free box. Two axes: grid size (fixed 20 steps) and step budget (fixed 24²).

### Grid sweep (20 steps, ν = 0.05, dt = 0.005)

| Grid | Sequential | Parallel | Speedup |
|-----:|-----------:|---------:|--------:|
| 16²  | 1.680 ms   | 5.410 ms | 0.31× (3.2× slower) |
| 24²  | 3.763 ms   | 8.577 ms | 0.44× (2.3× slower) |
| 32²  | 7.060 ms   | 12.215 ms | 0.58× (1.7× slower) |
| 48²  | 15.597 ms  | 21.724 ms | 0.72× (1.4× slower) |

### Step sweep (24² grid)

| Steps | Sequential | Parallel | Speedup |
|------:|-----------:|---------:|--------:|
| 10    | 2.063 ms   | 4.473 ms | 0.46× (2.2× slower) |
| 20    | 3.753 ms   | 8.714 ms | 0.43× (2.3× slower) |
| 40    | 7.142 ms   | 16.799 ms | 0.43× (2.3× slower) |

The parallel penalty **shrinks as the grid grows** (0.31× → 0.72× from 16² to 48²): larger grids give
each Rayon task more work to amortize the dispatch, so the curves are converging. It is constant across
the step sweep (~0.43×), as expected — more steps just repeat the same per-step fan-out.

### Parallel crossover — extended grid sweep

Where does `parallel` stop being a penalty and start winning? The default suite only goes to 48², so
the grid sweep was extended (20 steps, same ν / dt; `--measurement-time 3–4 --sample-size 10`; the
bench grid array was temporarily widened, then reverted).

| Grid | Cells  | Sequential | Parallel | Speedup |
|-----:|-------:|-----------:|---------:|--------:|
| 48²  | 2.3 k  | 16.10 ms   | 21.74 ms  | 0.74× |
| 64²  | 4.1 k  | 28.06 ms   | 34.15 ms  | 0.82× |
| 96²  | 9.2 k  | 61.63 ms   | 101.98 ms | 0.60× (noisy) |
| 128² | 16 k   | 110.27 ms  | 146.56 ms | 0.75× |
| 192² | 37 k   | 247.43 ms  | 310.09 ms | 0.80× |
| 256² | 66 k   | 450.9 ms   | 434.5 ms  | **1.04× (break-even)** |
| 384² | 147 k  | 995.5 ms   | 787.4 ms  | **1.26×** |
| 512² | 262 k  | 1791.9 ms  | 1222.6 ms | **1.47×** |

**Crossover: ~256² (≈65 k cells) is break-even; parallel becomes *noticeably* faster at ~384²
(≈1.25×) and keeps improving (1.47× at 512²).** Below 256² it is always a net loss. The win is modest
relative to the 16 cores — the per-step constrained projection (masked CG) and the march orchestration
are largely serial, so Amdahl's law caps the achievable speedup well under linear. The crossover and
the ceiling are hardware- and viscosity-dependent; re-measure on the target machine before relying on
the feature.

## 2. MMS verification (`bench_mms_verify`)

`CfdFlow::verify` against the analytic Taylor–Green field. Pointwise residual (no march) and the
decaying-amplitude march check.

### Pointwise (viscosity sweep)

| ν    | Sequential | Parallel | Speedup |
|-----:|-----------:|---------:|--------:|
| 0.01 | 246.2 ns   | 245.0 ns | 1.00× |
| 0.10 | 242.5 ns   | 243.9 ns | 0.99× |
| 1.00 | 243.3 ns   | 241.5 ns | 1.01× |

### Amplitude march (step sweep)

| Steps | Sequential | Parallel | Speedup |
|------:|-----------:|---------:|--------:|
| 10    | 1.048 µs   | 1.011 µs | 1.04× |
| 50    | 3.418 µs   | 3.401 µs | 1.00× |
| 100   | 6.228 µs   | 6.192 µs | 1.01× |

Verification is a closed-form pointwise evaluation; it never enters the parallelized DEC operator
loops, so the feature has **no measurable effect** (all within noise of 1.00×).

## 3. Operator accuracy (`bench_operator_study`)

`CfdFlow::operator_study` evaluating the viscous `δd` over resolution ladders of growing length.

| Ladder              | Sequential | Parallel | Speedup |
|---------------------|-----------:|---------:|--------:|
| `[16, 32]`          | 366.4 µs   | 370.5 µs | 0.99× |
| `[16, 32, 64]`      | 1.556 ms   | 1.571 ms | 0.99× |
| `[16, 32, 64, 128]` | 6.610 ms   | 7.355 ms | 0.90× |

Near-parity at the small rungs; the 128² rung is the only case with enough cells to enter the parallel
path, and there the fan-out overhead still slightly outweighs the gain (0.90×).

## Interpretation

- **Parallel is a net loss at small sizes.** The Rayon path parallelizes the inner DEC operator loops
  per call. On grids up to ~48²–128² the per-call dispatch (task splitting, thread hand-off, join)
  costs more than the elementwise/stencil work it distributes. The marching solver, which invokes those
  loops many times per step, pays this overhead repeatedly — hence the 1.4×–3.2× slowdowns.
- **The penalty narrows with size, and there is a measured crossover.** The extended march grid sweep
  (see [§1 crossover](#parallel-crossover--extended-grid-sweep)) breaks even at **~256² (≈65 k cells)**
  and becomes **noticeably faster at ~384² (1.26×), reaching 1.47× at 512²**. Below 256², parallel is
  always a net loss. The speedup stays well under the 16-core ideal because the per-step constrained
  projection and march orchestration are largely serial (Amdahl).
- **Verification is unaffected** because it does no DEC-loop work.
- **Guidance:** keep `parallel` **off** below ~256² (small/CI-scale workloads);
  enable it for grids ≳ 384², where it is a real win (≥1.25×). Re-measure to confirm the crossover on
  the target hardware.
- **Caveat:** these are single-machine medians at a short measurement window; verify at target configuration;