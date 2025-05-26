use std::sync::mpsc::Sender;

use crate::statistics::stats::WorkerStats;
use anyhow::Result;

pub trait Job: CloneJob {
    fn execute(&mut self, stats_sender: Sender<WorkerStats>) -> Result<()>;
}

pub trait CloneJob {
    fn clone_job<'a>(&self) -> Box<dyn Job + Send + Sync>;
}

impl Clone for Box<dyn Job> {
    fn clone(&self) -> Self {
        return self.clone_job();
    }
}
