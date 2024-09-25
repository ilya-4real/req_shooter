pub trait Job: CloneJob {
    fn execute(&self);
}

pub trait CloneJob {
    fn clone_job<'a>(&self) -> Box<dyn Job + Send + Sync>;
}

impl Clone for Box<dyn Job> {
    fn clone(&self) -> Self {
        return self.clone_job();
    }
}
