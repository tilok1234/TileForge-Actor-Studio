#[tauri::command]
fn studio_status() -> serde_json::Value {
    serde_json::json!({
        "name": "TileForge Actor Studio",
        "version": env!("CARGO_PKG_VERSION"),
        "contract": "tileforge-actor-32-v1",
        "approvalOwner": "user"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![studio_status])
        .run(tauri::generate_context!())
        .expect("error while running TileForge Actor Studio");
}
