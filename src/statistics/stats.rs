#[derive(Debug)]
pub struct Statistics {
    run_duration: usize,
    request_count: usize,
    error_count: usize,
    rps: Option<usize>,
}

impl Statistics {
    pub fn new() -> Self {
        return Statistics {
            run_duration: 0,
            request_count: 0,
            error_count: 0,
            rps: None,
        };
    }

    pub fn set_duration(&mut self, duration: usize) {
        self.run_duration = duration;
    }

    pub fn set_request_count(&mut self, request_count: usize) {
        self.request_count = request_count;
    }
    pub fn set_error_count(&mut self, error_count: usize) {
        self.error_count = error_count
    }

    pub fn calculate(&mut self) {
        self.rps = Some(self.request_count / self.run_duration)
    }
}
