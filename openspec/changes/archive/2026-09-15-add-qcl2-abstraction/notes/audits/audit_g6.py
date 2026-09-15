import subprocess, shutil, sys, os
# Named-defect audit, group 6: composition, the induced norm and the chains.
Q='deep_causality_quantum/'
CO=Q+'src/types/abstraction/composition.rs'; QM=Q+'src/types/circuit_model/qc_morphism.rs'; CH=Q+'src/types/abstraction/chains.rs'
S=sys.argv[1]
defects=[
 ("D1 constants swapped in the bound","coefficient",CO,"bound: post * epsilon_first + pre * epsilon_second,","bound: pre * epsilon_first + post * epsilon_second,"),
 ("D2 conjugate dropped from the natural representation","sign",QM,".map(|c| Complex::new(c.re, -c.im))\n                    .collect();\n                let conj = CausalTensor::from_slice(&conj, &[d_out, d_in]);",".map(|c| Complex::new(c.re, c.im))\n                    .collect();\n                let conj = CausalTensor::from_slice(&conj, &[d_out, d_in]);"),
 ("D3 first block instead of the largest","loop",QM,"            if norm > worst {\n                worst = norm;\n            }","            if worst == R::zero() {\n                worst = norm;\n            }"),
 ("D4 composite tau skips the first link","skipped case",CO,"                let tau = self\n                    .alignment()\n                    .tau_for_side(e.low(), side, caps)?\n                    .then(e.tau(), caps)?;","                let tau = e.tau().clone();"),
 ("D5 composite section in the wrong order","ordering",CO,"                let section = e.section().then(\n                    &self.alignment().section_for_side(e.low(), side, caps)?,\n                    caps,\n                )?;","                let section = self\n                    .alignment()\n                    .section_for_side(e.low(), side, caps)?\n                    .then(e.section(), caps)?;"),
 ("D6 measured taken from the second link","interpretation",CO,"            let (left, right) = abstraction.square(&query, caps)?;\n            let (measured, _) = left.frobenius_distance(&right, caps)?;","            let measured = epsilon_second;"),
 ("D7 middle wire map transposed","index",CH,"let middle_wire = |j: usize| (j / k_out) * n_out + j % k_out;","let middle_wire = |j: usize| (j % k_out) * n_out + j / k_out;"),
 ("D8 outer program shifted by k instead of n","coefficient",CH,"                            .map(|o| shifted(o, block * n)),\n                    );\n                }\n            }\n            GateOp::Cz","                            .map(|o| shifted(o, block * k)),\n                    );\n                }\n            }\n            GateOp::Cz"),
 ("D9 depolarising amplitude without the square root","normalisation",CH,"let stay = Complex::new((R::one() - p).sqrt(), R::zero());","let stay = Complex::new(R::one() - p, R::zero());"),
 ("D10 two-sided entries never split","branch",CO,"let sides: Vec<AlignmentSide> = if e.side() == AlignmentSide::Any && sided_here {","let sides: Vec<AlignmentSide> = if false && sided_here {"),
 ("D11 pre constant taken on the output side","interpretation",CO,"let pre = self.tau_in(q_m, &q_l, caps)?.frobenius_induced_norm(caps)?;","let pre = self.tau_out(q_m, &q_l, caps)?.frobenius_induced_norm(caps)?;"),
 ("D12 gram distance without the cross term","coefficient",QM,"                        squared -= two * overlap(x, y);","                        squared -= overlap(x, y);"),
]
targets=["//deep_causality_quantum:abstraction_tests/types/abstraction/composition_tests_test","//deep_causality_quantum:abstraction_tests/types/abstraction/type_alignment_tests_test"]
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
