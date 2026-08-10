# examples/avionics_examples/cfd/ — engineer-facing CFD demonstrations (plasma_blackout corridor/weather/retropulsion, nozzle_operating_map, flight_envelope_placard, viv_resonance_margin, turbulence_flow)

**Production readiness: `needs-work`**

The closed-form gas dynamics is genuinely correct: I reproduced the area-Mach kernel, the nozzle thrust coefficient (code 1.2539 vs textbook 1.2576), the first-critical ratio 0.9372, the whole isentropic-plus-Rankine-Hugoniot shock-position chain, the US-1976 atmosphere table to 3-4 digits per row, the Sutton-Graves arithmetic, and the Lorenz right-hand side. Constants documentation is unusually disciplined — most numeric literals carry a derivation, and retropulsion's constants.rs even records supersession history. But three gates presented in the verdict list cannot fail: weather's "(0) table integrity" folds over a hardcoded `errored: false`; corridor's "(4f) fine sweep refines the coarse winner" re-flies the coarse winner inside the fine round, so its inequality is structural; and corridor's "(4e)" compares against a "ballistic" miss that is identically AIM_CROSS_RANGE_M (printed as exactly 20.000 m). Separately, three committed deliverable tables carry undisclosed error: the placard's linearly-interpolated density overstates q by up to 39% at 36 km, the VIV Strouhal column is crossing-count quantized at 1/110 yet printed to 17 digits with two pairs of bit-identical rows, and four of retropulsion's "earned band" docstrings record measurements that do not match the committed output.txt. The placard README also advertises a third gate ("matrix integrity") that model.rs does not register. None of these is unfixable, and the physics core is sound — but an avionics reviewer would currently be unable to distinguish the gates that verify something from the ones that cannot fail.

- Files read: **58**
- Findings raised: **19** — surviving adversarial verification: **19** (refuted: 0)
- Surviving by severity: major 3, minor 14, info 2
- Independently confirmed-correct items: **16**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Area-Mach relation kernel used as the nozzle's analytic reference | `deep_causality_physics/src/kernels/fluids/compressible.rs:215-230` | A/A* = (1/M)·[(2/(γ+1))·(1+(γ−1)/2·M²)]^((γ+1)/(2(γ−1))) — Anderson, Modern Compressible Flow, Ch. 5 (the crate's own cited source) |
| Nozzle thrust coefficient at p_back/p0 = 0.10 | `deep_causality_cfd/src/types/flow/duct_march_run.rs:305; operating_map.csv row 0.1` | C_F = sqrt(2γ²/(γ−1)·(2/(γ+1))^((γ+1)/(γ−1))·(1−(p_e/p0)^((γ−1)/γ))) + (p_e−p_a)/p0·A_e/A_t — Sutton, Rocket Propulsion Elements |
| Closed-form internal-shock position construction | `examples/avionics_examples/cfd/nozzle_operating_map/model.rs:182-194` | T2/T1=(p2/p1)/(ρ2/ρ1); M2=M1·(u2/u1)·sqrt(T1/T2); p02/p01=(p2/p1)·(p02/p2)/(p01/p1); A*_2=A/(A/A*)(M2) — Anderson, normal-shock relations |
| First critical back-pressure ratio printed in output.txt | `examples/avionics_examples/cfd/nozzle_operating_map/output.txt (analytic references line)` | Subsonic root of A/A*=2 at γ=1.4 is M=0.3059; p/p0=(1+0.2M²)^(−3.5) |
| Rankine-Hugoniot post-shock ratios in FittedNormalShock | `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs:118-120` | ρ2/ρ1=(γ+1)M²/((γ−1)M²+2); u2/u1=1/(ρ2/ρ1); p2/p1=(2γM²−(γ−1))/(γ+1) — Anderson, normal-shock relations |
| US Standard Atmosphere 1976 table in the placard example | `examples/avionics_examples/cfd/flight_envelope_placard/constants.rs:60-70` | USSA-1976 (NOAA-S/T 76-1562) T, a, ρ at geometric altitude; n = ρ / 4.81e-26 kg |
| Sutton-Graves stagnation heating arithmetic | `examples/avionics_examples/cfd/flight_envelope_placard/model.rs:142-146; constants.rs:32` | q̇ = k·√(ρ∞/R_n)·V³ with k = 1.7415e-4 kg^0.5·m⁻¹ (SI, W/m²) — Sutton & Graves, NASA TR R-376, 1971 |
| Supersonic stagnation-temperature branch is exactly continuous with the isentropic branch | `examples/avionics_examples/cfd/flight_envelope_placard/model.rs:128-140` | A normal shock is adiabatic, so T0 is conserved across it for a calorically perfect gas: T0 = T∞·(1+(γ−1)/2·M∞²) |
| Lorenz / Saltzman rate field and chaotic parameters | `examples/avionics_examples/cfd/turbulence_flow/model.rs:87-91, 67-71` | dx/dt=σ(y−x); dy/dt=x(ρ−z)−y; dz/dt=xy−βz with σ=10, ρ=28, β=8/3 — Lorenz, J. Atmos. Sci. 20:130 (1963) |
| Leading Lyapunov exponent value | `examples/avionics_examples/cfd/turbulence_flow/model.rs:170` | λ₁ ≈ 0.90566 for the Lorenz attractor at (10, 28, 8/3) — Viswanath, 'Lyapunov exponents from random Fibonacci sequences to the Lorenz equations' (1998); Sprott, Chaos and Time-Series Analysis |
| Float106 is a legitimate ground truth over the simulated window | `examples/avionics_examples/cfd/turbulence_flow/main.rs:46, 90-92` | t_horizon ≈ ln(1/ε)/λ; the reference precision must out-live the comparison window |
| VIV dimensionalization and Reynolds number | `examples/avionics_examples/cfd/viv_resonance_margin/model.rs:63, 75-76` | Re = V·D/ν; f_s = St·V/D; margin = \|f_n − f_s\|/f_n |
| Weather-example cold-drift ratio obeys the t²-law it claims | `examples/avionics_examples/cfd/plasma_blackout/weather/output.txt (gate 4); weather_table.csv rows 1 and 4` | Dead-reckoning drift ∝ ½·\|b\|·t², so ratio = (bias departure) × (dwell ratio)² |
| Retropulsion gate (1) genuinely cross-checks a different artifact | `examples/avionics_examples/cfd/plasma_blackout/retropulsion/output.txt (gate 1); weather_table.csv` | Linear interpolation of the weather table's onset column at dT = −32 K between the −25 K and −40 K rows |
| Gate counts advertised in the crate README match the registered gates and the committed outputs | `examples/avionics_examples/README.md:24-26` | Claimed 13 (corridor) / 8 (weather) / 16 (retropulsion) |
| All twelve examples are declared and runnable | `examples/avionics_examples/Cargo.toml:12-58` | Each example directory must have a [[example]] stanza with a correct path |

## Findings

### 5.1 [MINOR] Weather gate '(0) table integrity' folds over a hardcoded false and cannot fail

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/weather/model.rs:223`
- **Auditor confidence:** confirmed

**Claim.** weather/model.rs:223 hardcodes `errored: false`, making gate '(0) table integrity' structurally incapable of reporting false. The condition it names is nevertheless enforced — a step error short-circuits `march_for` and exits 2 — so the defect is a redundant gate inflating the reported gate count from 7 discriminating to 8, not an unchecked failure path.

**Code evidence.**

```
model.rs:223 (in world_row): `errored: false, // a report that exists marched to completion; an error short-circuits march_for`
model.rs:262-267: `fn gate_integrity(v: &StudyView<'_, WorldRow>) -> (bool, String) { ( v.rows().iter().all(|r| !r.errored), "all six worlds completed without a captured step error".into(), ) }`
output.txt: `[PASS] (0) table integrity: all six worlds completed without a captured step error`
```

**Reference form.** A verification gate must have at least one reachable input under which it reports false. Here the predicate reduces to `all(|_| !false)` = `true` for every possible study, including one where every world diverged.

**Impact.** The verdict line reports a passing integrity check that verified nothing. An engineer reading the weather output counts eight passing gates when only seven can discriminate. If a future refactor lets a failed report reach world_row, the failure is silently certified as passing.

**Recommended fix.** Either remove the gate (and drop it from the crate README's 'Eight self-verifying gates' claim), or make `errored` a real observation — e.g. propagate a per-draw error flag from the march and set `errored: draws.iter().any(|d| d.had_step_error())`. If the short-circuit argument in the comment is correct, state it as a construction invariant in the README instead of dressing it as a gate.

**Adversarial check.** The mechanism is exactly as described and the quoted code is verbatim. `errored` is the literal `false` at weather/model.rs:223 and the gate at 262-267 folds `all(|r| !r.errored)`, so no input to `world_row` can make it report false. Contrast is real: corridor/model.rs:349 sets the same field from `pause.error().is_some()`. But the claimed IMPACT is overstated. `world_row` is the `reduce_ensemble` closure, reached only after `march_for` succeeded; a real step error resolves the study to `Err(StudyError)` at main.rs:84 `.verdict()?` and the program exits 2 with 'plasma-blackout weather table failed', not a green verdict. So the failure mode the auditor says is 'silently certified as passing' is in fact loudly caught one layer up, and the inline comment at :223 says exactly that. This is a redundant, non-discriminating reporting line — a verdict-hygiene defect — not a certification blocker.

> Evidence re-read: weather/model.rs:223 `errored: false, // a report that exists marched to completion; an error short-circuits march_for`; weather/model.rs:262-267 gate_integrity verbatim as quoted; weather/main.rs:72-84 the study chain ending `.verdict()?` inside a `Result<Verdict, StudyError>` closure, with main.rs:101-104 mapping Err to ExitCode 2; corridor/model.rs:349 `errored: pause.error().is_some()`; weather/output.txt shows 8 PASS lines including '(0) table integrity'.

---

