use crate::utils;
use colored::Colorize;

#[derive(Debug)]
pub struct WorkerStats {
    run_duration: usize,
    request_count: u32,
    error_count: u32,
    bad_requests: u32,
    received_data: usize,
    mean_latency: f64,
    stdev_latency: f64,
}

impl WorkerStats {
    pub fn new(
        run_duration: usize,
        request_count: u32,
        error_count: u32,
        bad_requests: u32,
        received_data: usize,
    ) -> Self {
        return WorkerStats {
            run_duration,
            request_count,
            error_count,
            bad_requests,
            received_data,
            mean_latency: 0.0,
            stdev_latency: 0.0,
        };
    }

    pub fn calculate_latencies(&mut self, latencies: Vec<f64>) {
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
    rps: u32,
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
            rps: total_requests / job_duration as u32,
        };
    }

    pub fn represent(&self) {
        let header = format!(
            "\n{}\n{}\n",
            "Statistics by workers:".cyan().bold(),
            "\tworker id\t mean latency\t\t stdev latency\t\t requests sent\t\t errors\t\t received data"
                .cyan()
                .underline()
        );
        print!("{header}");
        let mut non_200_300_requests = 0;
        let mut total_errors = 0;
        let mut mean_latencies = 0.0;
        let mut total_received_data = 0;
        for (index, worker_stat) in self.worker_stats.iter().enumerate() {
            println!(
                "\tworker {}\t {:.2}ms\t\t\t {:.2}ms\t\t\t {}\t\t\t {}\t\t {}",
                index,
                worker_stat.mean_latency / 1000.0,
                worker_stat.stdev_latency / 1000.0,
                worker_stat.request_count,
                worker_stat.error_count,
                utils::format_received_data_value(worker_stat.received_data)
            );
            non_200_300_requests += worker_stat.bad_requests;
            total_errors += worker_stat.error_count;
            total_received_data += worker_stat.received_data;
            mean_latencies += worker_stat.mean_latency;
        }
        let total_mean_latency = mean_latencies / self.worker_stats.len() as f64;
        println!();
        println!("{}", "Summary:".cyan().bold().underline());
        println!(
            "{}{}",
            "\tRequests per second:\t\t ".bright_green(),
            format!("{}", self.rps).bright_green()
        );
        println!(
            "\tTotal data received:\t\t {}",
            utils::format_received_data_value(total_received_data)
        );
        println!("\tMean latency:\t\t\t {:.2}ms", total_mean_latency / 1000.0);
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

#[cfg(test)]
mod test_statistics {
    use super::WorkerStats;

    #[test]
    fn test_mean_calculation() {
        let mut worker_stats = WorkerStats::new(1, 3, 0, 0, 0);
        let latencies = vec![1.0, 2.0, 3.0];
        worker_stats.calculate_latencies(latencies);
        assert_eq!(worker_stats.mean_latency, 2.0);
    }

    #[test]
    fn test_stdev_calculation() {
        let mut worker_stats = WorkerStats::new(1, 3, 0, 0, 0);
        let latencies = vec![1.0, 2.0, 3.0];
        worker_stats.calculate_latencies(latencies);
        let dispersion: f64 = 2.0 / 3.0;
        assert_eq!(worker_stats.stdev_latency, dispersion.sqrt());
    }
}
