/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The study grammar lowered onto the carrier: the event-fork `fork/branch/continue_for` path with
//! its refinement round, and the origin-fork `baseline/alternate/ensemble` campaign.

use super::{EnsRow, field_at_61km, world};
use deep_causality_cfd::{BlackoutTrigger, CfdFlow, CompressibleMarchConfig};

// ── The campaign event-fork path: `study.fork(&pause).branch(f).continue_for(n)` lowers onto the
// carrier fan-out above, landing on `Marched` so `reduce` reads each branch through a `CaseRun`.

#[derive(Clone)]
struct BranchRow {
    n_tot: f64,
}

impl deep_causality_cfd::TableRow for BranchRow {
    type Scalar = f64;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[("final_n_tot", "-")];
    fn cells(&self) -> Vec<f64> {
        vec![self.n_tot]
    }
}

fn both_branches_reduced(v: &deep_causality_cfd::StudyView<'_, BranchRow>) -> (bool, String) {
    (
        v.rows().len() == 2,
        format!("{} branches reduced", v.rows().len()),
    )
}

#[test]
fn campaign_fork_branch_continue_for_reduces_branches_in_case_order() {
    use deep_causality_cfd::{CaseRun, GateSeq};

    // One shared onset pause; the case axis is the branch command that names each world.
    let nominal = world("nominal_descent", 3.0, 8);
    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    let verdict = CfdFlow::study("corridor branches")
        .cases(vec![
            "shallow_branch".to_string(),
            "steep_branch".to_string(),
        ])
        .fork(&pause)
        // `branch` lowers each case onto its own world, built fresh here — a world the study owns,
        // living only in this phase, which the relaxed carrier lifetime now permits.
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce(
            |run: &CaseRun<'_, String, CompressibleMarchConfig<f64>, f64>| {
                // Reports come back in case order: the branch world resumed the case's command.
                assert_eq!(run.report().name(), run.case());
                // Every branch carries the alternation marker the manual fork chain produces.
                let log = format!("{}", run.report().effect_log().unwrap());
                assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");
                let n_tot = run
                    .report()
                    .series("final_n_tot")
                    .map(|s| s[0])
                    .unwrap_or(0.0);
                Ok(BranchRow { n_tot })
            },
        )
        .gates(GateSeq::new("branches").gate("both reduced", both_branches_reduced))
        .verdict()
        .expect("the fork campaign runs to a verdict");

    assert!(verdict.passed(), "{verdict}");
}

fn coarse_round_is_carried(v: &deep_causality_cfd::StudyView<'_, BranchRow>) -> (bool, String) {
    let coarse_ok = v.rounds().len() == 1 && v.rounds()[0].len() == 2;
    let fine_ok = v.rows().len() == 3;
    (
        coarse_ok && fine_ok,
        format!(
            "fine round {} rows over {} prior round(s)",
            v.rows().len(),
            v.rounds().len()
        ),
    )
}

#[test]
fn refine_reforks_the_same_onset_and_carries_the_prior_round() {
    use deep_causality_cfd::{CaseRun, CompressibleMarchConfig, GateSeq};

    let nominal = world("nominal_descent", 3.0, 8);
    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            field_at_61km(),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();

    // Reduce each branch to its peak final density (the actual value is immaterial to the test —
    // what matters is that both rounds fork the same onset and the coarse round is retained).
    fn score(
        runs: &[CaseRun<'_, String, CompressibleMarchConfig<f64>, f64>],
    ) -> Result<Vec<BranchRow>, deep_causality_physics::PhysicsError> {
        Ok(runs
            .iter()
            .map(|r| BranchRow {
                n_tot: r
                    .report()
                    .series("final_n_tot")
                    .map(|s| s[0])
                    .unwrap_or(0.0),
            })
            .collect())
    }

    let verdict = CfdFlow::study("two-round corridor")
        .cases(vec!["coarse_a".to_string(), "coarse_b".to_string()])
        .fork(&pause)
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce_all(score)
        // The refinement round re-forks the *same* onset; its three fine cases are derived from
        // the two coarse rows (here just fixed names), and the coarse round is retained.
        .refine(&pause, |coarse: &[BranchRow]| {
            assert_eq!(coarse.len(), 2);
            Ok(vec![
                "fine_a".to_string(),
                "fine_b".to_string(),
                "fine_c".to_string(),
            ])
        })
        .branch(|name| Ok(world(name, 3.0, 8)))
        .continue_for(3)
        .reduce_all(score)
        .gates(GateSeq::new("rounds").gate("coarse carried", coarse_round_is_carried))
        .verdict()
        .expect("the two-round study runs");

    assert!(verdict.passed(), "{verdict}");
}

// ── The origin-fork counterfactual + ensemble path: baseline → alternate → ensemble → couple →
// march_for → reduce_ensemble (the weather campaign's machinery).

fn origin_campaign_is_well_formed(v: &deep_causality_cfd::StudyView<'_, EnsRow>) -> (bool, String) {
    let rows = v.rows();
    // Two cases; the baseline case is unmarked, the alternated case carries the marker; each case
    // flew its full ensemble of draws.
    let ok =
        rows.len() == 2 && !rows[0].marked && rows[1].marked && rows.iter().all(|r| r.draws == 2);
    (
        ok,
        format!(
            "{} cases; baseline marked={}, alternate marked={}; draws {:?}",
            rows.len(),
            rows[0].marked,
            rows[1].marked,
            rows.iter().map(|r| r.draws).collect::<Vec<_>>(),
        ),
    )
}

#[test]
fn origin_campaign_alternates_from_baseline_and_ensembles_the_draws() {
    use deep_causality_cfd::{GateSeq, Report};

    let verdict = CfdFlow::study("origin campaign")
        .cases(vec!["standard".to_string(), "hot".to_string()])
        // The declared baseline built once; `alternate` binds one world per case. The case whose
        // world name matches the baseline ("standard") flies unmarked; "hot" is alternated + marked.
        .baseline(|| Ok(world("standard", 3.0, 4)))
        .alternate(|name| Ok(world(name, 3.0, 4)))
        .ensemble(2) // two receiver-noise draws per case
        .couple(|_case: &String, _draw: usize| ()) // the trivial no-op stack
        .march_for(3, field_at_61km)
        .reduce_ensemble(|_case: &String, draws: &[Report<f64>]| {
            // The ensemble reduction sees the whole draw set per case.
            let marked = draws[0]
                .effect_log()
                .map(|l| format!("{l}").contains("!!ContextAlternation!!"))
                .unwrap_or(false);
            Ok(EnsRow {
                marked,
                draws: draws.len(),
            })
        })
        .gates(GateSeq::new("origin").gate("well formed", origin_campaign_is_well_formed))
        .verdict()
        .expect("the origin campaign runs to a verdict");

    assert!(verdict.passed(), "{verdict}");
}
