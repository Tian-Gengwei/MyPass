# MyPass

> 本地优先、跨平台、端到端加密的现代密码管理器。

MyPass 是一个完全本地优先的密码管理器，零依赖云服务，所有数据加密后存储在用户设备上。支持 WebDAV 协议进行可选的端到端加密同步，浏览器扩展自动填充，以及生物识别/QuickKey 秒解锁。

## ✨ 核心特性

| 特性 | 描述 |
|------|------|
| 🔐 **端到端加密** | Argon2id KDF + XChaCha20-Poly1305 AEAD，零时清理 MEK |
| 🌐 **本地优先** | 数据永不离开设备，离线完全可用 |
| 🔄 **WebDAV 同步** | 支持 Nextcloud/ownCloud/标准 WebDAV 服务器 |
| 🧩 **导入器** | KeePass KDBX（签名检测）、Bitwarden JSON、Chrome/Firefox CSV |
| 🔑 **QuickKey** | 生物识别/PIN 秒解锁，无需每次输入主密码 |
| 📱 **TOTP** | 内置 6/8 位 TOTP 生成与验证（RFC 6238） |
| 🔍 **搜索索引** | 预计算 lowercase 字段的 O(1) 索引 |
| 🛡️ **安全特性** | 常量时间比较、速率限制、零时内存清理 |

## 🏗️ 技术栈

| 层次 | 技术 |
|------|------|
| 核心逻辑 | Rust 2021 (`forbid(unsafe_code)`) |
| 桌面应用 | Tauri 2.0 |
| 前端 | React 18 + TypeScript + TailwindCSS + Shadcn/UI |
| 状态管理 | Zustand 4 |
| 加密 | Argon2id + XChaCha20-Poly1305 + HKDF-SHA256 |
| KDF | Argon2id（OWASP 推荐参数） |
| 浏览器扩展 | 原生 MV2 + WXT 框架双轨 |
| 同步 | WebDAV（手写 HTTP/1.1 + rustls） |

## 📁 项目结构

```
MyPass/
├── Cargo.toml                    # Rust workspace
├── crates/
│   └── mypass-core/              # 核心加密与 Vault 库
│       ├── src/
│       │   ├── crypto/           # Argon2id, XChaCha20, HKDF, secure_random
│       │   ├── vault/            # Vault, Entry, Group, Manifest, Storage
│       │   ├── sync/             # WebDAV 同步 (S3 接口预留)
│       │   ├── auth/             # master_password, pin, quickkey
│       │   ├── otp/              # TOTP (RFC 6238)
│       │   ├── import/           # keepass, bitwarden, chrome, edge
│       │   ├── platform/         # biometric, keychain 接口
│       │   ├── security/         # zero_string, clipboard, screen_protection
│       │   ├── performance/      # LRU cache, search index
│       │   ├── extension/        # 扩展通信协议
│       │   ├── error.rs          # 错误类型定义
│       │   └── lib.rs            # 库入口
│       └── tests/                # 集成测试
│
├── src-tauri/                    # Tauri 2 桌面应用
│   ├── src/
│   │   ├── main.rs               # 应用入口
│   │   ├── error.rs              # TauriError + From 转换
│   │   └── commands/             # IPC 命令
│   │       ├── vault.rs          # 金库管理
│   │       ├── sync.rs           # WebDAV 同步
│   │       ├── import.rs         # 数据导入
│   │       ├── totp.rs           # TOTP 生成
│   │       ├── security.rs       # 锁定/会话
│   │       ├── biometric.rs      # 生物识别
│   │       ├── pin.rs            # PIN 验证
│   │       ├── quickkey.rs       # QuickKey
│   │       ├── webauthn.rs       # WebAuthn/Passkey
│   │       └── extension.rs      # 扩展 API
│   ├── capabilities/default.json # 权限配置
│   └── tauri.conf.json           # Tauri 配置
│
├── frontend/                     # React 前端
│   └── src/
│       ├── components/           # UI 组件（MainLayout, EntryForm, TotpTimer 等）
│       ├── stores/               # Zustand stores
│       └── lib/                  # 工具
│
├── extension/                    # Firefox 优先的浏览器扩展
│   ├── manifest.json             # MV2 manifest
│   └── src/
│
└── extensions/wxt/               # Chrome 优先的 WXT 扩展
    └── src/
```

