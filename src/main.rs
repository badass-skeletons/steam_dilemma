#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
pub use app::SteamDilemmaApp;

use rusqlite::Connection;
use steam_rs::steam_apps::get_app_list;
use steam_rs::{Steam, steam_id::SteamId};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // sqlite test
    {
        // let connection = Connection::open_in_memory().unwrap();
    }

    // Steam test
    {
        let steam_api_key = &std::env::var("STEAM_API_KEY").expect("Missing an API key");
        let steam = Steam::new(steam_api_key);

        // https://tradeit.gg/steam-id-finder
        let steam_id = SteamId::new(76561199101214612); // krupitskas TODO: figure out how to get SteamID from text

        let all_user_played_games = steam
            .get_owned_games(steam_id, true, true, 0, true, None, "english", true)
            .await
            .unwrap();

        for game in all_user_played_games.games {
            println!(
                "id {} name {}",
                game.appid,
                game.name.unwrap_or("unknown".to_string())
            );
        }

        println!("global games");

        let all_games = Steam::get_app_list().await.unwrap();

        for game in all_games.apps {
            println!("id {} name {}", game.appid, game.name);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Steam Dilemma",
        native_options,
        Box::new(|cc| Ok(Box::new(SteamDilemmaApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(SteamDilemmaApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
