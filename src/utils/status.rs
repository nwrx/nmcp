use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatusSystemCpu {
    /// The model of the CPU (e.g., `Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz`).
    pub model: String,

    /// The frequency of the CPU in MHz.
    pub frequency: u64,

    /// The CPU usage percentage.
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusSystem {
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
    pub freemem: u64,

    /// The total amount of memory in bytes.
    pub totalmem: u64,

    /// The amount of available memory in bytes (total memory - free memory).
    pub availmem: u64,

    /// The number of available parallelism threads.
    pub available_parallelismv: usize,

    /// The load average over the last 1, 5, and 15 minutes.
    pub loadavg: [f64; 3],

    /// A list of CPU information.
    pub cpus: Vec<StatusSystemCpu>,
}

impl Default for StatusSystem {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            uptime: System::uptime(),
            arch: System::cpu_arch(),
            family: std::env::consts::FAMILY.to_string(),
            platform: std::env::consts::OS.to_string(),
            version: System::kernel_long_version(),
            release: System::long_os_version().unwrap_or("Unknown".to_string()),
            freemem: system.free_memory(),
            totalmem: system.total_memory(),
            availmem: system.available_memory(),
            available_parallelismv: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            loadavg: {
                let load_avg = System::load_average();
                [load_avg.one, load_avg.five, load_avg.fifteen]
            },
            cpus: system
                .cpus()
                .iter()
                .map(|cpu| StatusSystemCpu {
                    model: cpu.brand().to_string(),
                    frequency: cpu.frequency(),
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
        let status = StatusSystem::default();
        assert!(status.uptime > 0);
        assert!(status.totalmem > 0);
        assert!(status.freemem <= status.totalmem);
        assert!(status.availmem <= status.totalmem);
        assert!(status.available_parallelismv > 0);
        assert!(status.loadavg.iter().all(|&x| x >= 0.0));
        assert!(!status.arch.is_empty());
        assert!(!status.family.is_empty());
        assert!(!status.platform.is_empty());
        assert!(!status.version.is_empty());
        assert!(!status.release.is_empty());
    }
}
