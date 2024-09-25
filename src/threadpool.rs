use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{spawn, JoinHandle},
};

use crate::jobs::job::Job;

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<Receiver<Box<dyn Job + Sync + Send>>>>) -> Worker {
        let thread: JoinHandle<()> = spawn(move || {
            let local_job: Box<dyn Job + Send + Sync> = receiver.lock().unwrap().recv().unwrap();
            drop(receiver);
            local_job.execute();
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    workers_pool: Vec<Worker>,
    sender: Option<Sender<Box<dyn Job + Send + Sync>>>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(num_threads);
        let (sender, receiver) = channel::<Box<dyn Job + Send + Sync>>();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..num_threads {
            println!("spawning worker: {}", id);
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers_pool: workers,
            sender: Some(sender),
        }
    }

    pub fn start(&self, j: Box<dyn Job + Send + Sync>) {
        for _ in 0..self.workers_pool.len() {
            let new_joba = j.clone_job();
            self.sender.as_ref().unwrap().send(new_joba).unwrap();
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
