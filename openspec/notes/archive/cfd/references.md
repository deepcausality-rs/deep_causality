# References for the CFD Note Deck

Status: living bibliography, 2026-06-11. Every entry lists the exact citation, a
stable link, and which notes in this folder rely on it. Cite from the other notes as
`(references.md: <Key>)`.

## A. Teschner group — practitioner evidence and adjacent research

**[Teschner-blog]** Teschner, T.-R. *The 6 Biggest and Unsolved Challenges in CFD.*
cfd.university, ca. 2025-12.
https://cfd.university/blog/the-6-biggest-and-unoslved-challenges-in-cfd
— Source of the 42:1 turbulence-to-grid publication ratio, the CAD-watertightness
gap, the "correlations alone and not physics" critique of transition models, the
solver-to-solver uncertainty-propagation gap, and the conservation/overfitting
caution on ML surrogates. Used by: `variable-grid-geometry.md` §0,
`causal_cfd.md` §0.2 verification.

**[Abolholl-2024]** Abolholl, H. A. A., Teschner, T.-R. & Moulitsas, I. (2024).
*A Hybrid Computer Vision and Machine Learning Approach for Robust Vortex Core
Detection in Fluid Mechanics Applications.* Journal of Computing and Information
Science in Engineering, 24(6), 061002.
https://asmedigitalcollection.asme.org/computingengineering/article-abstract/24/6/061002
— Documents that Q, Δ, and swirling-strength criteria produce spurious vortices
(false positives and negatives); "vortex core detection remains an unsolved
problem." Hybrid CNN+DNN detector, up to 2.6× faster than pure DNN; validated on
the Taylor–Green vortex (the shared benchmark of this deck). Used by:
`3DCausalFluidDynamics.md` B1b (robustness caveat + topological-detection
comparison target).

**[Rijns-2025-VSD]** Rijns, S., Teschner, T.-R., Blackburn, K., Siampis, E. &
Brighton, J. (2025). *Optimising vehicle performance with advanced active
aerodynamic systems.* Vehicle System Dynamics, ahead-of-print.
https://doi.org/10.1080/00423114.2025.2505619
— Active (incl. asymmetrically actuated) aero optimized via minimum-lap-time
simulation over a transient vehicle-dynamics model + CFD-derived aero data. Used
by: `causal_cfd.md` §6 demo 5 (closed-loop active aero is a live industrial
research program, not a hypothetical).

**[Rijns-2024-corner]** Rijns, S., Teschner, T.-R., Blackburn, K. & Brighton, J.
(2024). *Effects of cornering conditions on the aerodynamic characteristics of a
high-performance vehicle and its rear wing.* Physics of Fluids, 36(4), 045119.
https://pubs.aip.org/aip/pof/article-abstract/36/4/045119
— 20% downforce loss and 35% drag increase at a 2.9-car-length corner radius vs.
straight-line; moving reference frame + overset mesh, validated against
moving-ground wind tunnel. Empirical regime-change evidence for vehicle aero. Used
by: `causal_cfd.md` §6 demo 5; `variable-grid-geometry.md` §7 (overset).

**[Rijns-2024-yaw]** Rijns, S., Teschner, T.-R., Blackburn, K., Ramos Proenca, A. &
Brighton, J. (2024). *Experimental and numerical investigation of the aerodynamic
characteristics of high-performance vehicle configurations under yaw conditions.*
Physics of Fluids, 36(4).
— Wind-tunnel force/surface-pressure/wake measurements + URANS under yaw; second
regime-change axis. Used by: `causal_cfd.md` §6 demo 5.

**[Rijns-2024-balance]** Rijns, S., Teschner, T.-R., Blackburn, K. & Brighton, J.
(2024). *Performance analyses of active aerodynamic load balancing designs on
high-performance vehicles in cornering conditions.* Physics of Fluids, 36(8),
085199. https://pubs.aip.org/aip/pof/article-abstract/36/8/085199
— Active load-balancing actuation in cornering: the control problem of §6 demo 5.
Used by: `causal_cfd.md` §6 demo 5.

**[Rijns-2024-workflow]** Rijns, S., Teschner, T.-R., Blackburn, K. & Brighton, J.
(2024). *Integrated Numerical and Experimental Workflow for High-Performance
Vehicle Aerodynamics.* SAE Technical Paper 2024-01-5016.
https://doi.org/10.4271/2024-01-5016
— Four RANS models assessed against wind-tunnel and DDES data; novel CFD-based
blockage correction. The practitioner reality that typed multi-fidelity fusion
formalizes. Used by: `causal_cfd.md` §3.1 item 3.

**[Ng-2025-TVC]** Ng, M. K. & Teschner, T.-R. (2025). *On the Application of
Trapped Vortices in Motorsport Application for Improved Aerodynamic Performance
Using Passive and Active Flow Controls.* SAE Technical Paper 2025-01-5029.
https://saemobilus.sae.org/papers/2025-01-5029
— Trapped vortex cavities on NACA0012 and an inverted three-element F1 airfoil;
passive vs. active comparison. Used by: `causal_cfd.md` §3.4 item 1.

