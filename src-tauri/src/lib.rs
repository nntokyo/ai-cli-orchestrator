#[tauri::command]
fn bootstrap_status() -> &'static str {
    orchestrator_core::bootstrap_status()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![bootstrap_status])
        .run(tauri::generate_context!())
        .expect("error while running AI CLI Orchestrator");
}
