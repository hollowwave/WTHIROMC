fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            wthiromc_lib::commands::scan_processes,
            wthiromc_lib::commands::scan_startup_items,
            wthiromc_lib::commands::mark_process_safe,
            wthiromc_lib::commands::unmark_process_safe,
            wthiromc_lib::commands::mark_startup_safe,
            wthiromc_lib::commands::unmark_startup_safe
        ])
        .run(tauri::generate_context!())
        .expect("error while running WTHIROMC");
}

