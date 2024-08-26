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
