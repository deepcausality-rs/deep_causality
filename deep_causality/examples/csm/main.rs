// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use deep_causality_macros::make_run;

mod run;

// The macro assumes a mod named run with a pub method run().
fn main() { make_run!(); }
