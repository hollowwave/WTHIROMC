use wthiromc_lib::commands::scan_processes;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            wthiromc_lib::commands::scan_processes
        ])
        .run(tauri::generate_context!())
        .expect("error while running WTHIROMC");
}
