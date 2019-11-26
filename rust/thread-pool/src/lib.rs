use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub trait Runnable {
    fn run(&self);
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::SyncSender<Message>,
}

impl ThreadPool {
    pub fn new(queue_size: usize, number_of_threads: usize) -> Self {
        let mut workers = Vec::with_capacity(number_of_threads);
        let (sender, receiver) = mpsc::sync_channel(queue_size);
        let receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..number_of_threads {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }
        Self { workers, sender }
    }

    pub fn dispatch<T: Runnable + Send + 'static>(&mut self, runnable: T) {
        let job = Box::new(move || runnable.run());
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => job.call_box(),
                Message::Terminate => break,
            }
        });
        Worker {
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Condvar;

    #[derive(Debug, Clone)]
    struct CounterTask {
        pair: Arc<(Mutex<usize>, Condvar)>,
    }

    impl CounterTask {
        fn new() -> Self {
            let pair = Arc::new((Mutex::new(0), Condvar::new()));
            Self { pair }
        }

        fn wait_for_run_count(&self, count: usize) -> usize {
            let (lock, cvar) = &*self.pair;
            let mut state = lock.lock().unwrap();
            while *state < count {
                state = cvar.wait(state).unwrap();
            }
            *state
        }
    }

    impl Runnable for CounterTask {
        fn run(&self) {
            let (lock, cvar) = &*self.pair;
            let mut state = lock.lock().unwrap();
            *state += 1;
            cvar.notify_all();
        }
    }

    #[derive(Debug, Clone)]
    struct LatchTask {
        pair: Arc<(Mutex<(usize, usize)>, Condvar)>,
    }

    impl LatchTask {
        fn new(count: usize) -> Self {
            let pair = Arc::new((Mutex::new((count, 0)), Condvar::new()));
            Self { pair }
        }

        fn wait_for_latch_count(&self) {
            let (lock, cvar) = &*self.pair;
            let mut state = lock.lock().unwrap();
            while (*state).1 < (*state).0 {
                state = cvar.wait(state).unwrap();
            }
        }
    }

    impl Runnable for LatchTask {
        fn run(&self) {
            let (lock, cvar) = &*self.pair;
            let mut state = lock.lock().unwrap();
            (*state).1 += 1;
            cvar.notify_all();
            while (*state).1 < (*state).0 {
                state = cvar.wait(state).unwrap();
            }
        }
    }

    #[test]
    fn test_simple_dispatch() {
        let mut tp = ThreadPool::new(1, 1);
        let t = CounterTask::new();
        tp.dispatch(t.clone());
        t.wait_for_run_count(1);
    }

    #[test]
    fn test_simple_repeated_dispatch() {
        let mut tp = ThreadPool::new(1, 1);
        let t = CounterTask::new();
        for _ in 0..10 {
            tp.dispatch(t.clone());
        }
        t.wait_for_run_count(10);
    }

    #[test]
    fn test_complex_repeated_dispatch() {
        let mut tp = ThreadPool::new(10, 10);
        let t = CounterTask::new();
        for _ in 0..1000 {
            tp.dispatch(t.clone());
        }
        t.wait_for_run_count(1000);
    }

    #[test]
    fn test_complex_repeated_dispatch_2() {
        let mut tp = ThreadPool::new(10, 10);
        let mut tasks = vec![];
        for _ in 0..10 {
            tasks.push(CounterTask::new());
        }
        for _ in 0..100 {
            for t in &tasks {
                tp.dispatch(t.clone());
            }
        }
        for t in &tasks {
            t.wait_for_run_count(100);
        }
    }

    #[test]
    fn test_latch_simple_dispatch() {
        let number_of_threads = 10;

        let mut tp = ThreadPool::new(10, number_of_threads);
        let t = LatchTask::new(number_of_threads);
        for _ in 0..number_of_threads {
            tp.dispatch(t.clone());
        }
        t.wait_for_latch_count();
    }

    #[test]
    fn test_queue_size() {
        let size_of_queue = 10;
        let mut _tp = ThreadPool::new(size_of_queue, 1);
        // T.B.D.
    }

    #[test]
    fn test_latch_complex_dispatch() {
        let number_of_threads = 10;
        let mut tp = ThreadPool::new(10, number_of_threads);

        let mut tasks = vec![];
        for _ in 0..10 {
            tasks.push(LatchTask::new(number_of_threads));
        }
        for t in &tasks {
            for _ in 0..number_of_threads {
                tp.dispatch(t.clone());
            }
        }
        for t in &tasks {
            t.wait_for_latch_count();
        }
    }
}
