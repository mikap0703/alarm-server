use crate::alarm_handler::AlarmHandler;

mod config;
mod alarm_handler;
mod alarm;
mod mail_handler;
mod mail_parser;
mod apis;

#[tokio::main]
async fn main() {
    let configs = match config::parse_configs() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            panic!();
        }
    };

    // channel to send and receive alarms
    let (send_alarms, recv_alarms) = flume::unbounded();

    let alarm_handler = AlarmHandler::new(recv_alarms, configs.general.apis, configs.alarm_templates.clone());

    alarm_handler.start();

    for mail_source_config in configs.alarm_sources.mail_sources {
        let mail_handler = mail_handler::MailHandler::new(mail_source_config, send_alarms.clone(), false);
        mail_handler.start();
    }
}
