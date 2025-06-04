pub struct SteamGameLibrary {
    pub api_key: String,
}

// sqlite test
// {
//     let connection = Connection::open("steam_apps.db").unwrap();

//     let exists = table_exists(&connection, "people").unwrap();

//     if exists {
//         println!("Table 'people' exists.");
//     } else {
//         println!("Table 'people' does not exist. Creating it...");

//         connection.execute(
//             "CREATE TABLE people (
//                  id INTEGER PRIMARY KEY,
//                  name TEXT NOT NULL,
//                  age INTEGER
//              )",
//             [],
//         ).unwrap();

//         connection.execute(
//             "INSERT INTO people (name, age) VALUES (?1, ?2)",
//             params!["Alice", 30],
//         ).unwrap();
//     }

//     let mut stmt = connection.prepare("SELECT id, name, age FROM people").unwrap();
//     let person_iter = stmt.query_map([], |row| {
//         Ok((
//             row.get::<_, i32>(0)?,
//             row.get::<_, String>(1)?,
//             row.get::<_, i32>(2)?,
//         ))
//     }).unwrap();

//     for person in person_iter {
//         let (id, name, age) = person.unwrap();
//         println!("ID: {}, Name: {}, Age: {}", id, name, age);
//     }
// }

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
