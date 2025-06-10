use egui::{ColorImage, Image, TextureHandle};
use image::load_from_memory;
use library::{Consultant, CounterResponse, Customer, NewCustomerResponse, Room};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

#[derive(Clone)]
enum RequestState {
    Idle,
    Loading,
    Success(u64),
    Error(String),
}

#[derive(Clone, Serialize, Deserialize)] //
struct ClientState {
    pub current_room: Option<Room>,
    pub server_counter: Option<u64>,

    pub steam_id_str: String,
    pub current_customer: Option<Customer>,

    #[serde(skip)]
    test_texture: Option<egui::TextureHandle>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SteamDilemmaUi {
    client_state: ClientState,

    // UI-specific state
    label: String,
    room_id: Option<u64>,

    #[serde(skip)]
    value: f32,

    #[serde(skip)] // Don't serialize HTTP client
    http_client: Option<reqwest::Client>,

    #[serde(skip)] // Don't serialize request state
    request_state: Arc<Mutex<RequestState>>,

    #[serde(skip)] // Don't serialize shared client state for async operations
    shared_client_state: Arc<Mutex<ClientState>>,
}

fn parse_room_id_from_url() -> Option<u64> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let location_href = window.location().href().ok()?;
        let url = url::Url::parse(&location_href).ok()?;

        for (key, value) in url.query_pairs() {
            if key == "room_id" {
                return value.parse().ok();
            }
        }
    }
    None
}

async fn download_image(url: &str) -> Result<image::DynamicImage, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

fn image_to_color_image(img: &image::DynamicImage) -> egui::ColorImage {
    let img = img.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let pixels = img
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    egui::ColorImage { size, pixels }
}
async fn send_get_customer_library_request(
    client: reqwest::Client,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    let response_result = client
        .post("http://127.0.0.1:3000/api/get_customer_library")
        .json("76561199101214612")
        .send()
        .await;

    handle_get_customer_library_response(response_result, request_state, shared_client_state, ctx)
        .await;
}

async fn handle_get_customer_library_response(
    response_result: Result<reqwest::Response, reqwest::Error>,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    match response_result {
        Ok(response) => {
            handle_successful_custom_library_response(
                response,
                request_state,
                shared_client_state,
                ctx,
            )
            .await;
        }
        Err(e) => {
            update_request_state_error(&request_state, format!("Request failed: {}", e));
            ctx.request_repaint();
        }
    }
}

async fn send_increment_request(
    client: reqwest::Client,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    let response_result = client
        .post("http://127.0.0.1:3000/api/increment")
        .send()
        .await;

    handle_increment_response(response_result, request_state, shared_client_state, ctx).await;
}

async fn handle_increment_response(
    response_result: Result<reqwest::Response, reqwest::Error>,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    match response_result {
        Ok(response) => {
            handle_successful_response(response, request_state, shared_client_state, ctx).await;
        }
        Err(e) => {
            update_request_state_error(&request_state, format!("Request failed: {}", e));
            ctx.request_repaint();
        }
    }
}

async fn handle_successful_custom_library_response(
    response: reqwest::Response,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    match response.json::<NewCustomerResponse>().await {
        Ok(customer_response) => {
            // update_request_state_success(&request_state, counter_response.counter_value);
            // update_client_state_counter(&shared_client_state, counter_response.counter_value);

            if let Ok(mut client_state) = shared_client_state.lock() {
                log::debug!("{:?}", customer_response.customer);
                client_state.current_customer = Some(customer_response.customer);
            }

            let proxy_url = format!(
                "https://corsproxy.io/?{}",
                urlencoding::encode("https://media.steampowered.com/steamcommunity/public/images/apps/920210/568a07f03f8dba0b74c4a02f6ebb43ce5e09075c.jpg")
            );

            log::debug!("proxy_url: {}", proxy_url);

            let mut new_test_texture: Option<egui::TextureHandle> = None;

            let url = proxy_url;//"https://media.steampowered.com/steamcommunity/public/images/apps/920210/568a07f03f8dba0b74c4a02f6ebb43ce5e09075c.jpg";
            if let Ok(image) = download_image(url.as_str()).await {
                let color_image = image_to_color_image(&image);
                new_test_texture = Some(ctx.load_texture("my_image", color_image, egui::TextureOptions::default()));
            }

            if let Ok(mut client_state) = shared_client_state.lock() {
                client_state.test_texture = new_test_texture;
            }

            ctx.request_repaint();
        }
        Err(e) => {
            update_request_state_error(&request_state, format!("Failed to parse response: {}", e));
            ctx.request_repaint();
        }
    }
}

async fn handle_successful_response(
    response: reqwest::Response,
    request_state: Arc<Mutex<RequestState>>,
    shared_client_state: Arc<Mutex<ClientState>>,
    ctx: egui::Context,
) {
    match response.json::<CounterResponse>().await {
        Ok(counter_response) => {
            update_request_state_success(&request_state, counter_response.counter_value);
            update_client_state_counter(&shared_client_state, counter_response.counter_value);
            ctx.request_repaint();
        }
        Err(e) => {
            update_request_state_error(&request_state, format!("Failed to parse response: {}", e));
            ctx.request_repaint();
        }
    }
}

fn update_request_state_success(request_state: &Arc<Mutex<RequestState>>, counter_value: u64) {
    if let Ok(mut state) = request_state.lock() {
        *state = RequestState::Success(counter_value);
    }
}

fn update_request_state_error(request_state: &Arc<Mutex<RequestState>>, error_message: String) {
    if let Ok(mut state) = request_state.lock() {
        *state = RequestState::Error(error_message);
    }
}

