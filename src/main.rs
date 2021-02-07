#![feature(proc_macro_hygiene, decl_macro)]

use parking_lot::Mutex;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{request::Form, Request, Response, State};
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize, Debug)]
struct IncomingClient {
    // The raw, undecoded value. You _probably_ want `String` instead.
    address: String,
    value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct IncomingQuery {
    address: String,
}

#[post("/api/client_add", format = "json", data = "<incoming_client>")]
fn new_client(
    clients: State<'_, Mutex<HashMap<String, Vec<u8>>>>,
    incoming_client: Json<IncomingClient>,
) {
    println!("{:?}", incoming_client);
    let mut clients = clients.lock();
    clients.insert(
        incoming_client.address.clone(),
        incoming_client.value.clone(),
    );
}

#[options("/api/client_add", format = "json")]
fn new_client_preflight() {}

#[post("/api/client_query", format = "json", data = "<incoming_query>")]
fn query_clients(
    clients: State<'_, Mutex<HashMap<String, Vec<u8>>>>,
    incoming_query: Json<IncomingQuery>,
) -> Option<JsonValue> {
    let clients = clients.lock();
    let r = clients.get(&incoming_query.address);
    match r {
        Some(x) => Some(json!({ "key": x })),
        None => None,
    }
}

#[options("/api/client_query", format = "json")]
fn query_clients_preflight() {}

fn main() {
    let lookup = Mutex::from(HashMap::<String, Vec<u8>>::new());
    rocket::ignite()
        .attach(CORS())
        .mount(
            "/",
            routes![new_client, new_client_preflight, query_clients, query_clients_preflight],
        )
        .manage(lookup)
        .launch();
}

pub struct CORS();

impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to requests",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
