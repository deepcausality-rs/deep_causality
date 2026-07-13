# Tasks: transcribe-lean-num-complex-dual-verification

## 1. Fill the Complex & Dual page

- [ ] 1.1 Transcribe the 5 `complex.*` rows from `lean/THEOREM_MAP.md` into `website/docs/src/content/docs/formalization/complex-dual.md`, columns `| id | statement | Lean proof | Rust witness | Test |`, Lean cells directory-qualified (`Complex/Complex.lean :: complex_field_mul_inv`), Rust cells bare (`complex_tests.rs :: test_complex_field_mul_inv`)
- [ ] 1.2 Transcribe the 4 `quaternion.*` rows (`Complex/Quaternion.lean`, `quaternion_tests.rs`) in the same format
- [ ] 1.3 Transcribe the 6 `dual.*` rows (`Dual/Dual.lean`, `dual_tests.rs`) in the same format, preserving the map's witness names verbatim (`test_mul_comm`, not `test_dual_mul_comm`)
- [ ] 1.4 Extend the intro prose per the `num.md` pattern: state "Fifteen laws", link `lean/DeepCausalityFormal/` on GitHub (naming the Complex and Dual subdirectories), name both witness directories (`deep_causality_num_complex/tests/formalization_lean/`, `deep_causality_num_dual/tests/formalization_lean/`), keep the stub's framing of the negative results, note every row is `proved`
- [ ] 1.5 Delete the `:::caution` "good first issue" block and remove `draft: true` from the frontmatter (keep `sidebar: order: 5`)

## 2. Link from the formalization index

- [ ] 2.1 Add a `**[Complex & Dual](/formalization/complex-dual/)**` bullet to "The layers" in `website/docs/src/content/docs/formalization/index.md`, between Core and Topology, with a one-line summary of the layer
- [ ] 2.2 Update the "being documented" sentence to name only the Haft and Quantum layers

## 3. Verify

- [ ] 3.1 Cross-check the page against the source of truth: extracted ids equal the 15 `complex.*`/`quaternion.*`/`dual.*` ids in `THEOREM_MAP.md`; every Lean theorem name exists in the exact `.lean` file its cell names; every test name exists in the exact `_tests.rs` file its cell names
- [ ] 3.2 Build the site (`npm run build` in `website/docs/`) and confirm it exits zero with the `/formalization/complex-dual/` route present
