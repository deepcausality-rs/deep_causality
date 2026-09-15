import subprocess, shutil, sys, os
# Named-defect audit, group 5: fault sets and the fault-tolerance predicate.
Q='deep_causality_quantum/'
FS=Q+'src/types/abstraction/fault_set.rs'; FT=Q+'src/types/abstraction/fault_tolerance.rs'; QU=Q+'src/types/circuit_model/queries.rs'
S=sys.argv[1]
defects=[
 ("D1 binomial step divides by i+2","coefficient",FS,"binom = binom.checked_mul((n - i) as u64)? / (i as u64 + 1);","binom = binom.checked_mul((n - i) as u64)? / (i as u64 + 2);"),
 ("D2 cap refuses at equality","boundary",FS,"        if count > cap {","        if count >= cap {"),
 ("D3 Pauli pattern digit rotated","ordering",FS,"errors.push((sorted[i], PauliKind::ALL[p % 3]));","errors.push((sorted[i], PauliKind::ALL[(p + 1) % 3]));"),
 ("D4 X and Z parts swapped in the Pauli","interpretation",FS,".filter(|(_, p)| p.has_x())\n            .map(|(w, _)| *w)\n            .collect();\n        let zs",".filter(|(_, p)| p.has_z())\n            .map(|(w, _)| *w)\n            .collect();\n        let zs"),
 ("D5 fault inserted before the node's last box","index",QU,"members.iter().copied().max().map_or(0, |b| b + 1)","members.iter().copied().max().map_or(0, |b| b)"),
 ("D6 weight growth ignored","branch",FT,"let tolerated = constant && within;","let tolerated = constant;"),
 ("D7 weight bound strict","boundary",FT,"let within = weight <= fault.weight();","let within = weight < fault.weight();"),
 ("D8 residual inverted","sign",FT,"let residual = if tolerated { R::zero() } else { R::one() };","let residual = if tolerated { R::one() } else { R::zero() };"),
 ("D9 numeric square ignores the fault","skipped case",FT,"self.square_with(&Query::Io, &low_q, caps)?","self.square_with(&Query::Io, &Query::Io, caps)?"),
 ("D10 flips read from the Z part","interpretation",FT,"propagated.remainder.parity_flips(pauli.x())?","propagated.remainder.parity_flips(pauli.z())?"),
 ("D11 DEM duplicates kept","loop",FS,"            if seen.insert(f.clone()) {\n                faults.push(f);\n            }","            seen.insert(f.clone());\n            faults.push(f);"),
 ("D12 filter inverts holds","branch",FT,"            if report.holds() {\n                holds.push(gate.clone());","            if !report.holds() {\n                holds.push(gate.clone());"),
]
targets=["//deep_causality_quantum:abstraction_tests/types/abstraction/fault_set_tests_test","//deep_causality_quantum:abstraction_tests/types/abstraction/fault_tolerance_tests_test"]
rows=[]
for name,cls,path,old,new in defects:
    src=open(path).read()
    if src.count(old)!=1:
        rows.append((name,cls,"PATTERN NOT FOUND (%d)"%src.count(old))); continue
    shutil.copy(path, os.path.join(S, os.path.basename(path)+".orig"))
    open(path,'w').write(src.replace(old,new))
    try:
        r=subprocess.run(["bazel","test"]+targets,capture_output=True,text=True)
        rows.append((name,cls,"caught" if r.returncode!=0 else "SURVIVED"))
    finally:
        open(path,'w').write(src)
for row in rows: print("| %s | %s | %s |"%row)
