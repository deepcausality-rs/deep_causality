mod run;
mod types;
mod config;
mod utils;
mod io;

use deep_causality::prelude::time_execution;

fn main() {
    time_execution(run::run, "main_run");
}
