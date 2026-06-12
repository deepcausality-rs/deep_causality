/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Criterion benchmark of the periodic DEC-native Navier–Stokes solver.
//!
//! Measures the components of one march step on the 3D Taylor–Green case
//! (the Re-1600 example's workload) at 16³ and 32³, f64:
//!
//! - `rate_unprojected`: the raw RHS assembly `−i_u(du♭) − ν Δ_dR u♭` —
//!   wedge/interior-product dominated.
//! - `leray_project`: one gauge-fixed CG solve on the raw RHS.
//! - `solver_step`: the full projected step (four projected stage
//!   evaluations, the type-state re-entry, the CFL guard with its `sharp`
//!   call, and the divergence residual).
//!
//! Run serial vs. parallel to quantify the Rayon feature:
//!
//! ```text
//! cargo bench -p deep_causality_physics --bench dec_solver_benchmark
//! cargo bench -p deep_causality_physics --bench dec_solver_benchmark --features parallel
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use deep_causality_physics::{DecNsRate, DecNsSolver, SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const RE: f64 = 1600.0;
const DT: f64 = 0.2;

fn unit_manifold3(n: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn tg_vertex_tensor(
    manifold: &Manifold<LatticeComplex<3, f64>, f64>,
    n: usize,
) -> CausalTensor<f64> {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 3 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let (x, y, z) = (k * p[0] as f64, k * p[1] as f64, k * p[2] as f64);
        vertex[3 * vi] = x.sin() * y.cos() * z.cos();
        vertex[3 * vi + 1] = -x.cos() * y.sin() * z.cos();
    }
    CausalTensor::new(vertex, vec![3 * n0]).unwrap()
}

/// One benchmark fixture per grid: the warm manifold, the solver, the
/// seeded state, and the rate's marching input.
struct Fixture {
    manifold: Manifold<LatticeComplex<3, f64>, f64>,
    n: usize,
}

impl Fixture {
    fn new(n: usize) -> Self {
        Self {
            manifold: unit_manifold3(n),
            n,
        }
    }

    fn nu(&self) -> f64 {
        let k = 2.0 * std::f64::consts::PI / (self.n as f64);
        1.0 / (k * RE)
    }

    fn solver(&self) -> DecNsSolver<'_, 3, f64> {
        DecNsSolver::new(&self.manifold, self.nu(), DT, None).unwrap()
    }

    fn state(&self, solver: &DecNsSolver<'_, 3, f64>) -> SolenoidalField<f64> {
        solver
            .seed_from_vertex_vectors(&tg_vertex_tensor(&self.manifold, self.n))
            .unwrap()
    }
}

fn bench_dec_solver(c: &mut Criterion) {
    for n in [16usize, 32, 64] {
        let fixture = Fixture::new(n);
        let solver = fixture.solver();
        let state = fixture.state(&solver);
        // Warm the lattice matrix memos so the benches measure steady-state
        // marching, not first-call cache fills.
        let _ = solver.step(&state).unwrap();

        let rate = DecNsRate::new(&fixture.manifold, fixture.nu(), None).unwrap();
        let u = VelocityOneForm::new(state.as_one_form().clone(), &fixture.manifold).unwrap();
        let raw = rate.eval_unprojected(&u);

        let mut group = c.benchmark_group(format!("dec_solver_{n}x{n}x{n}_f64"));
        group.sample_size(10);

        group.bench_function("rate_unprojected", |b| {
            b.iter(|| black_box(rate.eval_unprojected(black_box(&u))))
        });

        // The generic compositional baseline the compiled stencil pipeline
        // is gated against (>= 2x serial required for the default switch).
        let generic_rate = DecNsRate::new(&fixture.manifold, fixture.nu(), None)
            .unwrap()
            .with_generic_assembly();
        group.bench_function("rate_unprojected_generic", |b| {
            b.iter(|| black_box(generic_rate.eval_unprojected(black_box(&u))))
        });

        // Opt-in spectral viscous term on the same fully periodic fixture.
        let spectral_rate = DecNsRate::new(&fixture.manifold, fixture.nu(), None)
            .unwrap()
            .with_spectral_diffusion()
            .unwrap();
        group.bench_function("rate_unprojected_spectral_diffusion", |b| {
            b.iter(|| black_box(spectral_rate.eval_unprojected(black_box(&u))))
        });

        group.bench_function("leray_project", |b| {
            b.iter(|| {
                black_box(
                    fixture
                        .manifold
                        .leray_project(black_box(raw.as_tensor()))
                        .unwrap(),
                )
            })
        });

        group.bench_function("solver_step", |b| {
            b.iter(|| black_box(solver.step(black_box(&state)).unwrap()))
        });

        group.finish();
    }
}

criterion_group!(benches, bench_dec_solver);
criterion_main!(benches);
