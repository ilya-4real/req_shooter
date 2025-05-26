use colored::Colorize;
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
};

use anyhow::Result;

use crate::{
    jobs::job::Job,
    statistics::stats::{SummaryStatistics, WorkerStats},
};

pub struct Worker {
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        receiver: Arc<Mutex<Receiver<Box<dyn Job + Sync + Send>>>>,
        stats_sender: Sender<WorkerStats>,
    ) -> Worker {
        let thread: JoinHandle<()> = spawn(move || {
            let mut thread_job: Box<dyn Job + Send + Sync> =
                receiver.lock().unwrap().recv().unwrap();
            let job_result = thread_job.execute(stats_sender);
            match job_result {
                Err(e) => {
                    println!("Got an error inside one of threads: {:?}", e)
                }
                _ => {}
            }
        });
        Worker {
            thread: Some(thread),
        }
    }
}
pub struct ThreadPool {
    workers_pool: Vec<Worker>,
    sender: Sender<Box<dyn Job + Send + Sync>>,
    stats_recvr: Receiver<WorkerStats>,
}

impl ThreadPool {
    pub fn new(num_threads: u8) -> ThreadPool {
        let mut workers = Vec::with_capacity(num_threads as usize);
        let (sender, receiver) = channel::<Box<dyn Job + Send + Sync>>();
        let receiver = Arc::new(Mutex::new(receiver));
        let (stats_tx, stats_rx) = channel::<WorkerStats>();
        println!(
            "{}",
            format!("Spawning workers: {}", num_threads).cyan().bold()
        );
        for _ in 0..num_threads {
            workers.push(Worker::new(Arc::clone(&receiver), stats_tx.clone()));
        }
        ThreadPool {
            workers_pool: workers,
            sender,
            stats_recvr: stats_rx,
        }
    }

    pub fn start(&self, job: Box<dyn Job + Send + Sync>) -> Result<()> {
        for _ in 0..self.workers_pool.len() {
            let new_job = job.clone_job();
            self.sender.send(new_job)?;
        }
        let mut workers_stats: Vec<WorkerStats> = vec![];
        for _ in 0..self.workers_pool.len() {
            let recvd_stats = self.stats_recvr.recv()?;
            workers_stats.push(recvd_stats);
        }
        SummaryStatistics::new(workers_stats).represent();
        return Ok(());
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers_pool {
            if let Some(thread) = worker.thread.take() {
                thread
                    .join()
                    .expect("Unable to join thread. Stopping the program");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThreadPool;

    #[test]
    #[ignore = "not yet implemented"]
    fn test_creating_threadpool() {
        let workers_number = 4;
        let threadpool = ThreadPool::new(workers_number);
        assert_eq!(threadpool.workers_pool.len(), workers_number as usize)
    }
}
