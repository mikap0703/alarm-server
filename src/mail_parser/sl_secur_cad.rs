use crate::alarm::{Alarm, Coordinates};
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::helpers::{get_table_key_order, parse_tables};
use crate::mail_parser::MailParser;
use log::{debug, info};

pub struct SecurCadParser;

impl MailParser for SecurCadParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm, config: MailConfig) -> Result<String, String> {
        let body = text_body.to_owned() + html_body;

        let table = parse_tables(&*body);
        for (key, value) in table.iter() {
            debug!("{}: {:?}", key, value);
        }

        let stichwoerter = config.stichwoerter.clone();

        // Einsatznummer - ID
        let id = match table.get("Auftragsnummer:") {
            Some(id) => id[0].clone(),
            None => "".to_string()
        };

        alarm.set_id(id);

        // Stichwort und Notfallgeschehen
        let mut stichwort = match table.get("Stichwort:") {
            Some(stichwort) => stichwort[0].clone(),
            None => "".to_string()
        };

        stichwort = match stichwoerter.get(&stichwort.to_uppercase()) {
            Some(stichwort) => stichwort.clone(),
            None => stichwort
        };

        let notfallgeschehen = match table.get("Notfallgeschehen:") {
            Some(notfallgeschehen) => notfallgeschehen[0].clone(),
            None => "".to_string()
        };

        if notfallgeschehen != "" {
            if !notfallgeschehen.is_empty() {
                let reg = regex::Regex::new(r"\((.*)\)").unwrap();
                if let Some(captures) = reg.captures(&notfallgeschehen) {
                    let title = match captures.get(1) {
                        Some(title) => title.as_str(),
                        None => &notfallgeschehen
                    };

                    alarm.set_title(title.to_string());
                } else {
                    alarm.set_title(notfallgeschehen);
                }
            } else if stichwort != "" {
                alarm.set_title(stichwort);
            }
        }

        // Objekt und Sachverhalt
        let objekt = match table.get("Objekt:") {
            Some(objekt) => objekt[0].clone(),
            None => "".to_string()
        };

        alarm.address.set_object(objekt.clone());

        let sachverhalt = match table.get("Sachverhalt:") {
            Some(sachverhalt) => sachverhalt[0].clone(),
            None => "".to_string()
        };

        if sachverhalt != "" {
            if objekt != "" {
                alarm.set_text(format!("{} - {}", sachverhalt, objekt));
            } else {
                alarm.set_text(sachverhalt.as_str().to_string());
            }
            alarm.set_text(sachverhalt.as_str().to_string());
        } else {
            alarm.set_text(objekt);
        }

        // Adresse
        // Straße
        let strasse = match table.get("Strasse:") {
            Some(strasse) => strasse[0].clone(),
            None => "".to_string()
        };

        if strasse != "" {
            alarm.address.set_street(strasse);
        }

        let strasse_nr = match table.get("Strasse / Hs.-Nr.:") {
            Some(strasse_nr) => strasse_nr[0].clone(),
            None => "".to_string()
        };

        // overwrite strasse without number if strasse_nr is not empty
        if strasse_nr != "" {
            alarm.address.set_street(strasse_nr);
        }

        // Ort
        let ort = match table.get("PLZ / Ort:") {
            Some(ort) => ort[0].clone(),
            None => "".to_string()
        };

        if ort != "" {
            alarm.address.set_city(ort);
        }

        // ort_info
        let ort_info = match table.get("Info:") {
            Some(ort_info) => ort_info[0].clone(),
            None => "".to_string()
        };

        if ort_info != "" {
            alarm.address.set_info(ort_info);
        }

        // Koordinaten
        // UTM
        let utm = match table.get("UTM:") {
            Some(utm) => utm[0].clone(),
            None => "".to_string()
        };

        if utm != "" {
            alarm.address.set_utm(utm);
        }

        // Lat Lon
        let lat_lon = match table.get("Geopositionen:") {
            Some(lat_lon) => lat_lon.clone(),
            None => vec![]
        };

        let lat_lon_reg = regex::Regex::new(r"/\d+,\d+/").unwrap();
        // parsing latitude - geogr. Breite
        let lat = match lat_lon.get(1) {
            Some(lat_str) => match lat_lon_reg.captures(lat_str) {
                Some(lat) => {
                    if let Some(lat_val) = lat.get(1) {
                        let lat_temp = lat_val.as_str().replace(",", "."); // replace ',' with '.'
                        lat_temp.parse::<f64>().ok() // parse as f64
                    } else {
                        None
                    }
                },
                None => None,
            },
            None => None,
        };

        // parsing longitude - geogr. Länge
        let lon = match lat_lon.get(0) {
            Some(lon_str) => match lat_lon_reg.captures(lon_str) {
                Some(lon) => {
                    if let Some(lon_val) = lon.get(1) {
                        let lon_temp = lon_val.as_str().replace(",", "."); // replace , with .
                        lon_temp.parse::<f64>().ok()
                    } else {
                        None
                    }
                },
                None => None,
            },
            None => None,
        };


        alarm.address.set_coords(Coordinates { lat, lon });

        // Apple Maps Link
        if let Some(lat) = lat {
            if let Some(lon) = lon {
                alarm.add_to_text(format!("\n \n https://maps.apple.com/?q={},{} \n \n", lat, lon));
            }
        }

        // Units
        let key_order = get_table_key_order(&*body);
        let unit_start_index = if let Some(index) = key_order.iter().position(|x| x == "Ressourcen") {
            index + 1
        } else {
            0
        };

        let unit_end_index = if let Some(index) = key_order.iter().position(|x| x == "Meldender des Hilfeersuchens") {
            index
        } else {
            0
        };

        if unit_end_index > unit_start_index {
            // vec of unit keys
            let unit_keys = &key_order[unit_start_index..unit_end_index];

            for key in unit_keys {
                if table.contains_key(key) {
                    // only add unit if it's not in the ignore_units list
                    if config.ignore_units.contains(&key.to_string()) {
                        continue;
                    } else {
                        alarm.add_unit(key.to_string());
                    }
                }
            }

            // save template names to apply them later
            for unit in alarm.units.clone().iter() {
                // look if a template name is given for a unit
                if let Some(template_name) = config.alarm_template_keywords.get(unit) {
                    alarm.add_template_name(template_name.clone());
                }
            }
        }

        info!("{:?}", alarm);

        //
        Ok(format!("Parsed SecurCad: {}", text_body))
    }
}
