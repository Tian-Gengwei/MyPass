//! Tauri 构建脚本
//!
//! 修复 GNU 工具链下 tauri-winres 失败的问题

fn main() {
    // 仅在 MSVC 工具链上运行 tauri_build
    // 在 GNU 工具链上跳过 windows 资源编译
    #[cfg(not(target_env = "msvc"))]
    {
        eprintln!("Skipping tauri_build on GNU toolchain (windows resource issue)");
        return;
    }

    #[cfg(target_env = "msvc")]
    tauri_build::build()
}
