use std::{fs::File, path::Path};
use std::io::prelude::*;
use egui::{Grid, TextEdit};
use egui::widgets::{Slider, DragValue, Button};
use egui_extras::DatePickerButton;
use serde::{Deserialize, Serialize};
use eframe::egui;
use chrono::NaiveDate;
use std::string::ToString;
use strum_macros::Display;

const APP_NAME: &str = "ALM Dynamic Model";
const NUM_YEARS_MODELED: usize = 10; // for the risk curve

// Here we define what our app keeps in its storage.
struct MyApp {
    json_data: JsonData,
    changed: bool,
    logo: Option<egui::TextureHandle>,
}

// Here we define what a "new" app looks like.
impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If you want to customize GUI fonts and visuals, do it here.
        // cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals
        let mut app = Self::default();
        let image = load_image_from_path(Path::new("res/logo.png"));
        match image {
            Ok(color_image) => {
                let img_options = Default::default();
                let logo = cc.egui_ctx.load_texture("logo", color_image, img_options);
                app.logo = Some(logo);
            }
            Err(e) => {
                eprintln!("Error parsing logo image: {}", e);
            }
        }
        app
    }
}

// Here we define the "default" app.
impl Default for MyApp {
    fn default() -> Self {
        // Load the initial JSON data from a file
        let json_data = match load_json_data() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error loading JSON data: {}", e);
                JsonData::default()
            }
        };

        let changed = false;

        MyApp { json_data, changed, logo: None } 
    }
}

fn load_image_from_path(path: &Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}


