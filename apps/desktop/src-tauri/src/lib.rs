// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod core;
mod commands;
mod state;

use crate::core::error::{AppError, ErrorDto};
use crate::commands::plants::create_plant;

#[tauri::command]
fn greet_safe(name: &str) -> Result<String, ErrorDto> {
    if name.trim().is_empty(){
        return Err(AppError::InvalidArgument("name is required".to_string()).into());
    }

    Ok(format!("Hello, {name}! (safe)"))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            greet_safe,
            create_plant
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
