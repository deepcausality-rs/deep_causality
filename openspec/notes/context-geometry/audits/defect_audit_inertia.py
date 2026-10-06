# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""The inertia rows of the context-geometry defect audit against utils/inertia.rs at commit
35bdf67c7: the same five defects as D5, D7, D8, D9 and D10 of defect_audit.py, with the patterns
quoting that text. Inject one at a time, run the context integration suite, record the failing
tests, restore the file byte for byte.

Usage: python3 defect_audit_inertia.py <repo root> [D5 D7 ...]. Results go to audit_results.json
beside this script."""
import subprocess, sys, re, json, pathlib
ROOT = pathlib.Path(sys.argv[1]); SRC = ROOT / "deep_causality_context/src"
D = [
 ("D5", 2, "utils/inertia.rs", "if diag + diag >= big {", "if diag + diag > big {"),
 ("D7", 7, "utils/inertia.rs", "if diag + diag >= big {", "if diag + diag >= big || true {"),
 ("D8", 9, "utils/inertia.rs", "            positive += 1;\n            negative += 1;", "            positive += 2;"),
 ("D9", 9, "utils/inertia.rs", "return (positive, negative, remaining);", "return (positive, negative, 0);"),
 ("D10", 3, "utils/inertia.rs", "a[j][k] -= a[j][p] * (a[p][k] / a[p][p]);", "a[j][k] += a[j][p] * (a[p][k] / a[p][p]);"),
]
only = set(sys.argv[2:])
results = []
for did, cls, rel, old, new in D:
    if only and did not in only: continue
    p = SRC / rel; original = p.read_bytes(); text = original.decode()
    n = text.count(old)
    if n != 1:
        results.append({"id": did, "class": cls, "file": rel, "status": f"PATTERN x{n}"}); print(did, "pattern count", n, flush=True); continue
    try:
        p.write_text(text.replace(old, new))
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
