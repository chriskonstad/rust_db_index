#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::json::{Json, JsonValue};
use rocket::State;

use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Data {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum Event {
    Set{key: String, value: String},
    Delete{key: String},
}

type MessageMap = HashMap<String, String>;
type EventLog = Vec<Event>;

struct AppState {
    db: MessageMap,
    log: EventLog,
}

#[post("/db", format = "json", data = "<data>")]
fn store(data: Json<Data>, state : State<Mutex<AppState>>) -> JsonValue {
    let s = &mut state.lock().expect("lock state");
    s.db.insert(data.key.clone(), data.value.clone());
    s.log.push(Event::Set{key: data.key.clone(), value: data.value.clone()});
    json!({"status": "ok"})
}

#[delete("/db/<key>")]
fn delete(key: String, state : State<Mutex<AppState>>) -> JsonValue {
    let s = &mut state.lock().expect("lock state");
    s.db.remove(&key);
    s.log.push(Event::Delete{key: key.clone()});
    json!({"status": "ok"})
}

#[get("/db/<key>", format = "json")]
fn get(key: String, state : State<Mutex<AppState>>) -> Option<Json<Data>> {
    let hashmap = &state.lock().unwrap().db;
    hashmap.get(&key).map(|contents| {
        Json(Data{
            key: key.clone(),
            value: contents.clone(),
        })
    })
}

#[get("/log/<from>", format = "json")]
fn log(from: usize, state : State<Mutex<AppState>>) -> Json<Vec<Event>> {
    trace!("From: {}", from);
    let log = &state.lock().expect("lock state").log;
    if log.len() < from {
        return Json([].to_vec())
    }
    Json(log[from..].to_vec())
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found.",
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![store, delete, get, log])
        .manage(Mutex::new(AppState{
            db: HashMap::<String, String>::new(),
            log: Vec::<Event>::new(),
        }))
        .register(catchers![not_found])
        .launch();
}
