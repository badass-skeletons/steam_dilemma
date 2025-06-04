use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub steam_name: String,
    pub steam_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: u64,
    pub name: String,
    pub tags: Vec<String>,
    pub app_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consultant {
    pub name: String,
    // TODO: use llama3 or mistral 7B
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGameLibrary {
    pub api_key: String,
}

impl SteamGameLibrary {
    pub fn new() -> SteamGameLibrary {
        // let steam_api_key = &std::env::var("STEAM_API_KEY").expect("Missing an API key");
        // let request = "https://store.steampowered.com/api/appdetails?appids=570";

        // let resp = reqwest::get(request).await.unwrap().text().await.unwrap();
        // println!("{:#?}", resp);

        SteamGameLibrary {
            api_key: "a".to_string(),
        }
    }
}

// API Response types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterResponse {
    pub counter_value: u64,
}

// Room management types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: u64,
    pub customers: Vec<Customer>,
    pub consultants: Vec<Consultant>,
}

pub fn test_lib() {
    println!("hello from shared lib");
}
