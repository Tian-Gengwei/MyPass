# Phase 2: 生物识别 + TOTP 详细计划

> **目标时间**: Week 4-5
> **核心价值**: 实现极速解锁机制（生物识别/PIN）+ TOTP 功能

---

## 2.1 QuickKey 机制 (P0)

### 解锁流程
```
1. 首次主密码解锁后:
   - 生成 256-bit 随机 QuickKey
   - 用 MEK 加密 QuickKey → quick_key.enc
   - QuickKey 存入系统密钥库（Keychain/Keystore）

2. 后续生物/PIN 解锁:
   - 生物识别/PIN 验证 → 获取 QuickKey
   - 读取 quick_key.enc → 用 QuickKey 解密 → 获取 MEK
   - 用 MEK 解锁 Vault
```

### 文件变更
| 操作 | 文件 | 说明 |
|------|------|------|
| 新建 | `crates/mypass-core/src/auth/quickkey.rs` | QuickKey 生成/加密/存储逻辑 |
| 修改 | `crates/mypass-core/src/auth/mod.rs` | 导出 QuickKeyManager |
| 新建 | `src-tauri/src/commands/quickkey.rs` | Tauri 命令：enable_quickkey / unlock_with_quickkey |

---

## 2.2 生物识别集成 (P0)

### 技术选型
- **Tauri 2 插件**: `tauri-plugin-biometric`
- **支持平台**: Windows Hello, macOS Touch ID, iOS Face ID/Touch ID, Android Fingerprint

### 实现步骤
1. 添加 Cargo 依赖 `tauri-plugin-biometric = "2.0"`
2. 添加前端依赖 `@tauri-apps/plugin-biometric`
3. 配置 `tauri.conf.json` 添加 biometric 权限
4. 实现 `commands/biometric.rs`
5. 前端添加"启用生物识别"按钮

---

## 2.3 PIN 速率限制完善 (P1)

已有 `auth/pin.rs` 实现，需在 Tauri 层暴露命令。

### 命令设计
| 命令 | 输入 | 输出 | 说明 |
|------|------|------|------|
| `set_pin` | PIN 码 | void | 设置/更改 PIN |
| `verify_pin` | PIN 码 | bool | 验证 PIN，返回是否锁定 |
| `is_pin_set` | void | bool | 检查是否已设置 PIN |

---

## 2.4 TOTP 前端组件 (P1)

### 组件设计
```tsx
interface TotpTimerProps {
  secret: string  // otpauth://totp/... URL
  onCopy?: (code: string) => void
}

export function TotpTimer({ secret, onCopy }: TotpTimerProps) {
  // 显示倒计时环形进度 + 一键复制
}
```

### 文件变更
| 操作 | 文件 | 说明 |
|------|------|------|
| 新建 | `frontend/src/components/TotpTimer.tsx` | TOTP 倒计时组件 |
| 修改 | `frontend/src/components/MainLayout.tsx` | 详情页集成 TotpTimer |
| 新建 | `src-tauri/src/commands/totp.rs` | generate_totp 命令 |

---

**最后更新**: 2026-05-11