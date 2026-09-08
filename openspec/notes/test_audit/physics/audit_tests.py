"""Inventory broken-by-design tests in deep_causality_physics/tests.

Three classes, per the maintainer's definition:
  circular    - the expected value is recomputed in the test from the same formula
  tautology   - the assertions constrain nothing about the answer
  cherry-pick - one hand-chosen input, a magic literal, no stated provenance
"""
import re, os, json
from collections import defaultdict

ROOT = "deep_causality_physics/tests"

def split_tests(src):
    out, lines, i = [], src.split("\n"), 0
    while i < len(lines):
        if lines[i].strip().startswith("#[test]"):
            j = i + 1
            while j < len(lines) and not re.match(r"\s*(pub )?fn ", lines[j]):
                j += 1
            if j >= len(lines): i += 1; continue
            m = re.match(r"\s*(?:pub )?fn\s+([A-Za-z0-9_]+)", lines[j])
            name = m.group(1) if m else "?"
            depth, started, k, body = 0, False, j, []
            while k < len(lines):
                depth += lines[k].count("{") - lines[k].count("}")
                body.append(lines[k])
                if "{" in lines[k]: started = True
                if started and depth <= 0: break
                k += 1
            out.append((name, "\n".join(body), j + 1))
            i = k + 1
        else: i += 1
    return out

ASSERT = re.compile(r"\bassert(_eq|_ne)?!")
ARITH  = re.compile(r"[+\-*/]|\.(sqrt|cbrt|powi|powf|ln|log|log10|exp|sin|cos|tan|asin|acos|atan|abs|hypot)\(")
PROV   = re.compile(r"(NIST|CODATA|reference|published|textbook|table\b|handbook|oracle|closed[- ]form|by hand|bisection|analytic|literature|Appendix|ISBN|doi|et al|20\d\d\)|scipy|numpy|mpmath|Wolfram)", re.I)

def strip_comments(body):
    return "\n".join(l for l in body.split("\n") if not l.strip().startswith("//"))

def classify(name, body, helpers):
    code = strip_comments(body)
    flags = []
    alines = [l for l in code.split("\n") if ASSERT.search(l)]

    # Assertions may live in a helper this test calls.
    delegates = any(h in code for h in helpers)

    if not alines and not delegates:
        flags.append("no-assertion")

    # --- tautology: no assertion says anything about a numeric answer.
    if alines:
        informative = 0
        for l in alines:
            if re.search(r"\.is_err\(\)|\.is_none\(\)|unwrap_err|expect_err|matches!", l) \
               or re.search(r"assert!\s*\(\s*!", l):
                informative += 1            # an error-path assertion, incl. `assert!(!x.is_ok())`
            elif re.search(r"\.is_ok\(\)|\.is_some\(\)|\.is_finite\(\)|!\s*[\w.()]*\.is_nan\(\)|\.is_sign_positive\(\)", l):
                pass                        # says nothing about the value
            else:
                informative += 1
        if informative == 0:
            flags.append("tautology")

    # --- circular: an expectation built by arithmetic inside the test.
    circ = False
    for m in re.finditer(r"let\s+\w*(expected|want|reference|exp|predicted|analytic|manual)\w*\s*(?::[^=]+)?=\s*([^;]+);", code, re.I):
        if ARITH.search(m.group(2)) and not re.match(r"^\s*-?[\d._eE]+\s*$", m.group(2)):
            circ = True; break
    if not circ:
        # inline: assert!((got - <arithmetic expression>).abs() < TOL)
        for l in alines:
            m = re.search(r"-\s*\(([^)]*[+*/][^)]*)\)\s*\)?\s*\.abs\(\)", l)
            if m and ARITH.search(m.group(1)):
                circ = True; break
    if circ: flags.append("circular")

    # --- cherry-pick: single input, magic literal, no provenance.
    varied = bool(re.search(r"\bfor\s+[\w(]|\bin\s*\[|\.iter\(\)|\bin\s*0\.\.", code))
    literal = bool(re.search(r"assert(_eq)?!\s*\(\s*\(?[^;]*?(-|,)\s*-?\d+\.\d+", code))
    if literal and not varied and not PROV.search(body):
        flags.append("cherry-picked")
    if not varied and alines:
        flags.append("single-input")
    return flags

# Collect helper fn names that themselves assert (so delegation is not "no assertion").
helpers = set()
for dirpath, _, names in os.walk(ROOT):
    for fn in names:
        if not fn.endswith(".rs"): continue
        src = open(os.path.join(dirpath, fn)).read()
        for m in re.finditer(r"fn\s+([A-Za-z0-9_]+)\s*(<[^>]*>)?\s*\([^)]*\)[^{]*\{", src):
            nm = m.group(1)
            seg = src[m.end(): m.end() + 4000]
            if ASSERT.search(seg): helpers.add(nm)

inv, totals, tests, files = defaultdict(list), defaultdict(int), 0, 0
per_file = defaultdict(lambda: defaultdict(int))
for dirpath, _, names in os.walk(ROOT):
    for fn in names:
        if not fn.endswith(".rs"): continue
        p = os.path.join(dirpath, fn)
        got = split_tests(open(p).read())
        if got: files += 1
        for nm, body, line in got:
            tests += 1
            for f in classify(nm, body, helpers):
                totals[f] += 1
                inv[f].append(f"{p}:{line}:{nm}")
                per_file[p][f] += 1

print(f"scanned {tests} tests in {files} files\n")
for k in sorted(totals, key=lambda x: -totals[x]):
    print(f"  {k:16} {totals[k]:5}  ({100*totals[k]/tests:5.1f}%)")

broken = defaultdict(int)
for p, d in per_file.items():
    broken[p] = d.get("circular",0) + d.get("tautology",0) + d.get("no-assertion",0) + d.get("cherry-picked",0)
print("\n  worst files (circular + tautology + no-assertion + cherry-picked):")
for p, n in sorted(broken.items(), key=lambda x: -x[1])[:15]:
    print(f"    {n:4}  {p}")
json.dump({k: v for k, v in inv.items()}, open("/tmp/audit_inv.json", "w"), indent=1)