// Here we define our app's UI.
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("leftpanel").show(ctx, |ui| {// create logo
            match &mut self.logo {
                Some(logo) => {
                    let size = logo.size_vec2() * 0.1;
                    ui.image(logo, size)
                },
                None => ui.label("Logo not available"),
            };
            ui.horizontal(|ui| {
                if ui.add(Button::new("Run").min_size(egui::vec2(122.00,10.0))).clicked() {
                    // TODO
                    eprintln!("Model not found");
                }
                let save_possible = self.changed;
                if ui.add_enabled(save_possible, Button::new("Save").min_size(egui::vec2(122.00,10.0))).clicked() {
                    // Save the JSON data when the "Save" button is clicked
                    match save_json_data(&self.json_data) {
                        Ok(_) => {
                            println!("Data saved successfully!");
                            self.changed = false;
                        },
                        Err(e) => eprintln!("Error saving data: {}", e),
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                ui.heading("Run Setup");
                Grid::new("run-setup-grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                    ui.label("Number of Stages");
                    self.changed |= ui.add(Slider::new(&mut self.json_data.n_stages, 1..=4)).changed();
                    ui.end_row();

                    ui.label("Step Size in Months");
                    self.changed |= ui.add(Slider::new(&mut self.json_data.step_size_months, 1..=6)).changed();
                    ui.end_row();

                    ui.label("Base Date");
                    self.changed |= ui.add(DatePickerButton::new(&mut self.json_data.base_date).id_source("base_date_picker")).changed();
                    ui.end_row();

                    ui.label("Start Date");
                    self.changed |= ui.add(DatePickerButton::new(&mut self.json_data.start_date).id_source("start_date_picker")).changed();
                    ui.end_row();

                    ui.label("Reports Folder");
                    self.changed |= ui.add(TextEdit::singleline(&mut self.json_data.reports_folder)).changed();
                    ui.end_row();

                    ui.label("Environment");
                    ui.horizontal(|ui| {
                        for option in [Environment::Production, Environment::Development, Environment::Testing] {
                            // self.changed |= ui.selectable_value(&mut self.json_data.environment, option, option.to_user_friendly_label()).changed();
                            self.changed |= ui.selectable_value(&mut self.json_data.environment, option, option.to_string()).changed();
                        }
                    });
                    ui.end_row();

                    ui.label("Assumption Profile");
                    ui.horizontal(|ui| {
                        for option in [AssumptionProfile::BaseCase, AssumptionProfile::Scenario1, AssumptionProfile::Scenario2, AssumptionProfile::Scenario3] {
                            self.changed |= ui.selectable_value(&mut self.json_data.assumption_profile, option, option.to_string()).changed();
                        }
                    });
                    ui.end_row();

                    ui.label("Optimizer");
                    ui.horizontal(|ui| {
                        for option in [Optimizer::Highs, Optimizer::Cbc, Optimizer::Gurobi] {
                            self.changed |= ui.selectable_value(&mut self.json_data.optimizer, option, option.to_string()).changed();
                        }
                    });
                    ui.end_row();

                    ui.label("Forward Start Swap");
                    ui.horizontal(|ui| {
                        for option in [IncludedOrExcluded::Included, IncludedOrExcluded::Excluded] {
                            self.changed |= ui.selectable_value(&mut self.json_data.fwd_start_swap, option, option.to_user_friendly_label()).changed();
                        }
                    });
                    ui.end_row();
                });

                ui.add_space(10.0); // Add some spacing between sections

                ui.heading("Liquidity Parameters");

                const LCR_BOUND_MIN: u32 = 100;
                const LCR_BOUND_MAX: u32 = 300;

                Grid::new("liq-par-grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                    // special logic for lower/upper limit dragvalues because they affect each other
                    ui.label("LCR Lower Limit:");
                    let response = ui.add(DragValue::new(&mut self.json_data.lcr_lower_limit).speed(0.5).clamp_range(LCR_BOUND_MIN..=LCR_BOUND_MAX));
                    if response.changed() {
                        self.changed = true;
                        if self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                            self.json_data.lcr_upper_limit = self.json_data.lcr_lower_limit;
                        }
                    }
                    ui.end_row();

                    ui.label("LCR Upper Limit:");
                    let response = ui.add(DragValue::new(&mut self.json_data.lcr_upper_limit).speed(0.5).clamp_range(LCR_BOUND_MIN..=LCR_BOUND_MAX));
                    if response.changed() {
                        self.changed = true;
                        if self.json_data.lcr_lower_limit > self.json_data.lcr_upper_limit {
                            self.json_data.lcr_lower_limit = self.json_data.lcr_upper_limit;
                        }
                    }
                    ui.end_row();

                    ui.label("USD Treasury Liquidity as % of Total Assets");
                    self.changed |= ui.add(DragValue::new(&mut self.json_data.lcr_average_dra_pd).clamp_range(0.0..=50.0)).changed();
                    ui.end_row();

                    const LIQUIDITY_FLOOR_MIN: u32 = 1000000; // at least set this as 0
                    const LIQUIDITY_FLOOR_MAX: u32 = 100000000;

                    ui.label("MXN Treasury Liquidity Floor (USD)");
                    self.changed |= ui.add(DragValue::new(&mut self.json_data.MXN_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX)).changed();
                    ui.end_row();

                    ui.label("COP Treasury Liquidity Floor (USD)");
                    self.changed |= ui.add(DragValue::new(&mut self.json_data.COP_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX)).changed();
                    ui.end_row();

                    ui.label("BRL Treasury Liquidity Floor (USD)");
                    self.changed |= ui.add(DragValue::new(&mut self.json_data.BRL_treasury_liquidity_floor).clamp_range(LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX)).changed();
                    ui.end_row();
                });
                
                ui.add_space(10.0); // Add some spacing between sections

                // funding gap
                ui.heading("Funding Gap Parameters");

                Grid::new("fund-gap-par")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Require Annual Benchmark");
                        ui.horizontal(|ui| {
                            for option in [true, false] {
                                self.changed |= ui.selectable_value(&mut self.json_data.require_annual_benchmark, option, if option { "True" } else { "False" }).changed();
                            }
                        });
                        ui.end_row();

                        ui.label("Must Borrow Benchmark in First Year");
                        ui.horizontal(|ui| {
                            for option in [true, false] {
                                self.changed |= ui.selectable_value(&mut self.json_data.must_borrow_benchmark_in_first_year, option, if option { "True" } else { "False" }).changed();
                            }
                        });
                        ui.end_row();
                    });

                ui.add_space(10.0); // Add some spacing between sections

                // interest rate risk
                ui.heading("Interest Rate Risk Parameters");
                
                Grid::new("irr-par")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("NII Horizon in Months");
                        self.changed |= ui.add(Slider::new(&mut self.json_data.delta_nii_horizon_months, 1..=36)).changed();
                        ui.end_row();

                        ui.label("Rate Shock Size");
                        Grid::new("curves_grid")
                            .striped(true)
                            .num_columns(NUM_YEARS_MODELED+1)
                            .show(ui, |ui| {
                                ui.label("");
                                for year in 1..=NUM_YEARS_MODELED {
                                    ui.label(format!("Y{}", year));
                                }
                                ui.end_row();
                                for (curve_id, curve_array) in self.json_data.delta_nii_shocks_bps.iter_mut() {
                                    ui.label(curve_id);
                                    for value in curve_array {
                                        self.changed |= ui.add(DragValue::new(value).clamp_range(-500..=500)).changed();
                                    }
                                    ui.end_row();
                                }
                            });
                    });
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
    delta_nii_shocks_bps: std::collections::BTreeMap<String, [i32; NUM_YEARS_MODELED]>,
    // Add more fields as needed
}

