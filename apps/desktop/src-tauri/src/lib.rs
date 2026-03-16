// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod core;
mod state;

use crate::commands::plants::{
    connect_plant, create_plant, disconnect_plant, get_plant, list_plants, pause_plant,
    remove_plant, resume_plant, update_plant,
};
use crate::commands::plugins::{create_plugin, get_plugin, list_plugins, list_plugins_by_type, update_plugin};
use crate::core::error::{AppError, ErrorDto};
use crate::state::AppState;

#[tauri::command]
fn greet_safe(name: &str) -> Result<String, ErrorDto> {
    if name.trim().is_empty() {
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
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            greet_safe,
            create_plant,
            update_plant,
            list_plants,
            get_plant,
            remove_plant,
            connect_plant,
            disconnect_plant,
            pause_plant,
            resume_plant,
            create_plugin,
            get_plugin,
            update_plugin,
            list_plugins,
            list_plugins_by_type,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