## 🚀 快速开始

### 环境要求
- Rust 1.75+ (stable)
- Node.js 18+ 和 npm 9+
- Windows: Visual Studio Build Tools (推荐) 或 MinGW-w64
- macOS: Xcode Command Line Tools
- Linux: build-essential, libssl-dev, libwebkit2gtk-4.1-dev

### 注意事项
- **sync 功能为必需特性**，默认已启用，支持 WebDAV 同步
- Windows 上推荐使用 MSVC 工具链而非 GNU 工具链

### 开发模式

```bash
# 1. 安装前端依赖
cd frontend && npm install

# 2. 启动 Tauri 开发模式（自动启动 Vite + 编译 Rust）
cd .. && npm run tauri dev

# 3. 运行 Rust 核心库测试
cd crates/mypass-core && cargo test
```

### 生产构建

```bash
# 构建桌面应用
npm run tauri build

# 产物在 target/release/mypass-tauri(.exe)
```

支持的 bundle 目标（通过 `tauri bundle`）：
- Windows: MSI, NSIS
- macOS: DMG, APP
- Linux: DEB, AppImage, RPM

## 🔒 安全模型

### 加密流程
1. 用户输入主密码
2. Argon2id 派生 KEK（64 MiB, 3 iter, 4 parallel）
3. 随机生成 256-bit MEK（主加密密钥）
4. 用 KEK 加密 MEK 存储到 `master_key.enc`
5. 用 MEK 加密每个对象，使用 XChaCha20-Poly1305 AEAD
6. 对象按 SHA-256 路径分片存储到 `objects/{ab}/{cd}/abcd1234.enc`

### 内存保护
- MEK 在 `Drop` 时自动 zeroize
- 使用 `OsRng` (CSPRNG) 而非 `thread_rng`
- 常量时间比较防止时序攻击
- HKDF 子密钥派生（用于日志加密等次要用途）

### 速率限制
- 主密码：5 次失败 → 5 分钟锁定
- PIN：5 次失败 → 5 分钟锁定（独立计数器）
- 自动锁定：5 分钟无活动（可配置）

## 📊 测试覆盖

- **集成测试**：Vault 完整生命周期
- RFC 6238 TOTP 测试向量
- Manifest 同步算法测试
- 加密安全测试（nonce 唯一性、错误密钥、篡改检测）

## 🗺️ 开发路线图

### ✅ 已完成
- [x] Rust 核心库（crypto, vault, auth, otp, sync, import）
- [x] Tauri 应用壳（IPC 命令）
- [x] React 前端（三栏布局、添加/编辑/删除、TOTP 显示）
- [x] Zustand 状态管理
- [x] 搜索索引
- [x] 零时内存清理
- [x] KeePass KDBX 签名检测
- [x] Bitwarden JSON 完整导入
- [x] Chrome/Firefox CSV 完整导入
- [x] WebDAV 同步完整实现
- [x] Sync 功能设为默认必需功能

### 📋 未来计划
- [ ] S3 同步（接口已预留）
- [ ] WebAuthn/Passkey 真实集成
- [ ] CI/CD（GitHub Actions）
- [ ] 移动端构建（Android/iOS）
- [ ] 端到端测试（Playwright）

## 🤝 贡献

欢迎贡献！请先阅读 [SECURITY.md](SECURITY.md) 了解漏洞披露政策。

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🔐 安全披露

如发现安全漏洞，请发送邮件至 security@mypass.app，**不要**公开 GitHub issue。
