
// cpu_logger.rs - Log periodico CPU server (cross-platform)

// Import di terze parti
use sysinfo::{CpuRefreshKind, RefreshKind, System, Pid, ProcessRefreshKind, ProcessesToUpdate};

// Funzioni per tempo CPU totale (user+system), una per OS
#[cfg(target_os = "linux")]
// Linux: somma utime+stime da /proc/self/stat
fn get_cpu_time() -> Option<f64> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = File::open("/proc/self/stat").ok()?;
    let mut line = String::new();
    BufReader::new(file).read_line(&mut line).ok()?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 17 { return None; }
    let utime: f64 = parts[13].parse().ok()?;
    let stime: f64 = parts[14].parse().ok()?;
    let clock_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    Some((utime + stime) / clock_ticks)
}

#[cfg(target_os = "windows")]
// Windows: somma FILETIME user+kernel (API Win)
fn get_cpu_time() -> Option<f64> {
    use std::mem::MaybeUninit;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::processthreadsapi::GetProcessTimes;
    use winapi::shared::minwindef::FILETIME;
    unsafe {
        let handle = GetCurrentProcess();
        let mut creation = MaybeUninit::<FILETIME>::uninit();
        let mut exit = MaybeUninit::<FILETIME>::uninit();
        let mut kernel = MaybeUninit::<FILETIME>::uninit();
        let mut user = MaybeUninit::<FILETIME>::uninit();
        let ok = GetProcessTimes(
            handle,
            creation.as_mut_ptr(),
            exit.as_mut_ptr(),
            kernel.as_mut_ptr(),
            user.as_mut_ptr(),
        );
        if ok == 0 { return None; }
        let kernel = kernel.assume_init();
        let user = user.assume_init();
        // FILETIME: somma i due campi pubblici per ottenere i 64 bit
        let filetime_to_u64 = |ft: &FILETIME| -> u64 {
            (ft.dwLowDateTime as u64) | ((ft.dwHighDateTime as u64) << 32)
        };
        // Ogni tick è 100 nanosecondi
        let filetime_to_secs = |ft: &FILETIME| -> f64 {
            filetime_to_u64(ft) as f64 * 1e-7
        };
        Some(filetime_to_secs(&kernel) + filetime_to_secs(&user))
    }
}

#[cfg(target_os = "macos")]
// macOS: somma user+system da getrusage
fn get_cpu_time() -> Option<f64> {
    use libc::{rusage, getrusage, RUSAGE_SELF};
    unsafe {
        let mut usage: rusage = std::mem::zeroed();
        if getrusage(RUSAGE_SELF, &mut usage) != 0 {
            return None;
        }
        let user = usage.ru_utime.tv_sec as f64 + usage.ru_utime.tv_usec as f64 * 1e-6;
        let sys = usage.ru_stime.tv_sec as f64 + usage.ru_stime.tv_usec as f64 * 1e-6;
        Some(user + sys)
    }
}
use chrono::Local;
use tokio::time::{interval, Duration};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

// Avvia un task che logga ogni N secondi:
// - percentuale CPU istantanea
// - tempo CPU usato solo nell'intervallo
// Log su "server_cpu_stats.log"
pub fn spawn_cpu_logger(period_secs: u64) {
    // Task periodico Tokio
    tokio::spawn(async move {
        // Configura sysinfo solo per CPU/processi
        let mut sys = System::new_with_specifics(
            RefreshKind::default()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        // PID processo server
        let pid = Pid::from_u32(std::process::id());
        // Timer periodico
        let mut ticker = interval(Duration::from_secs(period_secs));

        // Tempo CPU precedente
        let mut prev_cpu_time = get_cpu_time().unwrap_or(0.0);

        // Loop: ogni tick logga
        loop {
            ticker.tick().await; // Attende il prossimo tick
            // Aggiorna CPU/processi
            sys.refresh_specifics(RefreshKind::default().with_cpu(CpuRefreshKind::everything()));
            sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());

            // Percentuale CPU istantanea
            let process_cpu = sys.process(pid)
                .map(|proc| proc.cpu_usage())
                .unwrap_or(0.0);

            // Tempo CPU usato solo nell'intervallo
            let cpu_time_total = get_cpu_time().unwrap_or(-1.0);
            let cpu_time_interval = if cpu_time_total >= 0.0 {
                let delta = cpu_time_total - prev_cpu_time;
                prev_cpu_time = cpu_time_total;
                if delta >= 0.0 { delta } else { 0.0 }
            } else {
                -1.0
            };
            // Log: timestamp, %CPU, tempo CPU intervallo
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!(
                "[{}] Server CPU Usage: {:.5}% | CPU Time Usage: {:.3}s\n",
                timestamp, process_cpu, cpu_time_interval
            );
            // Scrive nel file di log
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open("server_cpu_stats.log").await
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(log_line.as_bytes()).await {
                        eprintln!("Impossibile scrivere log CPU: {}", e);
                    }
                },
                Err(e) => eprintln!("Impossibile aprire file log CPU: {}", e),
            }
        }
    });
}
