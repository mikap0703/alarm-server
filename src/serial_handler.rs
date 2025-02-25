use flume::Sender;
use std::time::Duration;
use std::io::Read;
use std::io::BufRead;
use std::io;
use encoding_rs::ISO_8859_2;
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
        let delimiter = self.config.delimiter.clone().replace("\\r", "\r").replace("\\n", "\n").replace("\\0", "\0");

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

        let mut temp_buffer: Vec<u8> = vec![0; 1024];
        let mut buffer: Vec<u8> = Vec::new();
        let end_sequence: &[u8] = delimiter.as_bytes();

        loop {
            match port.read(temp_buffer.as_mut_slice()) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        debug!("Buffer: {:02X?}", &temp_buffer[..bytes_read]);
                        buffer.extend_from_slice(&temp_buffer[..bytes_read]);

                        // Check if the buffer contains the delimiter
                        if buffer.ends_with(end_sequence) {
                            let (decoded_buffer, _, _) = ISO_8859_2.decode(&buffer);
                            debug!("Received: {}", decoded_buffer);

                            // Handle the data
                            let (final_decoded, _, _) = ISO_8859_2.decode(&buffer);
                            self.handle_dme_data(final_decoded.to_string());

                            buffer.clear();
                        }
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
        let lines: Vec<&str> = data.trim().lines().collect();

        if lines.len() < 3 {
            warn!("Received data is too short");
            return;
        }

        for i in 0..lines.len() {
            debug!("Line {}: {}", i, lines[i]);
        }

        alarm.set_dme_data(DmeData {
            date: lines[0].to_string(),
            ric: lines[1].to_string(),
            content: lines[2].to_string(),
        });

        let text = lines[2].to_string();

        alarm.set_text(text);

        print!("Alarm: {:?}", alarm);

        for stichwort in self.config.alarm_list.clone() {
            if lines[2].contains(stichwort.as_str()) {
                alarm.title = stichwort.to_string();
            }
        }

        // Populate alarm with data
        if let Err(e) = self.send_alarms.send(alarm) {
            error!("Failed to send alarm: {:?}", e);
        }
    }
}