# MyPass API 设计文档

> 本文档定义 MyPass 前后端通信的 Tauri 命令接口，包括请求/响应格式、错误处理规范。

---

## 一、命令分类概览

| 分类 | 前缀 | 说明 |
|------|------|------|
| Vault 管理 | `vault_*` | 金库创建/解锁/锁定 |
| 条目操作 | `entry_*` | CRUD 操作 |
| 组操作 | `group_*` | CRUD 操作 |
| 认证 | `auth_*` | 生物识别/PIN/QuickKey |
| 同步 | `sync_*` | 同步操作 |
| 导入 | `import_*` | 旧格式导入 |
| TOTP | `totp_*` | TOTP 生成 |

---

## 二、Vault 管理命令

### 2.1 create_vault
```typescript
// Request
{ password: string, name: string }
// Response
{ is_unlocked: boolean, entry_count: number, group_count: number }
```

### 2.2 unlock_vault
```typescript
// Request
{ password: string }
// Response
{ is_unlocked: boolean, entry_count: number, group_count: number }
```

### 2.3 lock_vault
```typescript
// Request: void
// Response: void
```

---

## 三、条目操作命令

### 3.1 get_entries
```typescript
// Response: Entry[]
interface Entry {
  id: string
  name: string
  username: string
  password: string
  url: string | null
  notes: string | null
  otp_auth_url: string | null
  group_id: string | null
  custom_fields: Record<string, string>
  created_at: number
  updated_at: number
  version: number
}
```

### 3.2 create_entry
```typescript
// Request
{ name: string, username: string, password: string, url?: string, notes?: string, group_id?: string }
// Response: Entry
```

### 3.3 update_entry
```typescript
// Request
{ id: string, name: string, username: string, password: string, url?: string, notes?: string }
// Response: Entry
```

### 3.4 delete_entry
```typescript
// Request: { id: string }
// Response: void
```

### 3.5 search_entries
```typescript
// Request: { query: string, limit?: number }
// Response: Entry[]
```

---

## 四、认证命令

### 4.1 enable_quickkey
```typescript
// Response: { success: boolean, quick_key_id: string }
```

### 4.2 unlock_with_quickkey
```typescript
// Request: { quick_key_id: string }
// Response: VaultMetadata
```

### 4.3 check_biometric_available
```typescript
// Response: { available: boolean, biometry_type: string | null }
```

### 4.4 authenticate_biometric
```typescript
// Request: { reason: string }
// Response: { success: boolean, error?: string }
```

---

## 五、同步命令

### 5.1 sync_vault
```typescript
// Request: { force: boolean }
// Response: { pulled: number, pushed: number, conflicts: number, duration_ms: number }
```

### 5.2 get_sync_status
```typescript
// Response: { last_sync: number | null, is_syncing: boolean, pending_changes: number, provider: string | null }
```

---

## 六、导入命令

### 6.1 import_keepass
```typescript
// Request: { file_path: string, password: string }
// Response: { entries_imported: number, groups_imported: number, warnings: string[] }
```

### 6.2 import_bitwarden
```typescript
// Request: { file_path: string, password?: string }
// Response: { entries_imported: number, groups_imported: number, warnings: string[] }
```

---

## 七、TOTP 命令

### 7.1 generate_totp
```typescript
// Request: { secret_url: string }
// Response: { code: string, remaining_secs: number }
```

---

## 八、错误处理规范

### 8.1 错误响应格式
```typescript
interface TauriError {
  message: string      // 人类可读的错误消息
  code: string         // 错误码
  data?: any           // 附加数据（如剩余时间）
}
```

### 8.2 常见错误码
| 错误码 | 说明 |
|--------|------|
| `INVALID_PASSWORD` | 密码错误 |
| `NOT_UNLOCKED` | 金库未解锁 |
| `ENTRY_NOT_FOUND` | 条目不存在 |
| `ACCOUNT_LOCKED` | 账户已锁定 |
| `SYNC_FAILED` | 同步失败 |
| `IO_ERROR` | 文件系统错误 |

---

**最后更新**: 2026-05-11
**维护者**: MyPass Team