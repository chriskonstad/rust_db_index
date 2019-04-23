#![feature(proc_macro_hygiene, decl_macro)]
#![feature(vec_remove_item)]

#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::json::{Json, JsonValue};
use rocket::State;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
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
fn get(key: String, state : State<Arc<Mutex<AppState>>>) -> Option<Json<Match>> {
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
            let mut keys_to_delete = vec![];
            for (k, v) in state.iter_mut() {
                v.remove_item(&key);
                if v.len() == 0 {
                    keys_to_delete.push(k.clone());
                }
            }
            for k in keys_to_delete {
                trace!("Deleting {}", k);
                state.remove(&k);
            }

            // Add new entry
            let entry = state.entry(value).or_insert(Vec::new());
            entry.push(key);
        }
        Event::Delete{key} => {
            let mut keys_to_delete = vec![];
            for (k, v) in state.iter_mut() {
                v.remove_item(&key);
                if v.len() == 0 {
                    keys_to_delete.push(k.clone());
                }
            }
            for k in keys_to_delete {
                trace!("Deleting {}", k);
                state.remove(&k);
            }
        }
    }
}

fn background_update(state: Arc<Mutex<AppState>>) {
    thread::spawn(move || {
        loop {
            {
                let mut s = state.lock().unwrap();
                let url = format!("http://localhost:8000/log/{}", s.offset);
                let events = reqwest::Client::new()
                    .get(&*url)
                    .send()
                    .unwrap()
                    .json::<Vec<Event>>().unwrap();
                let num_events = events.len();
                if num_events != 0 {
                    info!("Got {} new events", num_events);
                }
                for e in events.into_iter() {
                    info!("{:#?}", e);
                    apply(e, &mut s.db);
                }
                s.offset += num_events;
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

fn main() {
    let state = Arc::new(Mutex::new(AppState{
            db: HashMap::<String, Vec<String>>::new(),
            offset: 0,
    }));
    background_update(Arc::clone(&state));
    rocket::ignite()
        .mount("/", routes![get])
        .manage(state)
        .register(catchers![not_found])
        .launch();
}
