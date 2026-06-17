# Reverted: fix-graded-convective-consistency — superseded by findings

## Status

**Reverted before implementation. Never applied; no living-spec impact.** The change set
was authored and validated (`--strict`) but its premise was falsified by a measurement
correction during its own gating spike (Group A), so it is moved here intact rather than
archived into `openspec/specs/`. The capability delta it carried
(`graded-convective-order`) was **not** synced into the living specs — there is no defect
to specify a fix for.

Date reverted: 2026-06-14 (same day it was created).

## What the change set assumed

It targeted a measured "consistency defect": the discrete **convective** operator
`i_X ω` (interior product) appeared to lose formal second order on smoothly graded meshes
— collapsing toward first order in both the max- and L2-norms by an adjacent-spacing ratio
of ≈ 1.11 — while the **viscous** operator stayed second order. The proposed remedy was a
surgical off-centering consistency correction (default) with a Galerkin / Whitney (Q1)
Hodge star as the rough-mesh fallback, framed by the mapped-coordinate view.

## The finding that superseded it

The Group A spike re-ran the graded MMS and, in its **first experiment**, traced the
"order loss" to the *measurement*, not the operator. DEC operators act on **cochains —
integrals over cells**: a discrete 1-form on an edge is `∫ ω ≈ (tangential midpoint
value)·ℓ_edge`. The MMS had fed pointwise 1-form values **inconsistently** — scaling `X♭`
by `ℓ` but not `ω`, and comparing an edge-integral output to a *pointwise* analytic
reference. That inconsistency is invisible on a uniform mesh (`ℓ = 1`) but `O(ℓ)`-wrong on
a graded one. (The viscous MMS used 0-forms, which carry no length factor, so it was always
measured consistently — which is why only the convective number looked broken.)

With **consistent cochains** (both `ω` and `X♭` as edge-integrals `×ℓ`, the output
normalised `÷ℓ`), the convective operator is **second order in both norms at every grading
amplitude tested, up to a 3:1 spacing ratio** — exactly like the viscous operator. Only the
error *constant* grows mildly with grading; the *order* does not degrade.

```
CONVECTIVE  i_X ω — L2-norm order vs grading amplitude (corrected cochains)
  a=0.00  p ≈ 1.99      a=0.20  p ≈ 1.99
  a=0.10  p ≈ 1.99      a=0.30  p ≈ 1.99   (… and ≈ 1.98 at a=0.50, ratio 3:1)
```

## Conclusion

**There is no convective consistency defect.** "Smooth grading retains second order" is
empirically true for **both** operators; the R1 promise is delivered today with no Galerkin
star, no off-centering correction, and no follow-up. The corrected, convention-enforcing
study lives in `examples/avionics_examples/dec_graded_mms`.

## Cleanup still owed elsewhere (the same artifact)

The same cochain bug under-measured the order in two other places that should be corrected
to the true finding (tracked separately, not part of this revert):

- the `add-graded-metrics` fast test `cartan_formula_converges_under_smooth_grading`
  (passes, but on a loose "error decreases" threshold rather than a true second-order
  assertion);
- the living `graded-metrics` spec / archived design, which record the now-false
  "the convective operator loses formal second order under grading."
