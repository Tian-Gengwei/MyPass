# 🖥️ Tauri 2 与系统融合

<版本约束>
- 严格使用 Tauri 2.x API。绝对禁止混用 1.x 语法（如 `tauri::api::*` 或旧版窗口管理）。
- 前后端通信统一使用 `tauri::command` 和 `invoke`。
  </版本约束>

<系统级融入>
- 窗口：必须实现自定义 Titlebar + Vibrancy/毛玻璃效果。
- 后台：系统托盘常驻 + 全局快捷键唤起。
- 插件：优先使用 Tauri 2 官方插件生态（如 `tauri-plugin-biometric`）。
  </系统级融入>

<反模式_坚决抵制>
- ❌ 在 Tauri Command 中阻塞主线程。I/O 或计算密集型任务必须使用 `async` 命令。
- ❌ 将敏感明文存储在 `localStorage` 或未加密的 OS 文件系统中。
  </反模式_坚决抵制>