// Here we define the "default" JSON data.
impl Default for JsonData {
    fn default() -> Self {
        let default_array: [i32; NUM_YEARS_MODELED] = [100; NUM_YEARS_MODELED];
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
            // curve data
            delta_nii_shocks_bps: std::collections::BTreeMap::from([
                ("CURVE_USD_FED_FUNDS".to_string(), default_array.clone()),
                ("CURVE_TTD_LIBOR6M".to_string(), default_array.clone()),
                ("CURVE_USD_OVERNIGHTSOFR".to_string(), default_array.clone()),
                ("CURVE_USD_LIBOR1M".to_string(), default_array.clone()),
                ("CURVE_USD_LIBOR3M".to_string(), default_array.clone()),
                ("CURVE_USD_LIBOR6M".to_string(), default_array.clone()),
                ("CURVE_USD_LIBOR12M".to_string(), default_array.clone()),
                ("CURVE_MXN_TIIE28D".to_string(), default_array.clone()),
                ("CURVE_BRL_CDI".to_string(), default_array.clone()),
                ("CURVE_COP_OVIBR".to_string(), default_array.clone()),
                ("CURVE_USD_OIS".to_string(), default_array.clone()),
                ("CURVE_TTD_GORTT".to_string(), default_array.clone()),
                ("CURVE_PEN_V_USD6M".to_string(), default_array.clone()),
                ("CURVE_AUD_OIS".to_string(), default_array.clone()),
                ("CURVE_CLP_V_CAMARA".to_string(), default_array.clone()),
                ("CURVE_EUR_OIS".to_string(), default_array.clone()),
            ]),
        }
    }
}

// Here we define the possible environments.
#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Environment {
    Production,
    Development,
    Testing,
}

// Here we define the possible assumption profiles.
#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum AssumptionProfile {
    #[serde(rename = "BASE CASE")]
    #[strum(serialize = "Base Case")]
    BaseCase,
    #[serde(rename = "SCENARIO 1")]
    #[strum(serialize = "Scenario 1")]
    Scenario1,
    #[serde(rename = "SCENARIO 2")]
    #[strum(serialize = "Scenario 2")]
    Scenario2,
    #[serde(rename = "SCENARIO 3")]
    #[strum(serialize = "Scenario 3")]
    Scenario3,
}

// Here we define the possible optimizers.
#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum Optimizer {
    Highs,
    Cbc,
    Gurobi,
}

// Here we define the possible open fields.
#[derive(Debug, Display, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum IncludedOrExcluded {
    Included,
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
    // let mut options = eframe::NativeOptions::default();
    let _result = eframe::run_native(APP_NAME, options, Box::new(|cc| Box::new(MyApp::new(cc))));
}
