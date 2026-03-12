use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use crate::alarm_handler::AlarmHandler;
use log::{error, info, warn};
use colored::Colorize;

mod config;
mod alarm_handler;
mod alarm;
mod mail_handler;
mod mail_parser;
mod apis;
mod serial_handler;

struct RotatingFileWriter {
    path: PathBuf,
    max_bytes: u64,
    max_files: usize,
    current_size: u64,
    file: File,
}

impl RotatingFileWriter {
    fn new(path: impl Into<PathBuf>, max_bytes: u64, max_files: usize) -> io::Result<Self> {
        let path = path.into();

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let current_size = file.metadata()?.len();

        Ok(Self {
            path,
            max_bytes,
            max_files: max_files.max(1),
            current_size,
            file,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file.flush()?;

        let oldest = rotated_path(&self.path, self.max_files);
        if oldest.exists() {
            fs::remove_file(&oldest)?;
        }

        for idx in (1..self.max_files).rev() {
            let src = rotated_path(&self.path, idx);
            let dst = rotated_path(&self.path, idx + 1);
            if src.exists() {
                fs::rename(src, dst)?;
            }
        }

        if self.path.exists() {
            fs::rename(&self.path, rotated_path(&self.path, 1))?;
        }

        self.file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        self.current_size = 0;

        Ok(())
    }
}

impl Write for RotatingFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.current_size + buf.len() as u64 > self.max_bytes {
            self.rotate()?;
        }

        let written = self.file.write(buf)?;
        self.current_size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn rotated_path(base: &Path, idx: usize) -> PathBuf {
    let file_name = base
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "app.log".to_string());
    base.with_file_name(format!("{file_name}.{idx}"))
}

fn setup_logger() -> Result<(), fern::InitError> {
    let debug = false;
    let default_level = if debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    let max_bytes = std::env::var("LOG_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(10 * 1024 * 1024); // 10MB per file
    let max_files = std::env::var("LOG_MAX_FILES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(5);
    let log_file_path = std::env::var("LOG_FILE_PATH").unwrap_or_else(|_| "config/app.log".to_string());
    let log_to_file = std::env::var("LOG_TO_FILE")
        .map(|v| !matches!(v.as_str(), "0" | "false" | "FALSE" | "False"))
        .unwrap_or(true);

    let stdout_dispatch = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level_color = match record.level() {
                log::Level::Error => "ERROR".red().bold(),
                log::Level::Warn  => "WARN".yellow(),
                log::Level::Info  => "INFO".green(),
                log::Level::Debug => "DEBUG".blue(),
                log::Level::Trace => "TRACE".purple(),
            };

            out.finish(format_args!(
                "{} {:<5} {}",
                chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .cyan()
                    .bold(),
                level_color,
                message
            ));
        })
        .chain(std::io::stdout());

    let mut base = fern::Dispatch::new()
        .level(default_level)
        // Suppress noisy parser warnings like "foster parenting not implemented".
        .level_for("html5ever", log::LevelFilter::Error)
        .chain(stdout_dispatch);

    if log_to_file {
        match RotatingFileWriter::new(log_file_path, max_bytes, max_files) {
            Ok(writer) => {
                let file_dispatch = fern::Dispatch::new()
                    .format(move |out, message, record| {
                        out.finish(format_args!(
                            "{} {:<5} {}",
                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                            record.level(),
                            message
                        ));
                    })
                    .chain(Box::new(writer) as Box<dyn Write + Send>);
                base = base.chain(file_dispatch);
            }
            Err(err) => {
                eprintln!("Failed to initialize file logging, continuing with stdout only: {}", err);
            }
        }
    }

    base.apply()?;

    Ok(())
}

fn api_type_label(api_type: &config::general::ApiType) -> &'static str {
    match api_type {
        config::general::ApiType::Divera => "Divera",
        config::general::ApiType::Alamos => "Alamos",
        config::general::ApiType::Telegram => "Telegram",
        config::general::ApiType::Typst => "Typst",
    }
}

fn log_startup_config(configs: &config::Configs) {
    if configs.general.alarm {
        info!("Alarm werden weitergeleitet");
    } else {
        warn!("Alarm werden NICHT weitergeleitet");
    }

    // --- Mail Sources ---
    let active_mail_sources: Vec<_> = configs
        .alarm_sources
        .mail_sources
        .iter()
        .filter(|source| source.active)
        .collect();

    if active_mail_sources.is_empty() {
        info!("Active mail sources: none");
    } else {
        // 1. Print the summary first
        let names: Vec<String> = active_mail_sources.iter().map(|s| s.name.clone()).collect();
        info!("Aktive Mail Quellen: {}", names.join(", "));

        // 2. Print individual details
        for source in active_mail_sources {
            info!(
            "Mail Source {} wartet auf Mails von {} mit dem Betreff {}.",
            source.name, source.alarm_sender, source.alarm_subject
        );
        }
    }

    // --- Serial Sources ---
    let active_serial_sources: Vec<_> = configs
        .alarm_sources
        .serial_sources
        .iter()
        .filter(|source| source.active)
        .collect();

    if active_serial_sources.is_empty() {
        info!("Active serial sources: none");
    } else {
        // 1. Print the summary first
        let names: Vec<String> = active_serial_sources.iter().map(|s| s.name.clone()).collect();
        info!("Active serial sources: {}", names.join(", "));

        // 2. Print individual details
        for source in active_serial_sources {
            info!(
            "Serial Source {} wartet auf Daten von {} (baudrate {}).",
            source.name, source.port, source.baudrate
        );
        }
    }

    if configs.general.apis.is_empty() {
        info!("APIs configured: none");
    } else {
        for api in &configs.general.apis {
            info!("API configured: '{}' ({})", api.name, api_type_label(&api.api));
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = setup_logger() {
        eprintln!("Error setting up logger: {}", e);
        return;
    }

    let configs = match config::parse_configs() {
        Ok(config) => config,
        Err(e) => {
            error!("Error loading config: {}", e);
            panic!();
        }
    };

    log_startup_config(&configs);
    if configs.general.delay > 0 {
        info!(
            "Warte {}s nach Konfig-Ausgabe vor dem Start der Handler...",
            configs.general.delay
        );
        tokio::time::sleep(Duration::from_secs(configs.general.delay)).await;
    }

    // channel to send and receive alarms
    let (send_alarms, recv_alarms) = flume::unbounded();

    let alarm_handler = AlarmHandler::new(recv_alarms, configs.general, configs.alarm_templates.clone());

    alarm_handler.check_api_connections().await;
    alarm_handler.start();

    // starting handlers for mail sources
    for mail_source_config in configs.alarm_sources.mail_sources {
        if !mail_source_config.active {
            info!("Mail source '{}' is deactivated - skipping...", mail_source_config.name);
            continue;
        }
        let send_alarms = send_alarms.clone();
        thread::spawn(move || {
            let mail_handler = mail_handler::MailHandler::new(mail_source_config, send_alarms, false);
            mail_handler.start();
        });
    }

    // starting handlers for serial sources
    for serial_source_config in configs.alarm_sources.serial_sources {
        if !serial_source_config.active {
            info!("Serial source '{}' is deactivated - skipping...", serial_source_config.name);
            continue;
        }
        let send_alarms = send_alarms.clone();
        thread::spawn(move || {
            let serial_handler = serial_handler::SerialHandler::new(serial_source_config, send_alarms, true);
            serial_handler.start();
        });
    }

    loop {
        // Keep the main thread alive
        thread::park();
    }
}
