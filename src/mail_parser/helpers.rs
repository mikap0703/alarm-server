use quoted_printable::decode;
use std::collections::HashMap;
use quoted_printable::ParseMode::Strict;
use scraper::{Html, Selector};


pub fn parse_tables(body: &str) -> HashMap<String, Vec<String>> {
    let document = Html::parse_document(body);
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut result = HashMap::new();
    let mut current_key = String::new();

    // Select your table and row selectors (assumed to be defined)
    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let mut row_data = vec![];

            let mut cells = row.select(&cell_selector);

            // Process the first cell as a key (decoded with quoted-printable)
            if let Some(first_cell) = cells.next() {
                let key = first_cell.text().collect::<String>().trim().to_string();
                if !key.is_empty() {
                    let decoded_key = decode(&key, Strict).expect("Failed to decode key");
                    let key_str = String::from_utf8(decoded_key).expect("Invalid UTF-8 in key");
                    current_key = key_str;
                }
            }

            // Process remaining cells as data (decoded with quoted-printable)
            for cell in cells {
                let cell_text = cell.text().collect::<String>().replace(
                    &['\n', '\r', '\t'][..],
                    ""
                ).trim().to_string();
                let decoded_cell = decode(&cell_text, Strict).expect("Failed to decode cell");
                let cell_str = String::from_utf8(decoded_cell).expect("Invalid UTF-8 in cell");
                row_data.push(cell_str);
            }

            result.entry(current_key.clone()).or_insert_with(Vec::new).extend(row_data);
        }
    }

    result
}

pub fn get_table_key_order(body: &str) -> Vec<String> {
    let document = Html::parse_document(body);
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut result = vec![];

    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let mut cells = row.select(&cell_selector);

            // Process the first cell as a key
            if let Some(first_cell) = cells.next() {
                let key = first_cell.text().collect::<String>().trim().to_string();
                if !key.is_empty() {
                    result.push(key);
                }
            }
        }
    }

    result
}