// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality_macros::make_run;

mod run;
mod causal_state_machine;
mod causal_state;
mod causal_action;

// The macro assumes there is a mod run with a method run().
fn main() {
    make_run!();
}
