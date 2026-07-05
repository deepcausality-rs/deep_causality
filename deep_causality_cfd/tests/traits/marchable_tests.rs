/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Marchable::march` is the campaign's one-case-one-report seam. For each uncoupled family it
//! must produce exactly the report the family's canonical entry produces, so the study grammar
//! and the trajectory entry never diverge.

use deep_causality_cfd::{
    CfdConfigBuilder, CfdFlow, DuctAreaProfile, DuctConfig, DuctInlet, DuctStop, Marchable,
    MarchStop, Mesh, Observe, QttMarchConfig, QttMarchConfigBuilder, QttObserve, Seed,
};
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;

fn nozzle() -> DuctConfig<f64> {
    let p0 = 101_325.0;
    DuctConfig::new(
        DuctAreaProfile::ConvergingDiverging {
            inlet_area: 2.0,
            throat_area: 1.0,
            exit_area: 2.0,
            length: 1.0,
        },
        DuctInlet { p0, t0: 300.0 },
        1.4,
        p0 * 0.5,
        64,
        DuctStop {
            max_steps: 2_000,
            residual_tol: 1.0e-8,
        },
    )
    .expect("valid nozzle config")
}

fn cavity() -> deep_causality_cfd::MarchConfig<2, f64, (), ()> {
    CfdConfigBuilder::march::<2, f64>("dsl")
        .mesh(Mesh::box_domain([8, 8]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .build()
                .expect("valid solver config"),
        )
        .seed(Seed::Rest)
        .march_for(3)
        .observe(Observe::default().kinetic_energy())
        .build()
        .expect("valid march config")
}

fn taylor_green() -> QttMarchConfig<f64> {
    let n = 16usize;
    let dx = TAU / n as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    QttMarchConfigBuilder::<f64>::new()
        .name("tg")
        .grid(4, 4, dx, dx)
        .solver(0.02, 0.05, trunc)
        .seed_fn(|x: f64, y: f64| (-(x.cos() * y.sin()), x.sin() * y.cos()))
        .unwrap()
        .stop(MarchStop::Fixed(5))
        .observe(QttObserve::default().kinetic_energy())
        .build()
        .unwrap()
}

#[test]
fn duct_config_marchable_matches_the_entry() {
    let cfg = nozzle();
    let via_trait = cfg.march().expect("marchable duct");
    let via_entry = CfdFlow::duct_march(&cfg).run().expect("duct entry");
    assert_eq!(
        via_trait.series("mach_profile"),
        via_entry.series("mach_profile"),
        "Marchable::march equals the duct entry"
    );
}

#[test]
fn march_config_marchable_matches_run_owned() {
    let cfg = cavity();
    let via_trait = cfg.march().expect("marchable DEC");
    let via_entry = CfdFlow::march(&cfg).run_owned().expect("DEC run_owned");
    assert_eq!(
        via_trait.series("kinetic_energy"),
        via_entry.series("kinetic_energy"),
        "Marchable::march equals CfdFlow::march(..).run_owned()"
    );
}

#[test]
fn qtt_config_marchable_matches_the_entry() {
    let cfg = taylor_green();
    let via_trait = cfg.march().expect("marchable QTT");
    let via_entry = CfdFlow::qtt_march(&cfg).run().expect("QTT entry");
    assert_eq!(
        via_trait.series("kinetic_energy"),
        via_entry.series("kinetic_energy"),
        "Marchable::march equals CfdFlow::qtt_march(..).run()"
    );
}
