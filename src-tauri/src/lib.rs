pub mod analyzer;
pub mod catalog;
pub mod compression;
pub mod dbpf;
pub mod i18n;
pub mod manifest;
pub mod scanner;
pub mod taxonomy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scanner::scan_packages])
        .run(tauri::generate_context!())
        .expect("error while running S3CC Organizer");
}
