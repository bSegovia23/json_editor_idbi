use std::{fs::File, ops::RangeInclusive, path::Path};
use std::io::prelude::*;
use egui::{Ui, Vec2, ColorImage};
use serde::{Deserialize, Serialize};
use eframe::egui;
use chrono::NaiveDate;

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
                println!("Error parsing logo image: {}", e);
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
                println!("Error loading JSON data: {}", e);
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

// Generic function to render a widget.
fn render_widget_with_change_tracking(
    app: &mut MyApp,
    ui: &mut Ui,
    widget: impl egui::Widget,
){
    let response = ui.add(widget);
    if response.changed(){
        app.changed = true;
    }
}

// Generic function to render a slider with a label.
fn render_slider_with_label<T>(
    ui: &mut Ui,
    label: &str,
    value: &mut T,
    range: RangeInclusive<T>
)
where
    T: Clone + eframe::emath::Numeric,
{
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::widgets::Slider::new(value, range));
    });
}

// Generic function to render a DragView with a label.
fn render_drag_value_with_label<T>(
    ui: &mut Ui,
    label: &str,
    value: &mut T,
    range: RangeInclusive<T>
)
where
    T: Clone + eframe::emath::Numeric,
{
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).clamp_range(range));
    });
}

// Generic function to render a DragView without a label or new line.
fn render_drag_value_inline<T>(
    ui: &mut Ui,
    value: &mut T,
    range: RangeInclusive<T>
)
where
    T: Clone + eframe::emath::Numeric,
{
    ui.add(egui::DragValue::new(value).clamp_range(range));
}

// Generic function to render a date picker with a label.
fn render_date_picker_with_label(ui: &mut Ui, label: &str, selection: &mut NaiveDate, id: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui_extras::DatePickerButton::new(selection).id_source(id));
    });
}

// Generic function to render a single-line text editor with a label.
fn render_text_edit_with_label(ui: &mut Ui, label: &str, text: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::TextEdit::singleline(text));
    });
}

// Generic function to render enum options as radio buttons with a label.
fn render_enum_options_with_label<T>(
    ui: &mut Ui,
    ui_label: &str,
    current_value: &mut T,
    options: &[T],
    to_user_friendly_label: impl Fn(&T) -> &str,
) where
    T: PartialEq + Copy,
{
    ui.horizontal(|ui| {
        ui.label(ui_label);
        for option in options {
            ui.selectable_value(current_value, *option, to_user_friendly_label(option));
        }
    });
}

// Generic function to render a true/false option.
fn render_bool_options_with_label(
    ui: &mut Ui,
    ui_label: &str,
    current_value: &mut bool) {
        // ui.horizontal(|ui| {
        //     ui.label(ui_label);
        //     ui.radio_value(current_value, true, "True");
        //     ui.radio_value(current_value, false, "False");
        // });
        render_enum_options_with_label(
            ui,
            ui_label,
            current_value,
            &[true, false],
            |b| {
                match b {
                    true => "True",
                    false => "False"
                }
            })
}


