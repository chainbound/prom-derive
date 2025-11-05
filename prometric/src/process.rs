use prometheus::{
    Gauge, Registry,
    core::{AtomicU64, GenericGauge},
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, RefreshKind, System};

type UintGauge = GenericGauge<AtomicU64>;

type UintCounter = GenericGauge<AtomicU64>;

/// A collector for process metrics.
///
/// # Metrics
/// - `process_threads`: The number of OS threads used by the process (Linux only).
/// - `process_cpu_cores`: The number of logical CPU cores available in the system.
/// - `process_cpu_usage`: The CPU usage of the process as a percentage.
/// - `process_max_cpu_freq`: The maximum CPU frequency of all cores in MHz.
/// - `process_min_cpu_freq`: The minimum CPU frequency of all cores in MHz.
/// - `process_resident_memory_bytes`: The resident memory of the process in bytes.
/// - `process_resident_memory_usage`: The resident memory usage of the process as a percentage of
///   the total memory available.
/// - `process_start_time_seconds`: The start time of the process in UNIX seconds.
/// - `process_open_fds`: The number of open file descriptors of the process.
/// - `process_max_fds`: The maximum number of open file descriptors of the process.
/// - `process_disk_written_bytes_total`: The total written bytes to disk by the process.
///
/// # Example
/// ```rust
/// use prometheus::Registry;
/// use prometric::process::ProcessCollector;
///
/// let registry = Registry::new();
/// let mut collector = ProcessCollector::new(&registry);
///
/// // OR run with the default registry
/// let mut collector = ProcessCollector::default();
///
/// // Collect the metrics
/// collector.collect();
/// ```
pub struct ProcessCollector {
    specifics: RefreshKind,
    sys: System,
    pid: Pid,
    cores: u64,

    metrics: ProcessMetrics,
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self::new(prometheus::default_registry())
    }
}

impl ProcessCollector {
    /// Create a new `ProcessCollector` with the given registry.
    pub fn new(registry: &Registry) -> Self {
        let pid = Pid::from_u32(std::process::id());

        // Create the stats that will be refreshed
        let specifics = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::nothing().with_ram())
            .with_processes(
                ProcessRefreshKind::nothing()
                    .with_cpu()
                    .with_memory()
                    .with_disk_usage()
                    .with_tasks(),
            );

        let mut sys = sysinfo::System::new_with_specifics(specifics);

        // Refresh system information immediately for our first data point.
        sys.refresh_specifics(specifics);

        let cores = sys.cpus().len() as u64;
        let metrics = ProcessMetrics::new(registry);

        Self { specifics, sys, pid, cores, metrics }
    }

    /// The PID of the process being monitored.
    pub fn pid(&self) -> u32 {
        self.pid.as_u32()
    }

    /// Collect the metrics for the process.
    pub fn collect(&mut self) {
        self.sys.refresh_specifics(self.specifics);

        let cpus = self.sys.cpus();
        let min_cpu_freq = cpus.iter().map(|cpu| cpu.frequency()).min().unwrap();
        let max_cpu_freq = cpus.iter().map(|cpu| cpu.frequency()).max().unwrap();

        let process = self.sys.process(self.pid).unwrap();
        let threads = process.tasks().map(|tasks| tasks.len()).unwrap_or(0);
        let open_fds = process.open_files().unwrap_or(0);
        let max_fds = process.open_files_limit().unwrap_or(0);
        let cpu_usage = process.cpu_usage() / self.cores as f32;
        let resident_memory = process.memory();
        let resident_memory_usage = resident_memory as f64 / self.sys.total_memory() as f64;
        let disk_usage = process.disk_usage().total_written_bytes;

        self.metrics.cores.set(self.cores);
        self.metrics.max_cpu_freq.set(max_cpu_freq);
        self.metrics.min_cpu_freq.set(min_cpu_freq);

        self.metrics.threads.set(threads as u64);
        self.metrics.cpu_usage.set(cpu_usage as f64);
        self.metrics.resident_memory.set(resident_memory);
        self.metrics.resident_memory_usage.set(resident_memory_usage);
        self.metrics.start_time.set(process.start_time());
        self.metrics.open_fds.set(open_fds as u64);
        self.metrics.max_fds.set(max_fds as u64);
        self.metrics.disk_written_bytes.set(disk_usage);
    }
}

