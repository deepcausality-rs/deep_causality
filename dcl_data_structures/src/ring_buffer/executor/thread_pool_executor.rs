use crate::ring_buffer::prelude::*;

pub struct ThreadedExecutor<'a> {
    runnables: Vec<Box<dyn Runnable + 'a>>,
}

pub struct ThreadedExecutorHandle {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl<'a> EventProcessorExecutor<'a> for ThreadedExecutor<'a> {
    type Handle = ThreadedExecutorHandle;

    fn with_runnables(runnables: Vec<Box<dyn Runnable + 'a>>) -> Self {
        Self { runnables }
    }

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
    fn join(self) {
        drop(self)
    }
}

impl Drop for ThreadedExecutorHandle {
    fn drop(&mut self) {
        let threads = std::mem::take(&mut self.threads);
        for t in threads.into_iter() {
            t.join().unwrap();
        }
    }
}
