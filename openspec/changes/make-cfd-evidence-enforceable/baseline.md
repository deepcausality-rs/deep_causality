# Pre-change verification baseline

Recorded per task 1.1, **before** any gate was touched, so that a harness failing later in this change
is attributable to the gate repair rather than to the CI wiring (design D4, migration plan step 1).

Machine: Apple M3 Max, macOS, `--release`. Measured uncontended unless noted.

## Fast harnesses (pull-request cadence)

| Harness | Exit | Wall-clock |
|---|---|---|
| `mms_taylor_green_verification` | 0 | 0.73 s |
| `dec_graded_mms_verification` | 0 | 0.64 s |
| `dec_taylor_green_re1600_verification` | 0 | 1.38 s |
| `qtt_taylor_green_verification` | 0 | 1.57 s |
| `qtt_cylinder_verification` | 0 | 7.39 s |
| `qtt_park2t_blackout` | 0 | 4.18 s |
| `qtt_sod` | 0 | 1.62 s |
| `qtt_ramc_stagline` | 0 | 0.58 s |
| `qtt_blunt_body_2d` | 0 | 0.98 s |
| `qtt_reentry_3d` | 0 | 0.84 s |

**Fast total: 19.9 s**, against the ~20 s estimate in the proposal. Excludes build.

## Slow harnesses (nightly cadence)

Measured during the pre-certification audit while 16 agents ran concurrently, so these are inflated and
serve only as an upper bound and an exit-code record.

| Harness | Exit | Wall-clock (contended) | Documented |
|---|---|---|---|
| `dec_cylinder_wake_verification` | 0 | 187 s | ~155 s |
| `dec_cylinder_verification` | 0 | 578 s | ~510 s |
| `dec_lid_cavity_re1000_verification` | 0 | 1407 s | ~28 s at 33²; this run was the 65² default |

**Slow total: ~12 min** at documented rates.

## Baseline conclusion

**All 13 harnesses exit 0 before this change.**

That is the fact the change has to be read against, and it is weaker than it looks: several of those
zero exits come from gates that cannot fail, and `dec_cylinder_verification` has no gate at all — it
exits 0 even after a solver error. So a harness that starts failing during this change has not
regressed; it has become capable of failing for the first time.

Any such failure is triaged against `openspec/notes/cfd_audit/ACTION-LIST.md` before a bound is moved
(task 7.8).

## Unit-test baseline

```
cargo test -p deep_causality_cfd --release
→ 813 passed; 0 failed; 2 ignored
```
