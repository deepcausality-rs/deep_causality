//! Thread Pool Executor Implementation
//!
//! This module provides a thread-based executor for running tasks concurrently. It implements
//! the `EventProcessorExecutor` trait and manages a collection of `Runnable` tasks that can
//! be executed in parallel using native OS threads.
//!
//! # Features
//! - Concurrent execution of multiple tasks using native threads
//! - Safe handling of task completion and thread cleanup
//! - Support for both single and multiple task execution
//! - Automatic thread joining on handle drop
//!
//! # Examples
//! ```rust
//! use std::sync::atomic::{AtomicUsize, Ordering};
//! use std::sync::Arc;
//! use dcl_data_structures::ring_buffer::prelude::*;
//!
//! // Define a simple runnable task
//! struct CounterTask {
//!     counter: Arc<AtomicUsize>,
//! }
//!
//! impl Runnable for CounterTask {
//!     fn run(self: Box<Self>) {
//!         self.counter.fetch_add(1, Ordering::SeqCst);
//!     }
//! }
//!
//! // Create tasks that will increment a shared counter
//! let counter = Arc::new(AtomicUsize::new(0));
//! let tasks: Vec<Box<dyn Runnable>> = vec![
//!     Box::new(CounterTask { counter: counter.clone() }),
//!     Box::new(CounterTask { counter: counter.clone() }),
//! ];
//!
//! // Create and spawn the executor
//! let executor = ThreadedExecutor::with_runnables(tasks);
//! let handle = executor.spawn();
//!
//! // Wait for all tasks to complete
//! handle.join();
//!
//! // Both tasks should have incremented the counter
//! assert_eq!(counter.load(Ordering::SeqCst), 2);
//! ```

use crate::ring_buffer::prelude::*;

/// A thread-based executor that manages and runs a collection of tasks concurrently.
///
/// The executor takes ownership of the tasks and spawns a new OS thread for each task
/// when spawned. Each task runs independently and concurrently with other tasks.
///
/// # Type Parameters
/// - `'a`: Lifetime parameter for the contained runnable tasks
pub struct ThreadedExecutor<'a> {
    runnables: Vec<Box<dyn Runnable + 'a>>,
}

/// A handle to a spawned `ThreadedExecutor` that manages the lifecycle of spawned threads.
///
/// This handle is responsible for joining all spawned threads when dropped or when
/// explicitly joined. It ensures that all tasks complete before the executor is cleaned up.
pub struct ThreadedExecutorHandle {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl<'a> EventProcessorExecutor<'a> for ThreadedExecutor<'a> {
    type Handle = ThreadedExecutorHandle;

    /// Creates a new `ThreadedExecutor` with the given collection of runnable tasks.
    ///
    /// # Parameters
    /// - `runnables`: A vector of boxed tasks implementing the `Runnable` trait
    ///
    /// # Returns
    /// A new `ThreadedExecutor` instance that owns the provided tasks
    fn with_runnables(runnables: Vec<Box<dyn Runnable + 'a>>) -> Self {
        Self { runnables }
    }

    /// Spawns all tasks onto separate threads and returns a handle to manage them.
    ///
    /// This method takes ownership of the executor and its tasks, spawning each task
    /// onto its own OS thread. The threads begin execution immediately.
    ///
    /// # Returns
    /// A `ThreadedExecutorHandle` that can be used to join the spawned threads
    ///
    /// # Safety
    /// This method uses unsafe code to extend the lifetime of the runnable tasks
    /// to 'static. This is safe because:
    /// 1. The threads are joined before the handle is dropped
    /// 2. The tasks are guaranteed to complete within their original lifetime
    fn spawn(self) -> Self::Handle {
        let mut threads = Vec::new();
        for r in self.runnables.into_iter() {
            let b = unsafe {
                std::mem::transmute::<Box<dyn Runnable + 'a>, Box<dyn Runnable + 'static>>(r)
            };
            threads.push(std::thread::spawn(move || b.run()));
        }

        ThreadedExecutorHandle { threads }
    }
}

impl ExecutorHandle for ThreadedExecutorHandle {
    /// Joins all spawned threads, waiting for them to complete.
    ///
    /// This method is equivalent to dropping the handle, as the drop implementation
    /// performs the same joining operation.
    fn join(self) {
        drop(self)
    }
}

impl Drop for ThreadedExecutorHandle {
    /// Ensures all spawned threads are joined when the handle is dropped.
    ///
    /// This implementation:
    /// 1. Takes ownership of the thread handles
    /// 2. Joins each thread, ensuring it completes
    /// 3. Panics if any thread panicked during execution
    fn drop(&mut self) {
        let threads = std::mem::take(&mut self.threads);
        for t in threads.into_iter() {
            t.join().unwrap();
        }
    }
}
