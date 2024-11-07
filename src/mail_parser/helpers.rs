use std::collections::HashMap;
use scraper::{Html, Selector};


pub fn parse_tables(body: &str) -> HashMap<String, Vec<String>> {
    let document = Html::parse_document(body);
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let mut result = HashMap::new();
    let mut current_key = String::new();

    for table in document.select(&table_selector) {
        for row in table.select(&row_selector) {
            let mut row_data = vec![];

            let mut cells = row.select(&cell_selector);

            // Process the first cell as a key
            if let Some(first_cell) = cells.next() {
                let key = first_cell.text().collect::<String>().trim().to_string();
                if !key.is_empty() {
                    current_key = key;
                }
            }

            // Process remaining cells as data
            for cell in cells {
                let cell_text = cell.text().collect::<String>().replace(&['\n', '\r', '\t'][..], "").trim().to_string();
                row_data.push(cell_text);
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