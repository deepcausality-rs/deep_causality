---
title: Deployment
description: A DeepCausality model carries no runtime of its own. It runs synchronously, asynchronously on Tokio, and concurrently across threads behind a shared lock that does not become a bottleneck.
sidebar:
  order: 14
---

A DeepCausality model may run in a normal binary, on a background thread, or across a pool of Tokio worker threads.

This page works through the deployment story using the [`tokio_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/tokio_example) crate.

## A runtime-agnostic core

A [Causaloid](/concepts/causaloid/) is evaluated through `evaluate(&self, ...)`. The call borrows the Causaloid; it does not consume or mutate it. The rule itself is a function pointer, so evaluation allocates nothing on the heap and dispatches nothing through a vtable. One value goes in, one `PropagatingEffect` comes out.

Two consequences follow. A single Causaloid can be evaluated any number of times. And because evaluation is a shared read, it can be evaluated from many places at once and is therefore the thread-safe.

## Asynchronous serving with Tokio

For a service that ingests a stream of events, pair the model with [Tokio](https://tokio.rs) and run inference on a task. The example's entry point is just twelve lines:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_handler = EventHandler::new(build_causal_model());

    tokio::spawn(async move {
        if let Err(e) = event_handler.run_background_inference().await {
            eprintln!("inference error: {e}");
        }
    })
    .await
    .expect("Failed to spawn async background task");

    Ok(())
}
```

The model is built once, the handler takes ownership, and inference runs on a spawned task while the main task stays free. In a real service the task is long-lived: it reads events off a channel and the `.await` becomes a graceful-shutdown handle.

The inference inside the task is synchronous. There is no `.await` in the evaluation path, no async Causaloid, no futures threaded through the causal logic. The asynchrony sits at the runtime boundary. This keeps the causal code simple and keeps the hot path free of executor overhead.

## Multi-threaded serving

The model is shareable across threads, and the lock that makes it shareable does not serialize the work. The handler holds the model behind an `Arc<RwLock<...>>`, and the Causaloid inside it is itself an `Arc`:

```rust
pub struct EventHandler {
    model: Arc<RwLock<BaseModelTokio>>,
}

pub async fn run_background_inference(&self) -> Result<(), Box<dyn Error + Send>> {
    let causaloid = {
        let model = self.model.read().unwrap();
        Arc::clone(model.causaloid())
        // read guard dropped here
    };

    for d in data.into_iter() {
        self.handle_inference(d, &causaloid)?;
    }
    Ok(())
}
```

The pattern is deliberate. A worker takes the read lock, clones the inner `Arc<Causaloid>`, and drops the guard immediately. The lock is held for the few microseconds it takes to copy a pointer. After that, the worker evaluates against its own `Arc` clone with no lock at all.

That is what allows concurrent serving. An `RwLock` admits many readers at once, so any number of Tokio worker threads can clone the pointer in parallel; none of them blocks on inference because none of them holds the lock while inferring. The same model answers one request or a million, and the Causaloid's `evaluate(&self)` signature guarantees the shared access is safe. A writer takes the lock only when the model or its [Context](/concepts/context/) must be updated, and only then do readers wait.

## Run the example

```bash
git clone https://github.com/deepcausality-rs/deep_causality
cd deep_causality
cargo run --release -p tokio_example
```

The program builds the handler, starts the background task, and prints one line per inference before exiting cleanly.
