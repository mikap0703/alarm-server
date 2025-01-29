use std::collections::HashMap;
use std::time::{Instant};
use crate::config::alarm_templates::AlarmTemplateReceiver;

#[derive(Debug, Clone)]
pub struct Address {
    street: String,
    city: String,
    object: String,
    object_id: String,
    info: String,
    utm: String,
    coords: Coordinates,
}

#[derive(Debug, Clone)]
pub struct Coordinates {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AlarmReceiver {
    pub groups: Vec<String>,
    pub vehicles: Vec<String>,
    pub members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MailData {
    id: String,
    sender: String,
    subject: String,
    content: String,
    date: u64,
}

#[derive(Debug, Clone)]
pub struct DmeData {
    pub(crate) date: String,
    pub(crate) ric: String,
    pub(crate) content: String,
}

#[derive(Debug, Clone)]
pub struct Alarm {
    pub id: String,
    pub origin: String,
    pub title: String,
    pub text: String,
    pub time: Instant,
    pub address: Address,
    pub units: Vec<String>,
    pub receiver: HashMap<String, AlarmReceiver>,
    pub template_names: Vec<String>,
    pub groups: Vec<String>,
    pub vehicles: Vec<String>,
    pub members: Vec<String>,
    pub webhooks: Vec<String>,
    pub alarm_sources: Vec<String>,
    pub mail_data: MailData,
    pub dme_data: DmeData,
}

impl Address {
    pub fn new() -> Self {
        Address {
            street: "".to_string(),
            city: "".to_string(),
            object: "".to_string(),
            object_id: "".to_string(),
            info: "".to_string(),
            utm: "".to_string(),
            coords: Coordinates {
                lat: None,
                lon: None,
            },
        }
    }

    pub fn set_street(&mut self, street: String) {
        self.street = street;
    }

    pub fn set_city(&mut self, city: String) {
        self.city = city;
    }

    pub fn set_object(&mut self, object: String) {
        self.object = object;
    }

    pub fn set_object_id(&mut self, object_id: String) {
        self.object_id = object_id;
    }

    pub fn set_info(&mut self, info: String) {
        self.info = info;
    }

    pub fn set_utm(&mut self, utm: String) {
        self.utm = utm;
    }

    pub fn set_coords(&mut self, coords: Coordinates) {
        self.coords = coords;
    }
}

impl Alarm {
    pub fn new() -> Self {
        Alarm {
            id: "".to_string(),
            origin: "".to_string(),
            title: "".to_string(),
            text: "".to_string(),
            time: Instant::now(),
            address: Address::new(),  // Use Address::new() here
            units: vec![],
            receiver: HashMap::new(),
            template_names: vec![],
            groups: vec![],
            vehicles: vec![],
            members: vec![],
            webhooks: vec![],
            alarm_sources: vec![],
            mail_data: MailData {
                id: "".to_string(),
                sender: "".to_string(),
                subject: "".to_string(),
                content: "".to_string(),
                date: 0,
            },
            dme_data: DmeData {
                date: "".to_string(),
                ric: "".to_string(),
                content: "".to_string(),
            },
        }
    }

    // Setter functions
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_origin(&mut self, origin: String) {
        self.origin = origin;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn add_to_text(&mut self, text: String) {
        self.text.push_str(&text);
    }

    pub fn set_time(&mut self, time: Instant) {
        self.time = time;
    }

    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }

    pub fn set_units(&mut self, units: Vec<String>) {
        self.units = units;
    }

    pub fn add_unit(&mut self, unit: String) {
        self.units.push(unit);
    }

    pub fn set_groups(&mut self, groups: Vec<String>) {
        self.groups = groups;
    }

    pub fn set_vehicles(&mut self, vehicles: Vec<String>) {
        self.vehicles = vehicles;
    }

    pub fn set_members(&mut self, members: Vec<String>) {
        self.members = members;
    }

    pub fn set_webhooks(&mut self, webhooks: Vec<String>) {
        self.webhooks = webhooks;
    }

    pub fn alarm_source(&mut self, alarm_source: String) {
        self.alarm_sources.push(alarm_source);
    }

    pub fn set_mail_data(&mut self, mail_data: MailData) {
        self.mail_data = mail_data;
    }

    pub fn set_dme_data(&mut self, dme_data: DmeData) {
        self.dme_data = dme_data;
    }

    pub fn add_template_name(&mut self, name: String) {
        self.template_names.push(name);
    }

    pub fn apply_template(&mut self, target: String, template_receiver: AlarmTemplateReceiver) {
        match template_receiver {
            AlarmTemplateReceiver::Api { members, groups, vehicles } => {
                let receiver = self.receiver.entry(target).or_insert(AlarmReceiver {
                    groups: vec![],
                    vehicles: vec![],
                    members: vec![],
                });

                if let Some(members) = members {
                    receiver.members = members;
                }

                if let Some(groups) = groups {
                    receiver.groups = groups;
                }

                if let Some(vehicles) = vehicles {
                    receiver.vehicles = vehicles;
                }
            }
            AlarmTemplateReceiver::Webhooks( webhooks ) => {
                // add webhooks to alarm
                self.webhooks.extend(webhooks);
            }
        }
    }
}
