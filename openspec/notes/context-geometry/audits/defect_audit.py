# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Phase-3 defect audit for the context-crate geometry work: inject one defect at a time, run the
context integration suite, record the failing tests, restore the file byte for byte.

Usage: python3 defect_audit.py <repo root> [D1 D2 ...]. Results go to audit_results.json beside
this script.

The patterns quote deep_causality_context at commit 462077fcb, except D5 and D7, which quote the
pivot threshold of utils/inertia.rs before its rewrite to `diag + diag >= big`;
defect_audit_inertia.py repeats those rows against the current text. A pattern that does not
match the checked-out source is reported as PATTERN x<count> and not run."""
import subprocess, sys, re, json, pathlib
ROOT = pathlib.Path(sys.argv[1]); SRC = ROOT / "deep_causality_context/src"
G = "types/context_node_types"
D = [
 ("D1", 4, "utils/seconds.rs", "TimeScale::Minute => Some(t * lift(60.0)),", "TimeScale::Minute => Some(t * lift(61.0)),"),
 ("D2", 3, "utils/seconds.rs", "TimeScale::Millisecond => Some(t / lift(1_000.0)),", "TimeScale::Millisecond => Some(t * lift(1_000.0)),"),
 ("D3", 9, "utils/seconds.rs", "TimeScale::Year => Some(t * lift(31_556_952.0)),", "TimeScale::Year => Some(t * lift(31_536_000.0)),"),
 ("D4", 8, "utils/seconds.rs", "TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => None,", "TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => Some(t),"),
 ("D5", 2, "utils/inertia.rs", "if diag >= half * big {", "if diag > half * big {"),
 ("D7", 7, "utils/inertia.rs", "if diag >= half * big {", "if diag >= half * big || true {"),
 ("D8", 9, "utils/inertia.rs", "            positive += 1;\n            negative += 1;", "            positive += 2;"),
 ("D9", 9, "utils/inertia.rs", "return (positive, negative, remaining);", "return (positive, negative, 0);"),
 ("D10", 3, "utils/inertia.rs", "a[j][k] -= a[j][p] * a[p][k] / a[p][p];", "a[j][k] += a[j][p] * a[p][k] / a[p][p];"),
 ("D11", 8, f"{G}/space_time/tangent_spacetime/metric_tensor.rs", ".find(|&(i, j)| new_metric[i][j] != new_metric[j][i]);", ".find(|&(i, j)| i != i && new_metric[i][j] != new_metric[j][i]);"),
 ("D12", 6, f"{G}/space_time/tangent_spacetime/metric_tensor.rs", "if (positive, negative, zero) != (3, 1, 0) {", "if (positive, negative, zero) == (4, 0, 0) {"),
 ("D13", 3, f"{G}/space/geo_space/metric.rs", "let e2 = f * (lift::<R>(2.0) - f);", "let e2 = f * (lift::<R>(2.0) + f);"),
 ("D14", 4, f"{G}/space/geo_space/metric.rs", "let a: R = lift(6_378_137.0);", "let a: R = lift(6_371_000.0);"),
 ("D15", 5, f"{G}/space/geo_space/metric.rs", "((R::one() - e2) * nu + h) * sin_phi,", "(nu + h) * sin_phi,"),
 ("D16", 8, f"{G}/space/geo_space/metric.rs", "if self.datum != VerticalDatum::WGS84 || other.datum != VerticalDatum::WGS84 {", "if false {"),
 ("D17", 9, f"{G}/space/geo_space/metric.rs", "(nu + h) * phi.cos() * lambda.cos(),", "(nu + h) * phi.cos() * lambda.sin(),"),
 ("D18", 2, f"{G}/space/geo_space/mod.rs", "if lat < -ninety || lat > ninety {", "if lat < -ninety || lat >= ninety {"),
 ("D19", 4, f"{G}/space/geo_space/mod.rs", "let ninety: R = lift(90.0);", "let ninety: R = lift(180.0);"),
 ("D20", 8, f"{G}/space/geo_space/mod.rs", "if !(lat.is_finite() && lon.is_finite() && alt.is_finite()) {", "if false {"),
 ("D21", 8, f"{G}/space_time/galilean_spacetime/simultaneity.rs", "(None, None) if self.time_scale == other.time_scale => Some(self.t == other.t),", "(None, None) => Some(self.t == other.t),"),
 ("D22", 5, f"{G}/space_time/galilean_spacetime/simultaneity.rs", "(Some(a), Some(b)) => Some(a == b),", "(Some(_), Some(_)) => Some(self.t == other.t),"),
 ("D23", 8, f"{G}/space_time/galilean_spacetime/metric.rs", "if self.is_simultaneous_with(other) != Some(true) {", "if false {"),
 ("D24", 9, f"{G}/space_time/minkowski_spacetime/space_temporal_interval.rs", "seconds(self.t, self.time_scale).unwrap_or_else(R::nan)", "seconds(self.t, self.time_scale).unwrap_or(self.t)"),
 ("D25", 1, f"{G}/space_time/newtonian_spacetime/coordinate.rs", "0 => Ok(&self.t),", "0 => Ok(&self.x),"),
 ("D26", 9, f"{G}/space_time/newtonian_spacetime/metric_signature.rs", "Metric::PGA(4)\n", "Metric::Generic { p: 3, q: 0, r: 1 }\n"),
 ("D27", 9, f"{G}/space_time/galilean_spacetime/metric_signature.rs", "Metric::PGA(4)\n", "Metric::Euclidean(4)\n"),
 ("D28", 1, f"{G}/space_time/minkowski_spacetime/coordinate.rs", "0 => Ok(&self.t),", "0 => Ok(&self.x),"),
 ("D29", 1, f"{G}/space_time/tangent_spacetime/coordinate.rs", "0 => Ok(&self.t),", "0 => Ok(&self.x),"),
 ("D30", 1, f"{G}/space_time/galilean_spacetime/coordinate.rs", "3 => Ok(&self.z),", "3 => Ok(&self.y),"),
 ("D31", 1, f"{G}/space_time/newtonian_spacetime/adjustable.rs", "let new_t = array_grid.get(p1);", "let new_t = array_grid.get(p4);"),
 ("D32", 8, f"{G}/space_time/causal_set_spacetime/mod.rs", "parent_id != self.id && self.predecessors.insert(parent_id)", "self.predecessors.insert(parent_id)"),
 ("D33", 8, f"{G}/space/geo_space/adjustable.rs", "super::check(adjusted_lat, adjusted_lon, adjusted_alt).map_err(|e| AdjustmentError(e.0))?;", ""),
 ("D34", 3, f"{G}/space_time/tangent_spacetime/mod.rs", "[-(c * c), zero, zero, zero],", "[c * c, zero, zero, zero],"),
]
only = set(sys.argv[2:])
results = []
for did, cls, rel, old, new in D:
    if only and did not in only: continue
    p = SRC / rel; original = p.read_bytes(); text = original.decode()
    n = text.count(old)
    first_only = did == "D31"
    if n != 1 and not (first_only and n >= 1):
        results.append({"id": did, "class": cls, "file": rel, "status": f"PATTERN x{n}"}); print(did, "pattern count", n, flush=True); continue
    try:
        p.write_text(text.replace(old, new, 1) if first_only else text.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "deep_causality_context", "--test", "mod", "-q"], cwd=ROOT, capture_output=True, text=True)
        out = r.stdout + r.stderr
        failed = sorted(set(re.findall(r"^    (types::\S+|traits::\S+|alias::\S+|errors::\S+|extensions::\S+|utils_test::\S+)$", out, re.M)))
        if "error[" in out or "could not compile" in out:
            status = "COMPILE-ERROR"
        else:
            status = "KILLED" if r.returncode != 0 else "SURVIVED"
        results.append({"id": did, "class": cls, "file": rel, "status": status, "failed": failed})
        print(did, status, len(failed), flush=True)
    finally:
        p.write_bytes(original)
    assert p.read_bytes() == original
(pathlib.Path(sys.argv[0]).parent / "audit_results.json").write_text(json.dumps(results, indent=1))
