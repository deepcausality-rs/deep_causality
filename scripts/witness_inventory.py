#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Inventory of the HKT witness types under deep_causality_unified_math/: every
# `pub struct X*Witness` and every `impl Trait for X*Witness`.
#
# Two parser requirements, both learned by getting them wrong. Comments are stripped first,
# because a doc comment showing `pub struct MyCustomTypeWitness` in an example block is otherwise
# counted as a witness. And `impl` headers are matched across newlines, because
# `impl<C> Monad<ManifoldWitness<C>>\n    for ManifoldWitness<C>` is invisible to a line-oriented
# regex — joining them recovers six rows a line regex drops silently.
#
# Run from the repository root. Writes /tmp/witness_inv.json and prints a summary.
#
import re, pathlib, collections, json

ROOT = pathlib.Path("deep_causality_unified_math")

def decomment(src):
    """Blank out //-comments (incl. doc comments) and /* */ blocks, preserving offsets."""
    out, i, n = list(src), 0, len(src)
    while i < n:
        if src[i] == '"':                       # skip string literals
            i += 1
            while i < n and src[i] != '"':
                i += 2 if src[i] == '\\' else 1
            i += 1
        elif src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j): out[k] = ' '
            i = j
        elif src.startswith("/*", i):
            j = src.find("*/", i + 2)
            j = n if j < 0 else j + 2
            for k in range(i, j):
                if out[k] != '\n': out[k] = ' '
            i = j
        else:
            i += 1
    return "".join(out)

def strip_generics(s):
    out, depth = [], 0
    for ch in s:
        if ch == '<': depth += 1
        elif ch == '>':
            if depth: depth -= 1
        elif depth == 0: out.append(ch)
    return "".join(out).strip()

IMPL = re.compile(r"\bimpl\b(?P<body>.*?)(?=\s*\{|\s+where\b)", re.S)
DECL = re.compile(r"pub struct (?P<name>[A-Za-z0-9_]*Witness)\b")
PROJ = re.compile(r"type Type(?:<[^=]*?>)?\s*=\s*(?P<ty>[^;]+);", re.S)

decls, proj = {}, {}
impls   = collections.defaultdict(set)
impl_by = collections.defaultdict(set)
HKTS = {"HKT","HKT2","HKT3","HKT4","HKT5",
        "HKT2Unbound","HKT3Unbound","HKT4Unbound","HKT5Unbound","HKT6Unbound"}

for crate in sorted(ROOT.iterdir()):
    src = crate / "src"
    if not src.is_dir(): continue
    cname = crate.name.replace("deep_causality_", "")
    for f in sorted(src.rglob("*.rs")):
        if "utils_tests" in str(f): continue
        raw = f.read_text()
        txt = decomment(raw)
        rel = str(f.relative_to(crate))
        for m in DECL.finditer(txt):
            decls.setdefault(m.group("name"), (cname, rel))
        for m in IMPL.finditer(txt):
            body = m.group("body")
            depth, cut, i = 0, None, 0
            while i < len(body):
                c = body[i]
                if c == '<': depth += 1
                elif c == '>': depth = max(0, depth - 1)
                elif depth == 0 and body[i:i+5] == " for ": cut = i; break
                i += 1
            if cut is None: continue
            tp, sp = body[:cut].lstrip(), body[cut+5:]
            if tp.startswith("<"):
                d = 0
                for j, c in enumerate(tp):
                    if c == '<': d += 1
                    elif c == '>':
                        d -= 1
                        if d == 0: tp = tp[j+1:]; break
            tr, sf = strip_generics(tp), strip_generics(sp)
            if not tr or not sf or not sf.endswith("Witness"): continue
            if not re.fullmatch(r"[A-Za-z0-9_]+", tr): continue
            impls[sf].add(tr)
            impl_by[(sf, tr)].add(cname)
            if tr in HKTS:
                pm = PROJ.search(txt[m.end(): m.end() + 1200])
                if pm: proj.setdefault(sf, " ".join(pm.group("ty").split()))

json.dump({"decls": decls, "proj": proj,
           "impls": {k: sorted(v) for k, v in impls.items()},
           "impl_by": {f"{k[0]}|{k[1]}": sorted(v) for k, v in impl_by.items()}},
          open("/tmp/witness_inv.json","w"), indent=1)
print(f"declared: {len(decls)}   with impls: {len(impls)}   traits: {len({t for v in impls.values() for t in v})}")
print("no-decl :", sorted(set(impls) - set(decls)))
print("no-impl :", sorted(set(decls) - set(impls)))
