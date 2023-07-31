use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use eframe::{egui, epi};
use egui::CtxRef;

#[derive(Debug, Default, Deserialize, Serialize)]
struct JsonData {
    // Define your JSON data structure here
    // For example:
    name: String,
    age: u32,
    // Add more fields as needed
}

struct MyApp {
    json_data: JsonData,
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

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "JSON Editor" // Provide a name for your application
    }
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame) {
        // UI code goes here
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.name());

            // Create UI elements to edit your JSON data
            // For example:
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.json_data.name);
            });

            ui.horizontal(|ui| {
                ui.label("Age:");
                ui.add(egui::widgets::Slider::new(&mut self.json_data.age, 0..=100).text("age"));
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

fn main() {
    let app = MyApp::default();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 200.0)),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
