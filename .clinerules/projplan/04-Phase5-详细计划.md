# Phase 5: 旧生态迁移 + 安全打磨详细计划

> **目标时间**: Week 11-12
> **核心价值**: 降低迁移成本，完善边缘场景，提升安全性

---

## 5.1 KDBX 解析 (P1)

### 技术选型

- **Rust 库**: `keepass` crate
- **支持版本**: KDBX 3.1 / 4.x

### 实现

```rust
pub struct KeePassImporter;
impl KeePassImporter {
    pub fn import(&self, data: &[u8], password: &str) -> Result<ImportResult>
}
```

---

## 5.2 Bitwarden JSON 解析 (P1)

### 实现

```rust
pub struct BitwardenImporter;
impl BitwardenImporter {
    pub fn import(&self, data: &[u8], password: Option<&str>) -> Result<ImportResult>
}
```

---

## 5.3 附件处理 (P2)

### 存储结构

```
my_vault.vault/
├── attachments/
│   ├── a1/b2/a1b2...enc    # 附件加密文件
│   └── d4/e5/d4e5...enc
└── attachment_manifest.enc   # 附件索引
```

---

## 5.4 内存清零 (P1)

### 实现方案

1. 添加 `zeroize = "1.7"` 依赖
2. 包装敏感类型

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SensitiveKey([u8; 32]);
```

---

## 5.5 防截屏 (P2)

### 实现方案

**Windows**: `SetWindowDisplayAffinity`
**macOS**: `CGWindowListCreate` 排除窗口
**Tauri 配置**: `"security": { "preventScreenshots": true }`

---

## 5.6 剪贴板自动清空 (P2)

### 实现

```typescript
export async function copyWithAutoClear(text: string, delayMs = 30000) {
  await navigator.clipboard.writeText(text)
  setTimeout(async () => {
    await navigator.clipboard.writeText('')
  }, delayMs)
}
```

---

**最后更新**: 2026-05-11