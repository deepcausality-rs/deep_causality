import subprocess, shutil, sys, os
Q='deep_causality_quantum/'
DM=Q+'src/types/qcm/dem_model.rs'; DA=Q+'src/types/abstraction/decoder_abstraction.rs'; NS=Q+'src/types/circuit_model/numeric_semantics.rs'; TA=Q+'src/types/abstraction/type_alignment.rs'; AB=Q+'src/types/abstraction/abstraction.rs'
S=sys.argv[1]
defects=[
 ("D1 mechanism weight uses p for the non-firing branch","coefficient",DM,"                    weight *= 1.0 - m.probability;","                    weight *= m.probability;"),
 ("D2 the fault shift is dropped","skipped case",DM,"            let mut flips = shift;","            let mut flips = 0;"),
 ("D3 detector bit position off by one (observables land on detectors)","index",DM,"            bits |= 1 << (self.num_variables() - 1 - (self.num_detectors + o));","            bits |= 1 << (self.num_variables() - 1 - o);"),
 ("D4 phantom mechanism fires with probability one","interpretation",DM,"        self.mechanisms.push(Mechanism {\n            probability: 0.0,","        self.mechanisms.push(Mechanism {\n            probability: 1.0,"),
 ("D5 graph edges read in the wrong direction","interpretation",DM,".filter(|(_, d)| graph.contains_edge(node, **d))",".filter(|(_, d)| graph.contains_edge(**d, node))"),
 ("D6 unknown directives accepted","branch",DM,"                \"detector\" | \"logical_observable\" => {}\n                _ => {\n                    return Err(refuse(\n                        number,\n                        raw,\n                        \"is not an error, detector or logical_observable line\",\n                    ));\n                }","                _ => {}"),
 ("D7 stochastic rows not checked","branch",DA,"|| (sum - 1.0).abs() > tol {","|| false {"),
 ("D8 matrix read transposed (high given low swapped)","interpretation",DA,"                digits(x, low_counts),\n                digits(y, high_counts),","                digits(y, high_counts),\n                digits(x, low_counts),"),
 ("D9 the classical output map ignored on the output side","skipped case",AB,"    if side == AlignmentSide::Output\n        && let Some(map) = alignment.classical_output()","    if false\n        && let Some(map) = alignment.classical_output()"),
 ("D10 attribution ranked ascending","ordering",DA,"ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));","ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal));"),
 ("D11 zero-branch pruning drops non-zero branches (inverted)","branch",NS,"            if nb.cols.iter().all(|c| c.iter().all(|a| *a == zero)) {\n                continue;\n            }","            if nb.cols.iter().any(|c| c.iter().any(|a| *a == zero)) {\n                continue;\n            }"),
 ("D12 worst failure picks the smallest residual","ordering",DA,"                Some(a) if a.residual >= f.residual => Some(a),","                Some(a) if a.residual <= f.residual => Some(a),"),
]
targets=["//deep_causality_quantum:qcm_tests/types/qcm/dem_model_tests_test","//deep_causality_quantum:abstraction_tests/types/abstraction/decoder_abstraction_tests_test","//deep_causality_quantum:circuit_model_tests/types/circuit_model/numeric_semantics_tests_test"]
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