### 5.2 [MINOR] Corridor gate '(4f) fine sweep refines the coarse winner' cannot fail by construction

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:463`
- **Auditor confidence:** confirmed

**Claim.** The fine candidate set contains the coarse winner's exact bank angle, so gate (4f) can only fail if the second fork round fails to reproduce the first round's result at the same command. It is therefore a cross-round reproducibility check with a reachable failing input, not an identity — but it is non-discriminating with respect to the other ten fine candidates, and the docstring at model.rs:69 states this design openly.

**Code evidence.**

```
model.rs:96-97: `let offset = (k as FloatType - FINE_SPAN_STEPS as FloatType) * ft(FINE_STEP_DEG); let deg = (winner.bank_deg + offset).clamp(lo, hi);` — at k = 5 this is `winner.bank_deg` exactly.
model.rs:69-70 (docstring, an explicit admission): `The coarse winner flies again in the middle, so the refinement can only confirm or improve it.`
model.rs:468-469: `committed.outcome.miss_distance <= coarse_committed.outcome.miss_distance`
output.txt confirms the identity: coarse `15.0deg ... 3.120 m` and fine `15.0deg ... 3.120 m` are the same value to four digits (CSV: 3.120459333065107).
```

**Reference form.** A refinement gate should be able to detect a refinement that failed to improve — e.g. by requiring a strict improvement, or by bracketing without re-flying the incumbent. Including the incumbent in the candidate set makes `min(S ∪ {x}) <= x` an algebraic identity.

**Impact.** Presented in the verdict as validation that the two-round search works, this gate would still pass if the fine round were entirely broken (all other candidates returning NaN-free garbage worse than the incumbent). It provides no evidence for the '0.5-degree resolution' claim the corridor README builds on.

**Recommended fix.** Require a strict improvement over the incumbent excluding the incumbent's own re-flight, or gate on something falsifiable — e.g. that the fine minimum is interior to the bracket (13.5 deg has 13.0 and 14.0 on either side in the committed run, which is a real and checkable property). Alternatively keep the incumbent for safety but gate on `fine_min < coarse_min` strictly and document the tie case.

**Adversarial check.** Every quoted line is verbatim and the arithmetic holds: FINE_SPAN_STEPS=5 and FINE_STEP_DEG=0.5 put offset 0 at k=5, `(winner.bank_deg + 0).clamp(lo,hi)` is the winner exactly (the winner is always inside [lo,hi]), and output.txt/CSV confirm the re-flown 15.0 deg reproduces 3.120459333065107. But the auditor's own REFERENCE FORM refutes the CONFIRMED verdict on the tautology axis: `min(S ∪ {x}) <= x` is an algebraic identity only if the re-flown incumbent reproduces x. Here x is not a stored number — it is re-derived by forking the same paused onset a *second* time, building a differently-named world ('fine_bank_05' vs 'bank_15_deg'), re-flying 100 steps, and re-scoring against a *carried* aim rather than a re-derived one. If the second fork, the aim carry-through (fine_candidates line 89/101), or the branch determinism regressed, the re-flown incumbent would score worse than 3.1205 and every fine candidate with it, and the gate would fail. That is a reachable failing input, so the gate is not tautological — it is a cross-round reproducibility check. It is weak (it cannot detect a fine round whose *other* candidates are garbage), and the docstring at line 69 says so plainly. The claim that it 'provides no evidence for the 0.5-degree resolution' also ignores that the gate's detail string prints the actual 13.5 deg / 2.39 m improvement over 15 deg / 3.12 m.

> Evidence re-read: corridor/model.rs:96-97 the offset/clamp exactly as quoted; :74-105 fine_candidates carries `aim: Some(aim)` from the coarse winner; :463-479 gate_refinement verbatim; corridor/constants.rs:29,31 FINE_SPAN_STEPS=5, FINE_STEP_DEG=0.5; corridor/main.rs:102 `.refine(&onset, model::fine_candidates)` — a second fork of the same pause, not a reuse of coarse rows; output.txt coarse 15.0deg 3.120 m and fine 15.0deg 3.120 m; corridor_branches.csv row 15 = 3.120459333065107.

---

### 5.3 [MINOR] Corridor gate '(4e)' compares against a 'ballistic miss' that is identically the AIM_CROSS_RANGE_M constant

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:443`
- **Auditor confidence:** confirmed

**Claim.** Gate (4e)'s 'ballistic miss' is AIM_CROSS_RANGE_M by construction, so the gate is effectively `committed_miss <= 6.67 m` and the reported '8.4x better' is 20.0/committed_miss. This is a wording/presentation problem — the printed comparison implies two independent flight results — not a tautology: the threshold is discriminating on steering authority, and the constant's construction is fully documented at constants.rs:32-38.

**Code evidence.**

```
model.rs:192-198: `pub fn aim_point(ballistic_terminal: [FloatType; 3]) -> [FloatType; 3] { [ ballistic_terminal[0], ballistic_terminal[1], ballistic_terminal[2] - ft(AIM_CROSS_RANGE_M), ] }`
model.rs:447-451: `let ballistic_miss = coarse[zero_bank].outcome.miss_distance; ... committed_miss * ft(MISS_IMPROVEMENT_FACTOR) <= ballistic_miss`
constants.rs:38: `pub const AIM_CROSS_RANGE_M: f64 = 20.0;`
output.txt confirms the identity: the 0.0deg row reads `miss (traj) 20.000 m` exactly, and the gate detail reads `vs the ballistic 20.00 m (8.4x better)`.
```

**Reference form.** A 'beats the baseline by Nx' gate should compare two independently measured quantities. Here one side is a configuration constant re-emitted through a distance computation, so the reported '8.4x better' is `20.0 / committed_miss`, not a ratio of two flight results.

**Impact.** Both the output and the corridor README present '2.39 m off the aim vs the ballistic 20.00 m' as a measured steering benefit. It is a measurement against a constant the study placed. Worse, AIM_CROSS_RANGE_M's own docstring records that it was re-pinned after measuring reachable cross-range ('measured: 33.5 m at the clamped 40 deg, ~21 m at 15 deg'), so the target distance was sized against the achievable answer.

**Recommended fix.** State the gate for what it is: `committed_miss <= AIM_CROSS_RANGE_M / MISS_IMPROVEMENT_FACTOR`, and print it that way ('closes 20.0 m of commanded cross-range to 2.39 m'). Drop the 'vs the ballistic' framing from the gate detail and the README, or place the aim point independently of the ballistic run (e.g. at a fixed inertial coordinate) so the ballistic miss becomes a genuine measurement.

**Adversarial check.** The mechanism is verified: aim_point (model.rs:192-198) displaces the ballistic terminal by exactly AIM_CROSS_RANGE_M in z, and gate_guidance (model.rs:443-451) reads that same branch's miss, so ballistic_miss ≡ 20.0 m and output.txt prints '20.000 m' exactly. But the finding is filed under 'tautology-circular' and the gate is not circular: it reduces to `committed_miss <= 20/3 m`, a discriminating threshold on flown steering authority that a broken or under-authoritative steering model fails (the 40 deg clamped branch already misses 22.02 m; a vehicle with half the roll authority would miss more than 6.67 m and the gate would fail). The 'magic constant' charge also fails traceability: constants.rs:32-38 states in full that AIM_CROSS_RANGE_M is the cross-range offset from the ballistic terminal, why it was sized (so the optimum sits inside the envelope cap and between sweep candidates), and the measured reachable cross-range it was re-pinned against. That is disclosed demonstration design, not a hidden back-fit. What survives is a presentation defect: the detail string and README:78 frame '20.00 m' and '8.4x better' as a comparison of two flown results when one side is a placed constant re-emitted through a distance computation.

> Evidence re-read: corridor/model.rs:192-198 aim_point verbatim; :443-451 gate_guidance verbatim; corridor/constants.rs:38 `pub const AIM_CROSS_RANGE_M: f64 = 20.0;` with docstring at :32-37; corridor/constants.rs:41 MISS_IMPROVEMENT_FACTOR = 3.0; output.txt coarse table `0.0deg ... 20.000 m` and gate (4e) '2.39 m off the aim vs the ballistic 20.00 m (8.4x better)'; corridor/README.md:57,78.

---

### 5.4 [MAJOR] Placard README advertises three gates including a 'matrix integrity' gate that does not exist in the code

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `examples/avionics_examples/cfd/flight_envelope_placard/README.md:80`
- **Auditor confidence:** confirmed

**Claim.** The README's gate table and prose claim three gates, the third being 'matrix integrity | one computed row per file row | 16 of 16 matrix rows computed'. placard_gates() registers exactly two, and the committed output.txt prints exactly two PASS lines.

**Code evidence.**

```
README.md:80: `| matrix integrity | one computed row per file row | 16 of 16 matrix rows computed |`
README.md:30: `All three gates pass and the process exits 0.`
README.md:63-64: `3. **Gate.** Three gates: ... and matrix integrity (every file row computed).`
model.rs:161-165: `pub fn placard_gates() -> GateSeq<PlacardRow> { GateSeq::new("flight envelope placard") .gate("q-max placard", gate_q_max) .gate("stagnation temperature", gate_stagnation_temperature) }`
output.txt: only `[PASS] q-max placard: ...` and `[PASS] stagnation temperature: ...`
```

**Reference form.** Documentation of a verification suite must enumerate exactly the gates the code registers. The 'recorded detail' column in the README quotes a specific string ('16 of 16 matrix rows computed') that appears nowhere in the code or the committed output.

**Impact.** A reviewer auditing this example against its README would tick off a schedule-integrity check that was never run. Nothing in the code detects a matrix row silently dropped between read and sweep — the exact failure mode the phantom gate claims to cover.

**Recommended fix.** Either implement the gate (compare `view.rows().len()` against the row count returned by the table reader, which requires plumbing that count through) or delete the third row from the README table, fix line 30 to 'Both gates pass', and fix line 63 to 'Two gates'.

