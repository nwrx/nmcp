use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemStatusCpu {
    /// The model of the CPU (e.g., `Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz`).
    pub model: String,

    /// The frequency of the CPU in MHz.
    pub speed: u64,

    /// The CPU usage percentage.
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    /// The uptime of the system in seconds.
    pub uptime: u64,

    /// The architecture of the CPU (e.g., `x86_64`, `arm64`).
    pub arch: String,

    /// The family of the CPU architecture (e.g., `x86`, `arm`).
    pub family: String,

    /// The platform the system is running on (e.g., `linux`, `windows`, `darwin`).
    pub platform: String,

    /// The version of the kernel.
    pub version: String,

    /// The release version of the operating system.
    pub release: String,

    /// The amount of free memory in bytes.
    pub memory_free: u64,

    /// The amount of used memory in bytes.
    pub memory_used: u64,

    /// The total amount of memory in bytes.
    pub memory_total: u64,

    /// The amount of available memory in bytes (total memory - free memory).
    pub memory_available: u64,

    /// The number of available parallelism threads.
    pub available_parallelism: usize,

    /// The load average over the last 1, 5, and 15 minutes.
    pub cpu_average_load: [f64; 3],

    /// The average CPU speed in MHz.
    pub cpu_average_speed: u64,

    /// A list of CPU information.
    pub cpus: Vec<SystemStatusCpu>,
}

impl Default for SystemStatus {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        system.refresh_all();

        let cpu = system.cpus();

        Self {
            uptime: System::uptime(),
            arch: System::cpu_arch(),
            family: std::env::consts::FAMILY.to_string(),
            platform: std::env::consts::OS.to_string(),
            version: System::kernel_long_version(),
            release: System::long_os_version().unwrap_or("Unknown".to_string()),
            memory_free: system.free_memory(),
            memory_used: system.used_memory(),
            memory_total: system.total_memory(),
            memory_available: system.available_memory(),
            available_parallelism: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            cpu_average_load: {
                let load_avg = System::load_average();
                [load_avg.one, load_avg.five, load_avg.fifteen]
            },
            cpu_average_speed: cpu.iter().map(|c| c.frequency()).sum::<u64>() / cpu.len() as u64,
            cpus: cpu
                .iter()
                .map(|cpu| SystemStatusCpu {
                    model: cpu.brand().to_string(),
                    speed: cpu.frequency(),
                    usage: cpu.cpu_usage(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_system_default() {
        let status = SystemStatus::default();
        assert!(status.uptime > 0);
        assert!(status.memory_total > 0);
        assert!(status.memory_free <= status.memory_total);
        assert!(status.memory_available <= status.memory_total);
        assert!(status.available_parallelism > 0);
        assert!(status.cpu_average_load.iter().all(|&x| x >= 0.0));
        assert!(!status.arch.is_empty());
        assert!(!status.family.is_empty());
        assert!(!status.platform.is_empty());
        assert!(!status.version.is_empty());
        assert!(!status.release.is_empty());
    }
}
