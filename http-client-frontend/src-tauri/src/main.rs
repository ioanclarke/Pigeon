// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use reqwest;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::file_service::{AddCollectionRequest, Request};

mod file_service;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_collections, send_request, add_collection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize)]
struct Response {
    status: u16,
    size: String,
    body: String,
    headers: HashMap<String, String>,
}

fn hashmap_to_headers(hashmap: HashMap<String, String>) -> HeaderMap {
    let mut header_map = HeaderMap::new();
    for (k, v) in hashmap {
        header_map.append(HeaderName::from_bytes(k.as_bytes()).unwrap(), HeaderValue::from_str(&*v).unwrap());
    }
    return header_map;
}

#[tauri::command]
async fn send_request(url: String, headers: HashMap<String, String>) -> String {
    let response = reqwest::Client::new().get(&url).headers(hashmap_to_headers(headers)).send().await;
    let response = match response {
        Ok(response) => response,
        Err(e) => return e.to_string(),
    };
    let status = response.status().as_u16();
    let headers = response.headers();

    let mut headers_map = HashMap::new();
    for (key, value) in headers.iter() {
        let header_value = value.to_str().unwrap().to_string();
        headers_map.insert(key.to_string(), header_value);
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => return e.to_string(),
    };
    let size = body.len().to_string();
    let my_response = Response {
        status,
        body,
        size,
        headers: headers_map,
    };
    let historic_request: Request  = Request {
        name: String::from("test"),
        url,
        method: "GET".parse().unwrap()
    };
    file_service::add_history(historic_request);

    return serde_json::to_string(&my_response).expect("Error");
}

#[tauri::command]
fn get_collections() -> file_service::Requests {
    return file_service::get_files();
}

#[tauri::command]
fn add_collection(config: AddCollectionRequest) -> bool {
    return file_service::add_collection(config);
}