**Adversarial check.** Every quoted line is verbatim at the cited location and the code contains no such gate anywhere. README.md:80 tabulates 'matrix integrity | one computed row per file row | 16 of 16 matrix rows computed'; README.md:30 says 'All three gates pass'; README.md:63-66 enumerates three gates including 'matrix integrity (every file row computed)'. model.rs:161-165 registers exactly two. output.txt prints exactly two PASS lines. I additionally grepped main.rs and model_config.rs for any row-count or length check that could implement the claim under another name — there is none. The 'recorded detail' string '16 of 16 matrix rows computed' appears in no source file and in no committed output. Nothing in the example detects a matrix row dropped between read and sweep.

> Evidence re-read: flight_envelope_placard/README.md:30, :63-66, :78-80 (grep -n confirms all three verbatim); model.rs:161-165 `placard_gates()` with exactly `.gate("q-max placard", ...)` and `.gate("stagnation temperature", ...)`; output.txt with two `[PASS]` lines and no third; grep of main.rs/model_config.rs for `len()`/`integrity`/`rows` returns only an unrelated doc comment and an `.inspect` call.

---

### 5.5 [MINOR] Placard density is linearly interpolated across a 10 km gap, overstating q by up to 39% in the committed deliverable table

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `examples/avionics_examples/cfd/flight_envelope_placard/constants.rs:67`
- **Auditor confidence:** confirmed

**Claim.** Linear interpolation of number density across the 30-40 km gap overstates density by 25% at 33 km and 39% at 36 km, so placard_table.csv's q column is high by the same factor and its qdot column by ~12%/18%. Both errors are conservative (they overstate load and heating) and the T0 column is unaffected because temperature is genuinely linear in that layer. The defect is that README.md calls this 'benign' without quantifying it in a committed deliverable table.

**Code evidence.**

