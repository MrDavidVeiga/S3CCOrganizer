pub mod analyzer;
pub mod catalog;
pub mod cache;
pub mod compression;
pub mod conflicts;
pub mod dbpf;
pub mod duplicates;
pub mod executor;
pub mod i18n;
pub mod manifest;
pub mod operation;
pub mod package_family;
pub mod package_details;
pub mod planner;
pub mod restore;
pub mod restore_history;
pub mod resource_cfg;
pub mod scanner;
pub mod taxonomy;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scanner::scan_packages,
            duplicates::analyze_duplicates,
            conflicts::analyze_conflicts,
            operation::get_operation_status,
            operation::cancel_operation,
            cache::get_cache_info,
            cache::clear_cache,
            package_details::get_package_technical_details,
            restore_history::list_restore_history,
            planner::build_organization_plan,
            executor::execute_organization,
            restore::preview_restore,
            restore::execute_restore
        ])
        .run(tauri::generate_context!())
        .expect("error while running S3CC Organizer");
}
