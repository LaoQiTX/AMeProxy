use std::sync::atomic::Ordering;
use tauri::{generate_handler, Emitter, Manager};
mod commands;
mod proxy;
use commands::*;
use proxy::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            #[cfg(windows)]
            app.manage(
                proxy::system_proxy::InstanceGuard::acquire().map_err(std::io::Error::other)?,
            );
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            proxy::paths::init_paths(app.handle());
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let state = handle.state::<AppState>();
                    match state.check_health().await {
                        Ok(true) => {
                            let _ = handle.emit("proxy-status-changed", ());
                        }
                        Ok(false) => {}
                        Err(error) => {
                            if let Ok(mut message) = state.lifecycle_error.lock() {
                                if *message != error {
                                    *message = error.clone();
                                    let _ = handle.emit("proxy-shutdown-error", error.clone());
                                    log::error!("Proxy recovery failed: {}", error);
                                }
                            }
                        }
                    }
                }
            });
            // Frontend initialization uses the same start command, surfacing failures.
            Ok(())
        })
        .invoke_handler(generate_handler![
            start_core,
            stop_core,
            start_proxy,
            stop_proxy,
            get_proxy_status,
            is_proxy_running,
            get_proxies,
            change_proxy,
            test_proxy,
            get_providers,
            get_provider_proxies,
            trigger_provider_health_check,
            get_rules,
            set_rules,
            get_rule_config,
            save_subscription,
            set_proxy_provider_url,
            add_proxy_provider,
            set_active_subscription,
            update_proxy_provider,
            remove_proxy_provider,
            get_config,
            get_connections,
            close_connection,
            close_all_connections,
            get_logs,
            get_uptime,
            toggle_tun,
            get_tun_status,
            get_controller_config
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle().clone();
                if app.state::<AppState>().closing.swap(true, Ordering::SeqCst) {
                    return;
                }
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = app.state::<AppState>().stop_core().await {
                        log::error!("Proxy shutdown failed: {}", error);
                        app.state::<AppState>()
                            .closing
                            .store(false, Ordering::SeqCst);
                        let _ = app.emit(
                            "proxy-shutdown-error",
                            format!("退出前恢复系统代理失败，窗口已保留，请重试关闭代理: {error}"),
                        );
                        return;
                    }
                    app.exit(0);
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
