#![feature(proc_macro_hygiene, decl_macro)]
#![feature(vec_remove_item)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::json::{Json, JsonValue};
use rocket::State;

use std::sync::Mutex;
use std::collections::HashMap;

// TODO share these common structs between index + db

#[derive(Serialize, Deserialize)]
struct Match {
    keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Event {
    Set{key: String, value: String},
    Delete{key: String},
}

type MessageMap = HashMap<String, Vec<String>>;

struct AppState {
    db: MessageMap,
    offset: usize,
}

#[get("/<key>", format = "json")]
fn get(key: String, state : State<Mutex<AppState>>) -> Option<Json<Match>> {
    let hashmap = &state.lock().unwrap().db;
    hashmap.get(&key).map(|keys| {
        Json(Match{
            keys: keys.clone(),
        })
    })
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found.",
    })
}

fn apply(event: Event, state: &mut MessageMap) {
    match event {
        Event::Set{key, value} => {
            // Delete old entry
            for val in &mut state.values_mut() {
                val.remove_item(&key);
                // TODO if list is empty, remove it for good
            }

            // Add new entry
            let entry = state.entry(value).or_insert(Vec::new());
            entry.push(key);
        }
        Event::Delete{key} => {
            for val in &mut state.values_mut() {
                val.remove_item(&key);
            }
        }
    }
}

fn main() {
    let state = Mutex::new(AppState{
            db: HashMap::<String, Vec<String>>::new(),
            offset: 0,
    });
    let events = reqwest::Client::new()
        .get("http://localhost:8000/log/0")
        .send()
        .unwrap()
        .json::<Vec<Event>>().unwrap();

    println!("Got events: {}", events.len());
    {
        let db = &mut state.lock().unwrap().db;
        for e in events.into_iter() {
            println!("{:#?}", e);
            apply(e, db);
        }
    }
    rocket::ignite()
        .mount("/", routes![get])
        .manage(state)
        .register(catchers![not_found])
        .launch();
}