```
constants.rs:67-68: `(30_000.0, 3.827e23, 226.51, 301.7),` then `(40_000.0, 8.308e22, 250.35, 317.2),` — a 10 km gap.
model.rs:93-98: `let w = (alt_m - ft(a0)) / (ft(a1) - ft(a0)); return Ok(( ft(n0) + w * (ft(n1) - ft(n0)), ...` — linear in n.
At 36 km: w = 0.6, n = 3.827e23 − 0.6·2.9962e23 = 2.0293e23 → ρ = 0.0097609 kg/m³. USSA-1976 at 36 km (T = 228.65 + 2.8·4 = 239.85 K, p = 868.02·(228.65/239.85)^12.2013 = 484.4 Pa) gives ρ = 484.4/(287.053·239.85) = 0.0070354 kg/m³. Ratio 1.387.
placard_table.csv row `5,36,...`: `q = 11.80097370166` kPa; recomputing with true ρ gives 8.48 kPa.
README.md:101-104: `**The atmosphere interpolates linearly** between US-1976 rows spaced 5 to 10 km apart, which overstates density between rows ... For a placard demonstration the error is benign and conservative`
```

**Reference form.** US Standard Atmosphere 1976 (NOAA-S/T 76-1562): within the 32-47 km layer, p = p_b·(T_b/T)^(g0/(R·L)) with L = +2.8 K/km — density decays quasi-exponentially, not linearly. Linear interpolation across a 10 km span in this layer is a chord under a convex curve, hence a systematic overestimate.

**Impact.** The committed placard_table.csv is the artifact an engineer reads row by row. Its q column is 25% high at 33 km and 39% high at 36 km, and its q̇ column (∝√ρ) is 12% and 18% high. The error is conservative for a q-max placard but not for the heating column or for anyone reusing q as a load input. 'Benign' is not a defensible characterisation of a 39% error in a certification-adjacent table.

**Recommended fix.** Interpolate log(n) rather than n (a one-line change: `exp(ln(n0) + w*(ln(n1)-ln(n0)))`), which reduces the 36 km error to well under 1%. Failing that, add the 35 km USSA row to close the gap, and replace the README's 'benign' with the measured worst-case percentage at the worst grid point.

**Adversarial check.** I re-derived the reference independently and the auditor's numbers are right. constants.rs:67-68 does jump 30 km -> 40 km; model.rs:93-98 interpolates n linearly. At 36 km, w=0.6 gives n = 2.0293e23 -> rho = 9.761e-3 kg/m3. USSA-1976 in the 32-47 km layer (T_b=228.65 K at 32 km, L=+2.8 K/km, p_b=868.02 Pa, g0/(R*L)=12.201) gives T=239.85 K, p=484.4 Pa, rho=7.035e-3 — ratio 1.387. The 33 km point checks at 1.251. q at 36 km recomputes to 11.80 kPa (CSV 11.80097370166) with the interpolated density and ~8.5 kPa with the true one. The reference form (chord under a convex, quasi-exponentially decaying curve) is correct. Two corrections. (1) The IMPACT reasoning is wrong on the heating column: qdot ∝ sqrt(rho), so an overstated density overstates qdot too, which is conservative for a heating placard, not anti-conservative — the README's 'conservative' claim holds for both derived columns. (2) The T0 column is unaffected, because temperature in that layer is exactly linear in altitude, so the linear T interpolation is near-exact (240.81 K interpolated vs 239.85 K true). The genuine defect is that README.md's Limitations bullet says 'benign and conservative' without quantifying a 39% density error in a committed table — a disclosure gap in an example otherwise scrupulous about numbers, not an undisclosed physics error.

> Evidence re-read: flight_envelope_placard/constants.rs:60-70 ATMOSPHERE (30 km row 3.827e23, 40 km row 8.308e22 — a 10 km gap; all other gaps 5 or 7 km); model.rs:89-99 `let w = (alt_m - ft(a0)) / (ft(a1) - ft(a0))` with linear n, T and a; placard_table.csv row `5,36,11.80097370166,1444.884,9.148990622557834`; README.md Limitations bullet 'The atmosphere interpolates linearly ... the error is benign and conservative'; independent USSA-1976 32-47 km layer re-derivation.

---

### 5.6 [MINOR] VIV Strouhal numbers are crossing-count quantized at 1/110 but reported to 17 significant digits; the Re-trend is a quantization artifact

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `examples/avionics_examples/cfd/viv_resonance_margin/model.rs:72`
- **Auditor confidence:** confirmed

**Claim.** strouhal_number derives frequency by counting mean-crossings, so St is confined to integer multiples of 0.5/T_tail = 1/110 ≈ 0.00909. Two of the four committed rows are bit-identical and the other two are bit-identical to each other; the apparent St rise from Re 100 to Re 160 is exactly one crossing count. Neither the README nor constants.rs discloses the resolution.

**Code evidence.**

```
frequency.rs:45-48: `let periods = R::from_usize(crossings) * half; let total_time = R::from_usize(n - 1) * dt; periods / total_time`
model.rs:71-72: `let tail = &probe[probe.len() / 2..]; let strouhal = strouhal_number(tail, dt, ft(1.0), ft(1.0));`
constants.rs:53,57: `CFL: f64 = 0.4` and `STEPS: usize = 2200` → dt = 0.4/8 = 0.05, tail record length T = 55.0.
viv_resonance_margin.csv: rows 0.75 and 0.90 both read `0.18181818181818182` (= 20/110), rows 1.05 and 1.20 both read `0.19090909090909092` (= 21/110). Identical to the last bit.
constants.rs:68-70 claims a resolved trend: `Measured on this grid: St 0.1818 to 0.1909 across the four scheduled airspeeds, sitting on the unconfined laminar reference (Williamson: St rises from about 0.164 at Re 100 to about 0.185 at Re 160)`
```

**Reference form.** Williamson, Annu. Rev. Fluid Mech. 28:477 (1996): St rises from ≈0.164 at Re 100 to ≈0.185 at Re 160, a change of ≈0.021 — about 2.3 quantization steps of this estimator. A mean-crossing estimator resolves frequency to ±0.5 crossings over the record, i.e. ±1/(2T) = ±0.0091 here, which is ±5% of the reported value.

**Impact.** An engineer copying viv_resonance_margin.csv sees St resolved to 1e-17 and a clean monotone trend with Re. Both are illusory. The reported +10.9% offset at Re 100 (0.1818 vs Williamson's 0.164) is attributed in constants.rs to 'the coarse grid and the domain blockage', but at least half a quantization step (0.0045) of it is estimator resolution, and the estimator cannot distinguish Re 100 from Re 120 at all.

**Recommended fix.** Report St with an explicit uncertainty of ±1/(2·T_tail) and print it to 3 decimals in the CSV rather than 17. Document the resolution in constants.rs beside ST_BAND, and state that the four-airspeed trend is unresolved at this record length. To actually resolve the trend, lengthen the tail (STEPS) by ~3x or replace mean-crossing counting with a peak-interpolated periodogram.

**Adversarial check.** I re-derived the estimator from the library source and the auditor is exactly right. frequency.rs:23-49 counts mean-crossings and returns (crossings/2)/((n-1)*dt); model.rs:71-72 feeds it the second-half tail at dt = CFL/CELLS_PER_D = 0.4/8 = 0.05. The committed values are bit-exact rationals over 110: 0.18181818181818182 = 20/110 and 0.19090909090909092 = 21/110, which pins (n-1)*dt = 55.0 exactly and the resolution at 0.5/55 = 1/110 = 0.00909. Rows 0.75 and 0.90 m/s are bit-identical, rows 1.05 and 1.20 are bit-identical, and the entire reported Re 100 -> Re 160 'rise' of 0.0091 is precisely one crossing count. Williamson's ~0.164 -> ~0.185 over that Re range is ~2.3 steps of this estimator, so the trend is unresolvable on this record. constants.rs:65-70 presents 'St 0.1818 to 0.1909 across the four scheduled airspeeds' as measurement supporting a physical claim, and neither it nor README.md (lines 44, 51-53, 66) discloses the +/-0.0091 resolution. Severity: the finding is factual and material for anyone reading the CSV, but the gates it feeds are unaffected — ST_BAND is (0.16, 0.21), explicitly a dead-wake detector, and MARGIN_MIN=0.15 sits ~3 quantization steps below the worst measured margin of 0.236 — so this is a reporting/disclosure defect, not a failing physics gate.

> Evidence re-read: deep_causality_cfd/src/types/flow/frequency.rs:23-49 (dominant_frequency by mean-crossing; `periods / total_time` with `total_time = (n-1)*dt`) and :54-58 (strouhal_number); viv/model.rs:71-72 `let tail = &probe[probe.len()/2..]; let strouhal = strouhal_number(tail, dt, ft(1.0), ft(1.0));`; viv/constants.rs:31 CELLS_PER_D=8, :53 CFL=0.4, :57 STEPS=2200, :65-71 the ST_BAND docstring verbatim; viv_resonance_margin.csv all four rows; output.txt St column printed to 4 dp.

---

### 5.7 [MAJOR] Four retropulsion 'earned band' docstrings record measurements that contradict the committed output.txt

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/retropulsion/constants.rs:125`
- **Auditor confidence:** confirmed

**Claim.** constants.rs states each earned band was 'pinned from the first measured run' and records that measurement. For DRAG_COLLAPSE_MIN, FROZEN_DRAG_SEPARATION_MIN, BELIEF_SEPARATION_MIN_M and WINDOW_PREDICTION_TOL_S the recorded measurement does not match the committed output.txt, so the provenance chain from band to run is broken.

**Code evidence.**

```
constants.rs:125 `**Re-earned 2026-07-20: measured 0.178** (0.1310 at the 0.16 branch down to −0.0472 at 0.38)` vs output.txt `(4b) drag collapse: preserved drag falls 0.2510 -> -0.0611 across the 4 burning branches' realized throttles (collapse 0.3121, ...)`
constants.rs:115 `**Re-earned 2026-07-20: measured 147.3 m/s**` vs output.txt `(4c) ... depart the frozen-drag prediction by up to 139.3755 m/s`
constants.rs:144 `**Re-earned 2026-07-20: measured 23.44 m** (149.35 m informed against 125.91 m uninformed)` vs output.txt `(5) ... 23.45 m apart (153.19 m vs 129.75 m ...)`
constants.rs:164 `the flown window came in at 10.50 s onset and 59.60 s dwell ... errors of 0.04 s and 0.19 s` vs output.txt `dwell 59.50 s against 59.41 s (error 0.087 s)`
(By contrast FLOW_SPREAD_MIN's 'measured 0.0202' and MAX_BOND_GROWTH's 'Measured: 0' do match, and retropulsion/README.md:137 'by up to 139 m/s' matches — so the README is current and constants.rs is stale.)
```

**Reference form.** constants.rs:12 sets the standard itself: 'Bands marked *earned* were pinned from the first measured run and now gate regressions.' An earned band's recorded measurement must be reproducible from the committed artifact, otherwise the band's justification cannot be audited.

**Impact.** The DRAG_COLLAPSE_MIN docstring describes branches at realized throttles 0.16 and 0.38 — the pre-correction, envelope-clamped roster documented as superseded at constants.rs:53-56. An auditor cannot tell which of these bands were re-derived after the PLUME_S_REF_M2 correction and which were carried over unexamined. Since these constants are the acceptance criteria, stale provenance undermines every gate that reads them.

**Recommended fix.** Re-run the example and update the four docstrings to the values in the committed output.txt (0.3121, 139.4 m/s, 23.45 m / 153.19 vs 129.75, dwell 59.50 s / error 0.087 s). Consider adding a CI check that greps each 'measured X' docstring value out of constants.rs and asserts it appears in output.txt, so the two cannot drift again.

**Adversarial check.** I checked all six earned bands against the committed output. Four mismatch as claimed. (a) constants.rs:125 records 'measured 0.178 (0.1310 at the 0.16 branch down to -0.0472 at 0.38)'; output.txt gate (4b) reports 0.2510 -> -0.0611, collapse 0.3121, and the roster table shows realized throttles 0.20/0.40/0.60/0.7931 — the 0.16 and 0.38 branches described no longer exist. (b) constants.rs:115 'measured 147.3 m/s' vs output gate (4c) '139.3755 m/s'. (c) constants.rs:144 'measured 23.44 m (149.35 m informed against 125.91 m uninformed)' vs output gate (5) '23.45 m apart (153.19 m vs 129.75 m)' — the aggregate nearly matches but both components moved by ~4 m. (d) constants.rs:162 'flown window came in at 10.50 s onset and 59.60 s dwell ... errors of 0.04 s and 0.19 s' vs output gate (1) 'dwell 59.50 s against 59.41 s (error 0.087 s)'. The control cases hold: FLOW_SPREAD_MIN's 0.0202 and MAX_BOND_GROWTH's 0 both match output exactly, and README.md:137's '139 m/s' matches, confirming the README is current while constants.rs is stale. constants.rs:11-12 sets the standard the file fails ('Bands marked *earned* were pinned from the first measured run and now gate regressions'). Note the auditor's line cite for (d) is 164; the text is at 162. No gate is at risk — every band still bounds its measured value with margin — so the defect is a broken audit trail on the acceptance criteria, not a wrong threshold.

> Evidence re-read: retropulsion/constants.rs:11-12, :78 (MAX_BOND_GROWTH 'Measured 2026-07-20: 0'), :101 (FLOW_SPREAD_MIN 0.0202), :115, :125, :144, :162; retropulsion/output.txt gates (4a),(4b),(4c),(5),(1) and the mid-burn roster table showing realized throttles 0.0000/0.2000/0.4000/0.6000/0.7931; retropulsion/README.md:137 'by up to 139 m/s'.

---

### 5.8 [MAJOR] 'One continuous descent' is contradicted by the corridor's own provenance log, which records three fluid-state re-seeds

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/README.md:15`
- **Auditor confidence:** confirmed

**Claim.** Three documents claim the corridor flies one continuous descent. The committed output.txt shows the marched fluid state is discarded and re-seeded from the world seed at each of the three leg boundaries; only the coupled field (navigation, log, projections) is carried.

**Code evidence.**

```
corridor/README.md:15: `This example flies that corridor as **one continuous descent** in a single composed coupling:`
corridor/main.rs:12: `This example flies that corridor as **one continuous** descent** on the compressible carrier`
examples/avionics_examples/README.md:24: `One continuous Mach-25 descent through plasma blackout`
corridor/output.txt provenance section, three separate entries: `leg re-seeded from step 119 in world 'fine_bank_02': coupled field carried, marched fluid state re-seeded from the world seed`, and the same at step 108 and step 476.
```

**Reference form.** For a marched flow solution, 'continuous' means the fluid state at the end of segment n is the initial state of segment n+1. Re-seeding from a uniform world seed resets the evolved layer and restarts its convergence, which is the opposite of continuity in the quantity the example exists to march.

**Impact.** The continuity claim is what distinguishes this example from a sequence of independent snapshots, and it is the basis for treating leg-2's electron density as the descent's peak. An engineer sizing confidence in the reported n_e, heat flux and dwell from 'one continuous descent' is relying on a property the run does not have. Note the log itself is honest — only the prose is not.

**Recommended fix.** Rewrite the claim to match the log: 'one continuous coupled descent — the navigation state, provenance log and evolved projections are carried across leg boundaries; the marched fluid layer is re-seeded at each pause.' Fix all three locations. If true fluid continuity is intended, carry the marched state across `.from(pause.state())` instead of re-seeding.

**Adversarial check.** Confirmed by the library source, which is stronger evidence than the log line the auditor cited. coupled_march.rs:88-95, the docstring on the very `from()` the corridor calls at main.rs:131/147/159, states: 'A leg boundary carries the coupled field, not the marched fluid layer. ... the incoming leg rebuilds its carrier and re-quantizes the world's uniform seed, so the evolved conserved state, the inflow strip, any acoustic-envelope drift the previous leg earned, its rebuild count, and its plume-imprint budget are all discarded.' The three log entries at steps 119, 108 and 476 in corridor/output.txt are the emission of that path (coupled_march.rs:110-114). So the fluid layer — the quantity the example exists to march — restarts three times, while navigation, truth trajectory, projections and the EffectLog are genuinely carried. corridor/README.md:15 and main.rs:12 both say 'one continuous descent'; examples/avionics_examples/README.md:24 repeats it. The library itself calls the accepted defense quasi-steady re-convergence 'within a few steps', which mitigates but does not restore continuity. The auditor is right that the log is honest and only the prose is not.

> Evidence re-read: deep_causality_cfd/src/types/flow/coupled_march.rs:85-122 (the `from(MarchState)` docstring and the re-seed log emission at :110-114); corridor/main.rs:131 `.from(onset.state())`, :147 `.from(peak.state())`, :159 `.from(exit_pause.state())`; corridor/output.txt provenance section, three 'leg re-seeded from step {119,108,476} in world fine_bank_02' entries; corridor/README.md:15; corridor/main.rs:12; examples/avionics_examples/README.md:24.

---

### 5.9 [MINOR] Corridor gates 'peak n_e' at a sampled instant 14x below the descent-wide maximum the same baseline produces

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:527`
- **Auditor confidence:** likely

**Claim.** Gate (2) correctly compares the corridor's 61 km station peak against a station-scoped RAM-C II anchor, so it is not mismeasuring the named quantity. However, the weather example — same coupling, byte-identical atmosphere — records a descent-wide n_e maximum of 4.734e20 (47x the anchor) at a step no corridor gate samples, and neither example's documentation acknowledges or explains the 14x gap. That is an undocumented cross-example inconsistency worth a note, not a broken gate.

**Code evidence.**

```
corridor/model.rs:527-531: `let ne_ok = (ft(RAMC_NE_REFERENCE / 5.0)..=ft(RAMC_NE_REFERENCE * 5.0)).contains(&l.leg2.ne_peak);` with detail string `"peak n_e vs the RAM-C II anchor"`.
corridor/model.rs:355-358 (snapshot): `ne_peak: field.scalar("n_e").map(utils::peak)` — peak over cells at the pause instant only.
shared/stages.rs:193-199 (WeatherTelemetry): `let ne = field.scalar("n_e").map(utils::peak)...; if ne > utils::scalar0(field, "wx_ne_max") { field.set_scalar("wx_ne_max", Vec::from([ne])); }` — same per-instant peak, but maximized over every step.
corridor/output.txt: `n_e = 3.349e19 m^-3 at the 60.9 km passage, in [2.0e18, 5.0e19]`
weather/weather_table.csv standard_day row: `ne_max = 473379665202666500000` = 4.734e20.
shared/world.rs:48-50: `pub fn standard_atmosphere() -> Vec<AtmosphereRow<FloatType>> { weather_atmosphere(0.0, 1.0) }` — the corridor and weather standard_day fly the identical table.
```

**Reference form.** RAM-C II flight measurement, peak electron density ≈1e19 m⁻³ near 61 km (the value shared/constants.rs:73 cites). A gate named 'peak n_e vs the flight anchor' must compare the descent's maximum, not the value at whichever step an altitude predicate happened to fire.

**Impact.** The corridor's headline physics gate passes at 3.3x the flight anchor while the same physics, marched continuously, reaches 47x it. Gate (2) would fail if it read the true peak. Because the corridor re-seeds its fluid state at each leg boundary (see the continuity finding), leg-2's value is measured on a partially re-converged layer, which is plausibly why the two differ — but either way the quantity gated is not the quantity named.

**Recommended fix.** Accumulate a running maximum of n_e across the corridor's legs (the WeatherTelemetry stage already does exactly this and is available in shared) and gate that value, renaming the current one 'n_e at the 61 km passage' if it is still worth printing. Then re-derive the acceptance band against the true peak, and reconcile the corridor and weather numbers before either is presented as validated against RAM-C II.

**Adversarial check.** The numbers are all real and I confirmed the two examples fly a byte-identical atmosphere (world.rs:48-50 `standard_atmosphere() = weather_atmosphere(0.0, 1.0)`), so the 3.349e19 vs 4.734e20 gap is genuine and undocumented. But the claimed DEFECT rests on a reference form that is wrong. RAM-C II's anchor is not a descent-wide maximum: shared/constants.rs:72-73 defines it as 'The RAM-C II ~61 km peak electron density anchor' — the peak *over the sheath* at the ~61 km station, which is exactly what `field.scalar("n_e").map(utils::peak)` computes at the 60.9 km pause. The gate's own detail string names the station ('at the 60.9 km passage'), and README.md:64-67 scopes the claim to 'the 61 km RAM-C II station'. So the quantity gated IS the quantity named, and gating a descent-wide maximum against a station measurement would be the error. What survives is a real, unexplained cross-example inconsistency: the same coupling on the same atmosphere accumulates a descent-wide n_e maximum 47x the flight anchor at some step neither example reports, and no document acknowledges it. The auditor's own alternative explanation — the corridor's leg-2 layer is only 108 steps past a re-seed while weather marches 850 steps unbroken — is plausible and would itself be worth a note.

> Evidence re-read: corridor/model.rs:527-544 gate_anchor (band is RAMC_NE_REFERENCE/5 ..= *5, detail names the passage altitude); :355-358 snapshot ne_peak; shared/stages.rs WeatherTelemetry running max into 'wx_ne_max'; shared/constants.rs:72-73 `/// The RAM-C II ~61 km peak electron density anchor, m^-3.` `pub const RAMC_NE_REFERENCE: f64 = 1.0e19;`; shared/world.rs:48-50; corridor/output.txt gate (2); weather/weather_table.csv standard_day ne_max; corridor/README.md:64-67 and :182-186.

---

### 5.10 [MINOR] Nozzle area-Mach gate applies A* = throat area to unchoked rows, producing a spurious failure if a user adds a back pressure above the first critical ratio

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `examples/avionics_examples/cfd/nozzle_operating_map/model.rs:86`
- **Auditor confidence:** confirmed

**Claim.** area_mach_dev is computed for every shock-free row, including subsonic-throughout rows where the duct never chokes. For those rows A* is not the throat area and the supersonic-branch selection downstream of the throat is wrong, so the deviation is ~0.5 and gate_area_mach fails on physically correct output.

**Code evidence.**

```
model.rs:86-97: `let area_mach_dev = if shock_x.is_none() { ... let ratio = area_at(*xi) / ft(THROAT_AREA_M2); let analytic = mach_from_area_ratio(ratio, *xi > throat_x);`
duct_march_run.rs:317: `if max_mach > R::one() && exit_mach < R::one() {` — the shock series is emitted only in the shocked regime, so an unchoked subsonic run also reports `shock_x = None`.
model.rs:300-305 (gate_area_mach): `if row.shock_x.is_none() && let Some(dev) = row.area_mach_dev && dev > ft(AREA_MACH_BAND)` — no choking test.
output.txt prints `first critical p/p0 = 0.9372`; the committed schedule tops out at 0.90, so the path is latent, not currently exercised.
```

**Reference form.** The area-Mach relation A/A* = f(M) is referenced to the sonic area A*. For an unchoked converging-diverging duct the throat Mach is below 1, so A* < A_throat and A_throat/A* > 1; using A* = A_throat is invalid. The flow is also subsonic on both sides of the throat, so the `xi > throat_x` supersonic-branch flag is wrong there too.

**Impact.** The example's stated purpose is that engineers do not waste time debugging tools. A user who adds `0.98` to back_pressures.csv — the single most natural experiment, since the README itself names the unchoked window — gets a red gate on a physically correct run and no diagnostic pointing at the cause. The README's Limitations note says only that the window 'is not swept', not that sweeping it misfires the gate.

**Recommended fix.** Guard the computation on choking: compute `area_mach_dev` only when `p_ratio < subsonic_exit_pressure_ratio()`, and set it to `None` otherwise (the gate already skips `None`). Add a line to the README Limitations explaining that unchoked rows are excluded from the area-Mach gate because A* is undefined there.

**Adversarial check.** The code path and the physics both check out. model.rs:86-97 computes area_mach_dev whenever `shock_x.is_none()`, with `ratio = area_at(xi) / THROAT_AREA_M2` and the supersonic branch selected by `xi > throat_x`. duct_march_run.rs emits the shock series only under `max_mach > 1 && exit_mach < 1`, so a fully subsonic (unchoked) run also reports `shock_x = None` and falls into that branch. gate_area_mach (model.rs:300-305) has no choking test. The reference form is correct: A/A* is referenced to the sonic area, and for an unchoked converging-diverging duct the throat Mach is below 1 so A* < A_throat, making A_throat/A* > 1 — using A* = A_throat understates the true area ratio on both sides, and selecting the supersonic branch downstream of the throat is simply wrong for a flow that is subsonic throughout. The path is reachable by an ordinary user: main.rs:69-77 `in_path()` takes an arbitrary CSV as argv[1], and the run itself prints 'first critical p/p0 = 0.9372' while the committed schedule stops at 0.90. Severity moderated: it is latent, not exercised by any committed artifact, and README.md:78-79 does say 'the unchoked window above the first critical ratio is not swept' — it just does not say that sweeping it misfires the gate.

> Evidence re-read: nozzle_operating_map/model.rs:86-106 (area_mach_dev guarded only by `shock_x.is_none()`), :96-97 (ratio against THROAT_AREA_M2 and `*xi > throat_x`), :120-129 area_at, :132-153 mach_from_area_ratio; :299-318 gate_area_mach with no choking predicate; deep_causality_cfd/src/types/flow/duct_march_run.rs `if max_mach > R::one() && exit_mach < R::one()` guarding the shock_position series; nozzle main.rs:69-77 in_path() accepting argv[1]; back_pressures.csv (max 0.90); output.txt 'first critical p/p0 = 0.9372'; README.md:78-79.

---

### 5.11 [MINOR] turbulence_flow README states the horizon law with attractor scale L≈10 while the code uses L=1, and the README's own numbers use L=1

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `examples/avionics_examples/cfd/turbulence_flow/README.md:33`
- **Auditor confidence:** confirmed

**Claim.** The README's displayed equation says the roundoff seed 'reaches the scale of the attractor (L ≈ 10)', but Report::horizon_law computes −ln(ε)/λ, i.e. L = 1, and its own docstring says L ≈ 1. The README's tabulated horizons are computed with L = 1, contradicting its own prose by ln(10)/λ = 2.54 time units.

**Code evidence.**

```
README.md:33-37: `That seed of size ε grows like e^{λ t} and reaches the scale of the attractor (L ≈ 10) at` ... `t_horizon ≈ ln(L / ε) / λ`
model.rs:198-202: `/// The horizon the digit-count law predicts for a given machine epsilon: ln(L/ε) / λ, with the attractor scale L ≈ 1 at the unit threshold. pub fn horizon_law(epsilon: f64) -> f64 { -epsilon.ln() / LYAPUNOV }`
README.md:82-84 tabulates `f32 (ε≈1.2e-7): t ≈ 17.6`, `f64 (ε≈2.2e-16): t ≈ 39.8`, `F106 (ε≈1.0e-32): t ≈ 81.3` — all of which reproduce with L = 1 (−ln(2.2e-16)/0.906 = 39.79) and none with L = 10 (which would give 42.3).
```

**Reference form.** The predictability-horizon estimate is t ≈ ln(L/ε)/λ where L is the separation threshold at which the forecast is declared lost. The code's threshold is 1.0 (main.rs:90 `let threshold = 1.0;`), so L = 1 is the internally consistent choice and the README's L ≈ 10 is simply wrong for this program.

**Impact.** A reader who takes the README's equation at face value and recomputes the horizons gets numbers 2.54 higher than the ones printed immediately below, with no way to tell which is authoritative.

**Recommended fix.** Change README.md:33 to state the one-state-space-unit threshold the code actually uses ('reaches the unit separation threshold, L = 1') and note that the Lorenz attractor diameter is ~40, so the horizon quoted is conservative. Alternatively raise the code's threshold to the attractor scale and update both.

**Adversarial check.** Both texts verified verbatim and the arithmetic settles it. README.md:32-37 says the seed 'reaches the scale of the attractor (L ≈ 10)' at t ≈ ln(L/eps)/lambda. model.rs's Report::horizon_law docstring says 'with the attractor scale L ≈ 1 at the unit threshold' and the body is `-epsilon.ln() / LYAPUNOV`, i.e. L = 1. I recomputed the README's own table at lines 82-84: -ln(1.2e-7)/0.906 = 17.59, -ln(2.2e-16)/0.906 = 39.79, -ln(1e-32)/0.906 = 81.32 — matching the printed 17.6 / 39.8 / 81.3 to the digit. With L = 10 every value would rise by ln(10)/0.906 = 2.54 to 20.1 / 42.3 / 83.9, matching nothing. main.rs:90 `let threshold = 1.0;` confirms L = 1 is the program's actual separation threshold, so the auditor's corrected reference form is the right one.

> Evidence re-read: turbulence_flow/README.md:32-37 and :81-85; model.rs Report::horizon_law docstring ('the attractor scale `L ≈ 1` at the unit threshold') and body `-epsilon.ln() / LYAPUNOV`; model.rs LYAPUNOV = 0.906; main.rs:90 `let threshold = 1.0;`; independent recomputation of all three tabulated horizons at both L values.

---

### 5.12 [MINOR] turbulence_flow violates the repo file-layout convention, has no committed output.txt, and no gates

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `examples/avionics_examples/cfd/turbulence_flow/main.rs:43`
- **Auditor confidence:** confirmed

**Claim.** Alone among the six CFD examples, turbulence_flow names its printing module print_utils.rs (not utils_print.rs), has no constants.rs or model_config.rs, inlines every load-bearing numeric in main(), commits no output.txt, and registers no gates — main() returns () with no exit code.

**Code evidence.**

```
main.rs:43-47: `let dt = 0.005_f64;` `let ic = [1.0_f64, 1.0, 1.0];` `let sample_dt = 0.5_f64;` `let samples = 120usize;` `let steps_per_sample = (sample_dt / dt) as usize;`
main.rs:90: `let threshold = 1.0;` — the constant that determines every reported forecast horizon, undocumented and unnamed.
main.rs:31: `mod print_utils;` — every other CFD example declares `mod utils_print;`
main.rs:37: `fn main() {` — no `-> ExitCode`, unlike all five siblings.
Directory listing: turbulence_flow contains only README.md, main.rs, model.rs, print_utils.rs — no output.txt, no constants.rs, no model_config.rs.
```

**Reference form.** The repo convention (and the layout every other example in this directory follows) is main / model_config / model / model_types / utils_print / constants, with a committed output.txt as the reproducible reference run. The five sibling CFD examples all exit nonzero on gate regression.

**Impact.** The README's 'Sample output' block (lines 53-67) is the only record of what the program prints, and it does not match what the code produces: analyse() emits rows at t = 5,10,15,...,55 (11 rows) while the README shows 6 with 15, 30, 35, 50, 55 silently elided. With no committed output.txt there is nothing to diff a regression against. The `threshold = 1.0` literal sets all three reported horizons and is invisible to anyone scanning for constants.

**Recommended fix.** Rename print_utils.rs to utils_print.rs; move dt, ic, sample_dt, samples and especially threshold into a constants.rs with justifications; commit an output.txt from an actual run and replace the hand-elided README block with it. Consider whether this example should carry gates like its siblings — at minimum an exit code.

**Adversarial check.** Every factual assertion checks out. The directory contains exactly README.md, main.rs, model.rs, print_utils.rs — no output.txt, no constants.rs, no model_config.rs, no model_types.rs. main.rs:31 declares `mod print_utils;` where all five siblings use `utils_print`. main.rs:37 is `fn main() {` with no `-> ExitCode`, unlike every sibling. main.rs:43-47 inlines dt, ic, sample_dt, samples verbatim as quoted, and main.rs:90 `let threshold = 1.0;` is the unnamed literal that sets all three reported horizons. No GateSeq exists. The README mismatch is real: analyse() (main.rs:94-106) walks t = 5.0 to 55.0 in steps of 5.0, emitting 11 rows, while README.md:53-67 shows 6 (5, 10, 20, 25, 40, 45), eliding 15, 30, 35, 50, 55. One mitigating fact the auditor did not note: this example is not a CfdFlow study at all — it uses CausalFlow/PropagatingEffect and Rk4 and does not touch deep_causality_cfd — so the gate-and-exit-code convention is arguably inapplicable, though the constants-file and utils_print naming conventions and the missing reference output still stand.

> Evidence re-read: turbulence_flow/ directory listing (4 files, no output.txt); main.rs:30-31 `mod model; mod print_utils;`; main.rs:37 `fn main() {`; main.rs:42-47 all five inlined literals verbatim; main.rs:89-109 analyse() emitting t=5..55 step 5; main.rs:90 `let threshold = 1.0;`; README.md:53-67 sample-output block with 6 rows; main.rs:33-35 imports CausalFlow/PropagatingEffect, not CfdFlow.

---

### 5.13 [MINOR] Corridor counts carrier rebuilds by grepping a rendered log, which the library docstring explicitly tells callers not to do

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:302`
- **Auditor confidence:** confirmed

**Claim.** rebuild_count tallies the substring 'carrier rebuilt at step' out of a rendered provenance log to feed gate (5a). The library provides CompressiblePause::rebuilds() and its docstring names this exact substring-tally as the thing not to do. The retropulsion example does it correctly, so the corridor is the outlier.

**Code evidence.**

```
corridor/model.rs:302-307: `pub fn rebuild_count(rendered: &str) -> usize { rendered.lines().filter(|l| l.contains("carrier rebuilt at step")).count() }`
deep_causality_cfd/src/types/flow/carrier.rs:453-455: `/// Read this rather than tallying "carrier rebuilt at step" substrings in a rendered log: the count is per-carrier, so it is a **per-leg** number, and a carrier is rebuilt at every leg boundary.`
retropulsion/output.txt gate 8, doing it right: `1 carrier rebuild(s) across all legs (cap 6), read from the pause accessor rather than a log tally.`
```

**Reference form.** A gate should read a typed accessor whose value the producing code guarantees, not re-parse a human-readable rendering whose format is not part of any API contract.

**Impact.** These are the files engineers copy. The pattern silently breaks if the log message is ever reworded — the count drops to 0 and gate (5a) (`rebuilds <= 3`) passes vacuously with no signal. The corridor's committed run reports 0 rebuilds, which is indistinguishable from a broken match.

**Recommended fix.** Replace rebuild_count with the accessor: sum `pause.rebuilds()` over the four legs, mirroring what retropulsion already does. Note the docstring's warning that the count is per-leg, so the legs must be summed rather than read from the last pause.

**Adversarial check.** Both quotes are verbatim at the cited locations. corridor/model.rs:302-307 is exactly `rendered.lines().filter(|l| l.contains("carrier rebuilt at step")).count()`, fed to gate (5a) via main.rs:173. carrier.rs:453-455 says verbatim 'Read this rather than tallying "carrier rebuilt at step" substrings in a rendered log', and carrier.rs:64 repeats the instruction on the field itself. Retropulsion does it right — its gate (8) detail says 'read from the pause accessor rather than a log tally'. The fragility is real: the emitting site is compressible_march_run.rs:386 and a reworded message silently drops the count to 0, which the corridor's committed run already reports, making a broken match indistinguishable from a clean one. One nuance the auditor missed and that a fix must handle: `CarrierPause::rebuilds()` is documented as per-leg, so the corridor spanning four legs would need to sum four accessor reads rather than a single call — the log tally is at least answering the cumulative question. That makes it a defensible-but-fragile workaround rather than a plain mistake.

> Evidence re-read: corridor/model.rs:301-307 verbatim; corridor/main.rs:167,173 `let rendered_log = format!("{}", reacq.field().log());` -> `rebuilds: model::rebuild_count(&rendered_log)`; corridor/model.rs:603-612 gate_rebuilds against MAX_REBUILDS=3; deep_causality_cfd/src/types/flow/carrier.rs:64 and :450-458 (the `rebuilds()` accessor and its 'read this rather than tallying' docstring, including 'the count is per-carrier, so it is a per-leg number'); compressible_march_run.rs:386 the emitting format string; retropulsion/output.txt gate (8).

---

### 5.14 [INFO] Corridor 'miss (t2 x-check)' column is constant across every branch and measures a different quantity than the miss it claims to cross-check

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/corridor/model.rs:152`
- **Auditor confidence:** confirmed

**Claim.** The miss_t2 column is constant at 1.3191 m across all 17 branches because |b| and the branch dwell are both invariant. It estimates INS dead-reckoning drift, not guidance miss, so calling it a 'cross-check' beside miss_traj is loose wording — but the docstring names it a 't²-law dead-reckoning proxy', no gate reads it, and its constancy correctly reflects that every branch stayed denied for the full continuation.

**Code evidence.**

```
model.rs:152-154 (docstring): `/// The t²-law dead-reckoning proxy, printed as a cross-check beside the trajectory-derived miss: ½·|b|·dwell² with |b| the accelerometer-bias magnitude.`
model.rs:246-247: `let bias: [FloatType; 3] = core::array::from_fn(|i| ft(IMU_ACCEL_BIAS[i])); let t2_miss_m = ft(0.5) * norm3(bias) * dwell * dwell;`
Dwell is BRANCH_STEPS·DT_FLIGHT = 100·0.1 = 10.0 s for every branch (output.txt shows `10.00 s` on all 17 rows; CSV shows `9.99999999999998`). |b| = sqrt(0.02²+0.014²+0.01²) = 0.026382, so t2_miss = 0.5·0.026382·100 = 1.3191 m — exactly the value in every CSV row.
output.txt: the traj miss spans 2.39 m to 22.02 m while the 'x-check' column reads 1.3191 m on every line.
```

**Reference form.** A cross-check must be an independent estimate of the same quantity. ½·|b|·t² estimates inertial position drift under constant accelerometer bias; |terminal − aim| estimates guidance miss with perfect state knowledge. Neither bounds nor predicts the other.

**Impact.** An engineer scanning corridor_branches.csv sees a 'miss_t2' column that never varies and disagrees with 'miss_traj' by up to 16x, with the docstring asserting the two are cross-checks. The branch 'dwell' column is likewise saturated at the branch horizon for all rows, so it carries no discriminating information either.

**Recommended fix.** Rename the column to what it is (e.g. 'ins_drift_t2_m') and drop the 'cross-check' framing from the docstring and the printed header. Note in the README that dwell is the branch horizon, not a flow-resolved dwell, so both columns are branch-invariant by construction.

**Adversarial check.** The arithmetic is exact. IMU_ACCEL_BIAS = [2.0e-2, -1.4e-2, 1.0e-2] (shared/constants.rs:189), |b| = 0.0263818, dwell = BRANCH_STEPS*DT_FLIGHT = 10 s on every branch, so t2_miss = 0.5*0.0263818*100 = 1.319091 — matching 1.319090595827292 in all 17 CSV rows. The two quantities are indeed different: one is INS dead-reckoning drift under constant accelerometer bias, the other is guidance miss to the aim under perfect state knowledge, and neither bounds the other. But the code does not claim otherwise. model.rs:151-153 labels it 'The t²-law dead-reckoning proxy' — it names what it is — and README.md:160 calls it 'the analytic t^2 drift law printed beside it'. 'Cross-check' is loose phrasing on a correctly-labeled diagnostic, and nothing is gated on the column, so the 'tautology-circular' axis does not apply. The constancy is also informative rather than degenerate: it reflects that every branch stayed GNSS-denied for the entire 100-step continuation, which is a real property of the fork.

> Evidence re-read: corridor/model.rs:151-153 the docstring verbatim; :246-247 `let bias ...; let t2_miss_m = ft(0.5) * norm3(bias) * dwell * dwell;`; :243-245 dwell read from the report's blackout_dwell series; shared/constants.rs:189 IMU_ACCEL_BIAS; corridor/constants.rs:18 BRANCH_STEPS=100 and shared DT_FLIGHT=0.1; corridor_branches.csv all 11 fine rows at 1.319090595827292 with miss_traj spanning 2.393-6.144; output.txt 17 rows; corridor/README.md:154-160.

---

### 5.15 [MINOR] S_REF docstring claims the rebuild-on-drift mechanism fires during the descent; the committed run records zero rebuilds

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `examples/avionics_examples/src/shared/constants.rs:49`
- **Auditor confidence:** confirmed

**Claim.** S_REF is documented as 'deliberately snug' so that the peak-station inflow outgrows it once and the rebuild mechanism fires. The corridor's committed output reports 0 rebuilds, so the stated design intent is not realised in the run that is checked in.

**Code evidence.**

```
shared/constants.rs:48-50: `/// Reference wave speed of the implicit acoustic envelope. Deliberately snug: the peak-station inflow outgrows it once, so the rebuild-on-drift mechanism fires where the descent steepens.` `pub const S_REF: f64 = 1.8;`
corridor/output.txt: `[PASS] (5a) bounded schedule rebuilds: 0 carrier rebuild(s) while following the descent (cap 3)`
The corridor provenance log contains no 'carrier rebuilt at step' line at all, whereas retropulsion's does (`carrier rebuilt at step 1: s_ref 1.4 -> 2.7683613571273638 (rebuild 1)`), confirming the log message is emitted when the mechanism does fire.
```

**Reference form.** A constant's docstring describing an observable behaviour ('the mechanism fires') must match the behaviour of the committed reference run.

**Impact.** Gate (5a) is presented as evidence that rebuilds stay bounded, but with zero rebuilds it demonstrates only that the mechanism never engaged. The docstring leads a reader to believe the rebuild path is exercised by this example when it is not — so the path is untested here despite appearing covered. (The corridor's substring-grep counting method, reported separately, means a zero could also indicate a broken match.)

**Recommended fix.** Either update the docstring to say the mechanism does not fire on the corridor's current profile (and point to retropulsion, which does exercise it), or retune S_REF so the described behaviour actually occurs. Switching gate (5a) to the typed accessor first would remove the ambiguity about whether zero is real.

**Adversarial check.** Quotes verified verbatim and I strengthened the finding by checking the second consumer the auditor did not. shared/constants.rs:48-50 says S_REF is 'Deliberately snug: the peak-station inflow outgrows it once, so the rebuild-on-drift mechanism fires where the descent steepens.' The corridor's committed output reports 0 rebuilds and its provenance log contains no 'carrier rebuilt at step' line. I then checked the weather example, which flies the same S_REF = 1.8 for 850 coupled steps in ONE continuous leg with no re-seed to reset envelope drift — the most favourable case for the docstring's claim — and grepped all 49 committed audit logs: zero 'carrier rebuilt' entries anywhere, on logs that are otherwise populated (regime transitions present). So the stated behaviour is unrealised in every committed artifact that flies this constant. The message is demonstrably emitted when the mechanism does fire (retropulsion's terminal leg at S_REF_TERMINAL = 1.4 logs 'carrier rebuilt at step 1: s_ref 1.4 -> 2.768'), so this is not a missing-log artifact.

> Evidence re-read: examples/avionics_examples/src/shared/constants.rs:47-50 verbatim; corridor/output.txt gate (5a) '0 carrier rebuild(s)' and the full 16-entry provenance block with no rebuild line; weather/audit/*.log — `grep -l "carrier rebuilt"` returns nothing across all 49 files, while `grep -c "regime ->"` returns 4 on a sample log, confirming the logs carry EffectLog content; retropulsion/output.txt 'carrier rebuilt at step 1: s_ref 1.4 -> 2.7683613571273638 (rebuild 1)'.

---

### 5.16 [MINOR] Retropulsion is documented as forking a 'plume-coupled' state, while its own constants record that the marched layer carries no plume imprint

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `examples/avionics_examples/cfd/plasma_blackout/retropulsion/constants.rs:108`
- **Auditor confidence:** confirmed

**Claim.** Three documents describe the retropulsion fork as a fork of the marched, plume-coupled state. constants.rs records that because the Cordell-Braun plume-geometry model's validity band (Mach 2-4) does not overlap the burn (Mach 0.4-2.0), the marched layer carries no plume imprint at all through the burn.

**Code evidence.**

```
retropulsion/constants.rs:108-111: `The band's own limitation is unchanged and documented at CORDELL_MACH_MIN: with the plume geometry outside the Cordell-Braun envelope through the burn, the marched layer carries no plume imprint, so this spread is throttle -> trajectory -> post-shock density rather than a plume footprint.`
shared/constants.rs:345-353 confirms the non-overlap: `Jarvinen-Adams measured drag preservation over **Mach 0.4-2.0** ... Cordell-Braun validated the analytic plume boundary over **Mach 2-4**. ... They meet at a single point.`
plasma_blackout/README.md:20: `forks the marched plume-coupled state, and lands`; :66-67: `choosing a throttle means flying candidate throttles from the same marched, plume-coupled instant.`
retropulsion/README.md:63: `**The fork.** The marched, plume-coupled state is forked in O(1) through copy-on-write`
examples/avionics_examples/README.md:26: `forks the marched, plume-coupled state mid-burn`
```

**Reference form.** A state described as 'plume-coupled' should carry the plume's influence on the marched field. Here the coupling is throttle → trajectory → post-shock density; the plume mask is never imprinted because its geometry model stands down outside Mach 2-4.

**Impact.** The state-fork-vs-parameter-sweep argument is the retropulsion example's central claim, and 'plume-coupled' is what makes it a state fork rather than a trajectory fork. The distinction still holds (the marched flow state is genuinely forked), but the plume qualifier overstates what is in that state. constants.rs is candid; the three READMEs are not.

**Recommended fix.** Replace 'plume-coupled' with an accurate description in all three READMEs — e.g. 'the marched, burn-coupled state (the plume geometry stands down outside the Cordell-Braun Mach 2-4 band, so the layer carries the burn through trajectory and post-shock density rather than a plume imprint)' — and cross-reference the CORDELL_MACH_MIN note.

**Adversarial check.** All quotes verified verbatim. retropulsion/constants.rs:108-111 states that with the plume geometry outside the Cordell-Braun envelope through the burn 'the marched layer carries no plume imprint, so this spread is throttle -> trajectory -> post-shock density rather than a plume footprint.' shared/constants.rs:339-353 documents the non-overlap of Jarvinen-Adams (Mach 0.4-2.0) and Cordell-Braun (Mach 2-4): 'They meet at a single point.' The 'plume-coupled' phrasing is in fact more widespread than the three places cited — it also appears in retropulsion/main.rs:24 and :151, utils_print.rs:94, and is printed into output.txt:21 itself. One qualification in the code's favour: the marched layer is not wholly plume-independent, since PlumeObstruction's drag decrement changes the trajectory, which changes the freestream feeds driving the inflow strip. So the state genuinely differs per throttle — the fork is a real state fork, as the auditor concedes — but the differences are aerodynamic and trajectory-mediated, not a plume imprint on the layer. 'Plume-coupled' overstates by exactly one link in the chain. constants.rs is candid; the prose is not.

> Evidence re-read: retropulsion/constants.rs:101-111 verbatim; shared/constants.rs:339-353 the two-envelope block ending 'They meet at a single point'; :354-358 CORDELL_MACH_MIN/MAX = 2.0/4.0; shared/world.rs:386-391 PlumeObstruction `.with_geometry_mach_band(CORDELL_MACH_MIN, CORDELL_MACH_MAX)`; shared/world.rs:516-522 the plume_imprint docstring ('Without it ... the marched layer never learns the plume exists'); plasma_blackout/README.md:20,66; retropulsion/README.md:21,63; examples/avionics_examples/README.md:26; retropulsion/main.rs:24,151; retropulsion/output.txt:21.

---

### 5.17 [INFO] Stagnation-temperature placard of 1700 K is justified as 'nickel-alloy class', which no nickel superalloy supports

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `examples/avionics_examples/cfd/flight_envelope_placard/constants.rs:52`
- **Auditor confidence:** confirmed

**Claim.** T0_MAX_PLACARD_K is a fully traceable, explicitly-labeled demonstration value sized 13 percent above the recorded 1502 K peak — not an untraceable magic number. The defect is wording only: 1700 K bounds the post-shock gas stagnation temperature, not a structural temperature, so naming a nickel-alloy service class beside it invites a category error. A structural-temperature reading of the number would indeed be unphysical.

**Code evidence.**

```
constants.rs:48-52: `/// Post-shock stagnation-temperature placard, K. A chosen demonstration ceiling for an uncooled hot-structure leading edge (nickel-alloy class), not certification data: the recorded matrix peaks near 1502 K at M 5.0 / 40 km, so 1700 K bounds the corridor with about 13 percent margin.` `pub const T0_MAX_PLACARD_K: f64 = 1700.0;`
The arithmetic confirms the back-fit: 1502 × 1.13 = 1697 ≈ 1700, i.e. the bound was placed 13% above the measured peak.
output.txt: `[PASS] stagnation temperature: max T0 = 1502.1 K at M 5.00 / 40.0 km, inside the 1700 K placard`
```

**Reference form.** Nickel-base superalloy service limits: Inconel 718 ≈ 980 K, single-crystal turbine alloys ≈ 1350 K peak metal temperature; incipient melting ≈ 1550-1650 K. A 1700 K uncooled nickel structure is not physically realisable. (For reference, 1700 K is in the coated C/C or refractory-metal range.)

**Impact.** The example is otherwise scrupulous about labelling demonstration values, and this one is labelled — but the material class named does not support the number, so a reader calibrating intuition from it gets a wrong sense of what hot-structure margins look like. The gate also cannot fail on the committed matrix by construction, since the bound was placed above the observed peak.

**Recommended fix.** Either lower the placard to a defensible nickel-alloy figure (≈1250-1350 K), which would make the M 5.0 / 40 km point exceed it and give the example a second live negative scenario, or keep 1700 K and rename the material class to coated carbon-carbon / refractory. Also state plainly that the bound was placed relative to the observed peak, as Q_MAX_PLACARD_KPA's docstring already does.

**Adversarial check.** The docstring is verbatim as quoted and the back-fit arithmetic checks (1502 x 1.13 = 1697). The auditor's superalloy figures are also correct: Inconel 718 tops out near 980 K, single-crystal blade alloys near 1350 K, incipient melting 1550-1650 K. But the finding rests on a category error that has to be called out. T0_MAX_PLACARD_K bounds the post-shock *gas* stagnation temperature, not a metal temperature: model.rs:128-140 computes T0 from the Rankine-Hugoniot jump plus isentropic re-stagnation of the freestream, and no wall model, emissivity, or radiation-equilibrium calculation exists anywhere in the example. A radiation-equilibrium leading edge in a 1700 K stagnation-temperature stream sits far below 1700 K, so a 1700 K gas-T0 placard is a permissive proxy for a nickel structure, not a physically impossible one. The 'magic-number' axis also fails: the value is fully traceable — the docstring states it is a chosen demonstration ceiling, not certification data, gives the 1502 K peak it bounds and the 13 percent margin, and README.md:88-90 repeats the disclaimer. What is left is loose wording: naming a material class beside a gas-temperature limit invites exactly the misreading the auditor made.

> Evidence re-read: flight_envelope_placard/constants.rs:48-52 verbatim; model.rs:107-140 placard_point — t0_k derived from `shock.post_shock(...)` then `post.t2 * (1 + half_gm1 * m2 * m2)`, a freestream/post-shock gas quantity with no wall or radiation-equilibrium model anywhere in the file; model.rs:206-241 gate_stagnation_temperature comparing r.t0_k directly; output.txt 'max T0 = 1502.1 K'; README.md:88-90 Limitations disclaimer; constants.rs:41-46 the parallel Q_MAX docstring using the same disclosed sizing method.

---

### 5.18 [MINOR] Nozzle output.txt embeds an absolute machine-specific path, defeating the portability the same module deliberately implements elsewhere

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `examples/avionics_examples/cfd/nozzle_operating_map/utils_print.rs:44`
- **Auditor confidence:** confirmed

**Claim.** print_intro strips the path to a bare file name with an explicit comment that this keeps the recorded reference output portable, but print_footer prints the full absolute path, so the committed output.txt cannot be reproduced on any other machine or checkout.

**Code evidence.**

```
utils_print.rs:18-19 (the stated intent): `// The file name only, not the absolute manifest path, so the recorded reference output is portable across machines and checkouts.`
utils_print.rs:44: `println!("\noperating map written to {}", out_path.display());`
output.txt: `operating map written to /Users/marvin/RustroverProjects/dcl/deep_causality/examples/avionics_examples/cfd/nozzle_operating_map/operating_map.csv`
(The viv example gets this right throughout — utils_print.rs:40-41 uses the `file_name` helper in the footer as well.)
```

**Reference form.** A committed reference output used for regression comparison must be reproducible on any checkout. The module's own comment states this requirement.

**Impact.** Any diff of a fresh run against the committed output.txt shows a spurious mismatch on this line, and the checked-in artifact leaks a developer's home directory path. Minor, but it is the one line preventing this example's output from being byte-comparable.

**Recommended fix.** Apply the same `file_name` treatment in print_footer that print_intro and the viv example already use.

**Adversarial check.** Verified line for line. nozzle_operating_map/utils_print.rs:18-19 carries the comment 'The file name only, not the absolute manifest path, so the recorded reference output is portable across machines and checkouts' and print_intro strips the schedule path accordingly. print_footer at line 44 then prints `out_path.display()` unstripped, and output.txt contains the full '/Users/marvin/RustroverProjects/dcl/deep_causality/examples/...' path. The contrast the auditor cites is real: viv_resonance_margin/utils_print.rs defines a `file_name` helper and uses it in print_footer, with the identical portability comment. The module states the requirement and then violates it on exactly one line, and the committed artifact leaks a developer home directory.

> Evidence re-read: nozzle_operating_map/utils_print.rs:18-19 (comment, grep-confirmed at line 18), :20-25 print_intro stripping to file_name, :41-48 print_footer with `println!("\noperating map written to {}", out_path.display());` at line 44; nozzle_operating_map/output.txt line with the absolute path; nozzle main.rs:79-81 out_path() built from CARGO_MANIFEST_DIR; viv_resonance_margin/utils_print.rs print_footer using `file_name(table_path)` with the same portability comment on its helper.

---

### 5.19 [MINOR] LYAPUNOV constant is uncited despite being load-bearing for every reported horizon number

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `examples/avionics_examples/cfd/turbulence_flow/model.rs:170`
- **Auditor confidence:** confirmed

**Claim.** LYAPUNOV = 0.906 determines every value printed by horizon_law and appears in both the README and the module docstring as 'λ ≈ 0.906', but carries no source reference — unlike every other physics constant in this example set.

**Code evidence.**

```
model.rs:169-170: `/// Leading Lyapunov exponent of the classic Lorenz attractor, used to report the horizon law.` `pub const LYAPUNOV: f64 = 0.906;`
print_utils.rs:31, 39, 43, 47 — every horizon_law call divides by it.
Contrast with the same repo's practice, e.g. flight_envelope_placard/constants.rs:26-32 which cites 'NASA TR R-376, 1971' for Sutton-Graves.
```

**Reference form.** λ₁ ≈ 0.90566 for the Lorenz system at σ=10, ρ=28, β=8/3 — Viswanath, 'Lyapunov exponents from random Fibonacci sequences to the Lorenz equations', Cornell TR (1998); also Sprott, Chaos and Time-Series Analysis, Table. The value in the code is correct.

**Impact.** No numerical error — the value is right. But it is the one constant in this example that a reader cannot trace to a source, in a file set whose stated discipline is that every constant carries its justification. The example also does not measure λ from its own trajectories, so there is no in-repo check that would catch a typo here.

**Recommended fix.** Add the citation to the docstring (Viswanath 1998, λ₁ = 0.90566). Optionally add a cheap self-check: fit the slope of ln(distance) vs t over the exponential-growth window of the committed run and assert it lands near 0.906 — that would turn an asserted constant into a measured one.

**Adversarial check.** Verified. model.rs carries only '/// Leading Lyapunov exponent of the classic Lorenz attractor, used to report the horizon law.' above `pub const LYAPUNOV: f64 = 0.906;` — no author, publication, or method. The value itself is right: the accepted lambda_1 for sigma=10, rho=28, beta=8/3 is 0.9056, and the auditor's Viswanath and Sprott references are the standard sources, so there is no numerical error. The traceability contrast holds: flight_envelope_placard/constants.rs:26-32 cites 'Sutton, K. and Graves, R. A., ... NASA TR R-376, 1971' for its heating constant, and the repo convention is that physics constants carry their source. The constant is load-bearing — horizon_law divides by it for all three reported horizons — and the example never measures lambda from its own trajectories, so no in-repo check would catch a typo. Note that turbulence_flow also has no constants.rs (see the layout finding), which is why this literal sits in model.rs at all.

> Evidence re-read: turbulence_flow/model.rs LYAPUNOV declaration and its one-line docstring; model.rs Report::horizon_law `-epsilon.ln() / LYAPUNOV`; print_utils.rs horizon_law call sites; main.rs — no Lyapunov estimation anywhere in the pipeline (simulate/analyse compute only trajectory distances and threshold crossings); flight_envelope_placard/constants.rs:26-32 as the cited-practice contrast.

---
