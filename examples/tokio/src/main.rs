/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod handler;
mod model;
mod types;
mod utils;

use crate::handler::EventHandler;
use crate::model::build_causal_model;

const FN_NAME: &str = "examples/tokio";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{FN_NAME}: Build Event Handler with causal model and context");
    let event_handler = EventHandler::new(build_causal_model());

    println!("{FN_NAME}: Start the data handler as background task",);
    tokio::spawn(async move {
        if let Err(e) = event_handler.run_background_inference().await {
            eprintln!("{FN_NAME}]: inference error: {e}");
        }
    })
    .await
    .expect("Failed to spawn async background task");

    Ok(())
}
