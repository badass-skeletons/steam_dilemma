//! This module deals with a user's games library.

use library::Game;
use serde::Deserialize;
use std::fmt::Formatter;

use crate::steam::steam_client::SteamClient;
use crate::steam::steam_client::SteamError;

/// The Steam API "GetOwnedGames (v0001)" endpoint
const ENDPOINT_OWNED_GAMES: &str = "http://api.steampowered.com/IPlayerService/GetOwnedGames/v1";

// How to get icon:
// https://media.steampowered.com/steamcommunity/public/images/apps/{appid}/{hash}.jpg

// How to get tags and categories
// https://store.steampowered.com/api/appdetails?appids={APPID}
/*
{
  "APPID": {
    "success": true,
    "data": {
      "name": "Game Name",
      "steam_appid": APPID,
      "genres": [{"id": "1", "description": "Action"}],
      "categories": [{"id": "2", "description": "Multiplayer"}],
      "tags": {"RPG": 123, "Open World": 456}
    }
  }
}
*/

/// Helper struct used during deserializing the API response.
#[derive(Debug, Deserialize)]
struct OwnedGamesResponse {
    response: Option<SteamUserLibrary>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SteamUserLibrary {
    pub game_count: u32,
    pub games: Vec<SteamGame>,
}

impl From<OwnedGamesResponse> for SteamUserLibrary {
    fn from(value: OwnedGamesResponse) -> Self {
        let v = value.response.unwrap_or_default();
        Self {
            game_count: v.game_count,
            games: v.games,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SteamGame {
    #[serde(rename(deserialize = "appid"))]
    pub app_id: u64,
    pub name: String,
    #[serde(rename(deserialize = "playtime_forever"))]
    pub total_playtime: u64,
    pub img_icon_url: String,
}

impl PartialEq for SteamGame {
    fn eq(&self, other: &Self) -> bool {
        self.app_id == other.app_id
    }
}

impl std::fmt::Display for SteamGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game: id {}, name: {}, total time played: {}",
            self.app_id, self.name, self.total_playtime
        )
    }
}

impl SteamClient {
    pub async fn get_user_library(&self, steam_id: &str) -> Result<SteamUserLibrary, SteamError> {
        // ?key=YOUR_API_KEY&steamid=USER_ID&include_appinfo=1&include_played_free_games=1

        let response = self
            .get_request(
                ENDPOINT_OWNED_GAMES,
                vec![
                    ("steamid", steam_id),
                    ("include_appInfo", "1"),
                    ("include_played_free_games", "1"),
                ],
            )
            .await?;

        let games = self.parse_response::<OwnedGamesResponse, SteamUserLibrary>(response)?;

        Ok(games)
    }
}

impl From<SteamGame> for Game {
    fn from(game: SteamGame) -> Self {
        Game {
            id: 0,
            app_id: game.app_id,
            name: game.name,
        }
    }
}