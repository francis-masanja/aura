use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::process::Command;
use futures_lite::stream::StreamExt;
use brightness::Brightness;

#[tauri::command]
fn media_control(action: String) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    match action.as_str() {
        "play_pause" => {
            let _ = enigo.key(Key::MediaPlayPause, Direction::Click);
        }
        "next" => {
            let _ = enigo.key(Key::MediaNextTrack, Direction::Click);
        }
        "prev" => {
            let _ = enigo.key(Key::MediaPrevTrack, Direction::Click);
        }
        _ => {}
    }
}

#[tauri::command]
fn volume_control(delta: i32) {
    #[cfg(target_os = "linux")]
    {
        let sign = if delta >= 0 { "+" } else { "-" };
        let _ = Command::new("amixer")
            .arg("set")
            .arg("Master")
            .arg(format!("{}%{}", delta.abs(), sign))
            .spawn();
    }
    
    #[cfg(target_os = "windows")]
    {
        let volume_cmd = format!(
            "$obj = New-Object -ComObject WScript.Shell; for($i=0; $i<{}; $i++){{ $obj.SendKeys([char]17{}) }}",
            (delta.abs() / 2),
            if delta > 0 { "5" } else { "4" }
        );
        let _ = Command::new("powershell")
            .arg("-Command")
            .arg(volume_cmd)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "set volume output volume ((output volume of (get volume settings)) + {})",
            delta
        );
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .spawn();
    }
}

#[tauri::command]
async fn brightness_control(delta: i32) {
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        let mut devices = brightness::brightness_devices();
        if let Some(Ok(mut device)) = devices.next().await {
            if let Ok(current) = device.get().await {
                let new_val = (current as i32 + delta).clamp(0, 100) as u32;
                let _ = device.set(new_val).await;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("brightness")
            .arg(format!("{}", if delta > 0 { "+0.1" } else { "-0.1" }))
            .spawn();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            media_control,
            volume_control,
            brightness_control
        ])
        .setup(|app| {
            #[cfg(target_os = "linux")]
            {
                use tauri::Manager;
                use webkit2gtk::glib::ObjectExt;
                use webkit2gtk::WebViewExt;
                use webkit2gtk::PermissionRequestExt;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.with_webview(|webview| {
                        #[cfg(target_os = "linux")]
                        {
                            webview.inner().connect_permission_request(|_, request: &webkit2gtk::PermissionRequest| {
                                if request.is::<webkit2gtk::UserMediaPermissionRequest>() || 
                                   request.is::<webkit2gtk::DeviceInfoPermissionRequest>() {
                                    request.allow();
                                    return true;
                                }
                                false
                            });
                        }
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
