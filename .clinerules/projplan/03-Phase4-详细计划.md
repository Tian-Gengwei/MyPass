# Phase 4: 同步 + 移动端详细计划

> **目标时间**: Week 8-10
> **核心价值**: 多端数据同步与全平台覆盖

---

## 4.1 WebDAV 同步 (P1)

### 技术选型
- **Rust 库**: `webdav` crate
- **配置**: 服务器 URL、用户名、密码（加密存储）

### 核心方法
```rust
pub struct WebDavProvider {
    pub async fn fetch_manifest(&self) -> Result<Vec<u8>>
    pub async fn upload_manifest(&self, data: &[u8]) -> Result<()>
    pub async fn fetch_object(&self, hash: &str) -> Result<Vec<u8>>
    pub async fn upload_object(&self, hash: &str, data: &[u8]) -> Result<()>
}
```

---

## 4.2 S3 同步 (P2)

### 技术选型
- **Rust 库**: `aws-sdk-s3` 或 `s3` crate
- **支持**: AWS S3、MinIO、COS 等

---

## 4.3 冲突收敛 (P1)

### 冲突策略
| 场景 | 策略 |
|------|------|
| 本地新于远端 | 推送本地 |
| 远端新于本地 | 拉取远端 |
| 版本相同内容不同 | 保留两者 |

---

## 4.4 Android 构建 (P0)

### 构建步骤
```bash
# 1. 安装 Android SDK (Android Studio)
# 2. 配置 tauri.conf.json
npm run tauri build -- --target android
```

### 权限配置
```xml
<uses-permission android:name="android.permission.USE_BIOMETRIC" />
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.USE_FINGERPRINT" />
```

---

## 4.5 iOS 构建 (P0)

### 构建步骤
```bash
# 1. macOS + Xcode
# 2. 配置 tauri.conf.json
npm run tauri build -- --target ios
```

---

## 4.6 响应式 UI (P1)

### 断点设计
| 断点 | 宽度 | 布局 |
|------|------|------|
| Desktop | >= 1024px | 三栏 |
| Tablet | 768px - 1023px | 两栏 |
| Mobile | < 768px | 单栏堆栈 |

---

**最后更新**: 2026-05-11