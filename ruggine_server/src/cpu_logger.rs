
// ============================================================================
// cpu_logger.rs - Task asincrono per logging CPU del server
// Fornisce una funzione per avviare il task periodico che registra l'utilizzo
// della CPU del processo backend in un file di log.
// ============================================================================

// Import di terze parti
use sysinfo::{CpuRefreshKind, RefreshKind, System, Pid, ProcessRefreshKind, ProcessesToUpdate};
use chrono::Local;
use tokio::time::{interval, Duration};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

// Avvia un task asincrono che logga l'utilizzo della CPU del processo server ogni `period_secs` secondi.
// Il log viene scritto in append su "server_cpu_stats.log".
pub fn spawn_cpu_logger(period_secs: u64) {
    tokio::spawn(async move {
        // Configura sysinfo per monitorare solo CPU e processi
        let mut sys = System::new_with_specifics(
            RefreshKind::default()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        // Ottieni il PID del processo corrente (server)
        let pid = Pid::from_u32(std::process::id());
        // Timer periodico
        let mut ticker = interval(Duration::from_secs(period_secs));
        loop {
            ticker.tick().await; // Attende il prossimo tick
            // Aggiorna solo la CPU e i processi
            sys.refresh_specifics(RefreshKind::default().with_cpu(CpuRefreshKind::everything()));
            sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());

            // Calcola utilizzo CPU del processo server
            let process_cpu = sys.process(pid)
                .map(|proc| proc.cpu_usage())
                .unwrap_or(0.0);

            // Prepara la stringa di log con timestamp
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] Server CPU Usage: {:.5}%\n", timestamp, process_cpu);

            // Scrive la riga nel file di log (modalità append)
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