struct ProcessMetrics {
    // System metrics
    cores: UintGauge,
    max_cpu_freq: UintGauge,
    min_cpu_freq: UintGauge,

    // Process metrics
    threads: UintGauge,
    cpu_usage: Gauge,
    resident_memory: UintGauge,
    resident_memory_usage: Gauge,
    start_time: UintGauge,
    open_fds: UintGauge,
    max_fds: UintGauge,
    disk_written_bytes: UintCounter,
}

impl ProcessMetrics {
    pub fn new(registry: &prometheus::Registry) -> Self {
        let cores = UintGauge::new(
            "system_cpu_cores",
            "The number of logical CPU cores available in the system.",
        )
        .unwrap();
        let max_cpu_freq = UintGauge::new(
            "system_max_cpu_frequency",
            "The maximum CPU frequency of all cores in MHz.",
        )
        .unwrap();
        let min_cpu_freq = UintGauge::new(
            "system_min_cpu_frequency",
            "The minimum CPU frequency of all cores in MHz.",
        )
        .unwrap();

        let threads = UintGauge::new(
            "process_threads",
            "The number of OS threads used by the process (Linux only).",
        )
        .unwrap();
        let cpu_usage =
            Gauge::new("process_cpu_usage", "The CPU usage of the process as a percentage.")
                .unwrap();
        let resident_memory = UintGauge::new(
            "process_resident_memory_bytes",
            "The resident memory of the process in bytes. (RSS)",
        )
        .unwrap();
        let resident_memory_usage = Gauge::new(
            "process_resident_memory_usage",
            "The resident memory usage of the process as a percentage of the total memory available.",
        )
        .unwrap();
        let start_time = UintGauge::new(
            "process_start_time_seconds",
            "The start time of the process in UNIX seconds.",
        )
        .unwrap();
        let open_fds = UintGauge::new(
            "process_open_fds",
            "The number of open file descriptors of the process.",
        )
        .unwrap();
        let max_fds = UintGauge::new(
            "process_max_fds",
            "The maximum number of open file descriptors of the process.",
        )
        .unwrap();
        let disk_written_bytes = UintCounter::new(
            "process_disk_written_bytes_total",
            "The total written bytes to disk by the process.",
        )
        .unwrap();

        // Register all metrics with the registry
        registry.register(Box::new(cores.clone())).unwrap();
        registry.register(Box::new(max_cpu_freq.clone())).unwrap();
        registry.register(Box::new(min_cpu_freq.clone())).unwrap();

        registry.register(Box::new(threads.clone())).unwrap();
        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(resident_memory.clone())).unwrap();
        registry.register(Box::new(resident_memory_usage.clone())).unwrap();
        registry.register(Box::new(start_time.clone())).unwrap();
        registry.register(Box::new(open_fds.clone())).unwrap();
        registry.register(Box::new(max_fds.clone())).unwrap();
        registry.register(Box::new(disk_written_bytes.clone())).unwrap();

        Self {
            cores,
            max_cpu_freq,
            min_cpu_freq,
            threads,
            cpu_usage,
            resident_memory,
            resident_memory_usage,
            start_time,
            open_fds,
            max_fds,
            disk_written_bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_process_collector() {
        let registry = Registry::new();
        let mut collector = ProcessCollector::new(&registry);
        let start = Instant::now();
        collector.collect();
        let duration = start.elapsed();
        println!("Time taken for collection: {:?}", duration);

        let metrics = registry.gather();
        let encoder = prometheus::TextEncoder::new();
        let body = encoder.encode_to_string(&metrics).unwrap();
        println!("{}", body);
    }
}
