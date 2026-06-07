#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;

use commands::{vault, import, totp, security, webauthn, biometric, extension, pin, quickkey, sync};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Android 平台日志初始化
#[cfg(target_os = "android")]
fn init_android_logging() {
    // Android 使用 android_logger 输出到 logcat
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Info)
            .with_tag("MyPass"),
    );
}

fn main() {
    // Android 平台初始化
    #[cfg(target_os = "android")]
    init_android_logging();

    let log_level = std::env::var("MYPASS_LOG_LEVEL")
        .unwrap_or_else(|_| "warn".to_string());

    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("mypass={}", log_level).parse().unwrap())
        .add_directive("tauri=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();

    tracing::info!("Starting MyPass Tauri application");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(commands::pin::PinManagerState::default())
        .invoke_handler(tauri::generate_handler![
            // Vault 命令
            vault::create_vault,
            vault::unlock_vault,
            vault::lock_vault,
            vault::get_vault_info,
            vault::get_entries,
            vault::get_entry,
            vault::create_entry,
            vault::update_entry,
            vault::delete_entry,
            vault::get_groups,
            vault::create_group,
            vault::delete_group,
            vault::search_entries,
            // 导入命令
            import::import_keepass,
            import::import_bitwarden,
            import::import_bitwarden_csv,
            import::import_chrome_csv,
            import::get_supported_import_formats,
            // TOTP 命令
            totp::generate_totp,
            totp::verify_totp,
            totp::parse_totp_url_command,
            // 安全命令
            security::set_auto_lock_timeout,
            security::get_auto_lock_timeout,
            security::keep_alive,
            security::get_session_status,
            // WebAuthn 命令
            webauthn::webauthn_is_available,
            webauthn::webauthn_authenticate,
            webauthn::webauthn_register,
            webauthn::webauthn_get_supported_authenticators,
            // 生物识别命令
            biometric::check_biometric_available,
            biometric::authenticate_biometric,
            biometric::get_biometry_type,
            // 浏览器扩展命令
            extension::vault_status,
            extension::get_extension_entries,
            extension::extension_save_credential,
            // PIN 命令
            pin::set_pin,
            pin::verify_pin,
            pin::is_pin_set,
            // QuickKey 命令
            quickkey::enable_quickkey,
            quickkey::unlock_with_quickkey,
            quickkey::is_quickkey_enabled,
            quickkey::get_quickkey_id,
            // 同步命令
            sync::sync_vault,
            sync::get_sync_status,
            sync::configure_sync,
            sync::test_sync_connection,
        ])
        .setup(|_app| {
            tracing::info!("MyPass setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
