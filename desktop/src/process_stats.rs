use std::time::Instant;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessSample {
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

pub struct ProcessMeter {
    last_cpu_secs: f64,
    last_wall: Instant,
    last_sample: ProcessSample,
}

impl ProcessMeter {
    pub fn new() -> Self {
        let mut m = Self {
            last_cpu_secs: cpu_seconds(),
            last_wall: Instant::now(),
            last_sample: ProcessSample {
                cpu_percent: 0.0,
                memory_bytes: memory_bytes(),
            },
        };
        let _ = &mut m;
        m
    }

    pub fn sample(&mut self) -> ProcessSample {
        let now = Instant::now();
        let wall = now.duration_since(self.last_wall).as_secs_f64();
        let cpu_now = cpu_seconds();

        let cpu_percent = if wall > 0.05 {
            let delta = (cpu_now - self.last_cpu_secs).max(0.0);
            ((delta / wall) * 100.0).clamp(0.0, 100.0 * f64::from(num_cpus_f32())) as f32
        } else {
            self.last_sample.cpu_percent
        };

        self.last_cpu_secs = cpu_now;
        self.last_wall = now;
        self.last_sample = ProcessSample {
            cpu_percent,
            memory_bytes: memory_bytes(),
        };
        self.last_sample
    }
}

pub fn format_cpu(percent: f32) -> String {
    if percent < 10.0 {
        format!("{percent:.1}%")
    } else {
        format!("{percent:.0}%")
    }
}

pub fn format_memory(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{}MB", (bytes + MB / 2) / MB)
    } else if bytes >= KB {
        format!("{}KB", (bytes + KB / 2) / KB)
    } else {
        format!("{bytes}B")
    }
}

fn num_cpus_f32() -> f32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as f32)
        .unwrap_or(1.0)
        .max(1.0)
}

#[cfg(unix)]
fn cpu_seconds() -> f64 {
    unsafe {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        if libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) != 0 {
            return 0.0;
        }
        let u = usage.assume_init();
        timeval_secs(u.ru_utime) + timeval_secs(u.ru_stime)
    }
}

#[cfg(unix)]
fn timeval_secs(tv: libc::timeval) -> f64 {
    tv.tv_sec as f64 + (tv.tv_usec as f64) / 1_000_000.0
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn memory_bytes() -> u64 {
    unsafe {
        let mut info: libc::mach_task_basic_info = std::mem::zeroed();
        let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
        let kr = libc::task_info(
            libc::mach_task_self(),
            libc::MACH_TASK_BASIC_INFO,
            &mut info as *mut _ as *mut libc::integer_t,
            &mut count,
        );
        if kr == libc::KERN_SUCCESS {
            info.resident_size as u64
        } else {
            0
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn memory_bytes() -> u64 {
    let Ok(raw) = std::fs::read_to_string("/proc/self/statm") else {
        return 0;
    };
    let mut parts = raw.split_whitespace();
    let _ = parts.next();
    let resident_pages = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    resident_pages.saturating_mul(page_size())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn page_size() -> u64 {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 }
}

#[cfg(windows)]
fn cpu_seconds() -> f64 {
    use std::mem::MaybeUninit;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessTimes(
            process: isize,
            creation: *mut u64,
            exit: *mut u64,
            kernel: *mut u64,
            user: *mut u64,
        ) -> i32;
    }
    unsafe {
        let mut creation = MaybeUninit::<u64>::uninit();
        let mut exit = MaybeUninit::<u64>::uninit();
        let mut kernel = MaybeUninit::<u64>::uninit();
        let mut user = MaybeUninit::<u64>::uninit();
        if GetProcessTimes(
            GetCurrentProcess(),
            creation.as_mut_ptr(),
            exit.as_mut_ptr(),
            kernel.as_mut_ptr(),
            user.as_mut_ptr(),
        ) == 0
        {
            return 0.0;
        }
        let total = kernel.assume_init().wrapping_add(user.assume_init());
        (total as f64) / 10_000_000.0
    }
}

#[cfg(windows)]
fn memory_bytes() -> u64 {
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
    }
    #[link(name = "psapi")]
    extern "system" {
        fn GetProcessMemoryInfo(
            process: isize,
            ppsmem_counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    unsafe {
        let mut counters = std::mem::zeroed::<ProcessMemoryCounters>();
        counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) == 0 {
            return 0;
        }
        counters.working_set_size as u64
    }
}
