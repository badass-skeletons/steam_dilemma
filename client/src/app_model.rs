use crate::{consultant::Consultant, customer::Customer, steam_database::SteamGameLibrary};

pub struct AppModel {
    pub customers: Vec<Customer>,
    pub consultant: Consultant,
    pub game_library: SteamGameLibrary,
}

impl AppModel {


}
