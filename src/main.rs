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
            egui::ScrollArea::vertical().max_width(f32::INFINITY).show(ui, |ui| {
                // Create UI elements to edit our JSON data
                ui.heading("Model");
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

                ui.heading("Liquidity Risk");

                const LCR_BOUND_MIN: u32 = 100;
                const LCR_BOUND_MAX: u32 = 300;

                ui.horizontal(|ui| {
                    ui.label("LCR Lower Limit:");
                    let response = ui.add(egui::DragValue::new(&mut self.json_data.lcr_lower_limit).speed(0.5).clamp_range(LCR_BOUND_MIN..=LCR_BOUND_MAX));
                    if response.changed() {
                        if self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                            self.json_data.lcr_upper_limit = self.json_data.lcr_lower_limit;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("LCR Upper Limit:");
                    let response = ui.add(egui::DragValue::new(&mut self.json_data.lcr_upper_limit).speed(0.5).clamp_range(LCR_BOUND_MIN..=LCR_BOUND_MAX));
                    if response.changed() {
                        if self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                            self.json_data.lcr_lower_limit = self.json_data.lcr_upper_limit;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("USD Treasury Liquidity as % of Total Assets:");
                    ui.add(egui::DragValue::new(&mut self.json_data.lcr_average_dra_pd).speed(0.01).clamp_range(0.0..=50.0));
                });

                const LIQUIDITY_FLOOR_MIN: u32 = 1000000; // at least set this as 0
                const LIQUIDITY_FLOOR_MAX: u32 = 100000000;

                ui.horizontal(|ui| {
                    ui.label("MXN Treasury Liquidity Floor (USD):");
                    ui.add(egui::DragValue::new(&mut self.json_data.MXN_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("COP Treasury Liquidity Floor (USD):");
                    ui.add(egui::DragValue::new(&mut self.json_data.COP_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("BRL Treasury Liquidity Floor (USD):");
                    ui.add(egui::DragValue::new(&mut self.json_data.BRL_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("BRL Treasury Liquidity Floor (USD):");
                    ui.add(egui::DragValue::new(&mut self.json_data.BRL_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX));
                });
                
                // funding gap
                ui.horizontal(|ui| {
                    ui.label("Require Annual Benchmark:");
                    ui.radio_value(&mut self.json_data.require_annual_benchmark, true, "True");
                    ui.radio_value(&mut self.json_data.require_annual_benchmark, false, "False");
                });
                
                ui.horizontal(|ui| {
                    ui.label("Must Borrow Benchmark in First Year:");
                    ui.radio_value(&mut self.json_data.must_borrow_benchmark_in_first_year, true, "True");
                    ui.radio_value(&mut self.json_data.must_borrow_benchmark_in_first_year, false, "False");
                });

                // interest rate risk
                ui.horizontal(|ui| {
                    ui.label("NII Horizon in Months:");
                    ui.add(egui::widgets::Slider::new(&mut self.json_data.delta_nii_horizon_months, 1..=36));
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
        });
    }
}

// Here we define our JSON's data structure.
#[derive(Debug, Deserialize, Serialize)]
struct JsonData {
    // model
    n_stages: u32,
    step_size_months: u32,
    base_date: NaiveDate,
    start_date: NaiveDate,
    reports_folder: String,
    environment: Environment,
    assumption_profile: AssumptionProfile,
    optimizer: Optimizer,
    fwd_start_swap: IncludedOrExcluded,
    // liquidity risk
    lcr_lower_limit: f32, lcr_upper_limit: f32, // between 100 and 300
    lcr_average_dra_pd: f32,
    MXN_treasury_liquidity_floor: u32,
    COP_treasury_liquidity_floor: u32,
    BRL_treasury_liquidity_floor: u32,
    // funding gap
    require_annual_benchmark: bool,
    must_borrow_benchmark_in_first_year: bool,
    // interest rate risk
    delta_nii_horizon_months: u32,

    // Add more fields as needed
}

// Here we define the "default" JSON data.
impl Default for JsonData {
    fn default() -> Self {
        JsonData {
            // model
            n_stages: 2,
            step_size_months: 4,
            reports_folder: "Reports".to_string(),
            base_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            environment: Environment::Production,
            assumption_profile: AssumptionProfile::BaseCase,
            optimizer: Optimizer::Highs,
            fwd_start_swap: IncludedOrExcluded::Included,
            // liquidity risk
            lcr_lower_limit: 0.0, lcr_upper_limit: 100.0,
            lcr_average_dra_pd: 0.02,
            MXN_treasury_liquidity_floor: 1000000,
            COP_treasury_liquidity_floor: 1500000,
            BRL_treasury_liquidity_floor: 10000000,
            // funding gap
            require_annual_benchmark: false,
            must_borrow_benchmark_in_first_year: false,
            // interest rate risk
            delta_nii_horizon_months: 12,
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
