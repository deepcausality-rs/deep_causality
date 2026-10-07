// Trajectory level: march until the evolved sheath's n_e crosses the GPS L1 cutoff.
let onset = CfdFlow::march(&nominal)
    .couple(world::corridor_coupling(1.0, 0))
    .trigger(trigger)
    .from_field(world::initial_field())
    .until(|field, _| field.regime().map(|r| r.gnss_denied).unwrap_or(false))?;

// Campaign level: fork the paused onset once per candidate bank command, fly every
// branch concurrently, reduce to scored rows — then refine from the *same* onset with
// 0.5-degree candidates around the coarse winner, and gate the whole two-round result.
let corridor = CfdFlow::study("bank-angle corridor")
    .cases(model::coarse_commands())
    .fork(&onset)                            // the shared flow-resolved fork point
    .branch(model::bank_world)               // one alternated world per command, marked
    .continue_for(constants::BRANCH_STEPS)   // concurrent, copy-on-write
    .reduce_all(model::score_branches)       // aim point from the ballistic branch
    .refine(&onset, model::fine_candidates)  // second round, same paused onset
    .branch(model::bank_world)
    .continue_for(constants::BRANCH_STEPS)
    .reduce_all(model::score_branches)
    .gates(model::corridor_gates())          // steering beats ballistic; fine ≥ coarse
    .verdict()?;
