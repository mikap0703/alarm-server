use std::thread;
use crate::alarm::Alarm;

pub struct AlarmHandler {
    // channel to send and receive alarms
    recv_alarms: flume::Receiver<Alarm>,
}

impl AlarmHandler {
    pub fn new(recv_alarms: flume::Receiver<Alarm>) -> Self {
        Self {
            recv_alarms,
        }
    }

    pub fn start(&self) {
        thread::spawn(move || {
            loop {
                match self.recv_alarms.recv() {
                    Ok(alarm) => {
                        println!("AlarmHandler received alarm: {}", alarm.title);
                    }
                    Err(e) => {
                        eprintln!("Error receiving alarm: {}", e);
                        break;
                    }
                }
            }
        });
    }
}