use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use eframe::egui;
use egui_extras;
use chrono::NaiveDate;

const APP_NAME: &str = "JSON Editor";

// Here we define what our app keeps in its storage.
struct MyApp {
    json_data: JsonData,
}

// Here we define what a "new" app looks like.
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If you want to customize GUI fonts and visuals, do it here.
        // cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals
        Self::default()
    }
}

// Here we define the "default" app.
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

// Here we define our app's UI.
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Create UI elements to edit our JSON data
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
                ui.add(egui_extras::DatePickerButton::new(&mut self.json_data.base_date).id_source("base_date_picker"));
            });

            ui.horizontal(|ui| {
                ui.label("Start Date:");
                ui.add(egui_extras::DatePickerButton::new(&mut self.json_data.start_date).id_source("start_date_picker"));
            });

            ui.horizontal(|ui| {
                ui.label("Reports Folder:");
                ui.add(egui::TextEdit::singleline(&mut self.json_data.reports_folder));
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

            let assumption_profile_options = [
                AssumptionProfile::BaseCase,
                AssumptionProfile::Scenario1,
                AssumptionProfile::Scenario2,
                AssumptionProfile::Scenario3,
            ];

            ui.horizontal(|ui| {
                ui.label("Assumption Profile:");
                for option in assumption_profile_options {
                    ui.radio_value(&mut self.json_data.assumption_profile, option, option.to_user_friendly_label());
                }
            });

            let optimizer_options = [
                Optimizer::Highs,
                Optimizer::Cbc,
                Optimizer::Gurobi,
            ];

            ui.horizontal(|ui| {
                ui.label("Optimizer:");
                for option in optimizer_options {
                    ui.radio_value(&mut self.json_data.optimizer, option, option.to_user_friendly_label());
                }
            });

            let open_field_options = [
                IncludedOrExcluded::Included,
                IncludedOrExcluded::Excluded,
            ];

            ui.horizontal(|ui| {
                ui.label("Forward Start Swap:");
                for option in open_field_options {
                    ui.radio_value(&mut self.json_data.fwd_start_swap, option, option.to_user_friendly_label());
                }
            });

            ui.horizontal(|ui| {
                ui.label("LCR Lower Limit:");
                let response = ui.add(egui::Slider::new(&mut self.json_data.lcr_lower_limit, 0.0..=100.0).drag_value_speed(0.01));
                if response.changed() && self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                    self.json_data.lcr_upper_limit = self.json_data.lcr_lower_limit;
                }
            });

            ui.horizontal(|ui| {
                ui.label("LCR Upper Limit:");
                let response = ui.add(egui::Slider::new(&mut self.json_data.lcr_upper_limit, 0.0..=100.0).drag_value_speed(0.01));
                if response.changed() && self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                    self.json_data.lcr_lower_limit = self.json_data.lcr_upper_limit;
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

// Here we define our JSON's data structure.
#[derive(Debug, Deserialize, Serialize)]
struct JsonData {
    n_stages: u32,
    step_size_months: u32,
    base_date: NaiveDate,
    start_date: NaiveDate,
    reports_folder: String,
    environment: Environment,
    assumption_profile: AssumptionProfile,
    optimizer: Optimizer,
    fwd_start_swap: IncludedOrExcluded,
    lcr_lower_limit: f32, lcr_upper_limit: f32,

    // Add more fields as needed
}

// Here we define the "default" JSON data.
impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            n_stages: 2,
            step_size_months: 4,
            reports_folder: "Reports".to_string(),
            base_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            environment: Environment::Production,
            assumption_profile: AssumptionProfile::BaseCase,
            optimizer: Optimizer::Highs,
            fwd_start_swap: IncludedOrExcluded::Included,
            lcr_lower_limit: 0.0, lcr_upper_limit: 100.0,
        }
    }
}

// Here we define the possible environments.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum Environment {
    #[serde(rename = "PRODUCTION")] // what should be written in the JSON
    Production,
    #[serde(rename = "DEVELOPMENT")]
    Development,
    #[serde(rename = "TESTING")]
    Testing,
}

impl Environment {
    // Here we define labels for the UI.
    fn to_user_friendly_label(&self) -> &str {
        match self {
            Environment::Production => "Production",
            Environment::Development => "Development",
            Environment::Testing => "Testing",
        }
    }
}

// Here we define the possible assumption profiles.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum AssumptionProfile {
    #[serde(rename = "BASE CASE")]
    BaseCase,
    #[serde(rename = "SCENARIO 1")]
    Scenario1,
    #[serde(rename = "SCENARIO 2")]
    Scenario2,
    #[serde(rename = "SCENARIO 3")]
    Scenario3,
}

impl AssumptionProfile {
    fn to_user_friendly_label(&self) -> &str {
        match self {
            AssumptionProfile::BaseCase => "Base Case",
            AssumptionProfile::Scenario1 => "Scenario 1",
            AssumptionProfile::Scenario2 => "Scenario 2",
            AssumptionProfile::Scenario3 => "Scenario 3",
        }
    }
}

// Here we define the possible optimizers.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum Optimizer {
    #[serde(rename = "highs")]
    Highs,
    #[serde(rename = "cbc")]
    Cbc,
    #[serde(rename = "gurobi")]
    Gurobi,
}

impl Optimizer {
    fn to_user_friendly_label(&self) -> &str {
        match self {
            Optimizer::Highs => "Highs",
            Optimizer::Cbc => "CBC",
            Optimizer::Gurobi => "Gurobi",
        }
    }
}

// Here we define the possible open fields.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum IncludedOrExcluded {
    #[serde(rename = "included")]
    Included,
    #[serde(rename = "excluded")]
    Excluded,
}

impl IncludedOrExcluded {
    fn to_user_friendly_label(&self) -> &str {
        match self {
            IncludedOrExcluded::Included => "Included",
            IncludedOrExcluded::Excluded => "Excluded",
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
    let options = eframe::NativeOptions::default();
    let _result = eframe::run_native(APP_NAME, options, Box::new(|cc| Box::new(MyApp::new(cc))));
}
