use async_trait::async_trait;
use chrono::Local;
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::json;
use staticmap::{StaticMapBuilder};
use staticmap::tools::{Color, LineBuilder};
use crate::alarm::Alarm;
use crate::apis::Api;

pub struct Typst {
    pub name: String,
}

#[async_trait]
impl Api for Typst {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        let output_dir = PathBuf::from("config/typst/");
        let typst_bin = std::env::var("TYPST_BIN").unwrap_or_else(|_| "typst".to_string());
        let template_path =PathBuf::from("config/typst/template.typ");
        let alarm_clone = alarm.clone();

        let pdf_path = tokio::task::spawn_blocking(move || {
            render_alarm_pdf(output_dir, typst_bin, template_path, alarm_clone)
        })
            .await
            .map_err(|e| format!("Typst thread panicked: {}", e))??;

        info!("Typst API: Successfully created PDF at {}", pdf_path.display());
        Ok(())
    }

    async fn update_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        self.trigger_alarm(alarm).await
    }

    async fn check_connection(&self) -> Result<String, String> {
        let typst_bin = std::env::var("TYPST_BIN").unwrap_or_else(|_| "typst".to_string());
        let typst_bin_clone = typst_bin.clone();
        let output = tokio::task::spawn_blocking(move || {
            Command::new(&typst_bin_clone).arg("--version").output()
        })
        .await
        .map_err(|e| format!("Typst check thread panicked: {}", e))?
        .map_err(|e| format!("Failed to execute Typst binary: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Typst check failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok("typst OK".to_string())
        } else {
            Ok(stdout)
        }
    }
}

fn render_alarm_pdf(
    output_dir: PathBuf,
    typst_bin: String,
    template_path: PathBuf,
    alarm: Alarm,
) -> Result<PathBuf, String> {
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let ts = Local::now().format("%Y%m%d_%H%M%S%.3f").to_string();
    let origin = sanitize_file_component(&alarm.origin);
    let id = sanitize_file_component(&alarm.id);

    let base_name = if alarm.id.trim().is_empty() {
        format!("alarm_{}_{}", origin, ts)
    } else {
        format!("alarm_{}_{}", origin, id)
    };

    let json_path = output_dir.join(format!("{}.json", base_name));
    let pdf_path = output_dir.join(format!("{}.pdf", base_name));
    let map_path = output_dir.join(format!("{}_map.png", base_name));

    // 1. Generate the OSM Map Screenshot (100m x 100m)
    // Accessing lat/lon through alarm.address.coords as per your snippet
    let lat = alarm.address.coords.lat.ok_or("Alarm missing latitude")?;
    let lon = alarm.address.coords.lon.ok_or("Alarm missing longitude")?;

    generate_static_map(lat, lon, &map_path)?;

    // 2. Prepare JSON for Typst
    let json_value = json!(alarm);

    let json_data = serde_json::to_string_pretty(&json_value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(&json_path, json_data)
        .map_err(|e| format!("Failed to write JSON: {}", e))?;

    // 3. Compile Typst
    let output = Command::new(&typst_bin)
        .arg("compile")
        .arg("--input")
        .arg(format!("alarm_id={}", base_name))
        .arg(&template_path)
        .arg(&pdf_path)
        .output()
        .map_err(|e| format!("Failed to execute Typst binary: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Typst compilation failed: {}", stderr.trim()));
    }

    // Clean up temporary files
    let _ = fs::remove_file(&json_path);
    let _ = fs::remove_file(&map_path);

    Ok(pdf_path)
}

/// Generates a 100m x 100m map centered on the coordinates
fn generate_static_map(lat: f64, lon: f64, save_path: &Path) -> Result<(), String> {

    let mut map = StaticMapBuilder::default()
        .width(800)
        .height(400)
        .zoom(17)
        .url_template("https://tile.openstreetmap.de/{z}/{x}/{y}.png")
        .lon_center(lon)
        .lat_center(lat)
        .build()
        .map_err(|e| format!("Failed to initialize map builder: {}", e))?;

    // Define the style of the 'X'
    let x_color = Color::new(true, 255, 0, 0, 255); // Red color (RGBA)
    let offset = 0.00005; // Size of the X arms in degrees (adjust based on zoom)
    let stroke_width = 4.0;

    // Line 1: Top-Left to Bottom-Right
    let line1 = LineBuilder::default()
        .lat_coordinates(vec![lat - offset, lat + offset])
        .lon_coordinates(vec![lon - offset, lon + offset])
        .width(stroke_width)
        .color(x_color.clone())
        .build()
        .map_err(|e| format!("Failed to build X-line 1: {}", e))?;

    // Line 2: Top-Right to Bottom-Left
    let line2 = LineBuilder::default()
        .lat_coordinates(vec![lat - offset, lat + offset])
        .lon_coordinates(vec![lon + offset, lon - offset])
        .width(stroke_width)
        .color(x_color)
        .build()
        .map_err(|e| format!("Failed to build X-line 2: {}", e))?;

    // Add both lines to the map
    map.add_tool(line1);
    map.add_tool(line2);

    match map.save_png(save_path) {
        Ok(_) => info!("Map image saved to {}", save_path.display()),
        Err(e) => return Err(format!("Failed to save map image: {}", e)),
    }

    Ok(())
}

fn sanitize_file_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => out.push(ch),
            _ => out.push('_'),
        }
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.is_empty() {
        "alarm".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

// --- PUBLIC HELPER FUNCTIONS ---

pub fn default_output_dir() -> PathBuf {
    std::env::var_os("TYPST_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("config/typst/out"))
}

pub fn default_typst_bin() -> String {
    std::env::var("TYPST_BIN").unwrap_or_else(|_| "typst".to_string())
}

pub fn default_template_path() -> PathBuf {
    PathBuf::from("config/typst/template.typ")
}
