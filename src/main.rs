use crate::alarm_handler::AlarmHandler;
use fern::Dispatch;
use log::error;
use chrono::Local;
use colored::Colorize;

mod config;
mod alarm_handler;
mod alarm;
mod mail_handler;
mod mail_parser;
mod apis;

fn setup_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            // Apply colors based on the log level
            let level_color = match record.level() {
                log::Level::Error => "ERROR".red().bold(),
                log::Level::Warn => "WARN".yellow(),
                log::Level::Info => "INFO".green(),
                log::Level::Debug => "DEBUG".blue(),
                log::Level::Trace => "TRACE".purple(),
            };

            // Print the formatted log
            out.finish(format_args!(
                "{} {:<5} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string().cyan().bold(),
                level_color,
                message
            ))
        })
        .level(log::LevelFilter::Debug) // Minimum log level
        // Log to the console
        .chain(std::io::stdout())
        // Log to a file (without colors)
        .chain(fern::log_file("app.log")?)
        .apply()?;
    Ok(())
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

    // channel to send and receive alarms
    let (send_alarms, recv_alarms) = flume::unbounded();

    let alarm_handler = AlarmHandler::new(recv_alarms, configs.general.apis, configs.alarm_templates.clone());

    alarm_handler.start();

    for mail_source_config in configs.alarm_sources.mail_sources {
        let mail_handler = mail_handler::MailHandler::new(mail_source_config, send_alarms.clone(), true);
        mail_handler.start();
    }
}
