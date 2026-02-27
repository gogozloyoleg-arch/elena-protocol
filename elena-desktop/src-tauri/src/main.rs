// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::Manager;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const WALLET_NAME: &str = "default";
const NODE_ADMIN: &str = "127.0.0.1:9190";
const GATEWAY_LISTEN: &str = "127.0.0.1:9180";

struct NodeState {
    node_process: Option<Child>,
    gateway_process: Option<Child>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(NodeState {
            node_process: None,
            gateway_process: None,
        }))
        .invoke_handler(tauri::generate_handler![
            get_elena_data_dir,
            ensure_wallet,
            start_node,
            stop_node,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resource_bin(app: &tauri::AppHandle, name: &str) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;
    let bin_dir = resource_dir.join("bin");
    #[cfg(windows)]
    let path = bin_dir.join(format!("{}.exe", name));
    #[cfg(not(windows))]
    let path = bin_dir.join(name);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[tauri::command]
fn get_elena_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let elena_dir = data_dir.join("elena");
    std::fs::create_dir_all(&elena_dir).map_err(|e| e.to_string())?;
    Ok(elena_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn ensure_wallet(app: tauri::AppHandle) -> Result<(), String> {
    let data_dir = get_elena_data_dir(app.clone())?;
    let wallet_path = PathBuf::from(&data_dir)
        .join("wallets")
        .join(format!("{}.key", WALLET_NAME));
    if wallet_path.exists() {
        return Ok(());
    }
    let core_bin = resource_bin(&app, "elena-core")
        .ok_or_else(|| "elena-core не найден. Соберите бинарники и положите в resources/bin/ (см. README).".to_string())?;
    let mut cmd = Command::new(&core_bin);
    cmd.arg("-d").arg(&data_dir);
    cmd.arg("wallet").arg(WALLET_NAME);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let out = cmd.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Ошибка создания кошелька: {}", stderr));
    }
    Ok(())
}

#[tauri::command]
fn start_node(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<NodeState>>();
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if guard.node_process.is_some() {
        return Err("Узел уже запущен.".to_string());
    }
    let data_dir = get_elena_data_dir(app.clone())?;
    let core_bin = resource_bin(&app, "elena-core")
        .ok_or_else(|| "elena-core не найден. Соберите бинарники (см. README).".to_string())?;
    let gateway_bin = resource_bin(&app, "elena-gateway")
        .ok_or_else(|| "elena-gateway не найден. Соберите бинарники (см. README).".to_string())?;

    let mut node_cmd = Command::new(&core_bin);
    node_cmd
        .arg("-d")
        .arg(&data_dir)
        .arg("run")
        .arg("--wallet")
        .arg(WALLET_NAME)
        .arg("--admin")
        .arg(NODE_ADMIN)
        .arg("--listen")
        .arg("/ip4/127.0.0.1/tcp/9000");
    node_cmd.stdout(Stdio::null());
    node_cmd.stderr(Stdio::null());
    #[cfg(windows)]
    node_cmd.creation_flags(0x0800_0000);
    let node_child = node_cmd.spawn().map_err(|e| e.to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut gw_cmd = Command::new(&gateway_bin);
    gw_cmd
        .env("ELENA_NODE_ADMIN", NODE_ADMIN)
        .env("GATEWAY_LISTEN", GATEWAY_LISTEN);
    gw_cmd.stdout(Stdio::null());
    gw_cmd.stderr(Stdio::null());
    #[cfg(windows)]
    gw_cmd.creation_flags(0x0800_0000);
    let gateway_child = gw_cmd.spawn().map_err(|e| e.to_string())?;

    guard.node_process = Some(node_child);
    guard.gateway_process = Some(gateway_child);
    Ok(())
}

#[tauri::command]
fn stop_node(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<NodeState>>();
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if let Some(mut p) = guard.gateway_process.take() {
        let _ = p.kill();
    }
    if let Some(mut p) = guard.node_process.take() {
        let _ = p.kill();
    }
    Ok(())
}
