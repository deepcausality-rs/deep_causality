# deep_causality_sparse

> ## This crate is retired
>
> **0.2.6 is the last release, and it exports nothing.** Depend on
> [`deep_causality_linear`](https://crates.io/crates/deep_causality_linear) instead; the
> names are unchanged. Repoint `use deep_causality_sparse::CsrMatrix` to
> `use deep_causality_linear::CsrMatrix`, and likewise for `CsrMatrixWitness`, `cg_solve`,
> `cg_solve_preconditioned`, `cg_solve_preconditioned_from` and `CgFailure`.
>
> 0.2.5 re-exported these types from `deep_causality_linear` 0.1. Every crate in the
> project now uses `deep_causality_linear` 0.2, so a type taken from 0.2.5 does not match
> the type those crates expect (E0308, "expected `CsrMatrix`, found a different
> `CsrMatrix`").
>
> The crate lives in `yanked/` and is not a workspace member.
