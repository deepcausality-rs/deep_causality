mod run;

use deep_causality::prelude::time_execution;

fn main() {
    time_execution(run::run, "main_run");
}
