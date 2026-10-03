pub mod analyzer;
pub mod audit_report;
pub mod catalog;
pub mod cache;
pub mod compression;
pub mod conflicts;
pub mod dbpf;
pub mod duplicates;
pub mod executor;
pub mod i18n;
pub mod manifest;
pub mod mesh_info;
pub mod navigation;
pub mod operation;
pub mod package_family;
pub mod package_details;
pub mod planner;
pub mod quarantine;
pub mod restore;
pub mod restore_history;
pub mod review_store;
pub mod resource_cfg;
pub mod scanner;
pub mod structure_manager;
pub mod workspace;
pub mod snapshots;
pub mod health;
pub mod dependencies;
pub mod technical_search;
pub mod package_compare;
pub mod inbox;
pub mod selection_export;
pub mod history;
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
            navigation::open_directory,
            navigation::reveal_path,
            package_details::get_package_technical_details,
            restore_history::list_restore_history,
            review_store::load_conflict_decisions,
            review_store::set_conflict_decision,
            audit_report::save_audit_report,
            structure_manager::list_structure,
            structure_manager::list_structure_directories,
            structure_manager::create_structure_folder,
            structure_manager::move_structure_path,
            structure_manager::rename_structure_folder,
            structure_manager::list_manual_operations,
            structure_manager::undo_last_manual_operation,
            workspace::load_workspace,
            workspace::save_workspace,
            workspace::set_read_only,
            workspace::set_package_metadata,
            workspace::set_manual_classification,
            workspace::save_package_group,
            workspace::delete_package_group,
            snapshots::create_snapshot,
            snapshots::list_snapshots,
            snapshots::compare_snapshot_to_current,
            snapshots::compare_mods_roots,
            health::analyze_mods_health,
            health::remove_empty_folder,
            dependencies::analyze_dependencies,
            technical_search::technical_search,
            package_compare::compare_packages,
            inbox::scan_inbox,
            inbox::build_inbox_import_plan,
            inbox::execute_inbox_import,
            selection_export::save_selection_export,
            history::get_operation_history,
            quarantine::build_quarantine_plan,
            quarantine::execute_quarantine,
            quarantine::restore_quarantine,
            quarantine::recover_quarantine,
            planner::build_organization_plan,
            executor::execute_organization,
            restore::preview_restore,
            restore::execute_restore
        ])
        .run(tauri::generate_context!())
        .expect("error while running S3CC Organizer");
}