fn update_client_state_counter(shared_client_state: &Arc<Mutex<ClientState>>, counter_value: u64) {
    if let Ok(mut client_state) = shared_client_state.lock() {
        client_state.server_counter = Some(counter_value);
    }
}

fn render_top_panel(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("About").clicked() {
                    // Could show an about dialog
                }
            });
            ui.add_space(16.0);
            egui::widgets::global_theme_preference_buttons(ui);
        });
    });
}

fn render_room_info(ui: &mut egui::Ui, label: &mut String) {
    ui.horizontal(|ui| {
        ui.label("Room info: ");
        ui.text_edit_singleline(label);
    });
    ui.add_space(10.0);
}

fn render_server_counter_section(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut SteamDilemmaUi) {
    ui.separator();
    ui.heading("Server Counter");

    let can_send_request = get_can_send_request_status(&app.request_state);

    ui.add_enabled_ui(can_send_request, |ui| {
        if ui.button("Increment Server").clicked() {
            app.increment_server_counter(ctx);
        }
    });

    render_request_status(ui, &app.request_state);
    render_client_state_counter(ui, app.client_state.server_counter);
}

fn get_can_send_request_status(request_state: &Arc<Mutex<RequestState>>) -> bool {
    if let Ok(state) = request_state.lock() {
        !matches!(*state, RequestState::Loading)
    } else {
        false
    }
}

fn render_request_status(ui: &mut egui::Ui, request_state: &Arc<Mutex<RequestState>>) {
    if let Ok(state) = request_state.lock() {
        match &*state {
            RequestState::Idle => {
                ui.colored_label(egui::Color32::GRAY, "Ready to increment server counter");
            }
            RequestState::Loading => {
                ui.colored_label(egui::Color32::YELLOW, "Sending request...");
            }
            RequestState::Success(counter_value) => {
                ui.horizontal(|ui| {
                    ui.label("Last response:");
                    ui.colored_label(egui::Color32::GREEN, format!("{}", counter_value));
                });
            }
            RequestState::Error(error) => {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            }
        }
    }
}

fn render_client_state_counter(ui: &mut egui::Ui, server_counter: Option<u64>) {
    if let Some(counter) = server_counter {
        ui.horizontal(|ui| {
            ui.label("Client state counter:");
            ui.colored_label(egui::Color32::BLUE, format!("{}", counter));
        });
    }
}

fn render_steam_section(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut SteamDilemmaUi) {
    ui.add_space(10.0);
    ui.separator();

    ui.horizontal(|ui| {
        // ui.text_edit_singleline(&mut app.shared_client_state.lock().unwrap().steam_id_str);
        if ui.button("Load Library").clicked() {
            app.get_customer_game_library(ctx);
        }
    });

    if let Some(current_customer) = &app.client_state.current_customer {
        ui.vertical(|ui| {
            ui.label("Customer Data:");
            ui.label(&current_customer.steam_name);
        });

        ui.vertical(|ui| {
            for game in &current_customer.games {
                ui.horizontal(|ui| {
                    ui.label("Game:");
                    if let Some(texture) = &app.client_state.test_texture {
                        ui.add(Image::new(texture));
                    }
                    ui.label(&game.name);
                });
            }
        });
    }
}

fn render_central_panel(ctx: &egui::Context, app: &mut SteamDilemmaUi) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Steam Dilemma");

        render_room_info(ui, &mut app.label);
        render_server_counter_section(ui, ctx, app);
        render_steam_section(ui, ctx, app);
    });
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            current_room: None,
            server_counter: None,
            steam_id_str: "".to_owned(),
            current_customer: None,
            test_texture: None,
        }
    }
}

impl Default for SteamDilemmaUi {
    fn default() -> Self {
        let client_state = ClientState::default();
        Self {
            shared_client_state: Arc::new(Mutex::new(client_state.clone())),
            client_state,
            label: "Steam Dilemma Client".to_owned(),
            room_id: None,
            value: 2.1,
            http_client: None,
            request_state: Arc::new(Mutex::new(RequestState::Idle)),
        }
    }
}

impl SteamDilemmaUi {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app: SteamDilemmaUi = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app.room_id = parse_room_id_from_url();

        if let Some(room_id) = app.room_id {
            app.label = format!("Room ID: {}", room_id);
        }

        // Initialize HTTP client
        app.http_client = Some(reqwest::Client::new());

        app
    }

    fn sync_client_state(&mut self) {
        // Synchronize shared client state back to the main client state
        if let Ok(shared_state) = self.shared_client_state.lock() {
            self.client_state = shared_state.clone();
        }
    }

    fn increment_server_counter(&mut self, ctx: &egui::Context) {
        if let Some(client) = &self.http_client {
            let client = client.clone();
            let ctx = ctx.clone();
            let request_state = self.request_state.clone();
            let shared_client_state = self.shared_client_state.clone();

            // Set loading state
            if let Ok(mut state) = request_state.lock() {
                *state = RequestState::Loading;
            }

            wasm_bindgen_futures::spawn_local(async move {
                send_increment_request(client, request_state, shared_client_state, ctx).await;
            });
        }
    }

    fn get_customer_game_library(&mut self, ctx: &egui::Context) {
        if let Some(client) = &self.http_client {
            let client = client.clone();
            let ctx = ctx.clone();
            let request_state = self.request_state.clone();
            let shared_client_state = self.shared_client_state.clone();

            if let Ok(mut state) = request_state.lock() {
                *state = RequestState::Loading;
            }

            wasm_bindgen_futures::spawn_local(async move {
                send_get_customer_library_request(client, request_state, shared_client_state, ctx)
                    .await;
            });
        }
    }
}

impl eframe::App for SteamDilemmaUi {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_client_state();

        render_top_panel(ctx);
        render_central_panel(ctx, self);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
