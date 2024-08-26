use crate::alarm_handler::AlarmHandler;

mod config;
mod alarm_handler;
mod alarm;

fn main() {
    let configs = match config::parse_configs() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            panic!();
        }
    };

    // channel to send and receive alarms
    let (send_alarms, recv_alarms) = flume::unbounded();

    let alarm_handler = AlarmHandler::new(recv_alarms);
    alarm_handler.start()

    println!("Hello, world!");
}
