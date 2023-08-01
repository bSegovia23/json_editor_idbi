use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use eframe::egui;
use egui_extras;
use chrono::NaiveDate;

const APP_NAME: &str = "JSON Editor";

struct MyApp {
    json_data: JsonData,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If you want to customize GUI fonts and visuals, do it here.
        // cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals
        Self::default()
    }
}

impl Default for MyApp {
    fn default() -> Self {
        // Load the initial JSON data from a file
        let json_data = match load_json_data() {
            Ok(data) => data,
            Err(e) => {
                println!("Error loading JSON data: {}", e);
                JsonData::default()
            }
        };

        MyApp { json_data }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // UI code goes here
        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading();

            // Create UI elements to edit your JSON data
            ui.horizontal(|ui| {
                ui.label("Number of Stages:");
                ui.add(egui::widgets::Slider::new(&mut self.json_data.n_stages, 1..=4));
            });

            ui.horizontal(|ui| {
                ui.label("Step Size in Months:");
                ui.add(egui::widgets::Slider::new(&mut self.json_data.step_size_months, 1..=6));
            });

            ui.horizontal(|ui| {
                ui.label("Base Date:");
                ui.add(egui_extras::DatePickerButton::new(&mut self.json_data.base_date));
            });
            
            let environment_options = [
                Environment::Production,
                Environment::Development,
                Environment::Testing,
            ];

            ui.horizontal(|ui| {
                ui.label("Environment:");
                for option in environment_options {
                    ui.radio_value(&mut self.json_data.environment, option, option.to_user_friendly_label());
                }
            });

            // Add more UI elements as needed for other fields

            if ui.button("Save").clicked() {
                // Save the JSON data when the "Save" button is clicked
                match save_json_data(&self.json_data) {
                    Ok(_) => println!("Data saved successfully!"),
                    Err(e) => println!("Error saving data: {}", e),
                }
            }
        });
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonData {
    // Define your JSON data structure here
    n_stages: u32,
    step_size_months: u32,
    base_date: NaiveDate,
    start_date: NaiveDate,
    environment: Environment, // PRODUCTION, ENVIRONMENT, TESTING
    // assumption_profile: String, // BASE CASE, SCENARIO 1, 2, 3
    // optimizer: String, // "highs" ASK JC
    // open_field: String, // Include vs Exclude
    // lcr_lower_limit: f32, lcr_upper_limit: f32,
    // Add more fields as needed
}

impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            n_stages: 2,
            step_size_months: 4,
            base_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            environment: Environment::Production
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Environment {
    Production,
    Development,
    Testing,
}

impl Environment {
    fn to_user_friendly_label(&self) -> &str {
        match self {
            Environment::Production => "Production",
            Environment::Development => "Development",
            Environment::Testing => "Testing",
        }
    }
}

fn load_json_data() -> Result<JsonData, Box<dyn std::error::Error>> {
    // Read the JSON data from a file (change "data.json" to your file name)
    let mut file = File::open("data.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the JSON data using serde
    let json_data: JsonData = serde_json::from_str(&contents)?;

    Ok(json_data)
}

fn save_json_data(data: &JsonData) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize the JSON data using serde
    let serialized_data = serde_json::to_string_pretty(data)?;

    // Write the data to a file (change "data.json" to your file name)
    let mut file = File::create("data.json")?;
    file.write_all(serialized_data.as_bytes())?;

    Ok(())
}

fn main() {
    let app = MyApp::default();
    // let options = eframe::NativeOptions {
    //     initial_window_size: Some(egui::vec2(400.0, 200.0)),
    //     ..Default::default()
    // };
    let options = eframe::NativeOptions::default();
    eframe::run_native(APP_NAME, options, Box::new(|cc| Box::new(MyApp::new(cc))));
}
