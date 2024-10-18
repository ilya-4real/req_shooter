use colored::Colorize;

#[derive(Debug)]
pub struct WorkerStats {
    run_duration: usize,
    request_count: usize,
    error_count: usize,
    bad_requests: usize,
    mean_latency: f64,
    stdev_latency: f64,
}

impl WorkerStats {
    pub fn new() -> Self {
        return WorkerStats {
            run_duration: 0,
            request_count: 0,
            error_count: 0,
            bad_requests: 0,
            mean_latency: 0.0,
            stdev_latency: 0.0,
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

    pub fn set_bad_requests(&mut self, bad_requests: usize) {
        self.bad_requests = bad_requests;
    }

    pub fn calculate(&mut self, latencies: Vec<f64>) {
        self.mean_latency = latencies.iter().sum::<f64>() / self.request_count as f64;
        let latency_variation: f64 = latencies
            .iter()
            .map(|lat| (lat - self.mean_latency).powi(2))
            .sum();
        self.stdev_latency = (latency_variation / self.request_count as f64).sqrt();
    }
}

pub struct SummaryStatistics {
    worker_stats: Vec<WorkerStats>,
    rps: usize,
}

impl SummaryStatistics {
    pub fn new(workers_stats: Vec<WorkerStats>) -> SummaryStatistics {
        let mut total_requests = 0;
        let job_duration = workers_stats[0].run_duration;
        for worker in &workers_stats {
            total_requests += worker.request_count
        }
        return SummaryStatistics {
            worker_stats: workers_stats,
            rps: total_requests / job_duration,
        };
    }

    pub fn represent(&self) {
        let header = format!(
            "\n{}\n{}\n",
            "Statistics by workers:".cyan().bold(),
            "\tworker id\t mean latency\t\t stdev latency\t\t requests sent\t\t errors"
                .cyan()
                .underline()
        );
        print!("{header}");
        let mut non_200_300_requests = 0;
        let mut total_errors = 0;
        for (index, worker_stat) in self.worker_stats.iter().enumerate() {
            println!(
                "\tworker {}\t {:.2}ms\t\t\t {:.2}ms\t\t\t {}\t\t\t {}",
                index,
                worker_stat.mean_latency / 1000 as f64,
                worker_stat.stdev_latency / 1000 as f64,
                worker_stat.request_count,
                worker_stat.error_count
            );
            non_200_300_requests += worker_stat.bad_requests;
            total_errors += worker_stat.error_count
        }
        println!();
        println!("{}", "Summary:".cyan().bold().underline());
        println!(
            "{}{}",
            "\tRequests per second:\t\t ".bright_green(),
            format!("{}", self.rps).bright_green()
        );
        println!(
            "{}{}",
            "\tNot 2** or 3** server responses: ",
            format!("{}", non_200_300_requests)
        );
        println!(
            "{}{}",
            "\tConnection errors happened:\t ",
            format!("{}", total_errors)
        );
    }
}
