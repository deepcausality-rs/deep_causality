/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

mod run;
mod utils;

fn main() {
    utils::print_header("Smoking Example!");
    utils::time(run::run, "main_run");
}
