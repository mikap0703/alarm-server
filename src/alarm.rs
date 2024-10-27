use std::time::{Instant};

#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    object: String,
    object_id: String,
    info: String,
    utm: String,
    coords: Coordinates,
}

#[derive(Debug)]
struct Coordinates {
    lat: Option<f64>,
    lon: Option<f64>,
}

#[derive(Debug)]
struct MailData {
    id: String,
    sender: String,
    subject: String,
    content: String,
    date: u64,
}

#[derive(Debug)]
struct DmeData {
    content: String,
}

#[derive(Debug)]
pub struct Alarm {
    pub id: String,
    pub origin: String,
    pub title: String,
    pub text: String,
    pub time: Instant,
    pub address: Address,
    pub units: Vec<String>,
    pub groups: Vec<String>,
    pub vehicles: Vec<String>,
    pub members: Vec<String>,
    pub webhooks: Vec<String>,
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
            groups: vec![],
            vehicles: vec![],
            members: vec![],
            webhooks: vec![],
            mail_data: MailData {
                id: "".to_string(),
                sender: "".to_string(),
                subject: "".to_string(),
                content: "".to_string(),
                date: 0,
            },
            dme_data: DmeData {
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

    pub fn set_time(&mut self, time: Instant) {
        self.time = time;
    }

    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }

    pub fn set_units(&mut self, units: Vec<String>) {
        self.units = units;
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

    pub fn set_mail_data(&mut self, mail_data: MailData) {
        self.mail_data = mail_data;
    }

    pub fn set_dme_data(&mut self, dme_data: DmeData) {
        self.dme_data = dme_data;
    }
}