// Here we define our app's UI.
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // egui::SidePanel::right("logo-panel").show(ctx, |ui| {
        // });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                // create logo
                match &mut self.logo {
                    Some(logo) => {
                        let size = logo.size_vec2() * 0.1;
                        ui.image(logo, size)
                    },
                    None => ui.label("Logo not available"),
                };
                ui.heading("ALM Dynamic Model");
                // Create UI elements to edit our JSON data
                ui.heading("Run Setup");
                render_slider_with_label(ui, "Number of Stages", &mut self.json_data.n_stages, 1..=4);
                render_slider_with_label(ui, "Step Size in Months", &mut self.json_data.step_size_months, 1..=6);
                render_date_picker_with_label(ui, "Base Date", &mut self.json_data.base_date, "base_date_picker");
                render_date_picker_with_label(ui, "Start Date", &mut self.json_data.start_date, "start_date_picker");
                render_text_edit_with_label(ui, "Reports Folder", &mut self.json_data.reports_folder);
                render_enum_options_with_label(
                    ui,
                    "Environment",
                    &mut self.json_data.environment,
                    &[Environment::Production, Environment::Development, Environment::Testing],
                    |option| option.to_user_friendly_label()
                );
                render_enum_options_with_label(
                    ui,
                    "Assumption Profile",
                    &mut self.json_data.assumption_profile,
                    &[AssumptionProfile::BaseCase, AssumptionProfile::Scenario1, AssumptionProfile::Scenario2, AssumptionProfile::Scenario3],
                    |option| option.to_user_friendly_label()
                );
                render_enum_options_with_label(
                    ui,
                    "Optimizer",
                    &mut self.json_data.optimizer,
                    &[Optimizer::Highs, Optimizer::Cbc, Optimizer::Gurobi],
                    |option| option.to_user_friendly_label()
                );
                render_enum_options_with_label(
                    ui,
                    "Forward Start Swap",
                    &mut self.json_data.fwd_start_swap,
                    &[IncludedOrExcluded::Included, IncludedOrExcluded::Excluded],
                    |option| option.to_user_friendly_label()
                );

                ui.add_space(10.0); // Add some spacing between sections

                ui.heading("Liquidity Parameters");

                const LCR_BOUND_MIN: u32 = 100;
                const LCR_BOUND_MAX: u32 = 300;

                // special logic for lower/upper limit dragvalues because they affect each other
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

                render_drag_value_with_label(
                    ui,
                    "USD Treasury Liquidity as % of Total Assets",
                    &mut self.json_data.lcr_average_dra_pd,
                    0.0..=50.0
                );

                const LIQUIDITY_FLOOR_MIN: u32 = 1000000; // at least set this as 0
                const LIQUIDITY_FLOOR_MAX: u32 = 100000000;

                render_drag_value_with_label(
                    ui,
                    "MXN Treasury Liquidity Floor (USD)",
                    &mut self.json_data.MXN_treasury_liquidity_floor,
                    LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX
                );

                render_drag_value_with_label(
                    ui,
                    "COP Treasury Liquidity Floor (USD)",
                    &mut self.json_data.COP_treasury_liquidity_floor,
                    LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX
                );

                render_drag_value_with_label(
                    ui,
                    "BRL Treasury Liquidity Floor (USD)",
                    &mut self.json_data.BRL_treasury_liquidity_floor,
                    LIQUIDITY_FLOOR_MIN..=LIQUIDITY_FLOOR_MAX
                );
                
                ui.add_space(10.0); // Add some spacing between sections

                // funding gap
                ui.heading("Funding Gap Parameters");

                render_bool_options_with_label(
                    ui,
                    "Require Annual Benchmark",
                    &mut self.json_data.require_annual_benchmark
                );

                render_bool_options_with_label(
                    ui,
                    "Must Borrow Benchmark in First Year",
                    &mut self.json_data.must_borrow_benchmark_in_first_year
                );

                ui.add_space(10.0); // Add some spacing between sections

                // interest rate risk
                ui.heading("Interest Rate Risk Parameters");

                ui.horizontal(|ui| {
                    ui.label("NII Horizon in Months:");
                    ui.add(egui::widgets::Slider::new(&mut self.json_data.delta_nii_horizon_months, 1..=36));
                });

                egui::Grid::new("curves_grid")
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
                                render_drag_value_inline(ui, value, -500..=500)
                            }
                            ui.end_row();
                        }
                    });

                // Add more UI elements as needed for other fields

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // Save the JSON data when the "Save" button is clicked
                        match save_json_data(&self.json_data) {
                            Ok(_) => println!("Data saved successfully!"),
                            Err(e) => println!("Error saving data: {}", e),
                        }
                    }
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
    // let options = eframe::NativeOptions::default();
    let mut options = eframe::NativeOptions::default();
    let _result = eframe::run_native(APP_NAME, options, Box::new(|cc| Box::new(MyApp::new(cc))));
}
