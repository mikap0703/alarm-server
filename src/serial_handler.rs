use flume::Sender;
use std::time::Duration;
use std::io::Read;
use std::io::BufRead;
use std::io;
use crate::alarm::{Alarm, DmeData};
use crate::config::alarm_sources::SerialConfig;
use log::{debug, error, info, warn};
use serialport::SerialPort;

pub struct SerialHandler {
    config: SerialConfig,
    send_alarms: Sender<Alarm>,
    debug: bool,
}

impl SerialHandler {
    pub fn new(config: SerialConfig, send_alarms: Sender<Alarm>, debug: bool) -> Self {
        Self {
            config,
            send_alarms,
            debug,
        }
    }

    pub fn start(&self) {
        let port_name = &self.config.port;
        let baud_rate = self.config.baudrate;
        let delimiter = self.config.delimiter.clone();
        let send_alarms = self.send_alarms.clone();
        let debug = self.debug;

        let mut port = match serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(10))
            .open() {
            Ok(port) => port,
            Err(e) => {
                error!("Failed to open serial port: {:?}", e);
                return;
            }
        };

        info!("Serial port opened: {}, Baudrate: {}", port_name, baud_rate);

        let mut serial_buf: Vec<u8> = vec![0; 1024]; // Initialize buffer with a size

        loop {
            match port.read(serial_buf.as_mut_slice()) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        debug!("Buffer: {:02X?}", &serial_buf[..bytes_read]);
                        // Handle the data
                        let data = String::from_utf8_lossy(&serial_buf[..bytes_read]).to_string();
                        self.handle_dme_data(data);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    error!("Error reading from serial port: {:?}", e);
                    break;
                }
            }
        }
    }

    fn handle_dme_data(&self, data: String) {
        debug!("Received data: {}", data);

        let mut alarm = Alarm::new();

        // split the data after each line break
        let lines: Vec<&str> = data.split("\n").collect();

        if lines.len() < 3 {
            warn!("Received data is too short");
            return;
        }

        alarm.set_dme_data(DmeData {
            date: lines[lines.len() -3].to_string(),
            ric: lines[lines.len() -2].to_string(),
            content: lines[lines.len() -1].to_string(),
        });

        let text = lines[lines.len() -1].to_string();

        alarm.set_text(text);

        for stichwort in self.config.alarm_list.clone() {
            if lines[lines.len() -1].contains(stichwort.as_str()) {
                alarm.title = stichwort.to_string();
            }
        }

        // Populate alarm with data
        if let Err(e) = self.send_alarms.send(alarm) {
            error!("Failed to send alarm: {:?}", e);
        }
    }
}