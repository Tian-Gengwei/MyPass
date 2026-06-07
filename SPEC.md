# MyPass 规格文档

## 1. 项目概述

**项目名称**: MyPass
**类型**: 本地优先、跨平台、端到端加密的密码管理器
**核心价值**: 安全、快速、跨平台统一体验

## 2. 技术栈

| 层次 | 技术选型 |
|------|----------|
| 核心逻辑 | Rust (forbid(unsafe_code)) |
| 应用壳 | Tauri 2.0 |
| 前端 | React + TailwindCSS + Shadcn/UI + Framer Motion |
| 状态管理 | Zustand |
| 浏览器扩展 | WXT 框架 |
| 加密 | Argon2id + XChaCha20-Poly1305 |

## 3. 架构模式

- **系统操作**: Tauri API (`@tauri-apps/api`)
- **自定义服务**: 命令模式 (`invoke`)

## 4. Vault 存储结构

```
my_vault.vault/
├── vault.meta.json       # 库元数据
├── master_key.enc        # 加密的 MEK
├── manifest.enc          # 全局索引
└── objects/              # 条目存储 (按 Hash 切片)
```

## 5. 加密方案

- **KDF**: Argon2id (主密码 -> KEK)
- **数据加密**: XChaCha20-Poly1305 AEAD
- **密钥层次**:
  - MEK: 主加密密钥
  - KEK: 从主密码派生，用于加密 MEK

## 6. 功能优先级

### Phase 1 (Week 1-3): 核心引擎 + 桌面 UI
- [x] Rust 核心库初始化
- [x] 加密模块 (Argon2id + XChaCha20-Poly1305)
- [x] Vault 对象存储
- [x] Manifest 索引
- [x] Tauri 命令
- [x] React 前端 + 主密码 UI
- [x] PIN 码 + 系统密钥库
- [x] 三栏布局 + CRUD
- [x] 搜索 + WebDAV 同步
- [x] KDBX/Bitwarden 导入

### Phase 2 (Week 4-5): 导入 + 增强认证
- [ ] 生物识别集成
- [ ] TOTP 生成

### Phase 3 (Week 6-7): 浏览器扩展
- [ ] WXT 扩展
- [ ] 表单识别
- [ ] 长连接 + 自动填存

### Phase 4 (Week 8-10): 多端同步 + 移动端
- [ ] 冲突解决
- [ ] S3 同步
- [ ] Android/iOS 构建

## 7. UI/UX

- **布局**: 三栏 (桌面) / 堆栈 (移动)
- **主题**: 暗色模式优先，现代极简风
- **交互**: Framer Motion 动画

## 8. 安全特性

- 主密码 (必选)
- PIN 码 + 速率限制
- 生物识别 (指纹/Face ID/Windows Hello)
- TOTP (条目内)
- WebAuthn/Passkeys 代理
- ChaCha8/ChaCha20 日志加密
