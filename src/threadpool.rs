use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
};

use crate::{jobs::job::Job, statistics::stats::Statistics};

pub struct Worker {
    id: u8,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: u8,
        receiver: Arc<Mutex<Receiver<Box<dyn Job + Sync + Send>>>>,
        stats_sender: Sender<Statistics>,
    ) -> Worker {
        let thread: JoinHandle<()> = spawn(move || {
            let local_job: Box<dyn Job + Send + Sync> = receiver.lock().unwrap().recv().unwrap();
            drop(receiver);
            local_job.execute(stats_sender);
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
pub struct ThreadPool {
    workers_pool: Vec<Worker>,
    sender: Option<Sender<Box<dyn Job + Send + Sync>>>,
    stats_recvr: Receiver<Statistics>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> ThreadPool {
        let mut workers = Vec::with_capacity(num_threads as usize);
        let (sender, receiver) = channel::<Box<dyn Job + Send + Sync>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let (stats_tx, stats_rx) = channel::<Statistics>();
        for id in 0..num_threads {
            println!("spawning worker: {}", id);
            workers.push(Worker::new(id, Arc::clone(&receiver), stats_tx.clone()));
        }
        ThreadPool {
            workers_pool: workers,
            sender: Some(sender),
            stats_recvr: stats_rx,
        }
    }

    pub fn start(&self, j: Box<dyn Job + Send + Sync>) {
        for _ in 0..self.workers_pool.len() {
            let new_job = j.clone_job();
            self.sender.as_ref().unwrap().send(new_job).unwrap();
        }
        let mut total_stats: Vec<Statistics> = vec![];
        for _ in 0..self.workers_pool.len() {
            let recvd_stats = self.stats_recvr.recv().unwrap();
            println!("{recvd_stats:?}");
            total_stats.push(recvd_stats);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers_pool {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
            println!("shutting down worker: {}", worker.id);
        }
    }
}