**[Ng-2026-TVC]** Ng, M. K. & Teschner, T.-R. (2026). *Enhancing the Aerodynamic
Performance of Diffusers in High-Performance Vehicles Using Trapped Vortex
Cavities.* SAE Technical Paper 2026-01-5027.
https://saemobilus.sae.org/papers/2026-01-5027
— TVC on three diffuser geometries; 10% lift-to-drag improvement (straight
diffuser, active configuration, limited mass-flow). Geometry-variant campaigns as
manual counterfactuals. Used by: `causal_cfd.md` §3.4 item 1.

**[Xiong-2024-LBM]** Xiong, X., Teschner, T.-R., Moulitsas, I. & Józsa, T. I.
(2024). *Critical assessment of the lattice Boltzmann method for cavitation
modelling based on single bubble dynamics.* Discover Applied Sciences, 6, 241.
https://doi.org/10.1007/s42452-024-05895-1
— Sensitivity of LBM cavitation predictions to domain/bubble size and
Rayleigh–Plesset initial conditions; validation against Laplace's law and Maxwell's
area construction. Methodological kin to this deck's verification-first culture;
LBM is the Vision 2030 Area 3 "alternative method." Background reference.

## B. Strategic documents

**[NASA-2030]** Slotnick, J., Khodadoust, A., Alonso, J., Darmofal, D., Gropp, W.,
Lurie, E. & Mavriplis, D. (2014). *CFD Vision 2030 Study: A Path to Revolutionary
Computational Aerosciences.* NASA/CR-2014-218178.
https://ntrs.nasa.gov/citations/20140003093
— The six investment areas, seven findings, four grand challenges. Used by:
`causal_cfd.md` §0.2 (corrected scorecard), `variable-grid-geometry.md` §0.

**[NASA-2030-journal]** Slotnick, J. et al. (2014). *Enabling the environmentally
clean air transportation of the future: a vision of computational fluid dynamics
in 2030.* Phil. Trans. R. Soc. A, 372: 20130317.
https://pmc.ncbi.nlm.nih.gov/articles/PMC4095895/
— Open-access journal version of [NASA-2030]; source of the quotes in
`causal_cfd.md` §0.2.

**[Slotnick-2024]** Slotnick, J. & Heller, E. (2024). Vision 2030 progress report.
AIAA Aviation Forum 2024, AIAA 2024-4501.
— Progress slower than projected on UQ, validation infrastructure, and
multidisciplinary integration. Cited in `causal_cfd.md` §0.2.

## C. Mathematical foundations

**[Hirani-2003]** Hirani, A. N. (2003). *Discrete Exterior Calculus.* PhD thesis,
California Institute of Technology.
— Operator definitions; the discrete interior product `i_X ω = ±⋆(⋆ω ∧ X♭)` that
closes `cfd-gap.md` G1.

**[MHS-2016]** Mohamed, M. S., Hirani, A. N. & Samtaney, R. (2016). *Discrete
exterior calculus discretization of incompressible Navier–Stokes equations over
surface simplicial meshes.* Journal of Computational Physics, 312.
— The published solver shape `cfd-gap.md` §2 instantiates on cubical lattices.

**[Elcott-2007]** Elcott, S., Tong, Y., Kanso, E., Schröder, P. & Desbrun, M.
(2007). *Stable, circulation-preserving, simplicial fluids.* ACM Transactions on
Graphics, 26(1).
— DEC fluids lineage; circulation preservation as a structural property.

**[Loseille-Alauzet-2011]** Loseille, A. & Alauzet, F. (2011). *Continuous mesh
framework. Parts I & II.* SIAM Journal on Numerical Analysis, 49(1).
— Metric-based anisotropic adaptation; the framework whose "metric field" is the
native data structure in `variable-grid-geometry.md` R2.

## D. Validation reference data

**[Ghia-1982]** Ghia, U., Ghia, K. N. & Shin, C. T. (1982). *High-Re solutions for
incompressible flow using the Navier-Stokes equations and a multigrid method.*
Journal of Computational Physics, 48, 387–411.
— Lid-driven cavity centerline tables; Stage 3 exit criterion.

**[Driver-1985]** Driver, D. M. & Seegmiller, H. L. (1985). *Features of a
reattaching turbulent shear layer in divergent channel flow.* AIAA Journal, 23(2).
— Backward-facing step reattachment; `causal_cfd.md` §6 demo 1 baseline.

**[Lehmkuhl-2013]** Lehmkuhl, O. et al. (2013). Low-frequency unsteadiness in the
vortex-formation region of a circular cylinder. Physics of Fluids, 25, 085109.
— 3D cylinder reference (with the Williamson lineage); Stage 4 exit criterion.

**[HOW-TGV]** The Taylor–Green vortex at Re = 1600, standard test case of the
International Workshop on High-Order CFD Methods (energy-dissipation-rate
reference data).
— Stage 1 flagship validation (`cfd-gap.md` §7 item 7).

## E. Causal methodology

**[MSLD-2026]** Martínez-Sánchez, Á. & Lozano-Durán, A. (2026). Causal analysis of
turbulent flows (SURD methodology; exact venue per the published version).
— The methodology `3DCausalFluidDynamics.md` adapts; §3 synthetic benchmark and
§4.1 JHU channel-flow measurement.
